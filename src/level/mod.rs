use std::borrow::Cow;

use bevy::prelude::*;

pub mod chunk;

/// All required components to describe a level
#[derive(Bundle)]
pub struct LevelBundle {
    pub base: Base,
    pub age_and_time: AgeAndTime,
    pub chunks: chunk::LookupTable,
}

/// Required properties (part of Level)
#[derive(Component)]
pub struct Base {
    pub name: Cow<'static, str>,
    pub dimension_type: Cow<'static, str>,
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
