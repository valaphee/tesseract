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
    Decode, Encode,
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

    let mut framed_socket = Framed::new(socket, Codec::default());

    let client_socket = TcpStream::connect(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565))
        .await
        .unwrap();
    client_socket.set_nodelay(true).unwrap();

    let mut framed_client_socket = Framed::new(client_socket, Codec::default());

    match next(&mut framed_socket).await.decode() {
        c2s::HandshakePacket::Intention {
            protocol_version,
            host_name,
            port,
            intention,
        } => match intention {
            Intention::Status => {
                encode_and_send(
                    &mut framed_socket,
                    &c2s::HandshakePacket::Intention {
                        protocol_version,
                        host_name,
                        port,
                        intention: Intention::Status,
                    },
                )
                .await;

                let packet = next(&mut framed_socket).await.decode();
                if matches!(packet, c2s::StatusPacket::StatusRequest { .. }) {
                    encode_and_send(&mut framed_client_socket, &packet).await;
                } else {
                    unimplemented!()
                }

                let packet = next(&mut framed_client_socket).await.decode();
                if matches!(packet, s2c::StatusPacket::StatusResponse { .. }) {
                    encode_and_send(&mut framed_socket, &packet).await;
                } else {
                    unimplemented!()
                }

                let packet = next(&mut framed_socket).await.decode();
                if matches!(packet, c2s::StatusPacket::PingRequest { .. }) {
                    encode_and_send(&mut framed_client_socket, &packet).await;
                } else {
                    unimplemented!()
                }

                let packet = next(&mut framed_client_socket).await.decode();
                if matches!(packet, s2c::StatusPacket::PongResponse { .. }) {
                    encode_and_send(&mut framed_socket, &packet).await;
                } else {
                    unimplemented!()
                }
            }
            Intention::Login => {
                encode_and_send(
                    &mut framed_client_socket,
                    &c2s::HandshakePacket::Intention {
                        protocol_version,
                        host_name,
                        port,
                        intention: Intention::Login,
                    },
                )
                .await;

                let packet = next(&mut framed_socket).await.decode();
                if matches!(packet, c2s::LoginPacket::Hello { .. }) {
                    encode_and_send(&mut framed_client_socket, &packet).await;
                } else {
                    unimplemented!()
                }

                match next(&mut framed_client_socket).await.decode() {
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
                        encode_and_send(
                            &mut framed_client_socket,
                            &c2s::LoginPacket::Key {
                                key: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &key)
                                    .unwrap(),
                                nonce: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &nonce)
                                    .unwrap(),
                            },
                        )
                        .await;
                        framed_client_socket.codec_mut().enable_encryption(&key);

                        loop {
                            match next(&mut framed_client_socket).await.decode() {
                                s2c::LoginPacket::LoginCompression {
                                    compression_threshold,
                                } => {
                                    framed_client_socket.codec_mut().enable_compression(
                                        Compression::default(),
                                        compression_threshold.0 as u16,
                                    );
                                    encode_and_send(
                                        &mut framed_socket,
                                        &s2c::LoginPacket::LoginCompression {
                                            compression_threshold,
                                        },
                                    )
                                    .await;
                                    framed_socket.codec_mut().enable_compression(
                                        Compression::default(),
                                        compression_threshold.0 as u16,
                                    );
                                }
                                packet => {
                                    if matches!(packet, s2c::LoginPacket::GameProfile(..)) {
                                        encode_and_send(&mut framed_socket, &packet).await;
                                        break;
                                    } else {
                                        unimplemented!()
                                    }
                                }
                            }
                        }
                        tokio::spawn(async move {
                            loop {
                                tokio::select! {
                                    packet = next(&mut framed_socket) => {
                                        let packet = packet.decode::<c2s::GamePacket>();
                                        println!("Recv: {:?}", packet);
                                        encode_and_send(&mut framed_client_socket, &packet).await;
                                    }
                                    packet = next(&mut framed_client_socket) => {
                                        let packet = packet.decode::<s2c::GamePacket>();
                                        println!("Send: {:?}", packet);
                                        encode_and_send(&mut framed_client_socket, &packet).await;
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
