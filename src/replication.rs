use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::{
    math::DVec3,
    prelude::*,
    utils::{HashSet, Uuid},
};
use futures::{SinkExt, StreamExt};
use num::BigInt;
use rsa::{pkcs8::EncodePublicKey, rand_core::OsRng, Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::{digest::Update, Digest, Sha1};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc};
use tokio_util::codec::Framed;

use mojang_session_api::apis::{configuration::Configuration, default_api::has_joined_server};
use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, s2c},
    types::{
        Angle, Biome, Component as ChatComponent, DamageType, GameType, Intention, Json, Nbt,
        Registries, Registry, RegistryEntry, Status, StatusPlayers, StatusVersion, VarI32,
    },
    Decode, Encode,
};

use crate::{actor, actor::ActorBundle, level, registry::DataRegistry};

pub struct ReplicationPlugin {
    address: SocketAddr,

    compression: Compression,
    compression_threshold: Option<u16>,
}

impl Default for ReplicationPlugin {
    fn default() -> Self {
        Self {
            address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565).into(),

            compression: Default::default(),
            compression_threshold: Some(256),
        }
    }
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        let address = self.address;

        let compression = self.compression;
        let compression_threshold = self.compression_threshold;

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
            .add_systems(First, spawn_player)
            .add_systems(PreUpdate, load_players)
            .add_systems(PreUpdate, update_players)
            .add_systems(PostUpdate, subscribe_and_replicate_initial)
            .add_systems(PostUpdate, replicate_chunks_late)
            .add_systems(PostUpdate, replicate_actors)
            .add_systems(PostUpdate, replicate_actors_movement);
    }
}

#[derive(Component)]
struct Connection {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Connection {
    fn send(&self, packet: s2c::GamePacket) {
        let mut data = vec![];
        packet.encode(&mut data).unwrap();
        self.tx.send(data).unwrap();
    }
}

async fn handle_new_connection(
    socket: TcpStream,
    private_key: RsaPrivateKey,
    compression: Compression,
    compression_threshold: Option<u16>,
    new_connection_tx: mpsc::UnboundedSender<Connection>,
) {
    let mut framed_socket = Framed::new(socket, Codec::default());

    match next(&mut framed_socket).await.decode() {
        c2s::HandshakePacket::Intention { intention, .. } => match intention {
            Intention::Status => {
                match next(&mut framed_socket).await.decode() {
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
                    _ => unimplemented!(),
                }

                match next(&mut framed_socket).await.decode() {
                    c2s::StatusPacket::PingRequest { time } => {
                        encode_and_send(
                            &mut framed_socket,
                            &s2c::StatusPacket::PongResponse { time },
                        )
                        .await;
                    }
                    _ => unimplemented!(),
                };
            }
            Intention::Login => {
                let name = match next(&mut framed_socket).await.decode() {
                    c2s::LoginPacket::Hello { name, .. } => name,
                    _ => unimplemented!(),
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
                let key = match next(&mut framed_socket).await.decode() {
                    c2s::LoginPacket::Key { key, nonce } => {
                        private_key
                            .decrypt(Pkcs1v15Encrypt::default(), &nonce)
                            .unwrap();
                        private_key
                            .decrypt(Pkcs1v15Encrypt::default(), &key)
                            .unwrap()
                    }
                    _ => unimplemented!(),
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

                encode_and_send(&mut framed_socket, &s2c::LoginPacket::GameProfile(user)).await;

                let (rx_packet_tx, rx_packet_rx) = mpsc::unbounded_channel();
                let (tx_packet_tx, mut tx_packet_rx) = mpsc::unbounded_channel();
                if new_connection_tx
                    .send(Connection {
                        rx: rx_packet_rx,
                        tx: tx_packet_tx,
                    })
                    .is_ok()
                {}

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            packet = framed_socket.next() => {
                                if let Some(packet) = packet {
                                    rx_packet_tx.send(packet.unwrap()).unwrap();
                                } else {
                                    break;
                                }
                            }
                            packet = tx_packet_rx.recv() => {
                                if let Some(packet) = packet {
                                    framed_socket.send(&packet).await.unwrap();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    framed_socket.close().await.unwrap();
                    tx_packet_rx.close()
                });
            }
            _ => unimplemented!(),
        },
    }
}

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_player(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        commands.spawn(connection);
    }
}

fn load_players(
    biome_registry: Res<DataRegistry<Biome>>,
    damage_type_registry: Res<DataRegistry<DamageType>>,
    mut commands: Commands,
    levels: Query<(Entity, &level::Level)>,
    players: Query<(Entity, &Connection), Added<Connection>>,
) {
    for (player, connection) in players.iter() {
        let (level, level_data) = levels.single();
        commands
            .entity(player)
            .insert((
                ActorBundle {
                    position: actor::Position(DVec3::new(0.0, 0.0, 0.0)),
                    rotation: actor::Rotation {
                        pitch: 0.0,
                        yaw: 0.0,
                    },
                    head_rotation: actor::HeadRotation { head_yaw: 0.0 },
                },
                SubscriptionDistance::default(),
                Subscriptions::default(),
            ))
            .set_parent(level);

        connection.send(s2c::GamePacket::Login {
            player_id: player.index() as i32,
            hardcore: false,
            game_type: GameType::Creative,
            previous_game_type: 0,
            levels: vec![level_data.name.clone()],
            registry_holder: Nbt(Registries {
                dimension_type: Registry {
                    type_: "minecraft:dimension_type".to_string(),
                    value: vec![RegistryEntry {
                        name: level_data.name.clone(),
                        id: 0,
                        element: level_data.dimension.clone(),
                    }],
                },
                biome: biome_registry.registry().clone(),
                chat_type: Registry {
                    type_: "minecraft:chat_type".to_string(),
                    value: vec![],
                },
                damage_type: damage_type_registry.registry().clone(),
            }),
            dimension_type: level_data.name.clone(),
            dimension: level_data.name.clone(),
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
        connection.send(s2c::game::GamePacket::SetDefaultSpawnPosition {
            pos: IVec3::new(0, 200, 0),
            yaw: 0.0,
        });
        connection.send(s2c::game::GamePacket::PlayerPosition {
            pos: DVec3::new(0.0, 150.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
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
        &mut actor::Position,
        &mut actor::Rotation,
        &mut SubscriptionDistance,
    )>,
) {
    for (player, mut connection, mut position, mut rotation, mut subscription_distance) in
        players.iter_mut()
    {
        if connection.tx.is_closed() {
            commands.entity(player).remove::<Connection>();
        } else {
            while let Ok(packet) = connection.rx.try_recv() {
                match Packet(packet).decode() {
                    c2s::GamePacket::ClientInformation { view_distance, .. } => {
                        subscription_distance.0 = view_distance as u8 + 3;
                        connection.send(s2c::GamePacket::SetChunkCacheRadius {
                            radius: VarI32(view_distance as i32),
                        });
                    }
                    c2s::GamePacket::MovePlayerPos { x, y, z, .. } => {
                        position.0.x = x;
                        position.0.y = y;
                        position.0.z = z;
                    }
                    c2s::GamePacket::MovePlayerPosRot {
                        x,
                        y,
                        z,
                        pitch,
                        yaw,
                        ..
                    } => {
                        position.0.x = x;
                        position.0.y = y;
                        position.0.z = z;
                        rotation.pitch = pitch;
                        rotation.yaw = yaw;
                    }
                    c2s::GamePacket::MovePlayerRot { pitch, yaw, .. } => {
                        rotation.pitch = pitch;
                        rotation.yaw = yaw;
                    }
                    _ => {}
                }
            }
        }
    }
}

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

#[allow(clippy::type_complexity)]
fn subscribe_and_replicate_initial(
    mut commands: Commands,
    mut levels: Query<&mut level::chunk::LookupTable>,
    chunk_positions: Query<(&level::chunk::Position, &Parent)>,
    mut chunks: Query<(Option<&level::chunk::Terrain>, &mut Replication)>,
    actors: Query<(
        Entity,
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
        if let Ok((chunk_position, level)) = chunk_positions.get(chunk.get()) {
            let x = chunk_position.0.x;
            let z = chunk_position.0.y;
            connection.send(s2c::GamePacket::SetChunkCacheCenter {
                x: VarI32(chunk_position.0.x),
                z: VarI32(chunk_position.0.y),
            });

            let mut acquire_chunks = Vec::new();
            let mut target_subscriptions = HashSet::new();
            if !actual_subscriptions.0.contains(&chunk_position.0) {
                acquire_chunks.push(chunk_position.0);
            }
            target_subscriptions.insert(IVec2::new(x, z));

            // square radius
            let subscription_distance = subscription_distance.0 as i32;
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
                    let (_, mut replication) = chunks.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);

                    // connection: remove chunk and entities, cause: unsubscribe
                    connection.send(s2c::GamePacket::RemoveEntities {
                        entity_ids: replication
                            .replicated
                            .iter()
                            .map(|actor| VarI32(actor.index() as i32))
                            .collect(),
                    });
                    connection.send(s2c::GamePacket::ForgetLevelChunk {
                        x: chunk_position.x,
                        z: chunk_position.y,
                    });
                }
            }

            // acquire chunks
            for chunk_position in acquire_chunks {
                if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                    if let Ok((terrain, mut replication)) = chunks.get_mut(chunk) {
                        replication.subscriber.insert(player);

                        if let Some(terrain) = terrain {
                            // connection: add chunk and entities, cause: subscribe
                            for (actor, actor_position, actor_rotation, actor_head_rotation) in
                                actors.iter_many(&replication.replicated)
                            {
                                // except owner
                                if actor == player {
                                    continue;
                                }

                                connection.send(s2c::GamePacket::AddEntity {
                                    id: VarI32(actor.index() as i32),
                                    uuid: Uuid::new_v4(),
                                    type_: VarI32(72),
                                    pos: actor_position.0,
                                    pitch: Angle(actor_rotation.pitch),
                                    yaw: Angle(actor_rotation.yaw),
                                    head_yaw: Angle(actor_head_rotation.head_yaw),
                                    data: VarI32(0),
                                    xa: 0,
                                    ya: 0,
                                    za: 0,
                                });
                            }

                            let mut buffer = Vec::new();
                            let mut sky_y_mask = 0i64;
                            let mut sky_updates = Vec::new();
                            for (i, section) in terrain.sections.iter().enumerate() {
                                4096i16.encode(&mut buffer).unwrap();
                                section.encode(&mut buffer).unwrap();
                                0u8.encode(&mut buffer).unwrap();
                                VarI32(0).encode(&mut buffer).unwrap();
                                VarI32(0).encode(&mut buffer).unwrap();

                                sky_y_mask |= 1 << (i + 1);
                                sky_updates.push(vec![0xFF; 2048])
                            }

                            connection.send(s2c::GamePacket::LevelChunkWithLight {
                                x: chunk_position.x,
                                z: chunk_position.y,
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
                            });
                        }
                    }
                } else {
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
        (
            &level::chunk::Terrain,
            &level::chunk::Position,
            &Replication,
        ),
        Added<level::chunk::Terrain>,
    >,
    players: Query<&Connection>,
) {
    // early return
    for (terrain, chunk_position, replication) in chunks.iter() {
        let mut buffer = Vec::new();
        let mut sky_y_mask = 0i64;
        let mut sky_updates = Vec::new();
        for (i, section) in terrain.sections.iter().enumerate() {
            4096i16.encode(&mut buffer).unwrap();
            section.encode(&mut buffer).unwrap();
            0u8.encode(&mut buffer).unwrap();
            VarI32(0).encode(&mut buffer).unwrap();
            VarI32(0).encode(&mut buffer).unwrap();

            sky_y_mask |= 1 << (i + 1);
            sky_updates.push(vec![0xFF; 2048])
        }

        for &player in &replication.subscriber {
            // connection: add chunk, cause: subscribe (late)
            if let Ok(connection) = players.get(player) {
                connection.send(s2c::GamePacket::LevelChunkWithLight {
                    x: chunk_position.0.x,
                    z: chunk_position.0.y,
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
                        sky_updates: sky_updates.clone(),
                        block_updates: vec![],
                    },
                });
            } else {
                warn!("Replication requires a connection")
            }
        }
    }
}

fn replicate_actors(
    mut chunks: Query<(&Children, &mut Replication), Changed<Children>>,
    actors: Query<(&actor::Position, &actor::Rotation, &actor::HeadRotation)>,
    players: Query<&Connection>,
) {
    // early return
    if chunks.is_empty() {
        return;
    }

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

                // connection: add entity, cause: despawn
                if let Ok(connection) = players.get(player) {
                    connection.send(s2c::GamePacket::RemoveEntities {
                        entity_ids: vec![VarI32(actor.index() as i32)],
                    })
                }
            }
        }
    }

    for (actors_, replication) in chunks.iter() {
        for &actor in actors_
            .iter()
            .filter(|actor| !replication.replicated.contains(actor))
        {
            let (actor_position, actor_rotation, actor_head_rotation) = actors.get(actor).unwrap();

            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: add entity, cause: spawn
                if let Ok(connection) = players.get(player) {
                    connection.send(s2c::GamePacket::AddEntity {
                        id: VarI32(actor.index() as i32),
                        uuid: Uuid::new_v4(),
                        type_: VarI32(72),
                        pos: actor_position.0,
                        pitch: Angle(actor_rotation.pitch),
                        yaw: Angle(actor_rotation.yaw),
                        head_yaw: Angle(actor_head_rotation.head_yaw),
                        data: VarI32(0),
                        xa: 0,
                        ya: 0,
                        za: 0,
                    });
                }
            }
        }
    }

    for (actors, mut replication) in chunks.iter_mut() {
        replication.replicated.clear();
        replication.replicated.extend(actors.iter())
    }
}

#[allow(clippy::type_complexity)]
fn replicate_actors_movement(
    chunks: Query<&Replication>,
    actors: Query<
        (Entity, &Parent, &actor::Position, &actor::Rotation),
        Or<(Changed<actor::Position>, Changed<actor::Rotation>)>,
    >,
    players: Query<&Connection>,
) {
    for (actor, chunk, actor_position, actor_rotation) in actors.iter() {
        if let Ok(replication) = chunks.get(chunk.get()) {
            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: teleport entity, cause: movement
                if let Ok(connection) = players.get(player) {
                    connection.send(s2c::GamePacket::TeleportEntity {
                        id: VarI32(actor.index() as i32),
                        pos: actor_position.0,
                        pitch: Angle(actor_rotation.pitch),
                        yaw: Angle(actor_rotation.yaw),
                        on_ground: false,
                    });
                    connection.send(s2c::GamePacket::RotateHead {
                        entity_id: VarI32(actor.index() as i32),
                        head_yaw: Angle(actor_rotation.yaw),
                    });
                }
            }
        }
    }
}

struct Packet(Vec<u8>);

impl Packet {
    fn decode<'a, T: Decode<'a>>(&'a self) -> T {
        T::decode(&mut self.0.as_slice()).unwrap()
    }
}

async fn encode_and_send(socket: &mut Framed<TcpStream, Codec>, packet: &impl Encode) {
    let mut data = vec![];
    packet.encode(&mut data).unwrap();
    socket.send(&data).await.unwrap();
}

async fn next(socket: &mut Framed<TcpStream, Codec>) -> Packet {
    Packet(socket.next().await.unwrap().unwrap())
}
