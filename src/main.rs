use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};
use std::time::Duration;

mod actor;
mod block;
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
        .run();
}
