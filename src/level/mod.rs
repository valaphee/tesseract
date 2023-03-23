use bevy::prelude::*;

use tesseract_protocol::types::{DimensionType, MonsterSpawnLightLevel};
use crate::persistence::Persistence;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
struct LevelBundle {
    level: Level,
    lookup_table: chunk::LookupTable,
    persistence: Persistence
}

/// Level (Level)
#[derive(Component)]
pub struct Level {
    pub name: String,
    pub dimension: DimensionType,
}

/// System for initially spawning all levels
pub fn spawn_levels(mut commands: Commands) {
    commands.spawn(LevelBundle {
        level: Level {
            name: "minecraft:overworld".to_string(),
            dimension: DimensionType {
                fixed_time: None,
                has_skylight: true,
                has_ceiling: false,
                ultrawarm: false,
                natural: true,
                coordinate_scale: 1.0,
                bed_works: true,
                respawn_anchor_works: true,
                min_y: 0,
                height: 16 * 16,
                logical_height: 16 * 16,
                infiniburn: "#minecraft:infiniburn_overworld".to_string(),
                effects: "minecraft:overworld".to_string(),
                ambient_light: 1.0,
                piglin_safe: true,
                has_raids: true,
                monster_spawn_light_level: MonsterSpawnLightLevel::Scalar(0),
                monster_spawn_block_light_limit: 0,
            },
        },
        lookup_table: default(),
        persistence: Persistence::new("/home/valaphee/.minecraft/saves/Neue Welt/region")
    });
}