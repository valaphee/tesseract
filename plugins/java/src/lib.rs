#![feature(result_flattening)]

pub use persistence::{PersistencePlugin, PersistencePluginLevel};
pub use registry::RegistryPlugin;
pub use replication::ReplicationPlugin;

pub mod persistence;
pub mod registry;
pub mod replication;
