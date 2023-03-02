use bevy::math::DVec3;
use bevy::prelude::*;

/// Actor: Position in the dimension
#[derive(Component)]
pub struct Position(pub DVec3);

/// Actor: Rotation
#[derive(Component)]
pub struct Rotation(pub Vec2);

/// Actor: Head rotation
#[derive(Component)]
pub struct HeadRotation(pub f32);
