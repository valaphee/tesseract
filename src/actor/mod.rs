use bevy::prelude::*;
use tesseract_protocol::packet::s2c;
use tesseract_protocol::types::VarInt;
use crate::level::chunk;

#[derive(Component)]
pub struct Position(pub [f64; 3]);

#[derive(Component)]
pub struct Rotation(pub [f32; 2]);

#[derive(Component)]
pub struct HeadRotation(pub f32);

pub fn populate_packet_queue(
    mut packet_queues: Query<&mut chunk::PacketQueue>,

    actors: Query<(Entity, &Parent, Ref<Position>, Ref<Rotation>, Ref<HeadRotation>), Or<(Changed<Position>, Changed<Rotation>, Changed<HeadRotation>)>>,
) {
    for (entity, parent, position, rotation, head_rotation) in actors.iter() {
        let mut packet_queue = packet_queues.get_component_mut::<chunk::PacketQueue>(parent.get()).unwrap();
        if position.is_changed() {
            packet_queue.0.push(s2c::GamePacket::TeleportEntity {
                id: VarInt(entity.index() as i32),
                x: position.0[0],
                y: position.0[1],
                z: position.0[2],
                y_rot: 0,
                x_rot: 0,
                on_ground: false,
            });
        } else {
            if rotation.is_changed() {
                packet_queue.0.push(s2c::GamePacket::MoveEntityRot {
                    entity_id: VarInt(entity.index() as i32),
                    y_rot: 0,
                    x_rot: 0,
                    on_ground: false,
                });
            }
        }
        if head_rotation.is_changed() {
            packet_queue.0.push(s2c::GamePacket::RotateHead {
                entity_id: VarInt(entity.index() as i32),
                y_head_rot: 0,
            });
        }
    }
}
