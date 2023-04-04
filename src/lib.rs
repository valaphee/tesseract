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
    pub struct SubscriptionDistance(pub u8);

    #[derive(Default, Component)]
    pub struct Subscriptions(pub HashSet<IVec2>);
}
