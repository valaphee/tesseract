use std::collections::BTreeSet;

use bevy::prelude::*;

use tesseract_java_protocol::types::PalettedContainer;

use crate::{
    actor,
    hierarchy::{EntityCommandsExt, IndexedChildren, ParentWithIndex},
    replication,
};

/// All required components to describe a chunk
#[derive(Bundle)]
pub struct ChunkBundle {
    pub base: Base,
    pub update_queue: UpdateQueue,

    pub replication: replication::Replication, // TODO: see Replication
}

/// Required properties (part of Chunk)
#[derive(Component)]
pub struct Base;

/// Keeps the hierarchy of actors in chunks consistent
/// - if chunk has changed, place actor into new chunk
/// - if new chunk does not exist, create new chunk
pub fn update_hierarchy(
    mut commands: Commands,

    level_access: Query<&IndexedChildren<IVec2>>,
    chunk_access: Query<&ParentWithIndex<IVec2>>,
    for_actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    for (actor, actor_position, level_or_chunk) in for_actors.iter() {
        let chunk_position = IVec2::new(
            (actor_position.0.x as i32) >> 4,
            (actor_position.0.z as i32) >> 4,
        );
        let level = if let Ok(indexed_chunk) = chunk_access.get(level_or_chunk.get()) {
            // skip actors where the chunk hasn't changed
            if indexed_chunk.index == chunk_position {
                continue;
            }

            indexed_chunk.parent
        } else {
            level_or_chunk.get()
        };

        if let Ok(indexed_chunks) = level_access.get(level) {
            if let Some(&chunk) = indexed_chunks.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn(ChunkBundle {
                        base: Base,
                        update_queue: Default::default(),
                        replication: Default::default(),
                    })
                    .add_child(actor)
                    .id();
                commands
                    .entity(level)
                    .set_indexed_child(chunk_position, Some(chunk));
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

impl Data {
    pub fn new(
        section_count: u8,
        y_offset: u8,
        default_block_id: u32,
        default_biome_id: u32,
    ) -> Self {
        Self {
            sections: {
                let mut sections = vec![];
                for _ in 0..section_count {
                    sections.push(DataSection {
                        block_states: PalettedContainer::Single(default_block_id),
                        biomes: PalettedContainer::Single(default_biome_id),
                        block_state_changes: Default::default(),
                    })
                }
                sections
            },
            y_offset,
        }
    }
}

pub struct DataSection {
    pub block_states: PalettedContainer<{ 16 * 16 * 16 }, 4, 32, 32>,
    pub block_state_changes: BTreeSet<u16>,

    pub biomes: PalettedContainer<{ 4 * 4 * 4 }, 3, 3, 6>,
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
pub struct UpdateQueue(pub Vec<BlockPosition>);

pub fn queue_updates(mut for_chunks: Query<(&Data, &mut UpdateQueue), Changed<Data>>) {
    for (data, mut queued_updates) in for_chunks.iter_mut() {
        queued_updates.0.clear();
        for (section_y, section) in data.sections.iter().enumerate() {
            queued_updates
                .0
                .extend(
                    section
                        .block_state_changes
                        .iter()
                        .map(|&block_state_change| {
                            BlockPosition(
                                block_state_change as u8,
                                block_state_change >> 8 | (section_y as u16) << 4,
                            )
                        }),
                );
        }
    }
}

//====================================================================================== HELPER ====

#[derive(Eq, PartialEq, Hash)]
pub struct BlockPosition(u8, u16);

impl BlockPosition {
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
