use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fs::File,
    path::Path,
};

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use tesseract_base::{block, item};
use tesseract_java_protocol::types::{Biome, DamageType, DimensionType, Registry, RegistryEntry};

#[derive(Default)]
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Registries::new("generated/reports/registries.json"))
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
            ))
            .add_systems(Startup, build_mappings);
    }
}

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

    pub fn id(&self, type_: &str, name: &str) -> u32 {
        self.registries
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

    pub fn registry(&self) -> &Registry<T> {
        &self.registry
    }

    pub fn id(&self, name: &str) -> u32 {
        *self.id_by_name.get(name).unwrap_or(&0)
    }
}

#[derive(Resource)]
pub struct Mappings {
    // block
    pub id_by_block: HashMap<u32, u32>,

    // item
    pub item_by_id: HashMap<u32, u32>,
}

#[derive(Component)]
pub struct RegistryName(Cow<'static, str>);

impl RegistryName {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }
}

fn build_mappings(
    registries: Res<Registries>,

    mut commands: Commands,

    blocks: Query<(Entity, &RegistryName), With<block::Base>>,
    items: Query<(Entity, &RegistryName), With<item::Base>>,
) {
    let report = serde_json::from_reader::<_, HashMap<String, BlockReport>>(
        File::open("generated/reports/blocks.json").unwrap(),
    )
    .unwrap();
    let mut id_by_name = HashMap::with_capacity(report.len() * 16);
    for (name, block_report) in report {
        for block_state_report in block_report.states {
            if block_state_report.properties.is_empty() {
                id_by_name.insert(name.clone(), block_state_report.id);
            } else {
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
    }

    commands.insert_resource(Mappings {
        id_by_block: blocks
            .iter()
            .filter_map(|(block, block_base)| {
                id_by_name
                    .get(block_base.0.as_ref())
                    .map(|&id| (block.index(), id))
            })
            .collect(),
        item_by_id: items
            .iter()
            .map(|(item, item_base)| {
                (
                    registries.id("minecraft:item", item_base.0.split('[').next().unwrap()),
                    item.index(),
                )
            })
            .collect(),
    });
}
