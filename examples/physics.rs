use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, math::DVec3, prelude::*};

use tesseract_base::{hierarchy::IndexedChildren, *};

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
        (level::update_time, tesseract_physics::update_fluids),
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
    commands.spawn((
        block::Base,
        tesseract_java::block::Name::new("minecraft:air"),
    ));
    commands.spawn((
        block::Base,
        item::Base,
        tesseract_java::block::Name::new("minecraft:bedrock"),
    ));
    commands.spawn((
        block::Base,
        item::Base,
        tesseract_java::block::Name::new("minecraft:dirt"),
    ));
    commands.spawn((
        block::Base,
        item::Base,
        tesseract_java::block::Name::new("minecraft:grass_block"),
    ));
    commands.spawn((
        block::Base,
        tesseract_physics::Fluid { volume: 7 },
        tesseract_java::block::Name::new("minecraft:water[level=0]"),
    ));
    commands.spawn_batch((0..7).map(|volume| {
        (
            block::Base,
            tesseract_physics::Fluid { volume },
            tesseract_java::block::Name::new(format!("minecraft:water[level={}]", 7 - volume)),
        )
    }));
}

fn spawn_levels(mut commands: Commands) {
    commands.spawn((
        level::LevelBundle {
            base: level::Base::new("minecraft:overworld", "minecraft:overworld"),
            age_and_time: Default::default(),
        },
        IndexedChildren::<IVec2>::default(),
    ));
}

#[allow(clippy::type_complexity)]
fn spawn_players(
    mut commands: Commands,

    level_access: Query<Entity, With<level::Base>>,

    for_players: Query<
        (Entity, &tesseract_java::replication::Connection),
        (
            Added<tesseract_java::replication::Connection>,
            Without<actor::Base>,
        ),
    >,
) {
    for (player, connection) in for_players.iter() {
        commands
            .entity(player)
            .insert((actor::player::PlayerBundle {
                base: actor::Base {
                    id: connection.user().id,
                },
                position: actor::Position(DVec3::new(0.0, 6.0, 0.0)),
                rotation: Default::default(),
                interaction: Default::default(),
            },))
            .set_parent(level_access.single());
    }
}

fn spawn_chunks(mut commands: Commands, for_chunks: Query<Entity, Added<level::chunk::Base>>) {
    if for_chunks.is_empty() {
        return;
    }

    let air_id = 0;
    let bedrock_id = 1;
    let dirt_id = 2;
    let grass_block_id = 3;
    for chunk in for_chunks.iter() {
        let mut chunk_data = level::chunk::Data::new(24, 4, air_id, 0);

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
