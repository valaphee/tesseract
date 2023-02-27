use crate::{Decode, Encode, VarInt};

#[derive(Encode, Decode)]
pub enum HandshakePacket {
    Intention(Intention),
}

#[derive(Encode, Decode)]
pub struct Intention {
    pub protocol_version: VarInt,
    pub host_name: String,
    pub port: u16,
    pub intention: IntentionEnum,
}

#[derive(Encode, Decode)]
pub enum IntentionEnum {
    Game,
    Status,
    Login,
}
