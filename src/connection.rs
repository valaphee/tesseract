use std::net::{Ipv4Addr, SocketAddrV4};

use bevy::prelude::*;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;
use uuid::Uuid;

use tesseract_protocol::{
    codec::Codec,
    packet::{c2s, s2c},
    types::{GameProfile, Intention, Json, Status, StatusPlayers, StatusVersion},
    Decode, Encode,
};
use tesseract_protocol::types::VarInt;

#[derive(Default)]
pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(listen)
            .add_system(initialize_connection)
            .add_system(update_connection);
    }
}

#[derive(Resource)]
struct NewConnectionReceiver(mpsc::UnboundedReceiver<Connection>);

#[derive(Debug, Component)]
pub struct Connection {
    receiver: mpsc::UnboundedReceiver<c2s::GamePacket>,
    received: Vec<c2s::GamePacket>,

    pub sender: mpsc::UnboundedSender<s2c::GamePacket>,
}

fn listen(mut commands: Commands) {
    let (new_connection_tx, new_connection_rx) = mpsc::unbounded_channel();

    commands.insert_resource(NewConnectionReceiver(new_connection_rx));

    std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let listener =
                    TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565))
                        .await
                        .unwrap();

                loop {
                    let (socket, _) = listener.accept().await.unwrap();

                    let new_connection_tx = new_connection_tx.clone();
                    tokio::spawn(async move {
                        let mut framed_socket = Framed::new(
                            socket,
                            Codec::<s2c::StatusPacket, c2s::HandshakePacket>::new(),
                        );
                        match framed_socket.next().await.unwrap().unwrap() {
                            c2s::HandshakePacket::Intention { intention, .. } => match intention {
                                Intention::Status => {
                                    let mut framed_socket = framed_socket.map_codec(|_| {
                                        Codec::<s2c::StatusPacket, c2s::StatusPacket>::new()
                                    });
                                    framed_socket.next().await.unwrap().unwrap();
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
                                    match framed_socket.next().await.unwrap().unwrap() {
                                        c2s::StatusPacket::PingRequest { time } => {
                                            framed_socket
                                                .send(s2c::StatusPacket::PongResponse { time })
                                                .await
                                                .unwrap();
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                Intention::Login => {
                                    let mut framed_socket = framed_socket.map_codec(|_| {
                                        Codec::<s2c::LoginPacket, c2s::LoginPacket>::new()
                                    });
                                    match framed_socket.next().await.unwrap().unwrap() {
                                        c2s::LoginPacket::Hello { name, .. } => {
                                            /*framed_socket
                                                .send(s2c::LoginPacket::LoginCompression {
                                                    compression_threshold: VarInt(256),
                                                })
                                                .await
                                                .unwrap();
                                            framed_socket.codec_mut().compression_threshold = Some(256);*/

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

                                            let (receive_packet_tx, receive_packet_rx) =
                                                mpsc::unbounded_channel();
                                            let (send_packet_tx, mut send_packet_rx) =
                                                mpsc::unbounded_channel();
                                            new_connection_tx
                                                .send(Connection {
                                                    receiver: receive_packet_rx,
                                                    received: vec![],
                                                    sender: send_packet_tx,
                                                })
                                                .unwrap();

                                            let mut framed_socket = framed_socket.map_codec(|_| {
                                                Codec::<s2c::GamePacket, c2s::GamePacket>::new()
                                            });
                                            let (mut sink, mut stream) = framed_socket.split();

                                            tokio::spawn(async move {
                                                while let Some(frame) = stream.next().await {
                                                    let packet = frame.unwrap();
                                                    println!("Recv {:?}", packet);
                                                    receive_packet_tx.send(packet).unwrap();
                                                }
                                            });

                                            tokio::spawn(async move {
                                                while let Some(packet) = send_packet_rx.recv().await
                                                {
                                                    println!("Send {:?}", packet);
                                                    sink.send(packet).await.unwrap();
                                                }
                                            });
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            },
                        }
                    });
                }
            })
    });
}

fn initialize_connection(
    mut commands: Commands,
    mut new_connection_receiver: ResMut<NewConnectionReceiver>,
) {
    while let Ok(connection) = new_connection_receiver.0.try_recv() {
        commands.spawn(connection);
    }
}

fn update_connection(mut connections: Query<&mut Connection>) {
    for mut connection in connections.iter_mut() {
        connection.received.clear();
        while let Ok(packet) = connection.receiver.try_recv() {
            connection.received.push(packet);
        }
    }
}
