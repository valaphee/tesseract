use bevy::prelude::*;

use tesseract_protocol::types::PalettedContainer;

#[derive(Component)]
pub struct Terrain {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub blocks: PalettedContainer,
    pub biomes: PalettedContainer,
}
