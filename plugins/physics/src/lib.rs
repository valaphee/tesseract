#![feature(array_zip)]

use bevy::prelude::*;
use rand::prelude::*;

use tesseract_base::{block, level};

#[derive(Component)]
pub struct Fluid {
    pub volume: u8,
}

pub struct FluidCache([u32; 8]);

impl FromWorld for FluidCache {
    fn from_world(world: &mut World) -> Self {
        let mut fluids_ = [0; 8];
        for (fluid, fluid_base) in world.query::<(Entity, &Fluid)>().iter(world) {
            fluids_[fluid_base.volume as usize] = fluid.index();
        }
        FluidCache(fluids_)
    }
}

pub fn update_fluids(
    fluid_cache: Local<FluidCache>,

    blocks: Query<&block::Base>,
    fluids: Query<(Entity, &Fluid)>,

    mut chunks: Query<(&mut level::chunk::Data, &level::chunk::UpdateQueue)>,
) {
    for (mut chunk_data, chunk_queued_updates) in chunks.iter_mut() {
        if chunk_queued_updates.0.is_empty() {
            continue;
        }

        for queued_update in &chunk_queued_updates.0 {
            let y = queued_update.y();
            if y == 0 {
                continue;
            }

            let x = queued_update.x();
            let z = queued_update.z();
            if let Ok((_, fluid_base)) = fluids.get(Entity::from_raw(chunk_data.get(x, y, z))) {
                let value2volume = |value| {
                    fluids.get(Entity::from_raw(value)).map_or_else(
                        |_| {
                            if blocks
                                .get(Entity::from_raw(value))
                                .map_or(false, |replaceable| replaceable.collision)
                            {
                                u8::MAX
                            } else {
                                0
                            }
                        },
                        |(_, fluid_base)| fluid_base.volume + 1,
                    )
                };
                let volume2value = |volume| match volume {
                    0 => Some(0),
                    VOLUME_MIN..=VOLUME_MAX => Some(fluid_cache.0[(volume - 1) as usize]),
                    _ => None,
                };

                // falling
                let mut volume = fluid_base.volume + 1;
                let mut volume_below = value2volume(chunk_data.get(x, y - 1, z));
                if volume_below < VOLUME_MAX {
                    volume_below += volume;
                    if volume_below > VOLUME_MAX {
                        volume = volume_below - VOLUME_MAX;
                        volume_below = VOLUME_MAX;
                    } else {
                        volume = 0;
                    }
                    chunk_data.set(x, y - 1, z, fluid_cache.0[(volume_below - 1) as usize]);
                }

                // spreading
                let mut xz_positions = [
                    (x.wrapping_sub(1), z),
                    (x + 1, z),
                    (x, z.wrapping_sub(1)),
                    (x, z + 1),
                ];
                xz_positions.shuffle(&mut thread_rng());
                let mut volumes = xz_positions.map(|xz_position| {
                    value2volume(chunk_data.get(xz_position.0, y, xz_position.1))
                });

                let mut volume_index = 0;
                let mut spread = false;
                while volume > VOLUME_MIN {
                    if volumes[volume_index] < volume {
                        volumes[volume_index] += 1;
                        volume -= 1;
                        spread = true;
                    }

                    volume_index += 1;
                    if volume_index >= volumes.len() {
                        volume_index = 0;
                        if !spread {
                            break;
                        }
                        spread = false;
                    }
                }

                if let Some(value) = volume2value(volume) {
                    chunk_data.set(x, y, z, value);
                }
                for ((x, z), volume) in xz_positions.zip(volumes) {
                    if let Some(value) = volume2value(volume) {
                        chunk_data.set(x, y, z, value);
                    }
                }
            }
        }
    }
}

const VOLUME_MIN: u8 = 1;
const VOLUME_MAX: u8 = 8;
