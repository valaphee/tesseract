use std::fmt::{Debug, Formatter};
use anyhow::{bail, Result};
use std::io;
use std::io::{Error, IoSlice};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use bevy::prelude::*;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::timeout;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tesseract_protocol::packet::{c2s, s2c};
use tesseract_protocol::types::{GameProfile, Json, VarInt};
use uuid::Uuid;
use tesseract_protocol::{Codec, Decode, Encode};
use tesseract_protocol::packet::c2s::handshake::IntentionPacketIntention;

#[derive(Default)]
pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(listen)
            .add_system(initialize_connection)
            .add_system(update_connection);
    }
}

#[derive(Resource)]
struct NewConnectionReceiver(UnboundedReceiver<Connection>);

#[derive(Component)]
pub struct Connection {
    receiver: UnboundedReceiver<c2s::GamePacket>,
    received: Vec<c2s::GamePacket>,

    pub sender: UnboundedSender<s2c::GamePacket>,
}

impl Debug for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

fn listen(
    mut commands: Commands
) {
    let (new_connection_tx, new_connection_rx) = mpsc::unbounded_channel();

    commands.insert_resource(NewConnectionReceiver(new_connection_rx));

    std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565)).await.unwrap();

                loop {
                    let (socket, _) = listener.accept().await.unwrap();
                    let (input, output) = socket.into_split();

                    let new_connection_tx = new_connection_tx.clone();
                    tokio::spawn(async move {
                        let mut framed_input = Framed::new(DummyOutput {
                            input,
                        }, Codec::<c2s::HandshakePacket>::new());

                        match framed_input.next().await.unwrap().unwrap() {
                            c2s::HandshakePacket::Intention(packet) => {
                                match packet.intention {
                                    IntentionPacketIntention::Status => {
                                        let mut framed_input = framed_input.map_codec(|_| {
                                            Codec::<c2s::StatusPacket>::new()
                                        });

                                        let mut framed_output = Framed::new(DummyInput {
                                            output,
                                        }, Codec::<s2c::StatusPacket>::new());
                                    }
                                    IntentionPacketIntention::Login => {}
                                    _ => unreachable!()
                                }

                            }
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

fn update_connection(
    mut connections: Query<&mut Connection>,
) {
    for mut connection in connections.iter_mut() {
        connection.received.clear();
        while let Ok(packet) = connection.receiver.try_recv() {
            connection.received.push(packet);
        }
    }
}
