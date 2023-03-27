use std::collections::HashMap;

use bevy::prelude::*;

use tesseract_protocol::types::PalettedContainer;

use crate::{actor, replication};

/// All required components to describe a chunk
#[derive(Bundle)]
pub struct ChunkBundle {
    chunk: Chunk,
    replication: replication::Replication,
}

impl ChunkBundle {
    pub fn new(position: IVec2) -> Self {
        Self {
            chunk: Chunk(position),
            replication: default(),
        }
    }

    pub fn with_subscriber(position: IVec2, subscriber: Entity) -> Self {
        Self {
            chunk: Chunk(position),
            replication: replication::Replication::with_subscriber(subscriber),
        }
    }
}

/// Chunk by position look-up table (Level)
#[derive(Default, Component)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

/// Required properties (Chunk)
#[derive(Component)]
pub struct Chunk(pub IVec2);

/// Keeps the hierarchy of actors in chunks consistent
/// - if chunk has changed, place actor into new chunk
/// - if new chunk does not exist, create new chunk
pub fn update_hierarchy(
    mut commands: Commands,
    mut levels: Query<&mut LookupTable>,
    chunks: Query<(&Chunk, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    // early return
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

/// Terrain (Chunk)
#[derive(Component)]
pub struct Terrain(pub Vec<TerrainSection>);

pub struct TerrainSection {
    pub block_states: PalettedContainer<{ 16 * 16 * 16 }, 4, 8, 15>,
    pub biomes: PalettedContainer<{ 4 * 4 * 4 }, 3, 3, 6>,

    pub block_state_updates: Vec<u16>,
}

impl Terrain {
    pub fn block_state(&self, position: IVec3) -> u32 {
        let section = &self.0[(position.y >> 4) as usize];
        let index = (position.y as u16 & 0xF) << 8
            | (position.z as u16 & 0xF) << 4
            | (position.x as u16 & 0xF);
        section.block_states.get(index as u32)
    }

    pub fn set_block_state(&mut self, position: IVec3, value: u32) {
        let section = &mut self.0[(position.y >> 4) as usize];
        let index = (position.y as u16 & 0xF) << 8
            | (position.z as u16 & 0xF) << 4
            | (position.x as u16 & 0xF);
        if section.block_states.get_and_set(index as u32, value) != value {
            section.block_state_updates.push(index);
        }
    }
}
