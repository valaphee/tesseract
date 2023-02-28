use serde::{Deserialize, Serialize};

use crate::{Decode, Encode, types::Json};

#[derive(Encode, Decode)]
pub enum StatusPacket {
    StatusResponse(StatusResponsePacket),
    PongResponse(PongResponsePacket),
}

#[derive(Encode, Decode)]
pub struct StatusResponsePacket {
    pub status: Json<Status>,
}

#[derive(Encode, Decode)]
pub struct PongResponsePacket {
    pub time: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusPlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<StatusVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    pub previews_chat: bool
}

#[derive(Serialize, Deserialize)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusPlayersSample>
}

#[derive(Serialize, Deserialize)]
pub struct StatusPlayersSample {
    pub id: String,
    pub name: String,
}
