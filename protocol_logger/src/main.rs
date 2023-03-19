use std::net::{Ipv4Addr, SocketAddrV4};

use futures::{SinkExt, StreamExt};
use num::BigInt;
use rand::{rngs::OsRng, Rng};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, PublicKey, RsaPublicKey};
use sha1::{digest::Update, Digest, Sha1};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use uuid::Uuid;

use mojang_session_api::{
    apis::{configuration::Configuration, default_api::join_server},
    models::JoinServerRequest,
};
use tesseract_protocol::{
    codec::{Codec, Compression},
    packet::{c2s, s2c},
    types::Intention,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 25565))
        .await
        .unwrap();

    loop {
        if let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(handle_new_connection(socket));
        }
    }
}

async fn handle_new_connection(socket: TcpStream) {
    socket.set_nodelay(true).unwrap();
    let mut framed_socket = Framed::new(
        socket,
        Codec::<s2c::HandshakePacket, c2s::HandshakePacket>::default(),
    );

    let client_socket =
        TcpStream::connect(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565))
            .await
            .unwrap();
    client_socket.set_nodelay(true).unwrap();
    let mut framed_client_socket = Framed::new(
        client_socket,
        Codec::<c2s::HandshakePacket, s2c::HandshakePacket>::default(),
    );

    match framed_socket.next().await.unwrap().unwrap() {
        c2s::HandshakePacket::Intention {
            protocol_version,
            host_name,
            port,
            intention,
        } => match intention {
            Intention::Status => {
                framed_client_socket
                    .send(c2s::HandshakePacket::Intention {
                        protocol_version,
                        host_name,
                        port,
                        intention: Intention::Status,
                    })
                    .await
                    .unwrap();

                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::StatusPacket, c2s::StatusPacket>());
                let mut framed_client_socket = framed_client_socket
                    .map_codec(|codec| codec.cast::<c2s::StatusPacket, s2c::StatusPacket>());

                let packet = framed_socket.next().await.unwrap().unwrap();
                if matches!(packet, c2s::StatusPacket::StatusRequest { .. }) {
                    framed_client_socket.send(packet).await.unwrap();
                } else {
                    unimplemented!()
                }

                let packet = framed_client_socket.next().await.unwrap().unwrap();
                if matches!(packet, s2c::StatusPacket::StatusResponse { .. }) {
                    framed_socket.send(packet).await.unwrap();
                } else {
                    unimplemented!()
                }

                let packet = framed_socket.next().await.unwrap().unwrap();
                if matches!(packet, c2s::StatusPacket::PingRequest { .. }) {
                    framed_client_socket.send(packet).await.unwrap();
                } else {
                    unimplemented!()
                }

                let packet = framed_client_socket.next().await.unwrap().unwrap();
                if matches!(packet, s2c::StatusPacket::PongResponse { .. }) {
                    framed_socket.send(packet).await.unwrap();
                } else {
                    unimplemented!()
                }
            }
            Intention::Login => {
                framed_client_socket
                    .send(c2s::HandshakePacket::Intention {
                        protocol_version,
                        host_name,
                        port,
                        intention: Intention::Login,
                    })
                    .await
                    .unwrap();

                let mut framed_socket = framed_socket
                    .map_codec(|codec| codec.cast::<s2c::LoginPacket, c2s::LoginPacket>());
                let mut framed_client_socket = framed_client_socket
                    .map_codec(|codec| codec.cast::<c2s::LoginPacket, s2c::LoginPacket>());

                let packet = framed_socket.next().await.unwrap().unwrap();
                if matches!(packet, c2s::LoginPacket::Hello { .. }) {
                    framed_client_socket.send(packet).await.unwrap();
                } else {
                    unimplemented!()
                }

                match framed_client_socket.next().await.unwrap().unwrap() {
                    s2c::LoginPacket::Hello {
                        server_id,
                        public_key,
                        nonce,
                    } => {
                        let mut rng = OsRng::default();
                        let mut key = [0u8; 16];
                        rng.fill(&mut key);

                        join_server(
                            &Configuration::new(),
                            Some(JoinServerRequest {
                                access_token: "".to_string(),
                                selected_profile: Uuid::new_v4(),
                                server_id: BigInt::from_signed_bytes_be(
                                    &Sha1::new()
                                        .chain(server_id.as_bytes())
                                        .chain(key)
                                        .chain(&public_key)
                                        .finalize(),
                                )
                                .to_str_radix(16),
                            }),
                        )
                        .await
                        .unwrap();

                        let public_key = RsaPublicKey::from_public_key_der(&public_key).unwrap();
                        framed_client_socket
                            .send(c2s::LoginPacket::Key {
                                key: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &key)
                                    .unwrap(),
                                nonce: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &nonce)
                                    .unwrap(),
                            })
                            .await
                            .unwrap();
                        framed_client_socket
                            .codec_mut()
                            .enable_encryption(key.to_vec());

                        loop {
                            match framed_client_socket.next().await.unwrap().unwrap() {
                                s2c::LoginPacket::LoginCompression {
                                    compression_threshold,
                                } => {
                                    framed_client_socket.codec_mut().enable_compression(
                                        Compression::default(),
                                        compression_threshold.0 as u16,
                                    );
                                    framed_socket
                                        .send(s2c::LoginPacket::LoginCompression {
                                            compression_threshold,
                                        })
                                        .await
                                        .unwrap();
                                    framed_socket.codec_mut().enable_compression(
                                        Compression::default(),
                                        compression_threshold.0 as u16,
                                    );
                                }
                                packet => {
                                    if matches!(packet, s2c::LoginPacket::GameProfile(..)) {
                                        framed_socket.send(packet).await.unwrap();
                                        break;
                                    } else {
                                        unimplemented!()
                                    }
                                }
                            }
                        }
                        let (mut sink, mut stream) = framed_socket
                            .map_codec(|codec| codec.cast::<s2c::GamePacket, c2s::GamePacket>())
                            .split();
                        let (mut client_sink, mut client_stream) = framed_client_socket
                            .map_codec(|codec| codec.cast::<c2s::GamePacket, s2c::GamePacket>())
                            .split();
                        tokio::spawn(async move {
                            while let Some(packet) = stream.next().await {
                                let packet = packet.unwrap();
                                println!("Recv: {:?}", &packet);
                                client_sink.send(packet).await.unwrap();
                            }
                        });
                        tokio::spawn(async move {
                            while let Some(packet) = client_stream.next().await {
                                let packet = packet.unwrap();
                                match packet {
                                    s2c::GamePacket::LevelParticles { .. } => {}
                                    packet => {
                                        if !matches!(
                                            packet,
                                            s2c::GamePacket::LevelChunkWithLight { .. }
                                        ) {
                                            println!("Send: {:?}", &packet);
                                        }
                                        sink.send(packet).await.unwrap();
                                    }
                                }
                            }
                        });
                    }
                    _ => unimplemented!(),
                };
            }
            _ => unimplemented!(),
        },
    }
}
