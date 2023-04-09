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
pub use tesseract_base::replication::*;
use tesseract_base::{
    actor,
    hierarchy::{EntityCommandsExt, IndexedChildren, ParentWithIndex},
    item, level,
};
use tesseract_java_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, c2s::game::PlayerActionPacketAction, s2c},
    types::{
        Biome, Component as ChatComponent, DamageType, DimensionType, GameType, Intention, Json,
        Nbt, PalettedContainer, Registries, Registry, Status, StatusPlayers, StatusVersion, VarI32,
    },
    Decode, Encode,
};

use crate::{block, registry};

/// Support for Minecraft: Java Edition replication
pub struct ReplicationPlugin {
    pub address: SocketAddr,

    pub compression: Compression,
    pub compression_threshold: Option<u16>,
}

impl Default for ReplicationPlugin {
    fn default() -> Self {
        Self {
            address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565).into(),

            compression: Compression::default(),
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
                            if let Ok((socket, address)) = listener.accept().await {
                                tokio::spawn(handle_new_connection(
                                    socket,
                                    address,
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
            .add_systems(First, (spawn_player, update_players).before(UpdateFlush))
            .add_systems(First, apply_system_buffers.in_set(UpdateFlush))
            .add_systems(PostUpdate, render_chunks)
            .add_systems(
                Last,
                (
                    replicate_initial,
                    subscribe_and_replicate_chunks,
                    cleanup_chunks,
                    replicate_chunks_late,
                    replicate_chunks_delta,
                    replicate_actors,
                    replicate_actors_delta,
                ),
            );
    }
}

#[derive(Component)]
pub struct Connection {
    address: SocketAddr,
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
    address: SocketAddr,
    private_key: RsaPrivateKey,
    compression: Compression,
    compression_threshold: Option<u16>,
    new_connection_tx: mpsc::UnboundedSender<Connection>,
) -> tesseract_java_protocol::Result<()> {
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
                    _ => return Err(tesseract_java_protocol::Error::Unexpected),
                }

                match next(&mut framed_socket).await?.decode()? {
                    c2s::StatusPacket::PingRequest { time } => {
                        encode_and_send(
                            &mut framed_socket,
                            &s2c::StatusPacket::PongResponse { time },
                        )
                        .await;
                    }
                    _ => return Err(tesseract_java_protocol::Error::Unexpected),
                };
            }
            Intention::Login => {
                let name = match next(&mut framed_socket).await?.decode()? {
                    c2s::LoginPacket::Hello { name, .. } => name,
                    _ => return Err(tesseract_java_protocol::Error::Unexpected),
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
                    _ => return Err(tesseract_java_protocol::Error::Unexpected),
                };
                framed_socket.codec_mut().enable_encryption(&key);

                if let Ok(user) = has_joined_server(
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
                {
                    if let Some(compression_threshold) = compression_threshold {
                        encode_and_send(
                            &mut framed_socket,
                            &s2c::LoginPacket::LoginCompression {
                                compression_threshold: compression_threshold as i32,
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
                        address,
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
                } else {
                    return Err(tesseract_java_protocol::Error::Unexpected);
                }
            }
            _ => return Err(tesseract_java_protocol::Error::Unexpected),
        },
    }

    Ok(())
}

//====================================================================================== UPDATE ====

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_player(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        info!(
            "Player {} (UUID: {}) connected",
            connection.user.name, connection.user.id
        );

        commands.spawn((connection, Subscription::default()));
    }
}

fn update_players(
    mut commands: Commands,

    mut for_players: Query<(
        Entity,
        &mut Connection,
        &mut Subscription,
        &mut actor::Position,
        &mut actor::Rotation,
        &mut actor::player::Interaction,
    )>,
) {
    for (player, mut connection, mut subscription, mut position, mut rotation, mut interaction) in
        for_players.iter_mut()
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

            commands.entity(player).remove::<Connection>();
        } else {
            while let Ok(packet) = connection.rx.try_recv() {
                match Packet(packet).decode().unwrap() {
                    c2s::GamePacket::ClientInformation { view_distance, .. } => {
                        connection.send(&s2c::GamePacket::SetChunkCacheRadius {
                            radius: view_distance as i32,
                        });

                        let new_subscription_radius = view_distance as u8 + 2;
                        if subscription.radius != new_subscription_radius {
                            subscription.radius = new_subscription_radius;
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
                    c2s::GamePacket::PlayerAction {
                        action,
                        pos,
                        sequence,
                        ..
                    } => {
                        connection.send(&s2c::GamePacket::BlockChangedAck { sequence });

                        match action {
                            PlayerActionPacketAction::StartDestroyBlock => {
                                *interaction = actor::player::Interaction::BreakBlock(pos);
                            }
                            PlayerActionPacketAction::AbortDestroyBlock => {
                                *interaction = actor::player::Interaction::None;
                            }
                            _ => {}
                        }
                    }
                    c2s::GamePacket::SetCreativeModeSlot {
                        slot_num,
                        item_stack,
                    } => {
                        let slot = match slot_num {
                            5 => item::Slot::Head,
                            6 => item::Slot::Torso,
                            7 => item::Slot::Legs,
                            8 => item::Slot::Feet,
                            9..=35 => item::Slot::Inventory((slot_num - 9) as u8),
                            36..=44 => item::Slot::Hotbar((slot_num - 36) as u8),
                            _ => todo!(),
                        };

                        let item_instance = item_stack.map(|item_stack| {
                            commands
                                .spawn(item::Instance {
                                    item: Entity::from_raw(0),
                                    count: item_stack.count as u8,
                                })
                                .id()
                        });
                        commands
                            .entity(player)
                            .set_indexed_child(slot, item_instance);
                    }
                    c2s::GamePacket::UseItemOn {
                        block_pos,
                        direction,
                        sequence,
                        ..
                    } => {
                        connection.send(&s2c::GamePacket::BlockChangedAck { sequence });

                        *interaction = actor::player::Interaction::UseItemOn(block_pos, direction);
                    }
                    c2s::GamePacket::UseItem { sequence, .. } => {
                        connection.send(&s2c::GamePacket::BlockChangedAck { sequence });
                    }
                    _ => {}
                }
            }
        }
    }
}

//================================================================================= REPLICATION ====

fn replicate_initial(
    dimension_type_registry: Res<registry::DataRegistry<DimensionType>>,
    biome_registry: Res<registry::DataRegistry<Biome>>,
    damage_type_registry: Res<registry::DataRegistry<DamageType>>,

    level_access: Query<(&level::Base, &level::AgeAndTime)>,
    chunk_access: Query<&ParentWithIndex<IVec2>>,

    for_players: Query<
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
    for (player, connection, actor_position, actor_rotation, chunk) in for_players.iter() {
        let (level, level_age_and_time) = level_access
            .get(chunk_access.get(chunk.get()).unwrap().parent)
            .unwrap();
        connection.send(&s2c::GamePacket::Login {
            player_id: player.index() as i32,
            hardcore: false,
            game_type: GameType::Creative,
            previous_game_type: 0,
            levels: vec![level.name().into()],
            registry_holder: Nbt(Registries {
                dimension_type: Cow::Borrowed(dimension_type_registry.registry()),
                biome: Cow::Borrowed(biome_registry.registry()),
                chat_type: Cow::Owned(Registry {
                    type_: "minecraft:chat_type".into(),
                    value: vec![],
                }),
                damage_type: Cow::Borrowed(damage_type_registry.registry()),
            }),
            dimension_type: level.dimension_type().into(),
            dimension: level.name().into(),
            seed: 0,
            max_players: 0,
            chunk_radius: 0,
            simulation_distance: 0,
            reduced_debug_info: false,
            show_death_screen: false,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
        });
        connection.send(&s2c::GamePacket::SetDefaultSpawnPosition {
            pos: Default::default(),
            yaw: Default::default(),
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
            id: 0,
        });
    }
}

//=========================================================================== CHUNK REPLICATION ====

struct RenderCache {
    blocks: HashMap<u32, RenderBlock>,
}

impl FromWorld for RenderCache {
    fn from_world(world: &mut World) -> Self {
        let mut for_blocks = world.query::<(Entity, &block::Name, Option<&block::Auto>)>();
        let blocks_report = world.resource::<registry::BlocksReport>();

        let mut blocks = HashMap::new();
        for (block_id, block_name, block_auto) in for_blocks.iter(world) {
            let block_report = &blocks_report.0[&block_name.name];
            blocks.insert(
                block_id.index(),
                match block_auto {
                    Some(block::Auto::Snowy) => {
                        let block_state_reports = block_report
                            .states
                            .iter()
                            .filter(|block_state_report| {
                                block_state_report.properties.iter().all(
                                    |(property_key, property_value)| {
                                        block_name.properties.get(property_key).map_or(
                                            true,
                                            |other_property_value| {
                                                property_value == other_property_value
                                            },
                                        )
                                    },
                                )
                            })
                            .collect::<Vec<_>>();

                        RenderBlock::Snowy {
                            false_: block_state_reports
                                .iter()
                                .find(|block_state_report| {
                                    block_state_report.properties["snowy"] == "false"
                                })
                                .unwrap()
                                .id,
                            true_: block_state_reports
                                .iter()
                                .find(|block_state_report| {
                                    block_state_report.properties["snowy"] == "true"
                                })
                                .unwrap()
                                .id,
                        }
                    }
                    _ => RenderBlock::Default(
                        if block_name.properties.is_empty() {
                            block_report
                                .states
                                .iter()
                                .find(|block_state_report| block_state_report.default)
                        } else {
                            block_report.states.iter().find(|block_state_report| {
                                block_state_report.properties == block_name.properties
                            })
                        }
                        .unwrap()
                        .id,
                    ),
                },
            );
        }
        Self { blocks }
    }
}

pub enum RenderBlock {
    Default(u32),
    Snowy { false_: u32, true_: u32 },
}

#[derive(Component)]
struct RenderedChunk {
    sections: Vec<RenderedChunkSection>,
}

struct RenderedChunkSection {
    block_states: PalettedContainer<{ 16 * 16 * 16 }, 4, 8, 15>,
    biomes: PalettedContainer<{ 4 * 4 * 4 }, 3, 3, 6>,
}

fn render_chunks(
    mut commands: Commands,
    render_cache: Local<RenderCache>,

    mut for_chunks: Query<
        (Entity, &level::chunk::Data, Option<&mut RenderedChunk>),
        Changed<level::chunk::Data>,
    >,
) {
    for (chunk, chunk_data, rendered_chunk) in for_chunks.iter_mut() {
        if let Some(mut rendered_chunk) = rendered_chunk {
            for (section_y, section) in chunk_data.sections.iter().enumerate() {
                let rendered_section = &mut rendered_chunk.sections[section_y];
                for &block_state_change in &section.block_state_changes {
                    let block = section.block_states.get(block_state_change as u32);
                    rendered_section.block_states.get_and_set(
                        block_state_change as u32,
                        match render_cache.blocks.get(&block).unwrap() {
                            RenderBlock::Default(value) => *value,
                            RenderBlock::Snowy { false_, .. } => *false_,
                        },
                    );
                }
            }
        } else {
            commands.entity(chunk).insert(RenderedChunk {
                sections: chunk_data
                    .sections
                    .iter()
                    .map(|section| RenderedChunkSection {
                        block_states: match &section.block_states {
                            PalettedContainer::Single(block) => PalettedContainer::Single(
                                match render_cache.blocks.get(block).unwrap() {
                                    RenderBlock::Default(value) => *value,
                                    RenderBlock::Snowy { false_, .. } => *false_,
                                },
                            ),
                            PalettedContainer::Indirect { palette, storage } => {
                                PalettedContainer::Indirect {
                                    palette: palette
                                        .iter()
                                        .map(|block| {
                                            match render_cache.blocks.get(block).unwrap() {
                                                RenderBlock::Default(value) => *value,
                                                RenderBlock::Snowy { false_, .. } => *false_,
                                            }
                                        })
                                        .collect(),
                                    storage: storage.clone(),
                                }
                            }
                            PalettedContainer::Direct(_) => todo!(),
                        },
                        biomes: section.biomes.clone(),
                    })
                    .collect(),
            });
        }
    }
}

fn cleanup_chunks(
    mut commands: Commands,

    level_access: Query<&IndexedChildren<IVec2>>,
    chunk_access: Query<&ParentWithIndex<IVec2>>,
    mut subscription_chunk_access: Query<&mut Replication>,

    for_players: Query<(Entity, &Parent, &Subscription), Without<Connection>>,
) {
    for (player, chunk, subscription) in for_players.iter() {
        if let Ok(indexed_chunk) = chunk_access.get(chunk.get()) {
            commands.entity(player).remove::<Subscription>();

            let indexed_chunks = level_access.get(indexed_chunk.parent).unwrap();
            for chunk_position in ConcentricSquareIterator::new(
                subscription.last_center,
                subscription.last_radius as i32,
            ) {
                if let Some(&chunk) = indexed_chunks.0.get(&chunk_position) {
                    trace!("Release chunk: {:?}", chunk_position);

                    let mut replication = subscription_chunk_access.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);
                } else {
                    trace!("Release chunk: {:?} (not spawned)", chunk_position);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn subscribe_and_replicate_chunks(
    mut commands: Commands,

    level_access: Query<&IndexedChildren<IVec2>>,
    chunk_access: Query<&ParentWithIndex<IVec2>>,
    mut subscription_chunk_access: Query<(Option<&RenderedChunk>, &mut Replication)>,
    actor_access: Query<(Entity, &actor::Base, &actor::Position, &actor::Rotation)>,

    mut for_players: Query<
        (Entity, &Parent, &Connection, &mut Subscription),
        Or<(Changed<Parent>, Changed<Subscription>)>,
    >,
) {
    for (player, chunk, connection, mut subscription) in for_players.iter_mut() {
        if let Ok(indexed_chunk) = chunk_access.get(chunk.get()) {
            let indexed_chunks = level_access.get(indexed_chunk.parent).unwrap();

            let center = indexed_chunk.index;
            connection.send(&s2c::GamePacket::SetChunkCacheCenter {
                x: center.x,
                z: center.y,
            });

            let radius = subscription.radius as i32;
            let last_center = subscription.last_center;
            let last_radius = subscription.last_radius as i32;

            // release chunks
            for chunk_position in
                ConcentricSquareIterator::new(last_center, last_radius).filter(|position| {
                    position.x >= (center.x + radius)
                        || position.x <= (center.x - radius)
                        || position.y >= (center.y + radius)
                        || position.y <= (center.y - radius)
                })
            {
                if let Some(&chunk) = indexed_chunks.0.get(&chunk_position) {
                    trace!("Release chunk: {:?}", chunk_position);

                    let (_, mut replication) = subscription_chunk_access.get_mut(chunk).unwrap();
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
            for chunk_position in ConcentricSquareIterator::new(center, subscription.radius as i32)
                .filter(|position| {
                    position.x >= (last_center.x + last_radius)
                        || position.x <= (last_center.x - last_radius)
                        || position.y >= (last_center.y + last_radius)
                        || position.y <= (last_center.y - last_radius)
                })
            {
                if let Some(&chunk) = indexed_chunks.0.get(&chunk_position) {
                    if let Ok((rendered_chunk, mut replication)) =
                        subscription_chunk_access.get_mut(chunk)
                    {
                        trace!("Acquire chunk: {:?}", chunk_position);

                        replication.subscriber.insert(player);

                        if let Some(rendered_chunk) = rendered_chunk {
                            // connection: add chunk and actors, cause: subscribe
                            connection.send(&add_chunk_packet(chunk_position, rendered_chunk));
                            for (actor, actor_base, actor_position, actor_rotation) in
                                actor_access.iter_many(&replication.replicated)
                            {
                                // except owner
                                if actor == player {
                                    continue;
                                }

                                connection.send(&add_actor_packet(
                                    actor,
                                    actor_base,
                                    actor_position,
                                    actor_rotation,
                                ));
                            }
                        }
                    } else {
                        trace!("Acquire chunk: {:?} (not initialized)", chunk_position);
                    }
                } else {
                    trace!("Acquire chunk: {:?} (not spawned)", chunk_position);

                    let chunk = commands
                        .spawn(level::chunk::ChunkBundle {
                            base: level::chunk::Base,
                            update_queue: Default::default(),
                            replication: Replication {
                                subscriber: HashSet::from([player]),
                                replicated: vec![],
                            },
                        })
                        .id();
                    commands
                        .entity(indexed_chunk.parent)
                        .set_indexed_child(chunk_position, Some(chunk));
                }
            }

            subscription.last_center = center;
            subscription.last_radius = radius as u8;
        }
    }
}

fn replicate_chunks_late(
    player_access: Query<&Connection>,

    for_chunks: Query<
        (&ParentWithIndex<IVec2>, &RenderedChunk, &Replication),
        Added<RenderedChunk>,
    >,
) {
    for (indexed_chunk, rendered_chunk, replication) in for_chunks.iter() {
        let add_chunk_packet = add_chunk_packet(indexed_chunk.index, rendered_chunk);
        for &player in &replication.subscriber {
            // connection: add chunk, cause: subscribe (late)
            if let Ok(connection) = player_access.get(player) {
                connection.send(&add_chunk_packet);
            }
        }
    }
}

fn replicate_chunks_delta(
    player_access: Query<&Connection>,

    mut for_chunks: Query<
        (
            &ParentWithIndex<IVec2>,
            &RenderedChunk,
            &Replication,
            &mut level::chunk::Data,
        ),
        Changed<RenderedChunk>,
    >,
) {
    for (chunk_base, rendered_chunk, replication, mut chunk_data) in for_chunks.iter_mut() {
        let mut update_chunk_packets = vec![];
        {
            let y_offset = chunk_data.y_offset as i32;
            for (section_y, section) in chunk_data.sections.iter_mut().enumerate() {
                if section.block_state_changes.is_empty() {
                    continue;
                }

                if section.block_state_changes.len() == 1 {
                    let block_state_change = *section.block_state_changes.first().unwrap();
                    let chunk_position = chunk_base.index;
                    update_chunk_packets.push(s2c::GamePacket::BlockUpdate {
                        pos: IVec3::new(
                            chunk_position.x * 16 + (block_state_change as u8 & 0xF) as i32,
                            (section_y as i32 - y_offset) * 16 + (block_state_change >> 8) as i32,
                            chunk_position.y * 16 + (block_state_change as u8 >> 4 & 0xF) as i32,
                        ),
                        block_state: rendered_chunk.sections[section_y]
                            .block_states
                            .get(block_state_change as u32)
                            as i32,
                    })
                } else {
                    let chunk_position = chunk_base.index;
                    let rendered_section = &rendered_chunk.sections[section_y];
                    update_chunk_packets.push(s2c::GamePacket::SectionBlocksUpdate(
                        s2c::game::SectionBlocksUpdatePacket {
                            section_pos: IVec3::new(
                                chunk_position.x,
                                section_y as i32 - y_offset,
                                chunk_position.y,
                            ),
                            suppress_light_updates: true,
                            position_and_states: section
                                .block_state_changes
                                .iter()
                                .map(|&block_state_change| {
                                    s2c::game::SectionBlocksUpdatePacketPositionAndState {
                                        x: block_state_change as u8 & 0xF,
                                        y: (block_state_change >> 8) as u8,
                                        z: block_state_change as u8 >> 4 & 0xF,
                                        block_state: rendered_section
                                            .block_states
                                            .get(block_state_change as u32)
                                            as i64,
                                    }
                                })
                                .collect(),
                        },
                    ));
                }

                section.block_state_changes.clear();
            }
        }
        if update_chunk_packets.is_empty() {
            continue;
        }

        for &player in &replication.subscriber {
            if let Ok(connection) = player_access.get(player) {
                for chunk_update_packet in &update_chunk_packets {
                    connection.send(chunk_update_packet);
                }
            }
        }
    }
}

//=========================================================================== ACTOR REPLICATION ====

fn replicate_actors(
    actor_access: Query<(&actor::Base, &actor::Position, &actor::Rotation)>,
    player_access: Query<&Connection>,

    mut for_chunks: Query<(&Children, &mut Replication), Changed<Children>>,
) {
    // early return
    if for_chunks.is_empty() {
        return;
    }

    // collect all actors for removal
    let mut remove_actors_by_player = HashMap::<Entity, HashSet<Entity>>::new();
    for (actors, replication) in for_chunks.iter() {
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

    for (actors_, replication) in for_chunks.iter() {
        for &actor in actors_
            .iter()
            .filter(|actor| !replication.replicated.contains(actor))
        {
            let (actor_base, actor_position, actor_rotation) = actor_access.get(actor).unwrap();
            let add_actor_packet =
                add_actor_packet(actor, actor_base, actor_position, actor_rotation);

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
                    if let Ok(connection) = player_access.get(player) {
                        connection.send(&add_actor_packet);
                    }
                }
            }
        }
    }

    for (player, actors) in remove_actors_by_player {
        if !actors.is_empty() {
            // connection: remove actors, cause: despawn/unsubscribe
            if let Ok(connection) = player_access.get(player) {
                connection.send(&s2c::GamePacket::RemoveEntities {
                    entity_ids: actors
                        .into_iter()
                        .map(|actor| VarI32(actor.index() as i32))
                        .collect(),
                })
            }
        }
    }

    for (actors, mut replication) in for_chunks.iter_mut() {
        replication.replicated.clear();
        replication.replicated.extend(actors.iter())
    }
}

#[allow(clippy::type_complexity)]
fn replicate_actors_delta(
    chunks_access: Query<&Replication>,
    player_access: Query<&Connection>,

    for_actors: Query<
        (Entity, &Parent, Ref<actor::Position>, Ref<actor::Rotation>),
        Or<(Changed<actor::Position>, Changed<actor::Rotation>)>,
    >,
) {
    for (actor, chunk, actor_position, actor_rotation) in for_actors.iter() {
        if let Ok(replication) = chunks_access.get(chunk.get()) {
            let mut packets = vec![];
            if actor_position.is_changed() {
                /*if actor_rotation.is_changed() {
                    s2c::GamePacket::MoveEntityPosRot {
                        entity_id: actor.index() as i32,
                        xa: 0,
                        ya: 0,
                        za: 0,
                        yaw: actor_rotation.yaw,
                        pitch: actor_rotation.pitch,
                        on_ground: false,
                    };
                } else {
                    s2c::GamePacket::MoveEntityPos {
                        entity_id: actor.index() as i32,
                        xa: 0,
                        ya: 0,
                        za: 0,
                        on_ground: false,
                    };
                }*/
                packets.push(s2c::GamePacket::TeleportEntity {
                    id: actor.index() as i32,
                    pos: actor_position.0,
                    pitch: actor_rotation.pitch,
                    yaw: actor_rotation.yaw,
                    on_ground: false,
                });
            } else if actor_rotation.is_changed() {
                packets.push(s2c::GamePacket::MoveEntityRot {
                    entity_id: actor.index() as i32,
                    yaw: actor_rotation.yaw,
                    pitch: actor_rotation.pitch,
                    on_ground: false,
                });
                packets.push(s2c::GamePacket::RotateHead {
                    entity_id: actor.index() as i32,
                    head_yaw: actor_rotation.yaw,
                });
            }

            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                if let Ok(connection) = player_access.get(player) {
                    for packet in &packets {
                        connection.send(packet);
                    }
                }
            }
        }
    }
}

//====================================================================================== HELPER ====

struct Packet(Vec<u8>);

impl Packet {
    fn decode<'a, T: Decode<'a>>(&'a self) -> tesseract_java_protocol::Result<T> {
        T::decode(&mut self.0.as_slice())
    }
}

async fn encode_and_send(socket: &mut Framed<TcpStream, Codec>, packet: &impl Encode) {
    let mut data = vec![];
    packet.encode(&mut data).unwrap();
    socket.send(&data).await.unwrap();
}

async fn next(socket: &mut Framed<TcpStream, Codec>) -> tesseract_java_protocol::Result<Packet> {
    socket
        .next()
        .await
        .ok_or(tesseract_java_protocol::Error::UnexpectedEnd)
        .flatten()
        .map(Packet)
}

struct ConcentricSquareIterator {
    center_x: i32,
    center_z: i32,
    radius: i32,

    r: i32,
    n: i32,
    i: i32,
}

impl ConcentricSquareIterator {
    fn new(center: IVec2, radius: i32) -> Self {
        Self {
            center_x: center.x,
            center_z: center.y,
            radius,

            r: 0,
            n: 0,
            i: 0,
        }
    }
}

impl Iterator for ConcentricSquareIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.r >= self.radius {
            return None;
        }
        if self.n > self.r {
            self.r += 1;
            self.n = -self.r;
        } else if self.n == 0 && self.r == 0 {
            self.r = 1;
            self.n = -1;
            return Some(IVec2::new(self.center_x, self.center_z));
        }

        Some(match self.i {
            0 => {
                self.i = 1;

                IVec2::new(self.center_x + self.n, self.center_z - self.r)
            }
            1 => {
                self.i = 2;

                IVec2::new(self.center_x + self.r, self.center_z + self.n)
            }
            2 => {
                self.i = 3;

                IVec2::new(self.center_x - self.n, self.center_z + self.r)
            }
            _ => {
                self.i = 0;
                self.n += 1;

                IVec2::new(self.center_x - self.r, self.center_z - self.n)
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some((self.radius * self.radius * 4 + 1) as usize))
    }
}

fn add_chunk_packet<'a>(position: IVec2, rendered_chunk: &RenderedChunk) -> s2c::GamePacket<'a> {
    let mut buffer = Vec::new();
    let mut sky_y_mask = 0i64;
    let mut sky_updates = Vec::new();
    for (i, rendered_section) in rendered_chunk.sections.iter().enumerate() {
        4096i16.encode(&mut buffer).unwrap();
        rendered_section.block_states.encode(&mut buffer).unwrap();
        rendered_section.biomes.encode(&mut buffer).unwrap();

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
    actor: Entity,
    actor_base: &actor::Base,
    position: &actor::Position,
    rotation: &actor::Rotation,
) -> s2c::GamePacket<'a> {
    s2c::GamePacket::AddPlayer {
        entity_id: actor.index() as i32,
        player_id: actor_base.id,
        pos: position.0,
        pitch: rotation.pitch,
        yaw: rotation.yaw,
    }
}
