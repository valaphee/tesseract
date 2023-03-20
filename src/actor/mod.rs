use bevy::{
    math::DVec3,
    prelude::*,
    utils::{hashbrown::hash_map::Entry, HashMap, Uuid},
};

use tesseract_protocol::{
    packet::s2c,
    types::{Angle, VarI32},
};

use crate::level;

pub mod connection;

#[derive(Component)]
pub struct Position(pub DVec3);

#[derive(Component)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct HeadRotation {
    pub head_yaw: f32,
}

#[allow(clippy::type_complexity)]
pub fn replicate(
    chunks: Query<
        (&Children, &level::chunk::Replication),
        Or<(Changed<Children>, Changed<level::chunk::Replication>)>,
    >,
    actors: Query<(Entity, &Position)>,
    players: Query<(Entity, &connection::Connection)>,
) {
    // early return
    if chunks.is_empty() {
        return;
    }

    // go through all added childrens and map to every child all subsequent
    // replicators
    let mut add = HashMap::<Entity, Vec<Entity>>::new();
    for (children, replication) in chunks.iter() {
        for &added in children
            .iter()
            .filter(|child| !replication.replicated.contains(child))
        {
            add.insert(added, replication.subscriber.clone().into_iter().collect());
        }
    }

    // go through all removed childrens and check if they are already added again
    // for the replicator or else collect them in remove
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
