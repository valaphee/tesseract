use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use tesseract_protocol::packet::s2c;
use tesseract_protocol::types::VarInt;
use crate::{actor, connection};

// Level
#[derive(Default, Component)]
pub struct LookupTable(HashMap<[i32; 2], Entity>);

// Chunk
#[derive(Component)]
pub struct Position(pub [i32; 2]);

#[derive(Component)]
pub struct LoadedBy(HashSet<Entity>);

#[derive(Component)]
pub struct InitialPacket(pub s2c::GamePacket);

#[derive(Component)]
pub struct PacketQueue(pub Vec<s2c::GamePacket>);

// Actor
#[derive(Component)]
pub struct Load(pub u8);

#[derive(Default, Component)]
pub struct Loaded(HashSet<[i32; 2]>);

pub fn initialize_chunk(
    mut commands: Commands,

    mut lookup_table: Query<&mut LookupTable>,
    chunks_positions: Query<&Position>,
    mut chunks: Query<(&mut LoadedBy, Option<&InitialPacket>)>,

    actors_without_parent: Query<(Entity, &actor::Position), Without<Parent>>,
    mut actors: Query<(Entity, &Parent, &Load, &mut Loaded, Option<&connection::Connection>), Changed<Parent>>,
) {
    // TODO: support multiple levels
    let mut lookup_table = lookup_table.single_mut();

    for (actor, actor_position) in actors_without_parent.iter() {
        let position = [(actor_position.0[0] as i32) >> 4, (actor_position.0[2] as i32) >> 4];
        if !lookup_table.0.contains_key(&position) {
            lookup_table.0.insert(position, commands.spawn((
                Position(position),
                LoadedBy(HashSet::new())
            )).add_child(actor).id());
        }
    }

    for (actor, chunk, load, mut loaded, connection) in actors.iter_mut() {
        let position = chunks_positions.get_component::<Position>(chunk.get()).unwrap().0;
        if let Some(connection) = connection {
            connection.sender.send(s2c::GamePacket::SetChunkCacheCenter {
                x: VarInt(position[0]),
                z: VarInt(position[1]),
            }).unwrap();
        }

        // Chunks in distance
        let mut chunks_in_distance = HashSet::new();
        let distance = load.0 as i32;
        for x_r in -distance..distance {
            for z_r in -distance..distance {
                let x = position[0] + x_r;
                let z = position[1] + z_r;
                chunks_in_distance.insert([x, z]);
            }
        }

        // Release chunks
        for position in loaded.0.iter().filter(|&chunk| !chunks_in_distance.contains(chunk)) {
            if let Some(&chunk) = lookup_table.0.get(position) {
                let mut loaded_by = chunks.get_component_mut::<LoadedBy>(chunk).unwrap();
                loaded_by.0.remove(&actor);

                if let Some(connection) = connection {
                    connection.sender.send(s2c::GamePacket::ForgetLevelChunk {
                        x: position[0],
                        z: position[1],
                    }).unwrap();
                }
            }
        }

        // Acquire chunks
        for position in chunks_in_distance.iter().filter(|&chunk| !loaded.0.contains(chunk)) {
            if let Some(&chunk) = lookup_table.0.get(position) {
                let (mut loaded_by, initial_packet) = chunks.get_mut(chunk).unwrap();
                loaded_by.0.insert(actor);

                if let Some(connection) = connection {
                    if let Some(initial_packet) = initial_packet {
                        connection.sender.send(initial_packet.0.clone()).unwrap();
                    }
                }
            } else {
                lookup_table.0.insert(*position, commands.spawn((
                    Position(*position),
                    LoadedBy({
                        let mut loaded_by = HashSet::new();
                        loaded_by.insert(actor);
                        loaded_by
                    })
                )).id());
            }
        }

        loaded.0 = chunks_in_distance;
    }
}

pub fn update_chunk_hierarchy(
    mut commands: Commands,

    lookup_table: Query<&LookupTable>,
    chunks: Query<&Position>,

    actors: Query<(Entity, Option<&Parent>, &actor::Position), (Added<actor::Position>, Changed<actor::Position>)>,
) {
    // TODO: support multiple levels
    let lookup_table = lookup_table.single();

    for (actor, chunk, actor_position) in actors.iter() {
        let new_position = [(actor_position.0[0] as i32) >> 4, (actor_position.0[2] as i32) >> 4];
        if chunk.map_or(true, |chunk| new_position != chunks.get_component::<Position>(chunk.get()).unwrap().0) {
            if let Some(&chunk) = lookup_table.0.get(&new_position) {
                commands.entity(chunk).add_child(actor);
            }
        }
    }
}

pub fn send_initial_packet(
    connections: Query<&connection::Connection>,

    chunks: Query<(&LoadedBy, &InitialPacket), Added<InitialPacket>>
) {
    for (loaded_by, initial_packet) in chunks.iter() {
        for &loaded_by in loaded_by.0.iter() {
            if let Ok(connection) = connections.get_component::<connection::Connection>(loaded_by) {
                connection.sender.send(initial_packet.0.clone()).unwrap();
            }
        }
    }
}

pub fn process_packet_queue(
    connections: Query<&connection::Connection>,

    mut chunks: Query<(&LoadedBy, &mut PacketQueue), Changed<PacketQueue>>,
) {
    for (loaded_by, mut packet_queue) in chunks.iter_mut() {
        for &loaded_by in loaded_by.0.iter() {
            if let Ok(connection) = connections.get_component::<connection::Connection>(loaded_by) {
                for packet in &packet_queue.0 {
                    connection.sender.send(packet.clone()).unwrap();
                }
            }
        }
        packet_queue.0.clear();
    }
}
