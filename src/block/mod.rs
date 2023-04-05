use bevy::prelude::*;

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base {
    pub collision: bool,
}
