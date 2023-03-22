use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};

mod actor;
mod chunk;
mod connection;
mod level;
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
        .add_systems(Startup, level::spawn_levels)
        // game loop
        .add_systems(PreUpdate, chunk::populate)
        .add_systems(PostUpdate, chunk::update_hierarchy)
        .run();
}
