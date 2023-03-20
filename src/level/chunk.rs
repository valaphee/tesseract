use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use tesseract_protocol::{
    packet::s2c,
    types::{Nbt, VarI32},
    Encode,
};

use crate::{actor, level};

#[derive(Default, Component)]
pub struct LookupTable(pub HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct Position(pub IVec2);

#[derive(Default, Component)]
pub struct Replication {
    pub subscriber: HashSet<Entity>,
    pub replicated: Vec<Entity>,
}

#[derive(Component)]
pub struct SubscriptionDistance(pub u8);

#[derive(Default, Component)]
pub struct Subscribed(HashSet<IVec2>);

pub fn update_hierarchy(
    mut commands: Commands,
    mut chunk_luts: Query<&mut LookupTable>,
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

        if let Ok(mut chunk_lut) = chunk_luts.get_mut(level) {
            if let Some(&chunk) = chunk_lut.0.get(&chunk_position) {
                commands.entity(chunk).add_child(actor);
            } else {
                let chunk = commands
                    .spawn((Position(chunk_position), Replication::default()))
                    .set_parent(level)
                    .add_child(actor)
                    .id();
                chunk_lut.0.insert(chunk_position, chunk);
            }
        } else {
            warn!("Parent is neither a level nor a chunk")
        }
    }
}

pub fn subscribe(
    mut commands: Commands,
    mut chunk_luts: Query<&mut LookupTable>,
    chunks: Query<(&Position, &Parent)>,
    mut chunks_replication: Query<(&mut Replication, &Children)>,
    mut players: Query<
        (
            Entity,
            &SubscriptionDistance,
            &mut Subscribed,
            &actor::connection::Connection,
            &Parent,
        ),
        Changed<Parent>,
    >,
) {
    for (player, subscribtion_distance, mut subscribed, connection, chunk) in players.iter_mut() {
        if let Ok((chunk_position, level)) = chunks.get(chunk.get()) {
            connection.send(s2c::GamePacket::SetChunkCacheCenter {
                x: VarI32(chunk_position.0.x),
                z: VarI32(chunk_position.0.y),
            });

            // square radius
            let mut chunks_to_subscribe = HashSet::new();
            let subscription_distance = subscribtion_distance.0 as i32;
            for x_r in -subscription_distance..subscription_distance {
                for z_r in -subscription_distance..subscription_distance {
                    let x = chunk_position.0.x + x_r;
                    let z = chunk_position.0.y + z_r;
                    chunks_to_subscribe.insert(IVec2::new(x, z));
                }
            }

            let mut chunk_lut = chunk_luts.get_mut(level.get()).unwrap();

            // release chunks
            for chunk_position in subscribed
                .0
                .iter()
                .filter(|&chunk_position| !chunks_to_subscribe.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    let (mut replication, actors) = chunks_replication.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);

                    // connection: remove chunk and entities, cause: unsubscription
                    connection.send(s2c::GamePacket::RemoveEntities {
                        entity_ids: actors
                            .iter()
                            .map(|actor| VarI32(actor.index() as i32))
                            .collect(),
                    });
                    connection.send(s2c::GamePacket::ForgetLevelChunk {
                        x: chunk_position.x,
                        z: chunk_position.y,
                    })
                }
            }

            // acquire chunks
            for position in chunks_to_subscribe
                .iter()
                .filter(|&chunk_position| !subscribed.0.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(position) {
                    let mut replication = chunks_replication.get_component_mut::<Replication>(chunk).unwrap();
                    replication.subscriber.insert(player);
                } else {
                    chunk_lut.0.insert(
                        *position,
                        commands
                            .spawn((
                                Position(*position),
                                Replication {
                                    subscriber: {
                                        let mut subscriber = HashSet::new();
                                        subscriber.insert(player);
                                        subscriber
                                    },
                                    replicated: vec![],
                                },
                            ))
                            .set_parent(level.get())
                            .id(),
                    );
                }
            }

            subscribed.0 = chunks_to_subscribe;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn replicate(
    chunks: Query<
        (&level::terrain::Terrain, &Position, &Replication),
        Added<level::terrain::Terrain>,
    >,
    players: Query<&actor::connection::Connection>,
) {
    // early return
    for (terrain, chunk_position, replication) in chunks.iter() {
        let mut buffer = Vec::new();
        for section in &terrain.sections {
            i16::MAX.encode(&mut buffer).unwrap();
            section.blocks.encode(&mut buffer).unwrap();
            section.biomes.encode(&mut buffer).unwrap();
        }

        for &player in &replication.subscriber {
            // connection: add chunk, cause: subscription (lazy)
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
