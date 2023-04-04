use serde::{Deserialize, Serialize};

use crate::{BlockState, PalettedContainer};

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    #[serde(rename = "DataVersion")]
    pub version: i32,
    #[serde(rename = "xPos")]
    pub x: i32,
    #[serde(rename = "yPos")]
    pub y: i32,
    #[serde(rename = "zPos")]
    pub z: i32,
    pub sections: Vec<ChunkSection>,
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSection {
    #[serde(rename = "Y")]
    pub y: i8,
    pub block_states: PalettedContainer<BlockState>,
    pub biomes: PalettedContainer<String>,
}
