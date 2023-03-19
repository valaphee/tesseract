use bevy::{math::DVec3, prelude::*};

pub mod connection;

#[derive(Component)]
pub struct Position(pub DVec3);

#[derive(Component)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct HeadRotation {
    pub head_yaw: f32,
}
