use std::path::Path;
use bevy::prelude::*;
use tesseract_protocol::types::{BitStorage, PalettedContainer};
use tesseract_savegame::chunk::Chunk as SavegameChunk;
use tesseract_savegame::region::RegionStorage;
use crate::level;
use crate::registry::BlockStateRegistry;

#[derive(Component)]
pub struct Persistence {
    region_storage: RegionStorage
}

impl Persistence {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            region_storage: RegionStorage::new(path)
        }
    }
}

pub fn load_chunks(
    block_state_registry: Res<BlockStateRegistry>,
    mut commands: Commands,
    mut levels: Query<&mut Persistence>,
    chunks: Query<(Entity, &level::chunk::Position, &Parent), Without<level::chunk::Terrain>>,
) {
    for (chunk, chunk_position, level) in chunks.iter() {
        let region_storage = &mut levels.get_mut(level.get()).unwrap().region_storage;
        if let Some(savegame_chunk_data) = region_storage.read(chunk_position.0) {
            let savegame_chunk = tesseract_nbt::de::from_slice::<SavegameChunk>(&mut savegame_chunk_data.as_slice()).unwrap();
            let mut sections = Vec::new();
            for y in 0..16 {
                sections.push(if let Some(savegame_chunk_section) = savegame_chunk.sections.get(y) {
                    if let Some(data) = &savegame_chunk_section.block_states.data {
                        if savegame_chunk_section.block_states.palette.is_empty() {
                            PalettedContainer::Global(BitStorage::from_data(16 * 16 * 16, data.clone()))
                        } else {
                            PalettedContainer::Linear {
                                palette: savegame_chunk_section.block_states.palette.iter().map(|entry| block_state_registry.id(&entry.name())).collect(),
                                storage: BitStorage::from_data(16 * 16 * 16, data.clone()),
                                linear_max_bits: 0,
                                global_bits: 0,
                            }
                        }
                    } else {
                        PalettedContainer::SingleValue {
                            value: block_state_registry.id(&savegame_chunk_section.block_states.palette.first().unwrap().name()),
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
                });
            }
            commands.entity(chunk).insert(level::chunk::Terrain { sections });
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
            commands.entity(chunk).insert(level::chunk::Terrain { sections });
        }
    }
}
