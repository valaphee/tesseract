use std::time::Duration;

use bevy::{
    app::{MainScheduleOrder, ScheduleRunnerSettings},
    log::LogPlugin,
    math::DVec3,
    prelude::*,
};

use tesseract_base::*;
use tesseract_protocol::types::{Biome, DamageType, DimensionType, PalettedContainer};

fn main() {
    // create and run app
    let mut app = App::new();
    // required
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 10.0,
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
    .init_schedule(PreLoad)
    .init_schedule(Load)
    .init_schedule(PostLoad)
    .init_schedule(Save)
    .add_plugin(replication::ReplicationPlugin::default())
    .add_systems(PostUpdate, level::chunk::update_hierarchy)
    // gameplay
    .add_systems(Update, actor::player::update_interactions)
    .add_systems(Update, level::update_time)
    // custom
    .add_systems(PreStartup, spawn_level)
    .add_systems(PostLoad, spawn_players)
    .add_systems(PostLoad, spawn_chunks);

    // required
    let mut order = app.world.resource_mut::<MainScheduleOrder>();
    order.insert_after(First, PreLoad);
    order.insert_after(PreLoad, Load);
    order.insert_after(Load, PostLoad);
    order.insert_after(PostUpdate, Save);

    app.run();
}

fn spawn_level(mut commands: Commands) {
    commands.spawn(level::LevelBundle {
        level: level::Level {
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
    levels: Query<Entity, With<level::Level>>,
    players: Query<
        (Entity, &replication::Connection),
        (Added<replication::Connection>, Without<actor::Actor>),
    >,
) {
    for (player, connection) in players.iter() {
        commands
            .entity(player)
            .insert((actor::player::PlayerBundle {
                actor: actor::Actor {
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
    chunks: Query<Entity, Added<level::chunk::Chunk>>,
) {
    for chunk in chunks.iter() {
        let mut terrain = level::chunk::Terrain {
            sections: {
                let mut sections = vec![];
                for _ in 0..24 {
                    sections.push(level::chunk::TerrainSection {
                        block_states: PalettedContainer::SingleValue(0),
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

        let bedrock_id = block_state_registry.id("minecraft:bedrock");
        let dirt_id = block_state_registry.id("minecraft:dirt");
        let grass_block_id = block_state_registry.id("minecraft:grass_block");
        for x in 0..16 {
            for z in 0..16 {
                terrain.set(x, 0, z, bedrock_id);
                for y in 1..4 {
                    terrain.set(x, y, z, dirt_id);
                }
                terrain.set(x, 4, z, grass_block_id);
            }
        }
        for section in &mut terrain.sections {
            section.block_state_changes.clear();
        }

        commands.entity(chunk).insert(terrain);
    }
}
