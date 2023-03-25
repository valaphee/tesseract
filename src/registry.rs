use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    path::Path,
};

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use tesseract_protocol::types::{Registry, RegistryEntry};

#[derive(Resource)]
pub struct Registries {
    registries: HashMap<String, RegistryReport>,
}

impl Registries {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            registries: serde_json::from_reader::<_, HashMap<String, RegistryReport>>(
                File::open(path).unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn id(&self, type_: &str, value: &str) -> u32 {
        self.registries
            .get(type_)
            .and_then(|registry| registry.entries.get(value).map(|entry| entry.protocol_id))
            .unwrap_or(0)
    }
}

#[derive(Serialize, Deserialize)]
struct RegistryReport {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    default: Option<String>,
    entries: HashMap<String, RegistryEntryReport>,
}

#[derive(Serialize, Deserialize)]
struct RegistryEntryReport {
    protocol_id: u32,
}

#[derive(Resource)]
pub struct BlockStateRegistry {
    id_by_name: HashMap<String, u32>,
}

impl BlockStateRegistry {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let report =
            serde_json::from_reader::<_, HashMap<String, BlockReport>>(File::open(path).unwrap())
                .unwrap();
        let mut id_by_name = HashMap::with_capacity(report.len() * 16);
        for (name, block_report) in report {
            for block_state_report in block_report.states {
                if block_state_report.default {
                    id_by_name.insert(name.clone(), block_state_report.id);
                }
                id_by_name.insert(
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
        Self { id_by_name }
    }

    pub fn id(&self, value: &str) -> u32 {
        *self.id_by_name.get(value).unwrap_or(&0)
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

#[derive(Resource)]
pub struct DataRegistry<T> {
    registry: Registry<T>,

    id_by_name: HashMap<String, u32>,
}

impl<T: DeserializeOwned> DataRegistry<T> {
    pub fn new<P: AsRef<Path>>(path: P, type_: &str) -> Self {
        let mut paths = std::fs::read_dir(path)
            .unwrap()
            .map(|path| path.unwrap())
            .collect::<Vec<_>>();
        paths.sort_by_key(|path| path.file_name());

        let mut registry = Vec::with_capacity(paths.len());
        let mut id_by_name = HashMap::with_capacity(paths.len());
        for (id, path) in paths.into_iter().enumerate() {
            let name = path
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let id = id as u32;
            registry.push(RegistryEntry {
                name: name.clone(),
                id,
                element: serde_json::from_reader(File::open(path.path()).unwrap()).unwrap(),
            });
            id_by_name.insert(name.to_string(), id);
        }

        Self {
            registry: Registry {
                type_: type_.to_string(),
                value: registry,
            },
            id_by_name,
        }
    }

    pub fn registry(&self) -> &Registry<T> {
        &self.registry
    }

    pub fn id(&self, value: &str) -> u32 {
        *self.id_by_name.get(value).unwrap_or(&0)
    }
}
