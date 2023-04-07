#![feature(result_flattening)]

use std::borrow::Cow;

use bevy::prelude::*;

pub use persistence::{PersistencePlugin, PersistencePluginLevel};
pub use registry::RegistryPlugin;
pub use replication::ReplicationPlugin;

pub mod persistence;
pub mod registry;
pub mod replication;

#[derive(Component)]
pub struct Tag(pub Vec<Cow<'static, str>>);

pub mod block {
    use std::{borrow::Cow, collections::BTreeMap};

    use bevy::prelude::*;

    #[derive(Component, Eq, PartialEq, Hash)]
    pub struct Name {
        pub(crate) name: String,
        pub(crate) properties: BTreeMap<String, String>
    }

    impl Name {
        pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
            let name = name.into();
            if let (Some(properties_begin), Some(properties_end)) =
                (name.find('['), name.rfind(']'))
            {
                Self {
                    name: name[..properties_begin].to_string(),
                    properties: name[properties_begin + 1..properties_end]
                        .split(',')
                        .map(|property| {
                            let (property_key, property_value) = property.split_once('=').unwrap();
                            (property_key.to_string(), property_value.to_string())
                        })
                        .collect(),
                }
            } else {
                Self {
                    name: name.to_string(),
                    properties: BTreeMap::default()
                }
            }
        }
    }

    #[derive(Component)]
    pub enum Auto {
        Snowy,
    }
}
