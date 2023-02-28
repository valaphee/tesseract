pub mod connection;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use anyhow::bail;

use bevy::app::ScheduleRunnerSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tesseract_protocol::Decode;
use tesseract_protocol::packet::c2s;
use tesseract_protocol::types::VarInt;
use crate::connection::ConnectionPlugin;

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
