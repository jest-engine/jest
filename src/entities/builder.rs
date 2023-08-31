use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::world::World;

use super::{
    errors::{self, AlreadyExists},
    Entity, EntityId,
};

/// A builder for creating entities and adding them to a world.
#[derive(Default)]
pub struct EntityBuilder {
    components: HashMap<TypeId, Box<dyn Any + Send>>,
}
impl EntityBuilder {
    /// Creates a new entity builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a component of type `T` to the entity, returning [`AlreadyExists`](errors::AlreadyExists) if
    /// a component of the same type already exists.. `T` must satisfy
    /// [`'static`](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound)
    /// and [`Send`].
    pub fn add<T: Any + Send>(&mut self, component: T) -> Result<&mut Self, AlreadyExists> {
        match self.components.entry(TypeId::of::<T>()) {
            Entry::Occupied(_) => Err(errors::AlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(Box::new(component));
                Ok(self)
            }
        }
    }

    /// Builds the entity and adds it to the world, returning its ID.
    pub async fn build(self, world: &Arc<World>) -> EntityId {
        world
            .insert(Entity {
                components: self.components,
                _world: world.clone(),
            })
            .await
    }
}
