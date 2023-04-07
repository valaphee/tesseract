use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    path::Path,
};

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use tesseract_java_protocol::types::{Biome, DamageType, DimensionType, Registry, RegistryEntry};

/// Needed for Minecraft: Java Edition persistence & replication
#[derive(Default)]
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlocksReport::new("generated/reports/blocks.json"))
            .insert_resource(RegistriesReport::new("generated/reports/registries.json"))
            .insert_resource(DataRegistry::<DimensionType>::new(
                "generated/data/dimension_type",
                "minecraft:dimension_type",
            ))
            .insert_resource(DataRegistry::<Biome>::new(
                "generated/data/worldgen/biome",
                "minecraft:worldgen/biome",
            ))
            .insert_resource(DataRegistry::<DamageType>::new(
                "generated/data/damage_type",
                "minecraft:damage_type",
            ));
    }
}

#[derive(Resource)]
pub(crate) struct RegistriesReport(HashMap<String, RegistryReport>);

impl RegistriesReport {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self(
            serde_json::from_reader::<_, HashMap<String, RegistryReport>>(
                File::open(path).unwrap(),
            )
            .unwrap(),
        )
    }

    pub(crate) fn id(&self, type_: &str, name: &str) -> u32 {
        self.0
            .get(type_)
            .and_then(|registry| registry.entries.get(name).map(|entry| entry.protocol_id))
            .unwrap_or(0)
    }
}

#[derive(Serialize, Deserialize)]
struct RegistryReport {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    default: Option<String>,
    entries: HashMap<String, RegistryEntryReport>,
}

#[derive(Serialize, Deserialize, Clone)]
struct RegistryEntryReport {
    protocol_id: u32,
}

#[derive(Resource, Debug)]
pub(crate) struct BlocksReport(pub(crate) HashMap<String, BlockReport>);

impl BlocksReport {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self(
            serde_json::from_reader::<_, HashMap<String, BlockReport>>(File::open(path).unwrap())
                .unwrap(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BlockReport {
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub(crate) properties: BTreeMap<String, Vec<String>>,
    pub(crate) states: Vec<BlockStateReport>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BlockStateReport {
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub(crate) properties: BTreeMap<String, String>,
    pub(crate) id: u32,
    #[serde(default)]
    pub(crate) default: bool,
}

#[derive(Resource)]
pub(crate) struct DataRegistry<T> {
    registry: Registry<T>,

    id_by_name: HashMap<String, u32>,
}

impl<T: DeserializeOwned> DataRegistry<T> {
    fn new<P: AsRef<Path>>(path: P, type_: &str) -> Self {
        let mut paths = std::fs::read_dir(path)
            .unwrap()
            .map(|path| path.unwrap())
            .collect::<Vec<_>>();
        paths.sort_by_key(|path| path.file_name());

        let mut registry = Vec::with_capacity(paths.len());
        let mut id_by_name = HashMap::with_capacity(paths.len());
        for (id, path) in paths.into_iter().enumerate() {
            let name = format!(
                "minecraft:{}",
                path.path().file_stem().unwrap().to_str().unwrap()
            );
            let id = id as u32;
            registry.push(RegistryEntry {
                name: name.clone(),
                id,
                element: serde_json::from_reader(File::open(path.path()).unwrap()).unwrap(),
            });
            id_by_name.insert(name, id);
        }

        Self {
            registry: Registry {
                type_: type_.to_string(),
                value: registry,
            },
            id_by_name,
        }
    }

    pub(crate) fn registry(&self) -> &Registry<T> {
        &self.registry
    }

    pub(crate) fn id(&self, name: &str) -> u32 {
        *self.id_by_name.get(name).unwrap_or(&0)
    }
}
