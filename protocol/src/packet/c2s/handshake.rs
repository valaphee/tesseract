use crate::{types::VarInt, Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum HandshakePacket {
    Intention(IntentionPacket),
}

#[derive(Debug, Encode, Decode)]
pub struct IntentionPacket {
    pub protocol_version: VarInt,
    pub host_name: String,
    pub port: u16,
    pub intention: IntentionPacketIntention,
}

#[derive(Debug, Encode, Decode)]
pub enum IntentionPacketIntention {
    Game,
    Status,
    Login,
}
