use crate::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusRequest(StatusRequest),
    PingRequest(PingRequest),
}

#[derive(Encode, Decode)]
pub struct StatusRequest;

#[derive(Encode, Decode)]
pub struct PingRequest {
    pub time: u64,
}
