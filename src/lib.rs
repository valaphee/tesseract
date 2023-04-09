pub mod actor;
pub mod block;
pub mod hierarchy;
pub mod item;
pub mod level;

pub mod persistence {
    use bevy::prelude::*;

    /// Flushes all commands after loading
    #[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct UpdateFlush;
}

pub mod replication {
    use std::collections::HashSet;

    use bevy::prelude::*;

    /// Flushes all commands after processing player input
    #[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct UpdateFlush;

    // TODO: should be moved to replication
    #[derive(Default, Component)]
    pub struct Replication {
        pub subscriber: HashSet<Entity>,
        pub replicated: Vec<Entity>,
    }

    // TODO: should be moved to replication
    #[derive(Default, Component)]
    pub struct Subscription {
        pub radius: u8,

        pub last_center: IVec2,
        pub last_radius: u8,
    }
}
