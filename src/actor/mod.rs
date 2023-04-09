use bevy::{math::DVec3, prelude::*};
use uuid::Uuid;

pub mod player;

/// All required components to describe an actor
#[derive(Bundle)]
pub struct ActorBundle {
    pub base: Base,
    pub position: Position,
    pub rotation: Rotation,
}

/// Required properties (part of Actor)
#[derive(Component)]
pub struct Base {
    pub id: Uuid,
}

/// Position of the actor in the level (part of Actor)
#[derive(Component)]
pub struct Position(pub DVec3);

/// Rotation (part of Actor)
#[derive(Component, Default)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}
