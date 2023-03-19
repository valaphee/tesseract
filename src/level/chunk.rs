use bevy::{prelude::*, utils::HashMap};

use crate::actor;

#[derive(Default, Component)]
pub struct Lut(pub HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct Position(pub IVec2);

pub fn update_hierarchy(
    mut commands: Commands,
    mut chunk_luts: Query<&mut Lut>,
    chunks: Query<(&Position, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    for (actor, actor_position, level_or_chunk) in actors.iter() {
        let chunk_position = IVec2::new(
            (actor_position.0[0] as i32) >> 4,
            (actor_position.0[2] as i32) >> 4,
        );
        let level = (if let Ok((position, level)) = chunks.get(level_or_chunk.get()) {
            // Skip actors where the chunk hasn't changed
            if position.0 == chunk_position {
                continue;
            }

            level
        } else {
            level_or_chunk
        })
        .get();

        if let Ok(mut chunk_lut) = chunk_luts.get_mut(level) {
            if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn(Position(chunk_position))
                    .set_parent(level)
                    .add_child(actor)
                    .id();
                chunk_lut.0.insert(chunk_position, chunk);
            }
        } else {
            warn!("Parent is neither a level nor a chunk")
        }
    }
}
