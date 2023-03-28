use crate::{
    types::{Intention, VarI32},
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum HandshakePacket {
    Intention {
        protocol_version: VarI32,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}
