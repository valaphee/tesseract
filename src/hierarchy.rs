use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};
use std::{collections::HashMap, hash::Hash};

#[derive(Component)]
pub struct ParentWithIndex<T> {
    pub index: T,
    pub parent: Entity,
}

#[derive(Component, Default)]
pub struct IndexedChildren<T>(pub HashMap<T, Entity>);

struct SetChild<T> {
    index: T,
    parent: Entity,
    child: Option<Entity>,
}

impl<T: Sync + Send + 'static + Eq + PartialEq + Hash + Clone> Command for SetChild<T> {
    fn write(self, world: &mut World) {
        if let Some(child) = self.child {
            {
                let mut child = world.entity_mut(child);
                if child.contains::<ParentWithIndex<T>>() {
                    panic!("Child already has a parent")
                }
                child.insert(ParentWithIndex {
                    index: self.index.clone(),
                    parent: self.parent,
                });
            }

            let mut parent = world.entity_mut(self.parent);
            if let Some(mut indexed_children) = parent.get_mut::<IndexedChildren<T>>() {
                if let Some(child) = indexed_children.0.insert(self.index, child) {
                    remove_children::<T>(world, child);
                }
            } else {
                parent.insert(IndexedChildren(HashMap::from([(self.index, child)])));
            }
        } else {
            let mut parent = world.entity_mut(self.parent);
            if let Some(mut indexed_children) = parent.get_mut::<IndexedChildren<T>>() {
                if let Some(child) = indexed_children.0.remove(&self.index) {
                    remove_children::<T>(world, child);
                }
            }
        }
    }
}

fn remove_children<T: Sync + Send + 'static + Eq + PartialEq + Hash + Clone>(
    world: &mut World,
    parent: Entity,
) {
    let mut children = vec![parent];
    while let Some(child) = children.pop() {
        if let Some(indexed_children) = world.entity_mut(child).get::<IndexedChildren<T>>() {
            children.extend(indexed_children.0.values());
        }
        world.entity_mut(child).despawn();
    }
}

pub trait EntityCommandsExt<T: Sync + Send + 'static + Eq + PartialEq + Hash + Clone> {
    fn set_indexed_child(&mut self, index: T, child: Option<Entity>) -> &mut Self;
}

impl<'w, 's, 'a, T: Sync + Send + 'static + Eq + PartialEq + Hash + Clone> EntityCommandsExt<T>
    for EntityCommands<'w, 's, 'a>
{
    fn set_indexed_child(&mut self, index: T, child: Option<Entity>) -> &mut Self {
        let parent = self.id();
        self.commands().add(SetChild {
            index,
            parent,
            child,
        });
        self
    }
}
