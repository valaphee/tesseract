use bevy::{math::DVec3, prelude::*};
use uuid::Uuid;

pub mod player;

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
#[derive(Component, Default)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

/// Head rotation (Actor)
#[derive(Component, Default)]
pub struct HeadRotation {
    pub head_yaw: f32,
}
