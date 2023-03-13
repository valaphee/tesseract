use crate::{
    types::{Intention, VarInt32},
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum HandshakePacket {
    Intention {
        protocol_version: VarInt32,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}
