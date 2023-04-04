use std::collections::{BTreeSet, HashMap};

use bevy::prelude::*;

use tesseract_java_protocol::types::PalettedContainer;

use crate::actor;

/// All required components to describe a chunk
#[derive(Bundle)]
pub struct ChunkBundle {
    base: Base,
    queued_updates: UpdateQueue,
}

impl ChunkBundle {
    pub fn new(position: IVec2) -> Self {
        Self {
            base: Base(position),
            queued_updates: Default::default(),
        }
    }
}

/// Chunk by position look-up table (part of Level)
#[derive(Component, Default)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

/// Required properties (part of Chunk)
#[derive(Component)]
pub struct Base(pub IVec2);

/// Keeps the hierarchy of actors in chunks consistent
/// - if chunk has changed, place actor into new chunk
/// - if new chunk does not exist, create new chunk
pub fn update_hierarchy(
    mut commands: Commands,
    mut levels: Query<&mut LookupTable>,
    chunks: Query<(&Base, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    for (actor, actor_position, level_or_chunk) in actors.iter() {
        let chunk_position = IVec2::new(
            (actor_position.0.x as i32) >> 4,
            (actor_position.0.z as i32) >> 4,
        );
        let level = (if let Ok((chunk_base, level)) = chunks.get(level_or_chunk.get()) {
            // skip actors where the chunk hasn't changed
            if chunk_base.0 == chunk_position {
                continue;
            }

            level
        } else {
            level_or_chunk
        })
        .get();

        if let Ok(mut chunk_lut) = levels.get_mut(level) {
            if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn(ChunkBundle::new(chunk_position))
                    .set_parent(level)
                    .add_child(actor)
                    .id();
                chunk_lut.0.insert(chunk_position, chunk);
            }
        } else {
            warn!(
                "Parent ({:?}) of {:?} is neither a level nor a chunk",
                level, actor
            );
        }
    }
}

//======================================================================================== DATA ====

/// Data (part of Chunk)
#[derive(Component)]
pub struct Data {
    pub sections: Vec<DataSection>,
    pub y_offset: u8,
}

pub struct DataSection {
    pub block_states: PalettedContainer<{ 16 * 16 * 16 }, 4, 8, 15>,
    pub biomes: PalettedContainer<{ 4 * 4 * 4 }, 3, 3, 6>,

    pub block_state_changes: BTreeSet<u16>,
}

impl Data {
    pub fn get(&self, x: u8, y: u16, z: u8) -> u32 {
        let section_y = (y >> 4) as usize;
        // x,z are wrapping
        if section_y >= self.sections.len() {
            return 0;
        }

        let section = &self.sections[section_y];
        let index = (y & 0xF) << 8 | (z as u16 & 0xF) << 4 | (x as u16 & 0xF);
        section.block_states.get(index as u32)
    }

    pub fn set(&mut self, x: u8, y: u16, z: u8, value: u32) {
        let section_y = (y >> 4) as usize;
        // x,z are wrapping
        if section_y >= self.sections.len() {
            return;
        }

        let section = &mut self.sections[section_y];
        let index = (y & 0xF) << 8 | (z as u16 & 0xF) << 4 | (x as u16 & 0xF);
        if section.block_states.get_and_set(index as u32, value) != value {
            section.block_state_changes.insert(index);
        }
    }
}

//================================================================================ UPDATE QUEUE ====

#[derive(Component, Default)]
pub struct UpdateQueue(pub Vec<Update>);

#[derive(Eq, PartialEq, Hash)]
pub struct Update(u8, u16);

impl Update {
    pub fn x(&self) -> u8 {
        self.0 & 0xF
    }

    pub fn y(&self) -> u16 {
        self.1
    }

    pub fn z(&self) -> u8 {
        self.0 >> 4 & 0xF
    }
}

pub fn queue_updates(mut chunks: Query<(&Data, &mut UpdateQueue), Changed<Data>>) {
    for (chunk_data, mut chunk_queued_updates) in chunks.iter_mut() {
        chunk_queued_updates.0.clear();
        for (section_y, section) in chunk_data.sections.iter().enumerate() {
            chunk_queued_updates
                .0
                .extend(
                    section
                        .block_state_changes
                        .iter()
                        .map(|&block_state_change| {
                            Update(
                                block_state_change as u8,
                                block_state_change >> 8 | (section_y as u16) << 4,
                            )
                        }),
                );
        }
    }
}
