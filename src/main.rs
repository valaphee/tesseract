use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};

mod actor;
mod block;
mod level;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0, // 1.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        // custom
        .add_plugin(actor::connection::ConnectionPlugin::default())
        .add_systems(Startup, level::load_level)
        .add_systems(PreUpdate, level::terrain::populate)
        // hierarchy
        .add_systems(PostUpdate, level::chunk::update_hierarchy)
        // subscribe
        .add_systems(PostUpdate, level::chunk::subscribe)
        // replicate
        .add_systems(PostUpdate, actor::replicate.after(level::chunk::replicate))
        .add_systems(PostUpdate, level::chunk::replicate)
        // debug
        //.add_systems(Last, tickln)
        .run();
}

// fn tickln() {
//    println!(".")
//}
