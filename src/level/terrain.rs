use bevy::{math::DVec3, prelude::*};

use tesseract_protocol::types::PalettedContainer;

use crate::{actor, level};

#[derive(Component)]
pub struct Terrain {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub blocks: PalettedContainer,
    pub biomes: PalettedContainer,
}

pub fn populate(
    mut commands: Commands,
    chunks: Query<(Entity, &level::chunk::Position), Without<Terrain>>,
) {
    for (chunk, chunk_position) in chunks.iter() {
        let mut columns = Vec::new();
        for _ in 0..16 {
            let mut block_paletted_container = PalettedContainer::SingleValue {
                value: 0,
                storage_size: 16 * 16 * 16,
                linear_min_bits: 4,
                linear_max_bits: 8,
                global_bits: 15,
            };
            for x in 0..16 {
                for z in 0..16 {
                    block_paletted_container.get_and_set(0 << 16 | z << 4 | x, 1);
                }
            }

            let biome_paletted_container = PalettedContainer::SingleValue {
                value: 0,
                storage_size: 4 * 4 * 4,
                linear_min_bits: 3,
                linear_max_bits: 3,
                global_bits: 6,
            };

            columns.push(Section {
                blocks: block_paletted_container,
                biomes: biome_paletted_container,
            })
        }

        commands.entity(chunk).insert(Terrain { sections: columns });
        commands
            .spawn(actor::Position(DVec3::new(
                chunk_position.0.x as f64 * 16.0,
                50.0,
                chunk_position.0.y as f64 * 16.0,
            )))
            .set_parent(chunk);
    }
}
