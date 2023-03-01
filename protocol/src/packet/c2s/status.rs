use crate::{Decode, Encode};

#[derive(Clone, Debug, Encode, Decode)]
pub enum StatusPacket {
    StatusRequest,
    PingRequest { time: i64 },
}
