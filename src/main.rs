use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};

mod actor;
mod connection;
mod level;
mod persistence;
mod registry;
mod replication;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        // plugins
        .add_plugin(connection::ConnectionPlugin::default())
        .add_plugin(replication::ReplicationPlugin::default())
        // startup
        .insert_resource(registry::BlockStateRegistry::new("generated/reports/blocks.json"))
        .add_systems(Startup, level::spawn_levels)
        // game loop
        .add_systems(PreUpdate, persistence::load_chunks)
        .add_systems(PostUpdate, level::chunk::update_hierarchy)
        .run();
}
