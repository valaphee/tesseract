use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::prelude::*;
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
    types::{Intention, Json, Status, StatusPlayers, StatusVersion, VarI32},
};

/// Actor
#[derive(Component)]
pub struct Connection {
    rx: mpsc::UnboundedReceiver<c2s::GamePacket>,
    tx: mpsc::UnboundedSender<s2c::GamePacket>,

    pub(crate) incoming: Vec<c2s::GamePacket>,
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

        app.add_startup_system(listen)
            .add_system(spawn_new_connection)
            .add_system(update_connection);
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
                                    description: Some(
                                        tesseract_protocol::types::Component::Literal(
                                            "Tesseract".to_string(),
                                        ),
                                    ),
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

                framed_socket.codec_mut().enable_encryption(key);

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
                        incoming: vec![],
                    })
                    .is_ok()
                {}

                let (mut sink, mut stream) = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::GamePacket, c2s::GamePacket>())
                    .split();
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
            _ => unimplemented!(),
        },
    }
}

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_new_connection(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        commands.spawn(connection);
    }
}

fn update_connection(mut connections: Query<&mut Connection>) {
    for mut connection in connections.iter_mut() {
        connection.incoming.clear();
        while let Ok(packet) = connection.rx.try_recv() {
            connection.incoming.push(packet);
        }
    }
}
