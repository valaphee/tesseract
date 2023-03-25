use bevy::prelude::*;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
pub struct LevelBundle {
    pub name: Name,
    pub dimension_type: DimensionType,
    pub lookup_table: chunk::LookupTable,
}

#[derive(Component)]
pub struct DimensionType(pub String);
