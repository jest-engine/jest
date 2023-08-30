use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::world::World;

use super::Entity;

#[derive(Default)]
pub struct EntityBuilder {
    components: HashMap<TypeId, Box<dyn Any>>,
}
impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: 'static>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }

    pub fn build(self, world: Arc<World>) -> Entity {
        Entity {
            components: self.components,
            _world: world,
        }
    }
}
