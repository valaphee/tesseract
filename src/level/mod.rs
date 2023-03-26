use bevy::prelude::*;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
pub struct LevelBundle {
    pub level: Level,
    pub age_and_time: AgeAndTime,
    pub chunks: chunk::LookupTable,
}

/// Required properties (Level)
#[derive(Component)]
pub struct Level {
    pub name: String,
    pub dimension_type: String,
}

#[derive(Default, Component)]
pub struct AgeAndTime {
    pub age: u64,
    pub time: u64,
}

pub fn update_time(mut levels: Query<&mut AgeAndTime>) {
    for mut time in levels.iter_mut() {
        time.age += 1;
        time.time += 1;
    }
}
