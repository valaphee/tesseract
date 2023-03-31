use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;

/// Item by name look-up table
#[derive(Resource)]
pub struct LookupTable(pub HashMap<String, Entity>);

/// Required properties (part of Item)
#[derive(Component)]
pub struct Base(pub Cow<'static, str>);

/// Builds the look-up table
pub fn build_lut(mut commands: Commands, items: Query<(Entity, &Base)>) {
    commands.insert_resource(LookupTable(
        items
            .iter()
            .map(|(item, item_base)| (item_base.0.to_string(), item))
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
