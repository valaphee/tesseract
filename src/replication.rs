use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::{Duration, Instant},
};

use bevy::prelude::*;
use futures::{SinkExt, StreamExt};
use num::BigInt;
use rand::{thread_rng, RngCore};
use rsa::{pkcs8::EncodePublicKey, rand_core::OsRng, Pkcs1v15Encrypt, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha1::{digest::Update, Digest, Sha1};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::Framed;

use mojang_session_api::{
    apis::{configuration::Configuration, default_api::has_joined_server},
    models::User,
};
use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, c2s::game::PlayerActionPacketAction, s2c},
    types::{
        Angle, Biome, Component as ChatComponent, DamageType, DimensionType, GameType, Intention,
        Json, Nbt, Registries, Registry, Status, StatusPlayers, StatusVersion, VarI32,
    },
    Decode, Encode,
};

use crate::{actor, level, registry, PreLoad, Save};

#[derive(Serialize, Deserialize)]
pub struct ReplicationPlugin {
    address: SocketAddr,

    compression: u8,
    compression_threshold: i16,
}

impl Default for ReplicationPlugin {
    fn default() -> Self {
        Self {
            address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565).into(),

            compression: Compression::default().level() as u8,
            compression_threshold: 256,
        }
    }
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        let address = self.address;

        let compression = Compression::new(self.compression as u32);
        let compression_threshold = if self.compression_threshold < 0 {
            None
        } else {
            Some(self.compression_threshold as u16)
        };

        let listen = move |mut commands: Commands| {
            let (new_connection_tx, new_connection_rx) = mpsc::unbounded_channel();

            commands.insert_resource(NewConnectionRx(new_connection_rx));

            std::thread::spawn(move || {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let private_key = RsaPrivateKey::new(&mut OsRng, 1024).unwrap();
                        let listener = TcpListener::bind(address).await.unwrap();

                        info!("Listening on {}", address);

                        loop {
                            if let Ok((socket, _)) = listener.accept().await {
                                tokio::spawn(handle_new_connection(
                                    socket,
                                    private_key.clone(),
                                    compression,
                                    compression_threshold,
                                    new_connection_tx.clone(),
                                ));
                            }
                        }
                    })
            });
        };

        app.add_systems(PostStartup, listen)
            .add_systems(PreLoad, spawn_player)
            .add_systems(PreLoad, update_players)
            .add_systems(Save, replicate_initial)
            .add_systems(Save, subscribe_and_replicate_chunks)
            .add_systems(Save, cleanup_chunks)
            .add_systems(Save, replicate_chunks_late)
            .add_systems(Save, replicate_chunks_delta)
            .add_systems(Save, replicate_actors)
            .add_systems(Save, replicate_actors_delta);
    }
}

#[derive(Component)]
pub struct Connection {
    user: User,

    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    tx: mpsc::UnboundedSender<Vec<u8>>,

    keep_alive: Instant,
    keep_alive_id: Option<i64>,
    latency: u32,
}

impl Connection {
    fn send(&self, packet: &s2c::GamePacket) {
        let mut data = vec![];
        if packet.encode(&mut data).is_ok() {
            let _ = self.tx.send(data);
        }
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn latency(&self) -> u32 {
        self.latency
    }
}

async fn handle_new_connection(
    socket: TcpStream,
    private_key: RsaPrivateKey,
    compression: Compression,
    compression_threshold: Option<u16>,
    new_connection_tx: mpsc::UnboundedSender<Connection>,
) -> tesseract_protocol::Result<()> {
    socket.set_nodelay(true).unwrap();

    let mut framed_socket = Framed::new(socket, Codec::default());

    match next(&mut framed_socket).await?.decode()? {
        c2s::HandshakePacket::Intention { intention, .. } => match intention {
            Intention::Status => {
                match next(&mut framed_socket).await?.decode()? {
                    c2s::StatusPacket::StatusRequest => {
                        encode_and_send(
                            &mut framed_socket,
                            &s2c::StatusPacket::StatusResponse {
                                status: Json(Status {
                                    description: Some(ChatComponent::Literal(
                                        "Tesseract".to_string(),
                                    )),
                                    players: Some(StatusPlayers {
                                        max: 1,
                                        online: 0,
                                        sample: vec![],
                                    }),
                                    version: Some(StatusVersion {
                                        name: "1.19.4".to_string(),
                                        protocol: 762,
                                    }),
                                    favicon: None,
                                }),
                            },
                        )
                        .await;
                    }
                    _ => return Err(tesseract_protocol::Error::UnexpectedPacket),
                }

                match next(&mut framed_socket).await?.decode()? {
                    c2s::StatusPacket::PingRequest { time } => {
                        encode_and_send(
                            &mut framed_socket,
                            &s2c::StatusPacket::PongResponse { time },
                        )
                        .await;
                    }
                    _ => return Err(tesseract_protocol::Error::UnexpectedPacket),
                };
            }
            Intention::Login => {
                let name = match next(&mut framed_socket).await?.decode()? {
                    c2s::LoginPacket::Hello { name, .. } => name,
                    _ => return Err(tesseract_protocol::Error::UnexpectedPacket),
                };

                let nonce: [u8; 16] = rand::random();
                encode_and_send(
                    &mut framed_socket,
                    &s2c::LoginPacket::Hello {
                        server_id: "".to_string(),
                        public_key: private_key.to_public_key_der().unwrap().to_vec(),
                        nonce: nonce.to_vec(),
                    },
                )
                .await;
                let key = match next(&mut framed_socket).await?.decode()? {
                    c2s::LoginPacket::Key { key, nonce } => {
                        private_key
                            .decrypt(Pkcs1v15Encrypt::default(), &nonce)
                            .unwrap();
                        private_key
                            .decrypt(Pkcs1v15Encrypt::default(), &key)
                            .unwrap()
                    }
                    _ => return Err(tesseract_protocol::Error::UnexpectedPacket),
                };
                framed_socket.codec_mut().enable_encryption(&key);

                let user = has_joined_server(
                    &Configuration::new(),
                    &name,
                    &BigInt::from_signed_bytes_be(
                        &Sha1::new()
                            .chain(&key)
                            .chain(private_key.to_public_key_der().unwrap().as_bytes())
                            .finalize(),
                    )
                    .to_str_radix(16),
                    None,
                )
                .await
                .unwrap();

                if let Some(compression_threshold) = compression_threshold {
                    encode_and_send(
                        &mut framed_socket,
                        &s2c::LoginPacket::LoginCompression {
                            compression_threshold: VarI32(compression_threshold as i32),
                        },
                    )
                    .await;
                    framed_socket
                        .codec_mut()
                        .enable_compression(compression, compression_threshold);
                }

                encode_and_send(
                    &mut framed_socket,
                    &s2c::LoginPacket::GameProfile(user.clone()),
                )
                .await;

                let (rx_packet_tx, rx_packet_rx) = mpsc::unbounded_channel();
                let (tx_packet_tx, mut tx_packet_rx) = mpsc::unbounded_channel();
                let _ = new_connection_tx.send(Connection {
                    user,
                    rx: rx_packet_rx,
                    tx: tx_packet_tx,
                    keep_alive: Instant::now(),
                    keep_alive_id: None,
                    latency: 0,
                });

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            packet = framed_socket.next() => {
                                if let Some(Ok(packet)) = packet {
                                    let _ = rx_packet_tx.send(packet);
                                } else {
                                    break;
                                }
                            }
                            packet = tx_packet_rx.recv() => {
                                if let Some(packet) = packet {
                                    if framed_socket.send(&packet).await.is_err() {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    tx_packet_rx.close();
                    let _ = framed_socket.close().await;
                });
            }
            _ => return Err(tesseract_protocol::Error::UnexpectedPacket),
        },
    }

    Ok(())
}

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_player(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        info!(
            "Player {} (UUID: {}) connected",
            connection.user.name, connection.user.id
        );

        commands.spawn((
            connection,
            SubscriptionDistance::default(),
            Subscriptions::default(),
        ));
    }
}

fn replicate_initial(
    dimension_type_registry: Res<registry::DataRegistry<DimensionType>>,
    biome_registry: Res<registry::DataRegistry<Biome>>,
    damage_type_registry: Res<registry::DataRegistry<DamageType>>,
    levels: Query<(&level::Level, &level::AgeAndTime)>,
    chunks: Query<&Parent>,
    players: Query<
        (
            Entity,
            &Connection,
            &actor::Position,
            &actor::Rotation,
            &Parent,
        ),
        Added<Connection>,
    >,
) {
    for (player, connection, actor_position, actor_rotation, chunk) in players.iter() {
        let (level_base, level_age_and_time) =
            levels.get(chunks.get(chunk.get()).unwrap().get()).unwrap();
        connection.send(&s2c::GamePacket::Login {
            player_id: player.index() as i32,
            hardcore: false,
            game_type: GameType::Creative,
            previous_game_type: 0,
            levels: vec![level_base.name.to_string()],
            registry_holder: Nbt(Registries {
                dimension_type: Cow::Borrowed(dimension_type_registry.registry()),
                biome: Cow::Borrowed(biome_registry.registry()),
                chat_type: Cow::Owned(Registry {
                    type_: "minecraft:chat_type".into(),
                    value: vec![],
                }),
                damage_type: Cow::Borrowed(damage_type_registry.registry()),
            }),
            dimension_type: level_base.dimension_type.clone(),
            dimension: level_base.name.to_string(),
            seed: 0,
            max_players: VarI32(0),
            chunk_radius: VarI32(16),
            simulation_distance: VarI32(16),
            reduced_debug_info: false,
            show_death_screen: false,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
        });
        connection.send(&s2c::GamePacket::SetDefaultSpawnPosition {
            pos: default(),
            yaw: default(),
        });
        connection.send(&s2c::GamePacket::SetTime {
            game_time: level_age_and_time.age as i64,
            day_time: level_age_and_time.time as i64,
        });
        connection.send(&s2c::GamePacket::PlayerPosition {
            pos: actor_position.0,
            yaw: actor_rotation.yaw,
            pitch: actor_rotation.pitch,
            relative_arguments: 0,
            id: VarI32(0),
        });
    }
}

fn update_players(
    mut commands: Commands,
    mut players: Query<(
        Entity,
        &mut Connection,
        &mut SubscriptionDistance,
        &mut actor::Position,
        &mut actor::Rotation,
        &mut actor::player::Interaction,
    )>,
) {
    for (
        player,
        mut connection,
        mut subscription_distance,
        mut position,
        mut rotation,
        mut interaction,
    ) in players.iter_mut()
    {
        if connection.keep_alive.elapsed() >= Duration::from_secs(15) {
            if connection.keep_alive_id.is_none() {
                let keep_alive_id = thread_rng().next_u64() as i64;
                connection.keep_alive = Instant::now();
                connection.keep_alive_id = Some(keep_alive_id);

                connection.send(&s2c::GamePacket::KeepAlive { id: keep_alive_id });
            } else {
                connection.rx.close();
            }
        }

        if connection.tx.is_closed() {
            info!(
                "Player {} (UUID: {}) disconnected",
                connection.user.name, connection.user.id
            );

            commands
                .entity(player)
                .remove::<Connection>()
                .remove::<SubscriptionDistance>();

            // remove all subscriptions
            subscription_distance.0 = 0;
        } else {
            while let Ok(packet) = connection.rx.try_recv() {
                match Packet(packet).decode().unwrap() {
                    c2s::GamePacket::ClientInformation { view_distance, .. } => {
                        let new_subscription_distance = view_distance as u8 + 3;
                        if subscription_distance.0 != new_subscription_distance {
                            subscription_distance.0 = new_subscription_distance;
                        }
                    }
                    c2s::GamePacket::KeepAlive { id } => {
                        if let Some(current_id) = connection.keep_alive_id {
                            if current_id == id {
                                connection.keep_alive_id = None;
                                connection.latency = if connection.latency != 0 {
                                    (connection.latency * 3
                                        + connection.keep_alive.elapsed().as_millis() as u32)
                                        / 4
                                } else {
                                    connection.latency
                                };
                            }
                        }
                    }
                    c2s::GamePacket::MovePlayerPos { x, y, z, .. } => {
                        if position.0.x != x || position.0.y != y || position.0.z != z {
                            position.0.x = x;
                            position.0.y = y;
                            position.0.z = z;
                        }
                    }
                    c2s::GamePacket::MovePlayerPosRot {
                        x,
                        y,
                        z,
                        pitch,
                        yaw,
                        ..
                    } => {
                        if position.0.x != x || position.0.y != y || position.0.z != z {
                            position.0.x = x;
                            position.0.y = y;
                            position.0.z = z;
                        }
                        if rotation.pitch != pitch || rotation.yaw != yaw {
                            rotation.pitch = pitch;
                            rotation.yaw = yaw;
                        }
                    }
                    c2s::GamePacket::MovePlayerRot { pitch, yaw, .. } => {
                        if rotation.pitch != pitch || rotation.yaw != yaw {
                            rotation.pitch = pitch;
                            rotation.yaw = yaw;
                        }
                    }
                    c2s::GamePacket::PlayerAction { action, pos, .. } => match action {
                        PlayerActionPacketAction::StartDestroyBlock => {
                            *interaction = actor::player::Interaction::BlockBreak(pos);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}

//=========================================================================== CHUNK REPLICATION ====

#[derive(Default, Component)]
pub struct Replication {
    subscriber: HashSet<Entity>,
    replicated: Vec<Entity>,
}

impl Replication {
    pub fn with_subscriber(subscriber_: Entity) -> Self {
        Self {
            subscriber: {
                let mut subscriber = HashSet::new();
                subscriber.insert(subscriber_);
                subscriber
            },
            replicated: default(),
        }
    }
}

#[derive(Default, Component)]
struct SubscriptionDistance(pub u8);

#[derive(Default, Component)]
struct Subscriptions(HashSet<IVec2>);

fn cleanup_chunks(
    mut commands: Commands,
    levels: Query<&level::chunk::LookupTable>,
    chunks: Query<&Parent>,
    mut subscription_chunks: Query<&mut Replication>,
    players: Query<(Entity, &Parent, &Subscriptions), Without<Connection>>,
) {
    for (player, chunk, subscriptions) in players.iter() {
        if let Ok(level) = chunks.get(chunk.get()) {
            commands.entity(player).remove::<Subscriptions>();

            let chunk_lut = levels.get(level.get()).unwrap();
            for chunk_position in subscriptions.0.iter() {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    trace!("Release chunk: {:?}", chunk_position);

                    let mut replication = subscription_chunks.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);
                } else {
                    trace!("Release chunk: {:?} (not spawned)", chunk_position);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn subscribe_and_replicate_chunks(
    registries: Res<registry::Registries>,
    mut commands: Commands,
    mut levels: Query<&mut level::chunk::LookupTable>,
    chunks: Query<(&level::chunk::Chunk, &Parent)>,
    mut subscription_chunks: Query<(Option<&level::chunk::Terrain>, &mut Replication)>,
    actors: Query<(
        Entity,
        &actor::Actor,
        &actor::Position,
        &actor::Rotation,
        &actor::HeadRotation,
    )>,
    mut players: Query<
        (
            Entity,
            &Parent,
            &Connection,
            &SubscriptionDistance,
            &mut Subscriptions,
        ),
        Or<(Changed<Parent>, Changed<SubscriptionDistance>)>,
    >,
) {
    for (player, chunk, connection, subscription_distance, mut actual_subscriptions) in
        players.iter_mut()
    {
        if let Ok((chunk_base, level)) = chunks.get(chunk.get()) {
            let subscription_distance = subscription_distance.0 as i32;
            if subscription_distance == 0 {
                continue;
            }

            let x = chunk_base.0.x;
            let z = chunk_base.0.y;
            connection.send(&s2c::GamePacket::SetChunkCacheCenter {
                x: VarI32(x),
                z: VarI32(z),
            });

            let mut acquire_chunks = Vec::new();
            let mut target_subscriptions = HashSet::new();
            if !actual_subscriptions.0.contains(&chunk_base.0) {
                acquire_chunks.push(chunk_base.0);
            }
            target_subscriptions.insert(IVec2::new(x, z));

            // square radius
            for r in 1..subscription_distance {
                for n in -r..r {
                    // north
                    let chunk_position = IVec2::new(x + n, z - r);
                    if !actual_subscriptions.0.contains(&chunk_position) {
                        acquire_chunks.push(chunk_position);
                    }
                    target_subscriptions.insert(chunk_position);

                    // east
                    let chunk_position = IVec2::new(x + r, z + n);
                    if !actual_subscriptions.0.contains(&chunk_position) {
                        acquire_chunks.push(chunk_position);
                    }
                    target_subscriptions.insert(chunk_position);

                    // south
                    let chunk_position = IVec2::new(x - n, z + r);
                    if !actual_subscriptions.0.contains(&chunk_position) {
                        acquire_chunks.push(chunk_position);
                    }
                    target_subscriptions.insert(chunk_position);

                    // west
                    let chunk_position = IVec2::new(x - r, z - n);
                    if !actual_subscriptions.0.contains(&chunk_position) {
                        acquire_chunks.push(chunk_position);
                    }
                    target_subscriptions.insert(chunk_position);
                }
            }

            let mut chunk_lut = levels.get_mut(level.get()).unwrap();

            // release chunks
            for chunk_position in actual_subscriptions
                .0
                .iter()
                .filter(|&chunk_position| !target_subscriptions.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    trace!("Release chunk: {:?}", chunk_position);

                    let (_, mut replication) = subscription_chunks.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);

                    // connection: remove chunk and actors, cause: unsubscribe
                    connection.send(&s2c::GamePacket::RemoveEntities {
                        entity_ids: replication
                            .replicated
                            .iter()
                            .map(|actor| VarI32(actor.index() as i32))
                            .collect(),
                    });
                    connection.send(&s2c::GamePacket::ForgetLevelChunk {
                        x: chunk_position.x,
                        z: chunk_position.y,
                    });
                } else {
                    trace!("Release chunk: {:?} (not spawned)", chunk_position);
                }
            }

            // acquire chunks
            for chunk_position in acquire_chunks {
                if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                    if let Ok((terrain, mut replication)) = subscription_chunks.get_mut(chunk) {
                        trace!("Acquire chunk: {:?}", chunk_position);

                        replication.subscriber.insert(player);

                        if let Some(terrain) = terrain {
                            // connection: add chunk and actors, cause: subscribe
                            connection.send(&add_chunk_packet(chunk_position, terrain));
                            for (
                                actor,
                                actor_base,
                                actor_position,
                                actor_rotation,
                                actor_head_rotation,
                            ) in actors.iter_many(&replication.replicated)
                            {
                                // except owner
                                if actor == player {
                                    continue;
                                }

                                connection.send(&add_actor_packet(
                                    &registries,
                                    actor,
                                    actor_base,
                                    actor_position,
                                    actor_rotation,
                                    actor_head_rotation,
                                ));
                            }
                        }
                    } else {
                        trace!("Acquire chunk: {:?} (not initialized)", chunk_position);
                    }
                } else {
                    trace!("Acquire chunk: {:?} (not spawned)", chunk_position);

                    chunk_lut.0.insert(
                        chunk_position,
                        commands
                            .spawn(level::chunk::ChunkBundle::with_subscriber(
                                chunk_position,
                                player,
                            ))
                            .set_parent(level.get())
                            .id(),
                    );
                }
            }

            actual_subscriptions.0 = target_subscriptions;
        }
    }
}

fn replicate_chunks_late(
    chunks: Query<
        (&level::chunk::Chunk, &level::chunk::Terrain, &Replication),
        Added<level::chunk::Terrain>,
    >,
    players: Query<&Connection>,
) {
    for (chunk_base, terrain, replication) in chunks.iter() {
        let add_chunk_packet = add_chunk_packet(chunk_base.0, terrain);
        for &player in &replication.subscriber {
            // connection: add chunk, cause: subscribe (late)
            if let Ok(connection) = players.get(player) {
                connection.send(&add_chunk_packet);
            }
        }
    }
}

fn replicate_chunks_delta(
    mut chunks: Query<
        (
            &level::chunk::Chunk,
            &mut level::chunk::Terrain,
            &Replication,
        ),
        Changed<level::chunk::Terrain>,
    >,
    players: Query<&Connection>,
) {
    for (chunk_base, mut terrain, replication) in chunks.iter_mut() {
        let mut update_chunk_packets = vec![];
        let mut cleanup_sections = vec![];
        for (section_y, section) in terrain.0.iter().enumerate() {
            if section.block_state_updates.is_empty() {
                continue;
            }

            update_chunk_packets.push(s2c::GamePacket::SectionBlocksUpdate(
                s2c::game::SectionBlocksUpdatePacket {
                    section_pos: IVec3::new(chunk_base.0.x, section_y as i32, chunk_base.0.y),
                    suppress_light_updates: true,
                    position_and_states: section
                        .block_state_updates
                        .iter()
                        .map(|block_state_update| {
                            s2c::game::SectionBlocksUpdatePacketPositionAndState {
                                x: *block_state_update as u8 & 0xF,
                                y: (*block_state_update >> 8) as u8,
                                z: *block_state_update as u8 >> 4 & 0xF,
                                block_state: section.block_states.get(*block_state_update as u32)
                                    as i64,
                            }
                        })
                        .collect(),
                },
            ));

            // cleanup later to not trigger change detection unnecessarily
            cleanup_sections.push(section_y);
        }
        if update_chunk_packets.is_empty() {
            continue;
        }
        for section_y in cleanup_sections {
            terrain.0[section_y].block_state_updates.clear()
        }

        for &player in &replication.subscriber {
            if let Ok(connection) = players.get(player) {
                for chunk_update_packet in &update_chunk_packets {
                    connection.send(chunk_update_packet);
                }
            }
        }
    }
}

//=========================================================================== ACTOR REPLICATION ====

fn replicate_actors(
    registries: Res<registry::Registries>,
    mut chunks: Query<(&Children, &mut Replication), Changed<Children>>,
    actors: Query<(
        &actor::Actor,
        &actor::Position,
        &actor::Rotation,
        &actor::HeadRotation,
    )>,
    players: Query<&Connection>,
) {
    // early return
    if chunks.is_empty() {
        return;
    }

    // collect all actors for removal
    let mut remove_actors_by_player = HashMap::<Entity, HashSet<Entity>>::new();
    for (actors, replication) in chunks.iter() {
        for &actor in replication
            .replicated
            .iter()
            .filter(|actor| !actors.contains(actor))
        {
            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                match remove_actors_by_player.entry(player) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(HashSet::new()),
                }
                .insert(actor);
            }
        }
    }

    for (actors_, replication) in chunks.iter() {
        for &actor in actors_
            .iter()
            .filter(|actor| !replication.replicated.contains(actor))
        {
            let (actor_base, actor_position, actor_rotation, actor_head_rotation) =
                actors.get(actor).unwrap();
            let add_actor_packet = add_actor_packet(
                &registries,
                actor,
                actor_base,
                actor_position,
                actor_rotation,
                actor_head_rotation,
            );

            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // actor as been re-added
                if !remove_actors_by_player
                    .get_mut(&player)
                    .map_or(false, |actors| actors.remove(&actor))
                {
                    // connection: add actor, cause: spawn/subscribe
                    if let Ok(connection) = players.get(player) {
                        connection.send(&add_actor_packet);
                    }
                }
            }
        }
    }

    for (player, actors) in remove_actors_by_player {
        if !actors.is_empty() {
            // connection: remove actors, cause: despawn/unsubscribe
            if let Ok(connection) = players.get(player) {
                connection.send(&s2c::GamePacket::RemoveEntities {
                    entity_ids: actors
                        .into_iter()
                        .map(|actor| VarI32(actor.index() as i32))
                        .collect(),
                })
            }
        }
    }

    for (actors, mut replication) in chunks.iter_mut() {
        replication.replicated.clear();
        replication.replicated.extend(actors.iter())
    }
}

#[allow(clippy::type_complexity)]
fn replicate_actors_delta(
    chunks: Query<&Replication>,
    actors: Query<
        (Entity, &Parent, &actor::Position, &actor::Rotation),
        Or<(Changed<actor::Position>, Changed<actor::Rotation>)>,
    >,
    players: Query<&Connection>,
) {
    for (actor, chunk, actor_position, actor_rotation) in actors.iter() {
        if let Ok(replication) = chunks.get(chunk.get()) {
            let update_position_rotation_packet = s2c::GamePacket::TeleportEntity {
                id: VarI32(actor.index() as i32),
                pos: actor_position.0,
                pitch: Angle(actor_rotation.pitch),
                yaw: Angle(actor_rotation.yaw),
                on_ground: false,
            };
            let update_head_rotation_packet = s2c::GamePacket::RotateHead {
                entity_id: VarI32(actor.index() as i32),
                head_yaw: Angle(actor_rotation.yaw),
            };

            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: teleport entity, cause: movement
                if let Ok(connection) = players.get(player) {
                    connection.send(&update_position_rotation_packet);
                    connection.send(&update_head_rotation_packet);
                }
            }
        }
    }
}

//====================================================================================== HELPER ====

struct Packet(Vec<u8>);

impl Packet {
    fn decode<'a, T: Decode<'a>>(&'a self) -> tesseract_protocol::Result<T> {
        T::decode(&mut self.0.as_slice())
    }
}

async fn encode_and_send(socket: &mut Framed<TcpStream, Codec>, packet: &impl Encode) {
    let mut data = vec![];
    packet.encode(&mut data).unwrap();
    socket.send(&data).await.unwrap();
}

async fn next(socket: &mut Framed<TcpStream, Codec>) -> tesseract_protocol::Result<Packet> {
    socket
        .next()
        .await
        .ok_or(tesseract_protocol::Error::UnexpectedEnd)
        .flatten()
        .map(Packet)
}

fn add_chunk_packet<'a>(position: IVec2, terrain: &level::chunk::Terrain) -> s2c::GamePacket<'a> {
    let mut buffer = Vec::new();
    let mut sky_y_mask = 0i64;
    let mut sky_updates = Vec::new();
    for (i, section) in terrain.0.iter().enumerate() {
        4096i16.encode(&mut buffer).unwrap();
        section.block_states.encode(&mut buffer).unwrap();
        section.biomes.encode(&mut buffer).unwrap();

        sky_y_mask |= 1 << (i + 1);
        sky_updates.push(vec![0xFF; 2048])
    }

    s2c::GamePacket::LevelChunkWithLight {
        x: position.x,
        z: position.y,
        chunk_data: s2c::game::LevelChunkPacketData {
            heightmaps: Nbt(serde_value::Value::Map(Default::default())),
            buffer: buffer.clone(),
            block_entities_data: vec![],
        },
        light_data: s2c::game::LightUpdatePacketData {
            trust_edges: true,
            sky_y_mask: vec![sky_y_mask],
            block_y_mask: vec![0],
            empty_sky_y_mask: vec![0],
            empty_block_y_mask: vec![0],
            sky_updates,
            block_updates: vec![],
        },
    }
}

fn add_actor_packet<'a>(
    registries: &registry::Registries,
    actor: Entity,
    actor_base: &actor::Actor,
    position: &actor::Position,
    rotation: &actor::Rotation,
    head_rotation: &actor::HeadRotation,
) -> s2c::GamePacket<'a> {
    if actor_base.type_ == "minecraft:player" {
        s2c::GamePacket::AddPlayer {
            entity_id: VarI32(actor.index() as i32),
            player_id: actor_base.id,
            pos: position.0,
            pitch: Angle(rotation.pitch),
            yaw: Angle(rotation.yaw),
        }
    } else {
        s2c::GamePacket::AddEntity {
            id: VarI32(actor.index() as i32),
            uuid: actor_base.id,
            type_: VarI32(registries.id("minecraft:entity_type", &actor_base.type_) as i32),
            pos: position.0,
            pitch: Angle(rotation.pitch),
            yaw: Angle(rotation.yaw),
            head_yaw: Angle(head_rotation.head_yaw),
            data: VarI32(0),
            xa: 0,
            ya: 0,
            za: 0,
        }
    }
}
