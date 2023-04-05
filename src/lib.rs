pub mod actor;
pub mod block;
pub mod item;
pub mod level;

pub mod persistence {
    use bevy::prelude::*;

    #[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct UpdateFlush;
}

pub mod replication {
    use std::collections::HashSet;

    use bevy::prelude::*;

    #[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct UpdateFlush;

    #[derive(Default, Component)]
    pub struct Replication {
        pub subscriber: HashSet<Entity>,
        pub replicated: Vec<Entity>,
    }

    #[derive(Default, Component)]
    pub struct Subscription {
        pub radius: u8,

        pub last_center: IVec2,
        pub last_radius: u8,
    }
}
