use std::{collections::HashSet, time::Duration};

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, math::DVec3, prelude::*};

use tesseract_base::*;
use tesseract_protocol::types::{Biome, DamageType, DimensionType, PalettedContainer};

fn main() {
    // create and run app
    let mut app = App::new();
    // required
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 20.0,
    )))
    .add_plugins(MinimalPlugins)
    .add_plugin(LogPlugin::default())
    .insert_resource(registry::Registries::new(
        "generated/reports/registries.json",
    ))
    .insert_resource(registry::BlockStateRegistry::new(
        "generated/reports/blocks.json",
    ))
    .insert_resource(registry::DataRegistry::<DimensionType>::new(
        "generated/data/dimension_type",
        "minecraft:dimension_type",
    ))
    .insert_resource(registry::DataRegistry::<Biome>::new(
        "generated/data/worldgen/biome",
        "minecraft:worldgen/biome",
    ))
    .insert_resource(registry::DataRegistry::<DamageType>::new(
        "generated/data/damage_type",
        "minecraft:damage_type",
    ))
    .add_plugin(replication::ReplicationPlugin::default())
    .add_systems(PostUpdate, level::chunk::update_hierarchy)
    // gameplay
    .add_systems(Update, actor::player::update_interactions)
    .add_systems(Update, level::update_time)
    // custom
    .add_systems(PreStartup, spawn_level)
    .add_systems(
        First,
        (spawn_players, spawn_chunks).after(replication::UpdateFlush),
    )
    .add_systems(Update, update_fluids)
    .add_systems(PostUpdate, queue_updates);

    app.run();
}

fn spawn_level(block_state_registry: Res<registry::BlockStateRegistry>, mut commands: Commands) {
    commands.spawn((
        block::Base {
            id: block_state_registry.id("minecraft:water[level=15]"),
        },
        Fluid,
    ));

    commands.spawn(level::LevelBundle {
        base: level::Base {
            name: "minecraft:overworld".into(),
            dimension_type: "minecraft:overworld".into(),
        },
        age_and_time: default(),
        chunks: default(),
    });
}

#[allow(clippy::type_complexity)]
pub fn spawn_players(
    mut commands: Commands,
    levels: Query<Entity, With<level::Base>>,
    players: Query<
        (Entity, &replication::Connection),
        (Added<replication::Connection>, Without<actor::Base>),
    >,
) {
    for (player, connection) in players.iter() {
        commands
            .entity(player)
            .insert((actor::player::PlayerBundle {
                base: actor::Base {
                    id: connection.user().id,
                    type_: "minecraft:player".into(),
                },
                position: actor::Position(DVec3::new(0.0, 6.0, 0.0)),
                rotation: default(),
                head_rotation: default(),
                interaction: default(),
            },))
            .set_parent(levels.single());
    }
}

fn spawn_chunks(
    block_state_registry: Res<registry::BlockStateRegistry>,
    biome_registry: Res<registry::DataRegistry<Biome>>,
    mut commands: Commands,
    chunks: Query<Entity, Added<level::chunk::Base>>,
    blocks: Query<Entity, With<block::Base>>,
) {
    if chunks.is_empty() {
        return;
    }

    let air_id = block_state_registry.id("minecraft:air") | 1 << 31;
    let bedrock_id = block_state_registry.id("minecraft:bedrock") | 1 << 31;
    let dirt_id = block_state_registry.id("minecraft:dirt") | 1 << 31;
    let grass_block_id = block_state_registry.id("minecraft:grass_block") | 1 << 31;
    let water_block_id = blocks.single().index();
    for chunk in chunks.iter() {
        let mut chunk_data = level::chunk::Data {
            sections: {
                let mut sections = vec![];
                for _ in 0..24 {
                    sections.push(level::chunk::DataSection {
                        block_states: PalettedContainer::SingleValue(air_id),
                        biomes: PalettedContainer::SingleValue(
                            biome_registry.id("minecraft:plains"),
                        ),
                        block_state_changes: default(),
                    })
                }
                sections
            },
            y_offset: 4,
        };

        for x in 0..16 {
            for z in 0..16 {
                chunk_data.set(x, 0, z, bedrock_id);
                for y in 1..4 {
                    chunk_data.set(x, y, z, dirt_id);
                }
                chunk_data.set(x, 4, z, grass_block_id);
            }
        }
        for section in &mut chunk_data.sections {
            section.block_state_changes.clear();
        }
        chunk_data.set(8, 6, 8, water_block_id);

        commands
            .entity(chunk)
            .insert((chunk_data, level::chunk::QueuedUpdates(HashSet::default())));
    }
}

#[derive(Component)]
struct Fluid;

fn update_fluids(
    mut chunks: Query<(&mut level::chunk::Data, &level::chunk::QueuedUpdates)>,
    blocks: Query<&Fluid>,
) {
    for (mut chunk_data, chunk_queued_updates) in chunks.iter_mut() {
        if chunk_queued_updates.0.is_empty() {
            continue;
        }

        for &queued_update in &chunk_queued_updates.0 {
            let y = queued_update >> 8;
            if y == 0 {
                continue;
            }

            let x = queued_update as u8 & 0xF;
            let z = queued_update as u8 >> 4 & 0xF;
            let value = chunk_data.get(x, y, z);
            if let Ok(_fluid) = blocks.get(Entity::from_raw(value)) {
                chunk_data.set(x, y - 1, z, value);
            }
        }
    }
}

fn queue_updates(
    mut chunks: Query<
        (&level::chunk::Data, &mut level::chunk::QueuedUpdates),
        Changed<level::chunk::Data>,
    >,
) {
    for (chunk_data, mut chunk_queued_updates) in chunks.iter_mut() {
        chunk_queued_updates.0.clear();
        for (section_y, section) in chunk_data.sections.iter().enumerate() {
            for &block_state_change in &section.block_state_changes {
                chunk_queued_updates
                    .0
                    .insert(block_state_change | (section_y as u16) << 12);
            }
        }
    }
}
