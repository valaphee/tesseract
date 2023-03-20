use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::{math::DVec3, prelude::*};
use futures::{SinkExt, StreamExt};
use num::BigInt;
use rsa::{pkcs8::EncodePublicKey, rand_core::OsRng, Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::{digest::Update, Digest, Sha1};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::Framed;

use mojang_session_api::apis::{configuration::Configuration, default_api::has_joined_server};
use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, s2c},
    types::{
        Biome, BiomeEffects, Component as ChatComponent, GameType, Intention, Json, Nbt,
        Registries, Registry, RegistryEntry, Status, StatusPlayers, StatusVersion, VarI32,
    },
};

use crate::{actor, level, replication};

#[derive(Component)]
pub struct Connection {
    rx: mpsc::UnboundedReceiver<c2s::GamePacket>,
    tx: mpsc::UnboundedSender<s2c::GamePacket>,
}

impl Connection {
    pub fn send(&self, packet: s2c::GamePacket) {
        self.tx.send(packet).unwrap();
    }
}

pub struct ConnectionPlugin {
    address: SocketAddr,

    compression: Compression,
    compression_threshold: Option<u16>,
}

impl Default for ConnectionPlugin {
    fn default() -> Self {
        Self {
            address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565).into(),

            compression: Default::default(),
            compression_threshold: Some(256),
        }
    }
}

impl Plugin for ConnectionPlugin {
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
            .add_systems(PreUpdate, update_players);
    }
}

async fn handle_new_connection(
    socket: TcpStream,
    private_key: RsaPrivateKey,
    compression: Compression,
    compression_threshold: Option<u16>,
    new_connection_tx: mpsc::UnboundedSender<Connection>,
) {
    let mut framed_socket = Framed::new(
        socket,
        Codec::<s2c::HandshakePacket, c2s::HandshakePacket>::default(),
    );
    match framed_socket.next().await.unwrap().unwrap() {
        c2s::HandshakePacket::Intention { intention, .. } => match intention {
            Intention::Status => {
                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::StatusPacket, c2s::StatusPacket>());

                match framed_socket.next().await.unwrap().unwrap() {
                    c2s::StatusPacket::StatusRequest => {
                        framed_socket
                            .send(s2c::StatusPacket::StatusResponse {
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
                            })
                            .await
                            .unwrap();
                    }
                    _ => unimplemented!(),
                }

                match framed_socket.next().await.unwrap().unwrap() {
                    c2s::StatusPacket::PingRequest { time } => {
                        framed_socket
                            .send(s2c::StatusPacket::PongResponse { time })
                            .await
                            .unwrap();
                    }
                    _ => unimplemented!(),
                };
            }
            Intention::Login => {
                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::LoginPacket, c2s::LoginPacket>());

                let name = match framed_socket.next().await.unwrap().unwrap() {
                    c2s::LoginPacket::Hello { name, .. } => name,
                    _ => unimplemented!(),
                };

                let nonce: [u8; 16] = rand::random();
                framed_socket
                    .send(s2c::LoginPacket::Hello {
                        server_id: "".to_string(),
                        public_key: private_key.to_public_key_der().unwrap().to_vec(),
                        nonce: nonce.to_vec(),
                    })
                    .await
                    .unwrap();
                let key = match framed_socket.next().await.unwrap().unwrap() {
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
                    framed_socket
                        .send(s2c::LoginPacket::LoginCompression {
                            compression_threshold: VarI32(compression_threshold as i32),
                        })
                        .await
                        .unwrap();
                    framed_socket
                        .codec_mut()
                        .enable_compression(compression, compression_threshold);
                }

                framed_socket
                    .send(s2c::LoginPacket::GameProfile(user))
                    .await
                    .unwrap();

                let (rx_packet_tx, rx_packet_rx) = mpsc::unbounded_channel();
                let (tx_packet_tx, mut tx_packet_rx) = mpsc::unbounded_channel();
                if new_connection_tx
                    .send(Connection {
                        rx: rx_packet_rx,
                        tx: tx_packet_tx,
                    })
                    .is_ok()
                {}

                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::GamePacket, c2s::GamePacket>());
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            packet = framed_socket.next() => {
                                if let Some(packet) = packet {
                                    let packet = packet.unwrap();
                                    //println!("Recv: {:?}", &packet);
                                    rx_packet_tx.send(packet).unwrap();
                                } else {
                                    break;
                                }
                            }
                            packet = tx_packet_rx.recv() => {
                                if let Some(packet) = packet {
                                    //println!("Send: {:?}", &packet);
                                    framed_socket.send(packet).await.unwrap();
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
    mut commands: Commands,
    levels: Query<(Entity, &level::Level)>,
    players: Query<(Entity, &Connection), Added<Connection>>,
) {
    for (player, connection) in players.iter() {
        let (level, level_data) = levels.single();
        commands
            .entity(player)
            .insert((
                actor::Position(DVec3::new(0.0, 0.0, 0.0)),
                actor::Rotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
                actor::HeadRotation { head_yaw: 0.0 },
                replication::SubscriptionDistance(4),
                replication::Subscriptions::default(),
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
                biome: Registry {
                    type_: "minecraft:worldgen/biome".to_string(),
                    value: vec![RegistryEntry {
                        name: "minecraft:plains".to_string(),
                        id: 0,
                        element: Biome {
                            has_precipitation: true,
                            precipitation: "rain".to_string(),
                            temperature: 0.0,
                            temperature_modifier: None,
                            downfall: 0.0,
                            effects: BiomeEffects {
                                fog_color: 0,
                                water_color: 0,
                                water_fog_color: 0,
                                sky_color: 0,
                                foliage_color: None,
                                grass_color: None,
                                grass_color_modifier: None,
                                ambient_sound: None,
                                mood_sound: None,
                                additions_sound: None,
                                music: None,
                            },
                        },
                    }],
                },
                chat_type: Registry {
                    type_: "minecraft:chat_type".to_string(),
                    value: vec![],
                },
                damage_type: {
                    let data = std::fs::read("damage_type.nbt").unwrap();
                    tesseract_serde_nbt::de::from_slice(&mut data.as_slice()).unwrap()
                },
            }),
            dimension_type: level_data.name.clone(),
            dimension: level_data.name.clone(),
            seed: 0,
            max_players: VarI32(0),
            chunk_radius: VarI32(4),
            simulation_distance: VarI32(4),
            reduced_debug_info: false,
            show_death_screen: false,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
        });
        connection.send(s2c::game::GamePacket::SetDefaultSpawnPosition {
            pos: IVec3::new(0, 100, 0),
            yaw: 0.0,
        });
    }
}

fn update_players(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Connection, &mut actor::Position)>,
) {
    for (player, mut connection, mut position) in players.iter_mut() {
        if connection.tx.is_closed() {
            commands.entity(player).remove::<Connection>();
        } else {
            while let Ok(packet) = connection.rx.try_recv() {
                match packet {
                    c2s::GamePacket::MovePlayerPos { x, y, z, .. } => {
                        position.0.x = x;
                        position.0.y = y;
                        position.0.z = z;
                    }
                    c2s::GamePacket::MovePlayerPosRot { x, y, z, .. } => {
                        position.0.x = x;
                        position.0.y = y;
                        position.0.z = z;
                    }
                    _ => {}
                }
            }
        }
    }
}
