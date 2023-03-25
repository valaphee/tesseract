use bevy::prelude::*;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
pub struct LevelBundle {
    pub level: Level,
    pub chunks: chunk::LookupTable,
}

#[derive(Component)]
pub struct Level {
    pub name: String,
    pub dimension_type: String,
}
