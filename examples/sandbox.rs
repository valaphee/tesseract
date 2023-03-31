use std::time::Duration;

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
    .add_plugin(replication::ReplicationPlugin::default())
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
    .add_systems(PreStartup, register_blocks_and_items)
    .add_systems(Startup, spawn_levels)
    .add_systems(
        First,
        (spawn_players, spawn_chunks).after(replication::UpdateFlush),
    );

    app.run();
}

fn register_blocks_and_items(mut commands: Commands) {
    commands.spawn(block::Base("minecraft:air".into()));
    commands.spawn((
        block::Base("minecraft:bedrock".into()),
        item::Base("minecraft:bedrock".into()),
    ));
    commands.spawn((
        block::Base("minecraft:dirt".into()),
        item::Base("minecraft:dirt".into()),
    ));
    commands.spawn((
        block::Base("minecraft:grass_block".into()),
        item::Base("minecraft:grass_block".into()),
    ));

    {
        let empty_bucket = commands.spawn(item::Base("minecraft:bucket".into())).id();

        let water = commands
            .spawn((
                block::Base("minecraft:water[level=0]".into()),
                block::Fluid {
                    volume: 7,
                    filter: 0,
                },
            ))
            .id();
        commands.spawn_batch((0..7).map(|volume| {
            (
                block::Base(format!("minecraft:water[level={}]", 7 - volume).into()),
                block::Fluid { volume, filter: 0 },
            )
        }));
        let filled_water_bucket = commands
            .spawn((
                item::Base("minecraft:water_bucket".into()),
                item::Bucket {
                    fluid: water,
                    empty: empty_bucket,
                },
            ))
            .id();

        commands
            .entity(empty_bucket)
            .insert(item::EmptyBucket([(water, filled_water_bucket)].into()));
    }
}

fn spawn_levels(mut commands: Commands) {
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

    chunks: Query<Entity, Added<level::chunk::Base>>,
) {
    if chunks.is_empty() {
        return;
    }

    let air_id = block_lut.0["minecraft:air"].index();
    let bedrock_id = block_lut.0["minecraft:bedrock"].index();
    let dirt_id = block_lut.0["minecraft:dirt"].index();
    let grass_block_id = block_lut.0["minecraft:grass_block"].index();
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
