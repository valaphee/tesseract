use crate::{
    types::{Intention, VarI32},
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum HandshakePacket {
    Intention {
        protocol_version: VarI32,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}
