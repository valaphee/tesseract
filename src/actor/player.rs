use bevy::prelude::*;

use tesseract_java_protocol::types::Direction;

use crate::{actor, block, level};

#[derive(Bundle)]
pub struct PlayerBundle {
    // actor
    pub base: actor::Base,
    pub position: actor::Position,
    pub rotation: actor::Rotation,

    // player
    pub interaction: Interaction,
}

/// Current interaction (part of Player)
#[derive(Component, Default)]
pub enum Interaction {
    #[default]
    None,
    BreakBlock(IVec3),
    UseItemOn(IVec3, Direction),
}

pub fn update_interactions(
    blocks: Query<(), With<block::Base>>,
    levels: Query<&level::chunk::LookupTable>,
    mut chunks: Query<(&level::chunk::Base, &mut level::chunk::Data, &Parent)>,
    mut players: Query<(&mut Interaction, &Parent), Changed<Interaction>>,
) {
    for (mut interaction, chunk) in players.iter_mut() {
        match *interaction {
            Interaction::BreakBlock(position) => {
                if let Ok((chunk_base, chunk_data, level)) = chunks.get_mut(chunk.get()) {
                    let chunk_position = IVec2::new(position.x >> 4, position.z >> 4);
                    // shortcut if position is in actor's chunk
                    let chunk_data = if chunk_base.position() == chunk_position {
                        Some(chunk_data)
                    } else {
                        levels.get(level.get()).ok().and_then(|chunk_lut| {
                            chunk_lut.0.get(&chunk_position).and_then(|chunk| {
                                chunks.get_component_mut::<level::chunk::Data>(*chunk).ok()
                            })
                        })
                    };

                    if let Some(mut chunk_data) = chunk_data {
                        let y_offset = chunk_data.y_offset as i32 * 16;
                        chunk_data.set(
                            position.x as u8,
                            (position.y + y_offset) as u16,
                            position.z as u8,
                            0,
                        );
                    }
                }

                *interaction = Interaction::None;
            }
            Interaction::UseItemOn(position, direction) => {
                /*if let Some(item_stack) = inventory.content[36 + inventory.selected_slot as usize] {
                    if blocks.contains(item_stack.item) {
                        if let Ok((chunk_base, chunk_data, level)) = chunks.get_mut(chunk.get()) {
                            let position = position + direction.vector();
                            let chunk_position = IVec2::new(position.x >> 4, position.z >> 4);
                            // shortcut if position is in actor's chunk
                            let chunk_data = if chunk_base.position() == chunk_position {
                                Some(chunk_data)
                            } else {
                                levels.get(level.get()).ok().and_then(|chunk_lut| {
                                    chunk_lut.0.get(&chunk_position).and_then(|chunk| {
                                        chunks.get_component_mut::<level::chunk::Data>(*chunk).ok()
                                    })
                                })
                            };

                            if let Some(mut chunk_data) = chunk_data {
                                let y_offset = chunk_data.y_offset as i32 * 16;
                                chunk_data.set(
                                    position.x as u8,
                                    (position.y + y_offset) as u16,
                                    position.z as u8,
                                    item_stack.item.index(),
                                );

                            }
                        }
                    }
                }*/

                *interaction = Interaction::None;
            }
            _ => {}
        }
    }
}
