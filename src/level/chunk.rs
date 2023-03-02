use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use crate::actor;

/// Dimension: Look-up table for chunk positions to entities
#[derive(Default, Component)]
pub struct LookupTable(HashMap<IVec2, Entity>);

/// Chunk: Position of the chunk in the dimension
#[derive(Component)]
pub struct Position(pub IVec2);

/// Updates the hierarchy of actors in chunks according to their chunk position
pub fn update_hierarchy(
    mut commands: Commands,
    dimensions: Query<&LookupTable>,
    chunks: Query<(&Position, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>
) {
    for (actor, actor_position, dimension_or_chunk) in actors.iter() {
        let chunk_position = IVec2::new((actor_position.0[0] as i32) >> 4, (actor_position.0[2] as i32) >> 4);
        let dimension = (if let Ok((position, dimension)) = chunks.get(dimension_or_chunk.get()) {
            // Skip actors where the chunk hasn't changed
            if position.0 == chunk_position {
                continue;
            }

            dimension
        } else {
            dimension_or_chunk
        }).get();

        if let Ok(dimension) = dimensions.get(dimension) {
            if let Some(&chunk) = dimension.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            }
        }
    }
}

/// Chunk: Entities which are currently seeing this chunk
#[derive(Component)]
struct ViewedBy(HashSet<Entity>);

/// Actor: View
#[derive(Component)]
pub struct View(pub u8);

/// Actor: Chunks are currently in view of the actor
#[derive(Default, Component)]
pub struct InView(HashSet<IVec2>);
