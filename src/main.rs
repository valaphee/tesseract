use std::time::Duration;

use bevy::app::ScheduleRunnerSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;

use crate::connection::ConnectionPlugin;

pub mod connection;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)

        .add_plugin(ConnectionPlugin::default())

        .run();
}
