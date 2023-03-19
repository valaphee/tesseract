use bevy::{prelude::*, utils::HashMap};

use crate::actor;

#[derive(Default, Component)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct Position(pub IVec2);

#[derive(Component)]
pub struct Replication {
    pub initial: Vec<Entity>,
    pub subsequent: Vec<Entity>,

    pub children: Vec<Entity>,
}

pub fn update_hierarchy(
    mut commands: Commands,
    mut chunk_luts: Query<&mut LookupTable>,
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

        if let Ok(mut chunk_lut) = chunk_luts.get_mut(level) {
            if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn((
                        Position(chunk_position),
                        Replication {
                            initial: vec![actor],
                            subsequent: vec![],
                            children: vec![],
                        },
                    ))
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

pub fn update_replication(mut chunks: Query<&mut Replication, Changed<Replication>>) {
    for mut chunk_replication in chunks.iter_mut() {
        if !chunk_replication.initial.is_empty() {
            let initial = chunk_replication.initial.clone();
            chunk_replication.subsequent.extend_from_slice(&initial);
            chunk_replication.initial.clear()
        }
    }
}
