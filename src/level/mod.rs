use bevy::prelude::*;

pub mod chunk;

#[derive(Component)]
pub struct Dimension {
    pub name: String,
}
