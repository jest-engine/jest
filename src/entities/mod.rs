use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use slotmap::new_key_type;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::world::World;

/// imports the builder module for creating entities
pub mod builder;

/// Error types for entity operations
#[derive(Debug)]
pub struct AlreadyExists;
#[derive(Debug)]
pub struct DoesNotExist;

/// implementation of Display for error type `AlreadyExists`
impl fmt::Display for AlreadyExists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity already exists")
    }
}

/// implementation of Display for error type `DoesNotExist`
impl fmt::Display for DoesNotExist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity does not exist")
    }
}

new_key_type! {
    /// Unique identifier for an entity
    pub struct EntityId;
}

/// Entity is a collection of components
/// they can be identified by their EntityId
/// each entity has components that are composed
/// of a TypeId and a Boxed Any
pub struct Entity {
    components: HashMap<TypeId, Box<dyn Any>>,
    // reference counter to the world
    _world: Arc<World>,
}

/// implementation of Entity
/// allows for the creation, deletion, and retrieval of entities
impl Entity {
    /// Creates a new entity, takes in a reference to world
    pub fn new(world: Arc<World>) -> Self {
        Self {
            components: HashMap::new(),
            _world: world,
        }
    }

    /// Adds a component of type `T` to the entity
    pub fn add<T: 'static>(&mut self, component: T) -> Result<(), AlreadyExists> {
        match self.components.entry(TypeId::of::<T>()) {
            Entry::Occupied(_) => Err(AlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(Box::new(component));
                // TODO: notify world
                Ok(())
            }
        }
    }

    /// removes the entity from the world
    pub fn remove<T: 'static>(&mut self) -> Result<(), DoesNotExist> {
        match self.components.entry(TypeId::of::<T>()) {
            Entry::Occupied(entry) => {
                entry.remove();
                // TODO: notify world
                Ok(())
            }
            Entry::Vacant(_) => Err(DoesNotExist),
        }
    }

    /// get an immutable reference to the specified entity
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|c| c.downcast_ref::<T>().unwrap())
    }

    /// get a mutable reference to the specified entity
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|c| c.downcast_mut::<T>().unwrap())
    }
}

pub struct EntityRef<'a> {
    pub(crate) _outer: RwLockReadGuard<'a, ()>,
    pub(crate) inner: RwLockReadGuard<'a, Entity>,
}
impl Deref for EntityRef<'_> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct EntityMut<'a> {
    pub(crate) _outer: RwLockReadGuard<'a, ()>,
    pub(crate) inner: RwLockWriteGuard<'a, Entity>,
}
impl Deref for EntityMut<'_> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for EntityMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}