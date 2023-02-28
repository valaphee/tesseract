use crate::{
    types::{Intention, VarInt},
    Decode, Encode,
};

#[derive(Debug, Encode, Decode)]
pub enum HandshakePacket {
    Intention {
        protocol_version: VarInt,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}
