use std::net::SocketAddr;

use bevy::prelude::*;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;
use uuid::Uuid;

use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, s2c},
    types::{GameProfile, Intention, Json, Status, StatusPlayers, StatusVersion, VarInt},
};

pub struct ConnectionPlugin {
    address: SocketAddr,

    compression: Compression,
    compression_threshold: Option<u16>,
}

#[derive(Component)]
pub struct Connection(Framed<TcpStream, Codec<s2c::GamePacket, c2s::GamePacket>>);

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
                        let listener = TcpListener::bind(address).await.unwrap();

                        loop {
                            if let Ok((socket, _)) = listener.accept().await {
                                tokio::spawn(handle_new_connection(
                                    socket,
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

#[derive(Resource)]
struct NewConnectionRx(mpsc::UnboundedReceiver<Connection>);

fn spawn_new_connection(mut commands: Commands, mut new_connection_rx: ResMut<NewConnectionRx>) {
    while let Ok(connection) = new_connection_rx.0.try_recv() {
        commands.spawn(connection);
    }
}

async fn handle_new_connection(
    socket: TcpStream,
    compression: Compression,
    compression_threshold: Option<u16>,
    new_connection_tx: mpsc::UnboundedSender<Connection>,
) {
    let mut framed_socket = Framed::new(
        socket,
        Codec::<s2c::StatusPacket, c2s::HandshakePacket>::new(),
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

                if let Some(compression_threshold) = compression_threshold {
                    framed_socket
                        .send(s2c::LoginPacket::LoginCompression {
                            compression_threshold: VarInt(compression_threshold as i32),
                        })
                        .await
                        .unwrap();
                    framed_socket.codec_mut().compression = compression;
                    framed_socket.codec_mut().compression_threshold = Some(compression_threshold);
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

                if new_connection_tx
                    .send(Connection(framed_socket.map_codec(|codec| {
                        codec.cast::<s2c::GamePacket, c2s::GamePacket>()
                    })))
                    .is_ok()
                {}
            }
            _ => unreachable!(),
        },
    }
}
