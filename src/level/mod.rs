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
    name: Cow<'static, str>,
    dimension_type: Cow<'static, str>,
}

impl Base {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        dimension_type: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            name: name.into(),
            dimension_type: dimension_type.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dimension_type(&self) -> &str {
        &self.dimension_type
    }
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
