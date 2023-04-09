use bevy::prelude::*;

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base;

//==================================================================================== INSTANCE ====

/// Instance of a block (part of Block instance)
#[derive(Component)]
pub struct Instance {
    pub block: Entity,
}
