use std::borrow::Cow;

use bevy::{math::DVec3, prelude::*};
use uuid::Uuid;

pub mod player;

/// All required components to describe an actor
#[derive(Bundle)]
pub struct ActorBundle {
    pub base: Base,
    pub position: Position,
    pub rotation: Rotation,
    pub head_rotation: HeadRotation,
}

/// Required properties (part of Actor)
#[derive(Component)]
pub struct Base {
    pub id: Uuid,
    pub type_: Cow<'static, str>,
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

/// Head rotation (part of Actor)
#[derive(Component, Default)]
pub struct HeadRotation {
    pub head_yaw: f32,
}

#[derive(Component)]
pub struct Armor {
    pub head: Option<Entity>,
    pub torso: Option<Entity>,
    pub legs: Option<Entity>,
    pub feet: Option<Entity>,
}

#[derive(Component)]
pub struct Hand {
    pub main_hand: Option<Entity>,
    pub off_hand: Option<Entity>,
}
