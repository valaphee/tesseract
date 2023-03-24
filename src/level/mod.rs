use bevy::prelude::*;

use tesseract_protocol::types::DimensionType;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
pub struct LevelBundle {
    pub level: Level,
    pub lookup_table: chunk::LookupTable,
}

/// Level (Level)
#[derive(Component)]
pub struct Level {
    pub name: String,
    pub dimension: DimensionType,
}
