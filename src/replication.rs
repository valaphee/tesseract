use bevy::{
    prelude::*,
    utils::{HashSet, Uuid},
};

use tesseract_protocol::{
    packet::s2c,
    types::{Angle, Nbt, VarI32},
    Encode,
};

use crate::{actor, connection, level};

#[derive(Default)]
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, subscribe_and_replicate_initial)
            .add_systems(PostUpdate, replicate_chunks_late)
            .add_systems(PostUpdate, replicate_actors)
            .add_systems(PostUpdate, replicate_actors_movement);
    }
}

#[derive(Default, Component)]
pub struct Replication {
    subscriber: HashSet<Entity>,
    replicated: Vec<Entity>,
}

impl Replication {
    pub fn with_subscriber(subscriber_: Entity) -> Self {
        Self {
            subscriber: {
                let mut subscriber = HashSet::new();
                subscriber.insert(subscriber_);
                subscriber
            },
            replicated: default(),
        }
    }
}

#[derive(Default, Component)]
pub struct SubscriptionDistance(pub u8);

#[derive(Default, Component)]
pub struct Subscriptions(HashSet<IVec2>);

#[allow(clippy::type_complexity)]
fn subscribe_and_replicate_initial(
    mut commands: Commands,
    mut levels: Query<&mut level::chunk::LookupTable>,
    chunk_positions: Query<(&level::chunk::Position, &Parent)>,
    mut chunks: Query<(Option<&level::chunk::Terrain>, &mut Replication)>,
    actors: Query<(Entity, &actor::Position)>,
    mut players: Query<
        (
            Entity,
            &Parent,
            &connection::Connection,
            &SubscriptionDistance,
            &mut Subscriptions,
        ),
        Or<(Changed<Parent>, Changed<SubscriptionDistance>)>,
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
                    let (_, mut replication) = chunks.get_mut(chunk).unwrap();
                    replication.subscriber.remove(&player);

                    // connection: remove chunk and entities, cause: unsubscribe
                    connection.send(s2c::GamePacket::RemoveEntities {
                        entity_ids: replication
                            .replicated
                            .iter()
                            .map(|actor| VarI32(actor.index() as i32))
                            .collect(),
                    });
                    connection.send(s2c::GamePacket::ForgetLevelChunk {
                        x: chunk_position.x,
                        z: chunk_position.y,
                    });
                }
            }

            // acquire chunks
            for chunk_position in target_subscriptions
                .iter()
                .filter(|&chunk_position| !actual_subscriptions.0.contains(chunk_position))
            {
                if let Some(&chunk) = chunk_lut.0.get(chunk_position) {
                    let (terrain, mut replication) = chunks.get_mut(chunk).unwrap();
                    replication.subscriber.insert(player);

                    if let Some(terrain) = terrain {
                        // connection: add chunk and entities, cause: subscribe
                        for (actor, actor_position) in actors.iter_many(&replication.replicated) {
                            // except owner
                            if actor == player {
                                continue;
                            }

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

                        let mut buffer = Vec::new();
                        for section in &terrain.sections {
                            4096i16.encode(&mut buffer).unwrap();
                            section.encode(&mut buffer).unwrap();
                            0u8.encode(&mut buffer).unwrap();
                            VarI32(0).encode(&mut buffer).unwrap();
                            VarI32(0).encode(&mut buffer).unwrap();
                        }
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
                            .spawn(level::chunk::ChunkBundle::with_subscriber(
                                *chunk_position,
                                player,
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

fn replicate_chunks_late(
    chunks: Query<
        (
            &level::chunk::Terrain,
            &level::chunk::Position,
            &Replication,
        ),
        Added<level::chunk::Terrain>,
    >,
    players: Query<&connection::Connection>,
) {
    // early return
    for (terrain, chunk_position, replication) in chunks.iter() {
        let mut buffer = Vec::new();
        for section in &terrain.sections {
            4096i16.encode(&mut buffer).unwrap();
            section.encode(&mut buffer).unwrap();
            0u8.encode(&mut buffer).unwrap();
            VarI32(0).encode(&mut buffer).unwrap();
            VarI32(0).encode(&mut buffer).unwrap();
        }

        for &player in &replication.subscriber {
            // connection: add chunk, cause: subscribe (late)
            if let Ok(connection) = players.get(player) {
                connection.send(s2c::GamePacket::LevelChunkWithLight {
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
            } else {
                warn!("Replication requires a connection")
            }
        }
    }
}

fn replicate_actors(
    mut chunks: Query<(&Children, &mut Replication), Changed<Children>>,
    actors: Query<&actor::Position>,
    players: Query<&connection::Connection>,
) {
    // early return
    if chunks.is_empty() {
        return;
    }

    for (actors, replication) in chunks.iter() {
        for &actor in replication
            .replicated
            .iter()
            .filter(|actor| !actors.contains(actor))
        {
            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: add entity, cause: despawn
                if let Ok(connection) = players.get(player) {
                    connection.send(s2c::GamePacket::RemoveEntities {
                        entity_ids: vec![VarI32(actor.index() as i32)],
                    })
                }
            }
        }
    }

    for (actors_, replication) in chunks.iter() {
        for &actor in actors_
            .iter()
            .filter(|actor| !replication.replicated.contains(actor))
        {
            let actor_position = actors.get(actor).unwrap();

            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: add entity, cause: spawn
                if let Ok(connection) = players.get(player) {
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
    }

    for (actors, mut replication) in chunks.iter_mut() {
        replication.replicated.clear();
        replication.replicated.extend(actors.iter())
    }
}

#[allow(clippy::type_complexity)]
fn replicate_actors_movement(
    chunks: Query<&Replication>,
    actors: Query<
        (Entity, &Parent, &actor::Position, &actor::Rotation),
        Or<(Changed<actor::Position>, Changed<actor::Rotation>)>,
    >,
    players: Query<&connection::Connection>,
) {
    for (actor, chunk, actor_position, actor_rotation) in actors.iter() {
        if let Ok(replication) = chunks.get(chunk.get()) {
            for &player in replication.subscriber.iter() {
                // except owner
                if actor == player {
                    continue;
                }

                // connection: teleport entity, cause: movement
                if let Ok(connection) = players.get(player) {
                    connection.send(s2c::GamePacket::TeleportEntity {
                        id: VarI32(actor.index() as i32),
                        pos: actor_position.0,
                        pitch: Angle(actor_rotation.pitch),
                        yaw: Angle(actor_rotation.yaw),
                        on_ground: false,
                    });
                    connection.send(s2c::GamePacket::RotateHead {
                        entity_id: VarI32(actor.index() as i32),
                        head_yaw: Angle(actor_rotation.yaw),
                    });
                }
            }
        }
    }
}
