use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;

/// Block by name look-up table
#[derive(Resource)]
pub struct LookupTable(pub HashMap<String, Entity>);

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base(pub Cow<'static, str>);

#[derive(Component)]
pub struct Fluid {
    pub volume: u8,
    pub filter: u8,
}

/// Builds the lookup table
pub fn build_lut(mut commands: Commands, blocks: Query<(Entity, &Base)>) {
    commands.insert_resource(LookupTable(
        blocks
            .iter()
            .map(|(block, block_base)| (block_base.0.to_string(), block))
            .collect(),
    ));
}
