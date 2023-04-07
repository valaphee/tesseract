use bevy::prelude::*;

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base {
    pub collision: bool,
}

/// Instance of a block (part of BlockInstance)
pub struct Instance {
    pub block: Entity,
    pub position: IVec3,
}
