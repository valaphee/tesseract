use bevy::prelude::*;

/// Required properties (part of Item)
#[derive(Component)]
pub struct Base;

/// Instance of an item (part of ItemInstance)
pub struct Instance {
    pub item: Entity,
    pub slot: u8,
    pub count: u8,
}
