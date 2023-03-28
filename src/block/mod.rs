use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct LookupTable(pub HashMap<String, HashMap<(String, PropertyValue), Entity>>);

pub enum PropertyValue {
    Boolean(bool),
    Number(u8),
    String(String),
}

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base {
    pub id: u32,
}
