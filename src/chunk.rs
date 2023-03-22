use bevy::{math::DVec3, prelude::*, utils::HashMap};

use tesseract_protocol::types::PalettedContainer;

use crate::{actor, replication};

#[derive(Default, Component)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct Position(pub IVec2);

#[derive(Component)]
pub struct Terrain {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub blocks: PalettedContainer,
    pub biomes: PalettedContainer,
}

pub fn populate(mut commands: Commands, chunks: Query<(Entity, &Position), Without<Terrain>>) {
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

pub fn update_hierarchy(
    mut commands: Commands,
    mut levels: Query<&mut LookupTable>,
    chunks: Query<(&Position, &Parent)>,
    actors: Query<(Entity, &actor::Position, &Parent), Changed<actor::Position>>,
) {
    // early return
    for (actor, actor_position, level_or_chunk) in actors.iter() {
        let chunk_position = IVec2::new(
            (actor_position.0[0] as i32) >> 4,
            (actor_position.0[2] as i32) >> 4,
        );
        let level = (if let Ok((position, level)) = chunks.get(level_or_chunk.get()) {
            // skip actors where the chunk hasn't changed
            if position.0 == chunk_position {
                continue;
            }

            level
        } else {
            level_or_chunk
        })
        .get();

        if let Ok(mut chunk_lut) = levels.get_mut(level) {
            if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn((
                        Position(chunk_position),
                        replication::Replication::default(),
                    ))
                    .set_parent(level)
                    .add_child(actor)
                    .id();
                chunk_lut.0.insert(chunk_position, chunk);
            }
        } else {
            warn!("Parent of actor is neither a savegame nor a chunk")
        }
    }
}
