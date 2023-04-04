use std::{collections::HashMap, time::Duration};

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, math::DVec3, prelude::*};

use tesseract_base::*;
use tesseract_protocol::types::{Biome, PalettedContainer};

fn main() {
    // create and run app
    let mut app = App::new();
    // required
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 20.0,
    )))
    .add_plugins(MinimalPlugins)
    .add_plugin(LogPlugin::default())
    .add_plugin(registry::RegistryPlugin::default())
    .add_plugin(replication::ReplicationPlugin::default())
    .add_plugin(persistence::PersistencePlugin(HashMap::from([(
        "minecraft:overworld".into(),
        persistence::PersistencePluginLevel {
            path: "levels/overworld".into(),
        },
    )])))
    .add_systems(PreStartup, registry::register_noop_blocks_and_items)
    .add_systems(Startup, (item::build_lut, block::build_lut))
    .add_systems(
        PostUpdate,
        (level::chunk::update_hierarchy, level::chunk::queue_updates),
    )
    // gameplay
    .add_systems(
        Update,
        (level::update_time, actor::player::update_interactions),
    )
    // custom
    .add_systems(
        First,
        (spawn_players, spawn_chunks).after(replication::UpdateFlush),
    )
    .add_systems(Update, block::update_fluids);

    app.run();
}

#[allow(clippy::type_complexity)]
fn spawn_players(
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
                inventory: actor::player::Inventory {
                    content: vec![None; 46],
                    selected_slot: 0,
                },
            },))
            .set_parent(levels.single());
    }
}

fn spawn_chunks(
    block_lut: Res<block::LookupTable>,
    biome_registry: Res<registry::DataRegistry<Biome>>,
    mut commands: Commands,
    chunks: Query<Entity, (Added<level::chunk::Base>, Without<level::chunk::Data>)>,
) {
    if chunks.is_empty() {
        return;
    }

    let air_id = block_lut.id("minecraft:air");
    let bedrock_id = block_lut.id("minecraft:bedrock");
    let dirt_id = block_lut.id("minecraft:dirt");
    let grass_block_id = block_lut.id("minecraft:grass_block[snowy=false]");
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

        commands.entity(chunk).insert(chunk_data);
    }
}
