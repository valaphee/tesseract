use crate::{
    types::{Intention, VarI32},
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum HandshakePacket {
    Intention {
        #[using(VarI32)]
        protocol_version: i32,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}
