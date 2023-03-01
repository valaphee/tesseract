use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod terrain;
pub mod chunk;

#[derive(Component)]
pub struct Dimension {
    pub name: String,
}
