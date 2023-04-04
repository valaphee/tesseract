use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;

/// Item by name look-up table
#[derive(Resource)]
pub struct LookupTable(pub HashMap<String, u32>);

impl LookupTable {
    pub fn id(&self, name: &str) -> u32 {
        self.0[name]
    }
}

/// Required properties (part of Item)
#[derive(Component)]
pub struct Base {
    name: Cow<'static, str>,
}

impl Base {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Builds the look-up table
pub fn build_lut(mut commands: Commands, items: Query<(Entity, &Base)>) {
    commands.insert_resource(LookupTable(
        items
            .iter()
            .map(|(item, item_base)| (item_base.name.to_string(), item.index()))
            .collect(),
    ));
}

#[derive(Component)]
pub struct EmptyBucket(pub HashMap<Entity, Entity>);

#[derive(Component)]
pub struct Bucket {
    pub fluid: Entity,
    pub empty: Entity,
}
