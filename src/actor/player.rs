use bevy::prelude::*;

use tesseract_java_protocol::types::Direction;

use crate::actor;

#[derive(Bundle)]
pub struct PlayerBundle {
    // actor
    pub base: actor::Base,
    pub position: actor::Position,
    pub rotation: actor::Rotation,

    // player
    pub interaction: Interaction,
}

//================================================================================= INTERACTION ====

/// Current interaction (part of Player)
#[derive(Component, Default)]
pub enum Interaction {
    #[default]
    None,
    BreakBlock(IVec3),
    UseItemOn(IVec3, Direction),
    UseItem,
}
