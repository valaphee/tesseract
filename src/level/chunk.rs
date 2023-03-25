use std::collections::HashMap;

use bevy::prelude::*;

use tesseract_protocol::types::PalettedContainer;

use crate::{actor, replication};

/// All required components to describe a chunk
#[derive(Bundle)]
pub struct ChunkBundle {
    position: Position,
    replication: replication::Replication,
}

impl ChunkBundle {
    pub fn new(position: IVec2) -> Self {
        Self {
            position: Position(position),
            replication: default(),
        }
    }

    pub fn with_subscriber(position: IVec2, subscriber: Entity) -> Self {
        Self {
            position: Position(position),
            replication: replication::Replication::with_subscriber(subscriber),
        }
    }
}

/// Chunk by position (Level)
#[derive(Default, Component)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

/// Position of the chunk in the level (Chunk)
#[derive(Component)]
pub struct Position(pub IVec2);

/// Keeps the hierarchy of actors in chunks consistent
pub fn update_hierarchy(
    mut commands: Commands,
    mut levels: Query<&mut LookupTable>,
    chunks: Query<(&Position, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    // early return
    for (actor, actor_position, level_or_chunk) in actors.iter() {
        let chunk_position = IVec2::new(
            (actor_position.0[0] as i32) >> 4,
            (actor_position.0[2] as i32) >> 4,
        );
        let level = (if let Ok((position, level)) = chunks.get(level_or_chunk.get()) {
            // skip actors where the chunk hasn't changed
            if position.0 == chunk_position {
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
            warn!("Parent of actor is neither a level nor a chunk")
        }
    }
}

/// Terrain (Chunk)
#[derive(Component)]
pub struct Terrain {
    pub sections: Vec<(
        PalettedContainer<4096, 4, 8, 15>,
        PalettedContainer<64, 3, 3, 6>,
    )>,
}
