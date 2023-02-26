use crate::{Decode, Encode, VarInt};

#[derive(Encode, Decode)]
pub enum ClientboundPacket {
    StatusResponse { status: String },
    PongResponse { time: u64 },
}

#[derive(Encode, Decode)]
pub enum ServerboundPacket {
    StatusRequest,
    PingRequest { time: u64 },
}
