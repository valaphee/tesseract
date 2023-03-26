use bevy::{math::DVec3, prelude::*};
use uuid::Uuid;

use crate::{level, replication};

/// All required components to describe an actor
#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub position: Position,
    pub rotation: Rotation,
    pub head_rotation: HeadRotation,
}

/// Required properties (Actor)
#[derive(Component)]
pub struct Actor {
    pub id: Uuid,
    pub type_: String,
}

/// Position of the actor in the level (Actor)
#[derive(Component)]
pub struct Position(pub DVec3);

/// Rotation (Actor)
#[derive(Component)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

/// Head rotation (Actor)
#[derive(Component)]
pub struct HeadRotation {
    pub head_yaw: f32,
}

#[allow(clippy::type_complexity)]
pub fn initialize_players(
    mut commands: Commands,
    levels: Query<Entity, With<level::Level>>,
    players: Query<
        (Entity, &replication::Connection),
        (Added<replication::Connection>, Without<Actor>),
    >,
) {
    for (player, connection) in players.iter() {
        commands
            .entity(player)
            .insert(ActorBundle {
                actor: Actor {
                    id: connection.user().id,
                    type_: "minecraft:player".into(),
                },
                position: Position(DVec3::new(0.0, 127.0, 0.0)),
                rotation: Rotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
                head_rotation: HeadRotation { head_yaw: 0.0 },
            })
            .set_parent(levels.single());
    }
}
