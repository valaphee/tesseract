#![feature(result_flattening)]

use bevy::ecs::schedule::ScheduleLabel;

pub mod actor;
pub mod level;
pub mod persistence;
pub mod registry;
pub mod replication;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreLoad;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Load;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostLoad;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Save;
