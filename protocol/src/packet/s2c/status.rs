use crate::{types::{Json, Status}, Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusResponse {
        status: Json<Status>,
    },
    PongResponse {
        time: i64,
    },
}
