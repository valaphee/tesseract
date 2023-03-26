use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod chunk;
pub mod entity;
pub mod region;

#[derive(Serialize, Deserialize)]
pub struct PalettedContainer<T> {
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
