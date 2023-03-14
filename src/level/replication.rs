use std::collections::HashSet;

use bevy::prelude::*;

use tesseract_protocol::{
    packet::s2c,
    types::{Nbt, VarI32},
    Encode,
};

use crate::{actor, connection, level};

#[derive(Component)]
pub struct ReplicatedBy(HashSet<Entity>);

#[derive(Component)]
pub struct ReplicationDistance(pub u8);

#[derive(Default, Component)]
pub struct Replicated(HashSet<IVec2>);

pub fn replicate(
    mut commands: Commands,
    mut dimensions: Query<&mut level::chunk::LookupTable>,
    chunks: Query<(&level::chunk::Position, &Parent)>,
    mut chunks_replicated_by: Query<&mut ReplicatedBy>,
    chunks_terrain: Query<&level::terrain::Terrain>,
    mut actors: Query<
        (
            Entity,
            &ReplicationDistance,
            &mut Replicated,
            &connection::Connection,
            &Parent,
            &actor::Position,
        ),
        Changed<Parent>,
    >,
) {
    for (actor, replication_distance, mut replicated, connection, parent, actor_position) in
        actors.iter_mut()
    {
        if let Ok((chunk_position, dimension)) = chunks.get(parent.get()) {
            connection.send(s2c::GamePacket::SetChunkCacheCenter {
                x: VarI32(chunk_position.0.x),
                z: VarI32(chunk_position.0.y),
            });

            // Square radius
            let mut chunks_in_distance = HashSet::new();
            let replication_distance = replication_distance.0 as i32;
            for x_r in -replication_distance..replication_distance {
                for z_r in -replication_distance..replication_distance {
                    let x = chunk_position.0.x + x_r;
                    let z = chunk_position.0.y + z_r;
                    chunks_in_distance.insert(IVec2::new(x, z));
                }
            }

            let mut lookup_table = dimensions.get_mut(dimension.get()).unwrap();

            // Release chunks
            for chunk_position in replicated
                .0
                .iter()
                .filter(|&chunk_position| !chunks_in_distance.contains(chunk_position))
            {
                if let Some(&chunk) = lookup_table.0.get(chunk_position) {
                    let mut replicated_by = chunks_replicated_by.get_mut(chunk).unwrap();
                    replicated_by.0.remove(&actor);
                    connection.send(s2c::GamePacket::ForgetLevelChunk {
                        x: chunk_position.x,
                        z: chunk_position.y,
                    })
                }
            }

            // Acquire chunks
            for position in chunks_in_distance
                .iter()
                .filter(|&chunk_position| !replicated.0.contains(chunk_position))
            {
                if let Some(&chunk) = lookup_table.0.get(position) {
                    let mut replicated_by = chunks_replicated_by.get_mut(chunk).unwrap();
                    replicated_by.0.insert(actor);
                    if let Ok(terrain) = chunks_terrain.get(chunk) {
                        let mut buffer = Vec::new();
                        for section in &terrain.sections {
                            1i16.encode(&mut buffer).unwrap();
                            section.blocks.encode(&mut buffer).unwrap();
                            section.biomes.encode(&mut buffer).unwrap();
                        }
                        connection.send(s2c::GamePacket::LevelChunkWithLight {
                            x: position.x,
                            y: position.y,
                            chunk_data: s2c::game::LevelChunkPacketData {
                                heightmaps: Nbt(s2c::game::LevelChunkPacketDataHeightmap {}),
                                buffer,
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
                } else {
                    lookup_table.0.insert(
                        *position,
                        commands
                            .spawn((
                                level::chunk::Position(*position),
                                ReplicatedBy({
                                    let mut replicated_by = HashSet::new();
                                    replicated_by.insert(actor);
                                    replicated_by
                                }),
                            ))
                            .set_parent(dimension.get())
                            .id(),
                    );
                }
            }

            replicated.0 = chunks_in_distance;
        } else {
            if let Ok(mut lookup_table) = dimensions.get_mut(parent.get()) {
                let chunk_position = IVec2::new(
                    (actor_position.0[0] as i32) >> 4,
                    (actor_position.0[2] as i32) >> 4,
                );
                lookup_table.0.insert(
                    chunk_position,
                    commands
                        .spawn((
                            level::chunk::Position(chunk_position),
                            ReplicatedBy({
                                let mut replicated_by = HashSet::new();
                                replicated_by.insert(actor);
                                replicated_by
                            }),
                        ))
                        .set_parent(parent.get())
                        .id(),
                );
            }
        }
    }
}

pub fn replicate_2(
    chunks: Query<
        (
            &level::chunk::Position,
            &level::terrain::Terrain,
            &ReplicatedBy,
        ),
        Added<level::terrain::Terrain>,
    >,
    actors: Query<&connection::Connection>,
) {
    for (position, terrain, replicated_by) in chunks.iter() {
        let mut buffer = Vec::new();
        for section in &terrain.sections {
            1i16.encode(&mut buffer).unwrap();
            section.blocks.encode(&mut buffer).unwrap();
            section.biomes.encode(&mut buffer).unwrap();
        }
        let packet = s2c::GamePacket::LevelChunkWithLight {
            x: position.0.x,
            y: position.0.y,
            chunk_data: s2c::game::LevelChunkPacketData {
                heightmaps: Nbt(s2c::game::LevelChunkPacketDataHeightmap {}),
                buffer,
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
        };
        for &actor in replicated_by.0.iter() {
            actors.get(actor).unwrap().send(packet.clone())
        }
    }
}
