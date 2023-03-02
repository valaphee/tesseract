use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::prelude::*;
use futures::{SinkExt, StreamExt};
use rsa::{Pkcs1v15Encrypt, PublicKeyParts, RsaPrivateKey};
use rsa::rand_core::OsRng;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;
use uuid::Uuid;

use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, s2c},
    types::{GameProfile, Intention, Json, Status, StatusPlayers, StatusVersion, VarInt},
};
use tesseract_protocol::types::{Biome, BiomeEffects, BiomePrecipitation, DimensionType, GameType, Nbt, Registries, Registry, RegistryEntry};

#[derive(Component)]
pub struct Connection {
    rx: mpsc::UnboundedReceiver<c2s::GamePacket>,
    tx: mpsc::UnboundedSender<s2c::GamePacket>,
}

impl Connection {
    pub fn send(&mut self, packet: s2c::GamePacket) {
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

        app.add_startup_system(listen)
            .add_system(spawn_new_connection);
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
        Codec::<s2c::StatusPacket, c2s::HandshakePacket>::default(),
    );
    match framed_socket.next().await.unwrap().unwrap() {
        c2s::HandshakePacket::Intention { intention, .. } => match intention {
            Intention::Status => {
                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::StatusPacket, c2s::StatusPacket>());

                match framed_socket.next().await.unwrap().unwrap() {
                    c2s::StatusPacket::StatusRequest => (),
                    _ => unreachable!(),
                }
                framed_socket
                    .send(s2c::StatusPacket::StatusResponse {
                        status: Json(Status {
                            description: Some("Tesseract".to_string()),
                            players: Some(StatusPlayers {
                                max: 1,
                                online: 0,
                                sample: vec![],
                            }),
                            version: Some(StatusVersion {
                                name: "1.19.3".to_string(),
                                protocol: 761,
                            }),
                            favicon: None,
                            previews_chat: false,
                        }),
                    })
                    .await
                    .unwrap();

                let time = match framed_socket.next().await.unwrap().unwrap() {
                    c2s::StatusPacket::PingRequest { time } => time,
                    _ => unreachable!(),
                };
                framed_socket
                    .send(s2c::StatusPacket::PongResponse { time })
                    .await
                    .unwrap();
            }
            Intention::Login => {
                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::LoginPacket, c2s::LoginPacket>());

                let name = match framed_socket.next().await.unwrap().unwrap() {
                    c2s::LoginPacket::Hello { name, .. } => name,
                    _ => unreachable!(),
                };

                /*let nonce: [u8; 16] = rand::random();
                framed_socket
                    .send(s2c::LoginPacket::Hello {
                        server_id: "".to_string(),
                        public_key: rsa_der::public_key_to_der(&private_key.n().to_bytes_be(), &private_key.e().to_bytes_be()),
                        nonce: nonce.to_vec(),
                    })
                    .await
                    .unwrap();
                let key = match framed_socket.next().await.unwrap().unwrap() {
                    c2s::LoginPacket::Key { key, nonce } => {
                        private_key.decrypt(Pkcs1v15Encrypt::default(), &nonce).unwrap();
                        private_key.decrypt(Pkcs1v15Encrypt::default(), &key).unwrap()
                    },
                    _ => unreachable!(),
                };
                framed_socket.codec_mut().encryption(key);*/

                if let Some(compression_threshold) = compression_threshold {
                    framed_socket
                        .send(s2c::LoginPacket::LoginCompression {
                            compression_threshold: VarInt(compression_threshold as i32),
                        })
                        .await
                        .unwrap();
                    framed_socket.codec_mut().compression(compression, compression_threshold);
                }

                framed_socket
                    .send(s2c::LoginPacket::GameProfile {
                        game_profile: GameProfile {
                            id: Uuid::new_v4(),
                            name,
                            properties: vec![],
                        },
                    })
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

                let (mut sink, mut stream) = framed_socket.map_codec(|codec| {
                    codec.cast::<s2c::GamePacket, c2s::GamePacket>()
                }).split();
                tokio::spawn(async move {
                    while let Some(packet) = stream.next().await {
                        let packet = packet.unwrap();
                        println!("Recv: {:?}", &packet);
                        rx_packet_tx.send(packet).unwrap();
                    }
                });
                tokio::spawn(async move {
                    while let Some(packet) = tx_packet_rx.recv().await {
                        println!("Send: {:?}", &packet);
                        sink.send(packet).await.unwrap();
                    }
                });
            }
            _ => unreachable!(),
        },
    }
}

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_new_connection(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        connection
            .tx
            .send(s2c::GamePacket::Login {
                player_id: 1,
                hardcore: false,
                game_type: GameType::Survival,
                previous_game_type: 0,
                levels: vec!["minecraft:overworld".to_string()],
                registry_holder: Nbt(Registries {
                    biome_registry: Registry {
                        _type: "minecraft:worldgen/biome".to_string(),
                        value: vec![RegistryEntry {
                            name: "plains".to_string(),
                            id: 0,
                            element: Biome {
                                precipitation: BiomePrecipitation::None,
                                temperature: 0.0,
                                downfall: 0.0,
                                temperature_modifier: None,
                                effects: BiomeEffects {
                                    sky_color: 0xFFFF0000,
                                    water_fog_color: 0,
                                    fog_color: 0,
                                    water_color: 0,
                                    foliage_color: None,
                                    grass_color: None,
                                    grass_color_modifier: None,
                                    music: None,
                                    ambient_sound: None,
                                    additions_sound: None,
                                    mood_sound: None,
                                },
                            },
                        }],
                    },
                    dimension_type_registry: Registry {
                        _type: "minecraft:dimension_type".to_string(),
                        value: vec![RegistryEntry {
                            name: "minecraft:overworld".to_string(),
                            id: 0,
                            element: DimensionType {
                                piglin_safe: true,
                                has_raids: true,
                                monster_spawn_light_level: 0,
                                monster_spawn_block_light_limit: 0,
                                natural: true,
                                ambient_light: 1.0,
                                fixed_time: None,
                                infiniburn: "#minecraft:infiniburn_overworld".to_string(),
                                respawn_anchor_works: true,
                                has_skylight: true,
                                bed_works: true,
                                effects: "minecraft:overworld".to_string(),
                                min_y: 0,
                                height: 16 * 16,
                                logical_height: 16 * 16,
                                coordinate_scale: 1.0,
                                ultrawarm: false,
                                has_ceiling: false,
                            },
                        }],
                    },
                }),
                dimension_type: "minecraft:overworld".to_string(),
                dimension: "minecraft:overworld".to_string(),
                seed: 0,
                max_players: VarInt(0),
                chunk_radius: VarInt(0),
                simulation_distance: VarInt(0),
                reduced_debug_info: false,
                show_death_screen: false,
                is_debug: false,
                is_flat: false,
                last_death_location: None,
            })
            .unwrap();
        commands.spawn(connection);
    }
}
