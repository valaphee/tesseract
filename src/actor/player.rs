use bevy::{math::DVec3, prelude::*};

use crate::{actor, level, replication};

#[derive(Bundle)]
pub struct PlayerBundle {
    pub actor: actor::Actor,
    pub position: actor::Position,
    pub rotation: actor::Rotation,
    pub head_rotation: actor::HeadRotation,
    pub interaction: Interaction,
}

#[derive(Component, Default)]
pub enum Interaction {
    #[default]
    None,
    BlockBreak(IVec3),
}

#[allow(clippy::type_complexity)]
pub fn initialize(
    mut commands: Commands,
    levels: Query<Entity, With<level::Level>>,
    players: Query<
        (Entity, &replication::Connection),
        (Added<replication::Connection>, Without<actor::Actor>),
    >,
) {
    for (player, connection) in players.iter() {
        commands
            .entity(player)
            .insert((PlayerBundle {
                actor: actor::Actor {
                    id: connection.user().id,
                    type_: "minecraft:player".into(),
                },
                position: actor::Position(DVec3::new(0.0, 127.0, 0.0)),
                rotation: default(),
                head_rotation: default(),
                interaction: default(),
            },))
            .set_parent(levels.single());
    }
}

pub fn update_interactions(
    levels: Query<&level::chunk::LookupTable>,
    mut chunks: Query<(&level::chunk::Chunk, &mut level::chunk::Terrain, &Parent)>,
    mut players: Query<(&mut Interaction, &Parent), Changed<Interaction>>,
) {
    for (mut interaction, chunk) in players.iter_mut() {
        #[allow(clippy::single_match)]
        match *interaction {
            Interaction::BlockBreak(position) => {
                if let Ok((chunk_base, terrain, level)) = chunks.get_mut(chunk.get()) {
                    let chunk_position = IVec2::new(position.x >> 4, position.z >> 4);
                    let terrain = if chunk_base.0 == chunk_position {
                        Some(terrain)
                    } else {
                        levels.get(level.get()).ok().and_then(|chunk_lut| chunk_lut.0.get(&chunk_position).and_then(|chunk| chunks.get_component_mut::<level::chunk::Terrain>(*chunk).ok()))
                    };

                    if let Some(mut terrain) = terrain {
                        terrain.set_block_state(
                            position.x as u8,
                            position.y as i16,
                            position.z as u8,
                            0,
                        );
                    }
                }

                *interaction = Interaction::None;
            }
            _ => {}
        }
    }
}
