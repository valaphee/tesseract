use crate::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug)]
pub enum StatusPacket {
    StatusRequest,
    PingRequest { time: i64 },
}
