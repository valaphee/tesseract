use bevy::{
    prelude::*,
    utils::{Entry, HashMap, HashSet, Uuid},
};

use tesseract_protocol::{
    packet::s2c,
    types::{Angle, Nbt, VarI32},
    Encode,
};

use crate::{actor, chunk, connection};

#[derive(Default)]
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, subscribe_and_replicate_initial)
            .add_systems(PostUpdate, replicate_chunks_late)
            .add_systems(PostUpdate, replicate_actors);
    }
}

#[derive(Default, Component)]
pub struct Replication {
    subscriber: HashSet<Entity>,
    replicated: Vec<Entity>,
}

#[derive(Component)]
pub struct SubscriptionDistance(pub u8);

#[derive(Default, Component)]
pub struct Subscriptions(HashSet<IVec2>);

fn subscribe_and_replicate_initial(
    mut commands: Commands,
    mut levels: Query<&mut chunk::LookupTable>,
    chunk_positions: Query<(&chunk::Position, &Parent)>,
    mut chunks: Query<(Option<&chunk::Terrain>, &Children, &mut Replication)>,
    mut players: Query<
        (
            Entity,
            &Parent,
            &connection::Connection,
            &SubscriptionDistance,
            &mut Subscriptions,
        ),
        Changed<Parent>,
    >,
) {
    for (player, chunk, connection, subscription_distance, mut actual_subscriptions) in
        players.iter_mut()
    {
        if let Ok((chunk_position, level)) = chunk_positions.get(chunk.get()) {
            connection.send(s2c::GamePacket::SetChunkCacheCenter {
                x: VarI32(chunk_position.0.x),
                z: VarI32(chunk_position.0.y),
            });

            // square radius
            let mut target_subscriptions = HashSet::new();
            let subscription_distance = subscription_distance.0 as i32;
            for x_r in -subscription_distance..subscription_distance {
                for z_r in -subscription_distance..subscription_distance {
                    let x = chunk_position.0.x + x_r;
                    let z = chunk_position.0.y + z_r;
                    target_subscriptions.insert(IVec2::new(x, z));
                }
            }

            let mut chunk_lut = levels.get_mut(level.get()).unwrap();

            // release chunks
            for chunk_position in actual_subscriptions
                .0
                .iter()
                .filter(|&chunk_position| !target_subscriptions.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    let (_, actors, mut replication) = chunks.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);

                    // connection: remove chunk and entities, cause: unsubscribe
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
            for chunk_position in target_subscriptions
                .iter()
                .filter(|&chunk_position| !actual_subscriptions.0.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    let (terrain, _, mut replication) = chunks.get_mut(chunk).unwrap();
                    replication.subscriber.insert(player);

                    if let Some(terrain) = terrain {
                        println!("hey?");
                        let mut buffer = Vec::new();
                        for section in &terrain.sections {
                            i16::MAX.encode(&mut buffer).unwrap();
                            section.blocks.encode(&mut buffer).unwrap();
                            section.biomes.encode(&mut buffer).unwrap();
                        }

                        // connection: add chunk and entities, cause: subscribe
                        connection.send(s2c::GamePacket::LevelChunkWithLight {
                            x: chunk_position.x,
                            z: chunk_position.y,
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
                } else {
                    chunk_lut.0.insert(
                        *chunk_position,
                        commands
                            .spawn((
                                chunk::Position(*chunk_position),
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

            actual_subscriptions.0 = target_subscriptions;
        }
    }
}

#[allow(clippy::type_complexity)]
fn replicate_chunks_late(
    chunks: Query<(&chunk::Terrain, &chunk::Position, &Replication), Added<chunk::Terrain>>,
    players: Query<&connection::Connection>,
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
            // connection: add chunk, cause: subscribe (late)
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

#[allow(clippy::type_complexity)]
fn replicate_actors(
    chunks: Query<(&Children, &Replication), Or<(Changed<Children>, Changed<Replication>)>>,
    actors: Query<(Entity, &actor::Position)>,
    players: Query<(Entity, &connection::Connection)>,
) {
    // early return
    if chunks.is_empty() {
        return;
    }

    // go through all added actors and map to every actor all subsequent replicators
    let mut add = HashMap::<Entity, Vec<Entity>>::new();
    for (actors, replication) in chunks.iter() {
        for &actor in actors
            .iter()
            .filter(|actor| !replication.replicated.contains(actor))
        {
            add.insert(actor, replication.subscriber.clone().into_iter().collect());
        }
    }

    // go through all removed actors and check if they are already added again for
    // the replicator or else collect them in remove
    let mut remove = HashMap::<Entity, Vec<Entity>>::new();
    for (children, replication) in chunks.iter() {
        for &removed in replication
            .replicated
            .iter()
            .filter(|child| !children.contains(child))
        {
            for &player in add
                .remove(&removed)
                .unwrap()
                .iter()
                .filter(|player| replication.subscriber.contains(player))
            {
                match remove.entry(player) {
                    Entry::Occupied(occupied) => occupied.into_mut(),
                    Entry::Vacant(vacant) => vacant.insert(Vec::new()),
                }
                .push(removed);
            }
        }
    }

    for (player, actors) in remove {
        // connection: remove entity, cause: deleted or moved
        players
            .get_component::<connection::Connection>(player)
            .unwrap()
            .send(s2c::GamePacket::RemoveEntities {
                entity_ids: actors
                    .iter()
                    .map(|actor| VarI32(actor.index() as i32))
                    .collect(),
            })
    }

    for (actor, players_) in add {
        for (entity, connection) in players.iter_many(players_) {
            // except owner
            if actor == entity {
                continue;
            }

            // connection: add entity, cause: added or moved
            let (_, actor_position) = actors.get(actor).unwrap();
            connection.send(s2c::GamePacket::AddEntity {
                id: VarI32(actor.index() as i32),
                uuid: Uuid::new_v4(),
                type_: VarI32(72),
                pos: actor_position.0,
                pitch: Angle(0.0),
                yaw: Angle(0.0),
                head_yaw: Angle(0.0),
                data: VarI32(0),
                xa: 0,
                ya: 0,
                za: 0,
            });
        }
    }
}
