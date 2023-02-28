use crate::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusRequest,
    PingRequest {
        time: i64,
    },
}
