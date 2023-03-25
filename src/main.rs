use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use tesseract_protocol::types::{Biome, DamageType};

mod actor;
mod level;
mod persistence;
mod registry;
mod replication;

fn main() {
    // initialize logging
    let (non_blocking_file_appender, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily("logs", "tesseract.log"));
    tracing_subscriber::registry()
        .with(EnvFilter::try_new("debug").unwrap())
        .with(
            tracing_subscriber::fmt::Layer::new()
                .with_ansi(false)
                .with_writer(non_blocking_file_appender),
        )
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    // create and run app
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugins(MinimalPlugins)
        .insert_resource(registry::Registries::new(
            "generated/reports/registries.json",
        ))
        .insert_resource(registry::BlockStateRegistry::new(
            "generated/reports/blocks.json",
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
        .add_plugin(persistence::PersistencePlugin::default())
        .add_systems(PostUpdate, level::chunk::update_hierarchy)
        .run();
}
