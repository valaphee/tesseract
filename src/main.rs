#![feature(result_flattening)]

use std::{fs::File, path::Path, time::Duration};

use bevy::{
    app::{MainScheduleOrder, ScheduleRunnerSettings},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tesseract_protocol::types::{Biome, DamageType, DimensionType};

use crate::{persistence::PersistencePlugin, replication::ReplicationPlugin};

mod actor;
mod level;
mod persistence;
mod registry;
mod replication;

fn main() {
    // initialize logging
    let (non_blocking_file_appender, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily("logs", "tesseract.log"));
    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_new("info,tesseract=debug").unwrap());

    #[cfg(feature = "tracing")]
    let registry = registry
        .with(tracing_tracy::TracyLayer::new());

    registry.with(
            tracing_subscriber::fmt::Layer::new()
                .with_ansi(false)
                .with_writer(non_blocking_file_appender),
        )
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    // load or create config
    let config_path = Path::new("config.json");
    let config = if config_path.exists() {
        serde_json::from_reader(File::open(config_path).unwrap()).unwrap()
    } else {
        let config = Config::default();
        serde_json::to_writer_pretty(File::create(config_path).unwrap(), &config).unwrap();
        config
    };

    // create and run app
    let mut app = App::new();
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 20.0,
    )))
    .add_plugins(MinimalPlugins)
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
    .add_plugin(config.persistence)
    .add_plugin(config.replication)
    .add_systems(PostLoad, actor::initialize_players)
    .add_systems(PostUpdate, level::update_time)
    .add_systems(PostUpdate, level::chunk::update_hierarchy);

    let mut order = app.world.resource_mut::<MainScheduleOrder>();
    order.insert_after(First, PreLoad);
    order.insert_after(PreLoad, Load);
    order.insert_after(Load, PostLoad);
    order.insert_after(PostUpdate, Save);

    app.run();
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct PreLoad;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Load;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct PostLoad;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Save;

#[derive(Default, Serialize, Deserialize)]
struct Config {
    persistence: PersistencePlugin,
    replication: ReplicationPlugin,
}
