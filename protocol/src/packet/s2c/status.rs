use crate::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusResponse(StatusResponse),
    PongResponse(PongResponse),
}

#[derive(Encode, Decode)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Encode, Decode)]
pub struct PongResponse {
    pub time: u64,
}
