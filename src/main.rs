use std::time::Duration;

use crate::{actor::load_connection, level::load_level};
use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};

mod actor;
mod block;
mod chunk;
mod connection;
mod level;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(connection::ConnectionPlugin::default())
        .add_systems(Startup, load_level)
        .add_systems(PreUpdate, load_connection)
        .run();
}
