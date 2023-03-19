use bevy::prelude::*;

use tesseract_protocol::{
    packet::s2c,
    types::{Nbt, PalettedContainer},
    Encode,
};

use crate::{actor, level};

#[derive(Component)]
pub struct Terrain {
    sections: Vec<Section>,
}

struct Section {
    blocks: PalettedContainer,
    biomes: PalettedContainer,
}

#[allow(clippy::type_complexity)]
pub fn replicate(
    chunks: Query<
        (
            &Terrain,
            &level::chunk::Position,
            &level::chunk::Replication,
        ),
        Or<(Added<Terrain>, Changed<level::chunk::Replication>)>,
    >,
    players: Query<&actor::connection::Connection>,
) {
    // Early return
    for (terrain, chunk_position, replication) in chunks.iter() {
        let mut buffer = Vec::new();
        for section in &terrain.sections {
            1i16.encode(&mut buffer).unwrap();
            section.blocks.encode(&mut buffer).unwrap();
            section.biomes.encode(&mut buffer).unwrap();
        }

        for &player in &replication.initial {
            players
                .get(player)
                .unwrap()
                .send(s2c::GamePacket::LevelChunkWithLight {
                    x: chunk_position.0.x,
                    z: chunk_position.0.y,
                    chunk_data: s2c::game::LevelChunkPacketData {
                        heightmaps: Nbt(serde_value::Value::Map(Default::default())),
                        buffer: buffer.clone(),
                        block_entities_data: vec![],
                    },
                    light_data: s2c::game::LightUpdatePacketData {
                        trust_edges: true,
                        sky_y_mask: vec![],
                        block_y_mask: vec![],
                        empty_sky_y_mask: vec![],
                        empty_block_y_mask: vec![],
                        sky_updates: vec![],
                        block_updates: vec![],
                    },
                });
        }
    }
}
