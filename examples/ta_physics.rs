use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, math::DVec3, prelude::*};

use tesseract_base::*;
use tesseract_java_protocol::types::Biome;

fn main() {
    // create and run app
    let mut app = App::new();
    // required (Bevy)
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 20.0,
    )))
    .add_plugins(MinimalPlugins)
    .add_plugin(LogPlugin::default())
    // required (Tesseract)
    .add_systems(Startup, (item::build_lut, block::build_lut))
    .add_systems(
        PostUpdate,
        (level::chunk::update_hierarchy, level::chunk::queue_updates),
    )
    // required (Java Edition)
    .add_plugin(tesseract_java::RegistryPlugin::default())
    .add_plugin(tesseract_java::ReplicationPlugin::default())
    // gameplay
    .add_systems(
        Update,
        (
            level::update_time,
            actor::player::update_interactions,
            tesseract_ta_physics::update_fluids,
        ),
    )
    // gameplay (custom)
    .add_systems(PreStartup, register_blocks_and_items)
    .add_systems(Startup, spawn_levels)
    .add_systems(
        First,
        (spawn_players, spawn_chunks).after(replication::UpdateFlush),
    );

    app.run();
}

fn register_blocks_and_items(mut commands: Commands) {
    commands.spawn(block::Base::new("minecraft:air"));
    commands.spawn((
        block::Base::new("minecraft:bedrock"),
        item::Base::new("minecraft:bedrock"),
    ));
    commands.spawn((
        block::Base::new("minecraft:dirt"),
        item::Base::new("minecraft:dirt"),
    ));
    commands.spawn((
        block::Base::new("minecraft:grass_block[snowy=false]"),
        item::Base::new("minecraft:grass_block"),
    ));

    commands.spawn((
        block::Base::new("minecraft:water[level=0]"),
        tesseract_ta_physics::Fluid { volume: 7 },
        item::Base::new("minecraft:water_bucket"),
    ));
    commands.spawn_batch((0..7).map(|volume| {
        (
            block::Base::new(format!("minecraft:water[level={}]", 7 - volume)),
            tesseract_ta_physics::Fluid { volume },
        )
    }));
}

fn spawn_levels(mut commands: Commands) {
    commands.spawn(level::LevelBundle {
        base: level::Base::new("minecraft:overworld", "minecraft:overworld"),
        age_and_time: default(),
        chunks: default(),
    });
}

#[allow(clippy::type_complexity)]
fn spawn_players(
    mut commands: Commands,
    levels: Query<Entity, With<level::Base>>,
    players: Query<
        (Entity, &tesseract_java::replication::Connection),
        (
            Added<tesseract_java::replication::Connection>,
            Without<actor::Base>,
        ),
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
    biome_registry: Res<tesseract_java::registry::DataRegistry<Biome>>,
    mut commands: Commands,
    chunks: Query<Entity, Added<level::chunk::Base>>,
) {
    if chunks.is_empty() {
        return;
    }

    let air_id = block_lut.id("minecraft:air");
    let bedrock_id = block_lut.id("minecraft:bedrock");
    let dirt_id = block_lut.id("minecraft:dirt");
    let grass_block_id = block_lut.id("minecraft:grass_block[snowy=false]");
    for chunk in chunks.iter() {
        let mut chunk_data =
            level::chunk::Data::new(24, 4, air_id, biome_registry.id("minecraft:plains"));

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
