use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    path::Path,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct BlockStateRegistry {
    id_by_value: HashMap<String, u32>,
}

impl BlockStateRegistry {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let report =
            serde_json::from_reader::<_, HashMap<String, BlockReport>>(File::open(path).unwrap())
                .unwrap();
        let mut id_by_value = HashMap::with_capacity(report.len() * 16);
        for (name, block_report) in report {
            for block_state_report in block_report.states {
                if block_state_report.default {
                    id_by_value.insert(name.clone(), block_state_report.id);
                }
                id_by_value.insert(
                    format!(
                        "{name}[{}]",
                        block_state_report
                            .properties
                            .iter()
                            .map(|(key, value)| format!("{key}={value}"))
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    block_state_report.id,
                );
            }
        }
        Self { id_by_value }
    }

    pub fn id(&self, value: &str) -> u32 {
        return *self.id_by_value.get(value).unwrap_or(&0);
    }
}

#[derive(Serialize, Deserialize)]
struct BlockReport {
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    properties: BTreeMap<String, Vec<String>>,
    states: Vec<BlockStateReport>,
}

#[derive(Serialize, Deserialize)]
struct BlockStateReport {
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    properties: BTreeMap<String, String>,
    id: u32,
    #[serde(default)]
    default: bool,
}
