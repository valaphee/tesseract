use crate::{types::VarInt, Decode, Encode};

#[derive(Encode, Decode)]
pub enum HandshakePacket {
    Intention(IntentionPacket),
}

#[derive(Encode, Decode)]
pub struct IntentionPacket {
    pub protocol_version: VarInt,
    pub host_name: String,
    pub port: u16,
    pub intention: Intention,
}

#[derive(Encode, Decode)]
pub enum Intention {
    Game,
    Status,
    Login,
}
