use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use slotmap::new_key_type;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::world::World;

pub mod builder;

pub struct AlreadyExists;
pub struct DoesNotExist;

new_key_type! {
    pub struct EntityId;
}

pub struct Entity {
    components: HashMap<TypeId, Box<dyn Any>>,
    _world: Arc<World>,
}
impl Entity {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            components: HashMap::new(),
            _world: world,
        }
    }

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

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|c| c.downcast_ref::<T>().unwrap())
    }
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
