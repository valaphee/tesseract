use crate::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusRequest(StatusRequestPacket),
    PingRequest(PingRequestPacket),
}

#[derive(Encode, Decode)]
pub struct StatusRequestPacket;

#[derive(Encode, Decode)]
pub struct PingRequestPacket {
    pub time: i64,
}
