use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use tesseract_protocol::types::{Biome, BitStorage, PalettedContainer};
use tesseract_savegame::{chunk::Chunk as RegionChunk, region::RegionStorage};

use crate::{
    level,
    registry::{BlockStateRegistry, DataRegistry},
};

#[derive(Serialize, Deserialize)]
pub struct PersistencePlugin {
    path: PathBuf,
}

impl Default for PersistencePlugin {
    fn default() -> Self {
        Self {
            path: "levels".into(),
        }
    }
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let path = self.path.clone();

        let spawn_levels = move |mut commands: Commands| {
            commands.spawn((
                level::LevelBundle {
                    name: "minecraft:overworld".into(),
                    dimension_type: level::DimensionType("minecraft:overworld".into()),
                    lookup_table: default(),
                },
                Persistence {
                    region_storage: RegionStorage::new(path.join("overworld/region")),
                },
            ));
        };

        app.add_systems(PreStartup, spawn_levels)
            .add_systems(PreUpdate, load_chunks);
    }
}

#[derive(Component)]
struct Persistence {
    region_storage: RegionStorage,
}

fn load_chunks(
    block_state_registry: Res<BlockStateRegistry>,
    biome_registry: Res<DataRegistry<Biome>>,
    mut commands: Commands,
    mut levels: Query<&mut Persistence>,
    chunks: Query<(Entity, &level::chunk::Position, &Parent), Without<level::chunk::Terrain>>,
) {
    for (chunk, chunk_position, level) in chunks.iter() {
        let region_storage = &mut levels.get_mut(level.get()).unwrap().region_storage;
        if let Some(region_chunk_data) = region_storage.read(chunk_position.0) {
            let region_chunk =
                tesseract_nbt::de::from_slice::<RegionChunk>(&mut region_chunk_data.as_slice())
                    .unwrap();
            let sections = region_chunk
                .sections
                .into_iter()
                .map(|region_chunk_section| {
                    (
                        if let Some(data) = region_chunk_section.block_states.data {
                            if region_chunk_section.block_states.palette.is_empty() {
                                PalettedContainer::Global(BitStorage::from_data(16 * 16 * 16, data))
                            } else {
                                PalettedContainer::Linear {
                                    palette: region_chunk_section
                                        .block_states
                                        .palette
                                        .iter()
                                        .map(|entry| block_state_registry.id(&entry.name()))
                                        .collect(),
                                    storage: BitStorage::from_data(16 * 16 * 16, data),
                                }
                                .fix()
                            }
                        } else {
                            PalettedContainer::SingleValue(
                                block_state_registry.id(&region_chunk_section
                                    .block_states
                                    .palette
                                    .first()
                                    .unwrap()
                                    .name()),
                            )
                        },
                        if let Some(data) = region_chunk_section.biomes.data {
                            if region_chunk_section.biomes.palette.is_empty() {
                                PalettedContainer::Global(BitStorage::from_data(4 * 4 * 4, data))
                            } else {
                                PalettedContainer::Linear {
                                    palette: region_chunk_section
                                        .biomes
                                        .palette
                                        .iter()
                                        .map(|entry| biome_registry.id(entry))
                                        .collect(),
                                    storage: BitStorage::from_data(4 * 4 * 4, data),
                                }
                                .fix()
                            }
                        } else {
                            PalettedContainer::SingleValue(
                                biome_registry.id(region_chunk_section
                                    .biomes
                                    .palette
                                    .first()
                                    .unwrap()),
                            )
                        },
                    )
                })
                .collect::<Vec<_>>();

            commands
                .entity(chunk)
                .insert(level::chunk::Terrain { sections });
        }
    }
}
