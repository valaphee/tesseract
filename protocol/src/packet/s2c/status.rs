use crate::{
    types::{Json, Status},
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum StatusPacket {
    StatusResponse { status: Json<Status> },
    PongResponse { time: i64 },
}
