use bevy::prelude::*;

/// Required properties (part of Item)
#[derive(Component)]
pub struct Base;

//==================================================================================== INSTANCE ====

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Slot {
    Cursor,
    Hotbar(u8),
    Inventory(u8),
    Feet,
    Legs,
    Torso,
    Head,
    Offhand,
}

/// Instance of an item (part of Item instance)
#[derive(Component)]
pub struct Instance {
    pub item: Entity,
    pub count: u8,
}
