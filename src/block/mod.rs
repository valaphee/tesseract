use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;

use crate::level;

/// Block by name look-up table
#[derive(Resource)]
pub struct LookupTable(pub HashMap<String, Entity>);

/// Required properties (part of Block)
#[derive(Component)]
pub struct Base(pub Cow<'static, str>);

#[derive(Component)]
pub struct Fluid(pub u8);

/// Builds the look-up table
pub fn build_lut(mut commands: Commands, blocks: Query<(Entity, &Base)>) {
    commands.insert_resource(LookupTable(
        blocks
            .iter()
            .map(|(block, block_base)| (block_base.0.to_string(), block))
            .collect(),
    ));
}

pub struct FluidCache([u32; 8]);

impl FromWorld for FluidCache {
    fn from_world(world: &mut World) -> Self {
        let mut fluids_ = [0; 8];
        for (fluid, fluid_base) in world.query::<(Entity, &Fluid)>().iter(world) {
            fluids_[fluid_base.0 as usize] = fluid.index();
        }
        FluidCache(fluids_)
    }
}

pub fn update_fluids(
    fluids: Query<(Entity, &Fluid)>,
    fluid_cache: Local<FluidCache>,

    mut chunks: Query<(&mut level::chunk::Data, &level::chunk::QueuedUpdates)>,
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
                fn get_volume(
                    fluids: &Query<(Entity, &Fluid)>,
                    chunk_data: &level::chunk::Data,
                    x: u8,
                    y: u16,
                    z: u8,
                ) -> u8 {
                    let value = chunk_data.get(x, y, z);
                    if value == 0 {
                        return 0;
                    }

                    fluids
                        .get(Entity::from_raw(value))
                        .map_or(u8::MAX, |(_, fluid_base)| fluid_base.0 + 1)
                }

                let mut volume = fluid_base.0 + 1;
                let mut volume_below = get_volume(&fluids, &chunk_data, x, y - 1, z);
                if volume_below <= 7 {
                    volume_below += fluid_base.0;
                    if volume_below >= 8 {
                        volume = volume_below - 8;
                        volume_below = 7;
                    } else {
                        volume = 0;
                    }
                    chunk_data.set(x, y - 1, z, fluid_cache.0[volume_below as usize]);
                }

                let mut volumes = [
                    0,
                    get_volume(&fluids, &chunk_data, x.wrapping_sub(1), y, z),
                    get_volume(&fluids, &chunk_data, x + 1, y, z),
                    get_volume(&fluids, &chunk_data, x, y, z.wrapping_sub(1)),
                    get_volume(&fluids, &chunk_data, x, y, z + 1),
                ];

                let mut volume_index = 0;
                let mut volume_maximum = 1;
                while volume > 0 {
                    if volumes[volume_index] < volume_maximum {
                        volumes[volume_index] += 1;
                        volume -= 1;
                    }

                    volume_index += 1;
                    if volume_index >= volumes.len() {
                        volume_index = 0;
                        volume_maximum += 1;
                    }
                }

                fn set_volume(
                    fluid_cache: &FluidCache,
                    chunk_data: &mut level::chunk::Data,
                    x: u8,
                    y: u16,
                    z: u8,
                    volume: u8,
                ) {
                    match volume {
                        0 => chunk_data.set(x, y, z, 0),
                        u8::MAX => {}
                        _ => chunk_data.set(x, y, z, fluid_cache.0[(volume - 1) as usize]),
                    }
                }

                set_volume(&fluid_cache, &mut chunk_data, x, y, z, volumes[0]);
                set_volume(
                    &fluid_cache,
                    &mut chunk_data,
                    x.wrapping_sub(1),
                    y,
                    z,
                    volumes[1],
                );
                set_volume(&fluid_cache, &mut chunk_data, x + 1, y, z, volumes[2]);
                set_volume(
                    &fluid_cache,
                    &mut chunk_data,
                    x,
                    y,
                    z.wrapping_sub(1),
                    volumes[3],
                );
                set_volume(&fluid_cache, &mut chunk_data, x, y, z + 1, volumes[4]);
            }
        }
    }
}
