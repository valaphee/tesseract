use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
    pub block_states: ChunkSectionPalette<BlockState>,
    pub biomes: ChunkSectionPalette<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSectionPalette<T> {
    pub palette: Vec<T>,
    pub data: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize)]
pub struct BlockState {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(
        rename = "Properties",
        skip_serializing_if = "BTreeMap::is_empty",
        default
    )]
    pub properties: BTreeMap<String, String>,
}

impl BlockState {
    pub fn name(&self) -> String {
        format!(
            "{}[{}]",
            self.name,
            self.properties
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
