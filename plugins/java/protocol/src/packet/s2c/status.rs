use crate::{
    types::{Json, Status},
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum StatusPacket {
    StatusResponse { status: Json<Status> },
    PongResponse { time: i64 },
}
