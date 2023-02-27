use crate::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusResponse(StatusResponsePacket),
    PongResponse(PongResponsePacket),
}

#[derive(Encode, Decode)]
pub struct StatusResponsePacket {
    pub status: String,
}

#[derive(Encode, Decode)]
pub struct PongResponsePacket {
    pub time: i64,
}
