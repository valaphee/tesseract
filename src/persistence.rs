use std::path::Path;

use bevy::prelude::*;

use tesseract_protocol::types::{
    BitStorage, DimensionType, MonsterSpawnLightLevel, PalettedContainer,
};
use tesseract_savegame::{chunk::Chunk as RegionChunk, region::RegionStorage};

use crate::{level, registry::BlockStateRegistry};

#[derive(Default)]
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_levels)
            .add_systems(PreUpdate, load_chunks);
    }
}

#[derive(Component)]
struct Persistence {
    region_storage: RegionStorage,
}

impl Persistence {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            region_storage: RegionStorage::new(path),
        }
    }
}

/// System for initially spawning all levels
fn spawn_levels(mut commands: Commands) {
    commands.spawn((
        level::LevelBundle {
            level: level::Level {
                name: "minecraft:overworld".to_string(),
                dimension: DimensionType {
                    fixed_time: None,
                    has_skylight: true,
                    has_ceiling: false,
                    ultrawarm: false,
                    natural: true,
                    coordinate_scale: 1.0,
                    bed_works: true,
                    respawn_anchor_works: true,
                    min_y: 0,
                    height: 16 * 16,
                    logical_height: 16 * 16,
                    infiniburn: "#minecraft:infiniburn_overworld".to_string(),
                    effects: "minecraft:overworld".to_string(),
                    ambient_light: 1.0,
                    piglin_safe: true,
                    has_raids: true,
                    monster_spawn_light_level: MonsterSpawnLightLevel::Scalar(0),
                    monster_spawn_block_light_limit: 0,
                },
            },
            lookup_table: default(),
        },
        Persistence::new("/home/valaphee/.minecraft/saves/Neue Welt2/region"),
    ));
}

fn load_chunks(
    block_state_registry: Res<BlockStateRegistry>,
    mut commands: Commands,
    mut levels: Query<&mut Persistence>,
    chunks: Query<(Entity, &level::chunk::Position, &Parent), Without<level::chunk::Terrain>>,
) {
    for (chunk, chunk_position, level) in chunks.iter() {
        let region_storage = &mut levels.get_mut(level.get()).unwrap().region_storage;
        if let Some(savegame_chunk_data) = region_storage.read(chunk_position.0) {
            let region_chunk =
                tesseract_nbt::de::from_slice::<RegionChunk>(&mut savegame_chunk_data.as_slice())
                    .unwrap();
            let mut sections = Vec::new();
            for y in 0..16 {
                sections.push(
                    if let Some(region_chunk_section) = region_chunk.sections.get(y) {
                        if let Some(data) = &region_chunk_section.block_states.data {
                            if region_chunk_section.block_states.palette.is_empty() {
                                PalettedContainer::Global(BitStorage::from_data(
                                    16 * 16 * 16,
                                    data.clone(),
                                ))
                            } else {
                                PalettedContainer::Linear {
                                    palette: region_chunk_section
                                        .block_states
                                        .palette
                                        .iter()
                                        .map(|entry| block_state_registry.id(&entry.name()))
                                        .collect(),
                                    storage: BitStorage::from_data(16 * 16 * 16, data.clone()),
                                    linear_max_bits: 0,
                                    global_bits: 0,
                                }
                            }
                        } else {
                            PalettedContainer::SingleValue {
                                value: block_state_registry.id(&region_chunk_section
                                    .block_states
                                    .palette
                                    .first()
                                    .unwrap()
                                    .name()),
                                storage_size: 16 * 16 * 16,
                                linear_min_bits: 4,
                                linear_max_bits: 8,
                                global_bits: 15,
                            }
                        }
                    } else {
                        PalettedContainer::SingleValue {
                            value: 0,
                            storage_size: 16 * 16 * 16,
                            linear_min_bits: 4,
                            linear_max_bits: 8,
                            global_bits: 15,
                        }
                    },
                );
            }
            commands
                .entity(chunk)
                .insert(level::chunk::Terrain { sections });
        } else {
            let mut sections = Vec::new();
            for _ in 0..16 {
                sections.push(PalettedContainer::SingleValue {
                    value: 0,
                    storage_size: 16 * 16 * 16,
                    linear_min_bits: 4,
                    linear_max_bits: 8,
                    global_bits: 15,
                });
            }
            commands
                .entity(chunk)
                .insert(level::chunk::Terrain { sections });
        }
    }
}
