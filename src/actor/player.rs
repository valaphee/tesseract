use bevy::prelude::*;

use crate::{actor, block, item, level};

#[derive(Bundle)]
pub struct PlayerBundle {
    pub base: actor::Base,
    pub position: actor::Position,
    pub rotation: actor::Rotation,
    pub head_rotation: actor::HeadRotation,

    pub inventory: Inventory,
    pub interaction: Interaction,
}

#[derive(Component)]
pub struct Inventory {
    pub content: Vec<Option<Entity>>,
    pub selected_slot: u8,
}

#[derive(Component, Default)]
pub enum Interaction {
    #[default]
    None,
    BlockBreak(IVec3),
    BlockPlace(IVec3),
}

pub fn update_interactions(
    block_items: Query<(), (With<block::Base>, With<item::Base>)>,

    levels: Query<&level::chunk::LookupTable>,
    mut chunks: Query<(&level::chunk::Base, &mut level::chunk::Data, &Parent)>,
    mut players: Query<(&mut Interaction, &Inventory, &Parent), Changed<Interaction>>,
) {
    for (mut interaction, inventory, chunk) in players.iter_mut() {
        #[allow(clippy::single_match)]
        match *interaction {
            Interaction::BlockBreak(position) => {
                if let Ok((chunk_base, chunk_data, level)) = chunks.get_mut(chunk.get()) {
                    let chunk_position = IVec2::new(position.x >> 4, position.z >> 4);
                    // shortcut if position is in actor's chunk
                    let chunk_data = if chunk_base.0 == chunk_position {
                        Some(chunk_data)
                    } else {
                        levels.get(level.get()).ok().and_then(|chunk_lut| {
                            chunk_lut.0.get(&chunk_position).and_then(|chunk| {
                                chunks.get_component_mut::<level::chunk::Data>(*chunk).ok()
                            })
                        })
                    };

                    // TODO: clean-up (just for testing)
                    if let Some(mut chunk_data) = chunk_data {
                        chunk_data.set(
                            position.x as u8,
                            (position.y + 64) as u16,
                            position.z as u8,
                            0,
                        );
                    }
                }

                *interaction = Interaction::None;
            }
            Interaction::BlockPlace(position) => {
                if let Some(item) = inventory.content[36 + inventory.selected_slot as usize] {
                    if block_items.contains(item) {
                        if let Ok((chunk_base, chunk_data, level)) = chunks.get_mut(chunk.get()) {
                            let chunk_position = IVec2::new(position.x >> 4, position.z >> 4);
                            // shortcut if position is in actor's chunk
                            let chunk_data = if chunk_base.0 == chunk_position {
                                Some(chunk_data)
                            } else {
                                levels.get(level.get()).ok().and_then(|chunk_lut| {
                                    chunk_lut.0.get(&chunk_position).and_then(|chunk| {
                                        chunks.get_component_mut::<level::chunk::Data>(*chunk).ok()
                                    })
                                })
                            };

                            if let Some(mut chunk_data) = chunk_data {
                                chunk_data.set(
                                    position.x as u8,
                                    (position.y + 64) as u16,
                                    position.z as u8,
                                    item.index(),
                                );
                            }
                        }
                    }
                }

                *interaction = Interaction::None;
            }
            _ => {}
        }
    }
}
