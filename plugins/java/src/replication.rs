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
use tesseract_base::{actor, level};
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
            .add_systems(First, (spawn_player, update_players).before(UpdateFlush))
            .add_systems(First, apply_system_buffers.in_set(UpdateFlush))
            .add_systems(PostUpdate, condense_chunks)
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

    mut players: Query<(
        Entity,
        &mut Connection,
        &mut Subscription,
        &mut actor::Position,
        &mut actor::Rotation,
        &mut actor::player::Interaction,
        &mut actor::player::Inventory,
    )>,
) {
    for (
        player,
        mut connection,
        mut subscription,
        mut position,
        mut rotation,
        mut interaction,
        mut inventory,
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
                                *interaction = actor::player::Interaction::BlockBreak(pos);
                            }
                            PlayerActionPacketAction::AbortDestroyBlock => {
                                *interaction = actor::player::Interaction::None;
                            }
                            _ => {}
                        }
                    }
                    c2s::GamePacket::SetCarriedItem { slot } => {
                        inventory.selected_slot = slot as u8;
                    }
                    c2s::GamePacket::SetCreativeModeSlot {
                        slot_num,
                        item_stack,
                    } => {
                        /*inventory.content.insert(
                            slot_num as usize,
                            item_stack.map(|item_stack| {
                                Entity::from_raw(mappings.item_by_id[&(item_stack.item as u32)])
                            }),
                        );*/
                    }
                    c2s::GamePacket::UseItemOn {
                        block_pos,
                        direction,
                        sequence,
                        ..
                    } => {
                        connection.send(&s2c::GamePacket::BlockChangedAck { sequence });

                        *interaction =
                            actor::player::Interaction::BlockPlace(block_pos + direction.vector());
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

    levels: Query<(&level::Base, &level::AgeAndTime)>,
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
            levels: vec![level_base.name().into()],
            registry_holder: Nbt(Registries {
                dimension_type: Cow::Borrowed(dimension_type_registry.registry()),
                biome: Cow::Borrowed(biome_registry.registry()),
                chat_type: Cow::Owned(Registry {
                    type_: "minecraft:chat_type".into(),
                    value: vec![],
                }),
                damage_type: Cow::Borrowed(damage_type_registry.registry()),
            }),
            dimension_type: level_base.dimension_type().into(),
            dimension: level_base.name().into(),
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

struct CondenseCache(HashMap<u32, CondenseBlock>);

impl FromWorld for CondenseCache {
    fn from_world(world: &mut World) -> Self {
        let mut blocks_query = world.query::<(Entity, &block::Name, Option<&block::Auto>)>();
        let blocks_report = world.resource::<registry::BlocksReport>();

        let mut blocks = HashMap::new();
        for (block, block_name, block_auto) in blocks_query.iter(world) {
            let block_report = &blocks_report.0[&block_name.name];
            blocks.insert(
                block.index(),
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
                        CondenseBlock::Snowy {
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
                    _ => CondenseBlock::Default(
                        block_report
                            .states
                            .iter()
                            .find(|block_state_report| {
                                block_state_report.properties == block_name.properties
                            })
                            .unwrap()
                            .id,
                    ),
                },
            );
        }
        Self(blocks)
    }
}

pub enum CondenseBlock {
    Default(u32),
    Snowy { false_: u32, true_: u32 },
}

#[derive(Component)]
struct CondenseChunk {
    sections: Vec<CondenseChunkSection>,
}

struct CondenseChunkSection {
    block_states: PalettedContainer<{ 16 * 16 * 16 }, 4, 8, 15>,
    biomes: PalettedContainer<{ 4 * 4 * 4 }, 3, 3, 6>,
}

fn condense_chunks(
    mut commands: Commands,

    condense_cache: Local<CondenseCache>,

    mut chunks: Query<
        (Entity, &level::chunk::Data, Option<&mut CondenseChunk>),
        Changed<level::chunk::Data>,
    >,
) {
    for (chunk, chunk_data, condense_chunk) in chunks.iter_mut() {
        if let Some(mut condense_chunk) = condense_chunk {
            for (section_y, section) in chunk_data.sections.iter().enumerate() {
                let condense_section = &mut condense_chunk.sections[section_y];
                for &block_state_change in &section.block_state_changes {
                    let block = section.block_states.get(block_state_change as u32);
                    condense_section.block_states.get_and_set(
                        block_state_change as u32,
                        match condense_cache.0.get(&block).unwrap() {
                            CondenseBlock::Default(value) => *value,
                            CondenseBlock::Snowy { false_, .. } => *false_,
                        },
                    );
                }
            }
        } else {
            commands.entity(chunk).insert(CondenseChunk {
                sections: chunk_data
                    .sections
                    .iter()
                    .map(|section| CondenseChunkSection {
                        block_states: match &section.block_states {
                            PalettedContainer::Single(block) => {
                                PalettedContainer::Single(match condense_cache.0.get(block).unwrap() {
                                    CondenseBlock::Default(value) => *value,
                                    CondenseBlock::Snowy { false_, .. } => *false_,
                                })
                            }
                            PalettedContainer::Indirect { palette, storage } => {
                                PalettedContainer::Indirect {
                                    palette: palette
                                        .iter()
                                        .map(|block| match condense_cache.0.get(block).unwrap() {
                                            CondenseBlock::Default(value) => *value,
                                            CondenseBlock::Snowy { false_, .. } => *false_,
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

    levels: Query<&level::chunk::LookupTable>,
    chunks: Query<&Parent>,
    mut subscription_chunks: Query<&mut Replication>,
    players: Query<(Entity, &Parent, &Subscription), Without<Connection>>,
) {
    for (player, chunk, subscription) in players.iter() {
        if let Ok(level) = chunks.get(chunk.get()) {
            commands.entity(player).remove::<Subscription>();

            let chunk_lut = levels.get(level.get()).unwrap();
            for chunk_position in
                SquareIterator::new(subscription.last_center, subscription.last_radius as i32)
            {
                if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn subscribe_and_replicate_chunks(
    registries: Res<registry::RegistriesReport>,

    mut commands: Commands,

    mut levels: Query<&mut level::chunk::LookupTable>,
    chunks: Query<(&level::chunk::Base, &Parent)>,
    mut subscription_chunks: Query<(Option<&CondenseChunk>, &mut Replication)>,
    actors: Query<(
        Entity,
        &actor::Base,
        &actor::Position,
        &actor::Rotation,
        &actor::HeadRotation,
    )>,
    mut players: Query<
        (Entity, &Parent, &Connection, &mut Subscription),
        Or<(Changed<Parent>, Changed<Subscription>)>,
    >,
) {
    for (player, chunk, connection, mut subscription) in players.iter_mut() {
        if let Ok((chunk_base, level)) = chunks.get(chunk.get()) {
            let mut chunk_lut = levels.get_mut(level.get()).unwrap();

            let center = chunk_base.position();
            connection.send(&s2c::GamePacket::SetChunkCacheCenter {
                x: center.x,
                z: center.y,
            });

            let radius = subscription.radius as i32;
            let last_center = subscription.last_center;
            let last_radius = subscription.last_radius as i32;

            // release chunks
            for chunk_position in SquareIterator::new(last_center, last_radius).filter(|position| {
                position.x >= (center.x + radius)
                    || position.x <= (center.x - radius)
                    || position.y >= (center.y + radius)
                    || position.y <= (center.y - radius)
            }) {
                if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
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
            for chunk_position in SquareIterator::new(chunk_base.position(), subscription.radius as i32)
                .filter(|position| {
                    position.x >= (last_center.x + last_radius)
                        || position.x <= (last_center.x - last_radius)
                        || position.y >= (last_center.y + last_radius)
                        || position.y <= (last_center.y - last_radius)
                })
            {
                if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                    if let Ok((condense_chunk, mut replication)) = subscription_chunks.get_mut(chunk) {
                        trace!("Acquire chunk: {:?}", chunk_position);

                        replication.subscriber.insert(player);

                        if let Some(condense_chunk) = condense_chunk {
                            // connection: add chunk and actors, cause: subscribe
                            connection.send(&add_chunk_packet(chunk_position, condense_chunk));
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
                            .spawn(level::chunk::ChunkBundle {
                                base: level::chunk::Base::new(chunk_position),
                                update_queue: Default::default(),
                                replication: Replication {
                                    subscriber: HashSet::from([player]),
                                    replicated: vec![],
                                },
                            })
                            .set_parent(level.get())
                            .id(),
                    );
                }
            }

            subscription.last_center = chunk_base.position();
            subscription.last_radius = subscription.radius;
        }
    }
}

fn replicate_chunks_late(
    chunks: Query<(&level::chunk::Base, &CondenseChunk, &Replication), Added<level::chunk::Data>>,
    players: Query<&Connection>,
) {
    for (chunk_base, condense_chunk, replication) in chunks.iter() {
        let add_chunk_packet = add_chunk_packet(chunk_base.position(), condense_chunk);
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
            &level::chunk::Base,
            &mut level::chunk::Data,
            &CondenseChunk,
            &Replication,
        ),
        Changed<level::chunk::Data>,
    >,
    players: Query<&Connection>,
) {
    for (chunk_base, mut chunk_data, condense_chunk, replication) in chunks.iter_mut() {
        let mut update_chunk_packets = vec![];
        {
            let y_offset = chunk_data.y_offset as i32;
            for (section_y, section) in chunk_data.sections.iter_mut().enumerate() {
                if section.block_state_changes.is_empty() {
                    continue;
                }

                let chunk_position = chunk_base.position();
                let condense_section = &condense_chunk.sections[section_y];
                update_chunk_packets.push(s2c::GamePacket::SectionBlocksUpdate(
                    s2c::game::SectionBlocksUpdatePacket {
                        section_pos: IVec3::new(chunk_position.x, section_y as i32 - y_offset, chunk_position.y),
                        suppress_light_updates: true,
                        position_and_states: section
                            .block_state_changes
                            .iter()
                            .map(|&block_state_change| {
                                s2c::game::SectionBlocksUpdatePacketPositionAndState {
                                    x: block_state_change as u8 & 0xF,
                                    y: (block_state_change >> 8) as u8,
                                    z: block_state_change as u8 >> 4 & 0xF,
                                    block_state: condense_section
                                        .block_states
                                        .get(block_state_change as u32)
                                        as i64,
                                }
                            })
                            .collect(),
                    },
                ));

                section.block_state_changes.clear();
            }
        }
        if update_chunk_packets.is_empty() {
            continue;
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
    registries: Res<registry::RegistriesReport>,

    mut chunks: Query<(&Children, &mut Replication), Changed<Children>>,
    actors: Query<(
        &actor::Base,
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
                id: actor.index() as i32,
                pos: actor_position.0,
                pitch: actor_rotation.pitch,
                yaw: actor_rotation.yaw,
                on_ground: false,
            };
            let update_head_rotation_packet = s2c::GamePacket::RotateHead {
                entity_id: actor.index() as i32,
                head_yaw: actor_rotation.yaw,
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

struct SquareIterator {
    center_x: i32,
    center_z: i32,
    radius: i32,

    r: i32,
    n: i32,
    i: i32,
}

impl SquareIterator {
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

impl Iterator for SquareIterator {
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
        (0, Some((self.radius * self.radius * 2) as usize))
    }
}

fn add_chunk_packet<'a>(position: IVec2, condense_chunk: &CondenseChunk) -> s2c::GamePacket<'a> {
    let mut buffer = Vec::new();
    let mut sky_y_mask = 0i64;
    let mut sky_updates = Vec::new();
    for (i, condense_section) in condense_chunk.sections.iter().enumerate() {
        4096i16.encode(&mut buffer).unwrap();
        condense_section.block_states.encode(&mut buffer).unwrap();
        condense_section.biomes.encode(&mut buffer).unwrap();

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
    registries: &registry::RegistriesReport,
    actor: Entity,
    actor_base: &actor::Base,
    position: &actor::Position,
    rotation: &actor::Rotation,
    head_rotation: &actor::HeadRotation,
) -> s2c::GamePacket<'a> {
    if actor_base.type_ == "minecraft:player" {
        s2c::GamePacket::AddPlayer {
            entity_id: actor.index() as i32,
            player_id: actor_base.id,
            pos: position.0,
            pitch: rotation.pitch,
            yaw: rotation.yaw,
        }
    } else {
        s2c::GamePacket::AddEntity {
            id: actor.index() as i32,
            uuid: actor_base.id,
            type_: registries.id("minecraft:entity_type", &actor_base.type_) as i32,
            pos: position.0,
            pitch: rotation.pitch,
            yaw: rotation.yaw,
            head_yaw: head_rotation.head_yaw,
            data: 0,
            xa: 0,
            ya: 0,
            za: 0,
        }
    }
}
