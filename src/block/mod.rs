use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;

/// Block by name look-up table
#[derive(Resource)]
pub struct LookupTable(HashMap<String, u32>);

impl LookupTable {
    pub fn id(&self, name: &str) -> u32 {
        self.0[name]
    }
}

/// Required properties (part of Block)
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
pub fn build_lut(mut commands: Commands, blocks: Query<(Entity, &Base)>) {
    commands.insert_resource(LookupTable(
        blocks
            .iter()
            .map(|(block, block_base)| (block_base.name.to_string(), block.index()))
            .collect(),
    ));
}
