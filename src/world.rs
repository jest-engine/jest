use std::{cell::UnsafeCell, sync::Arc};

use slotmap::HopSlotMap;
use tokio::sync::RwLock;

use crate::entities::{Entity, EntityId, EntityMut, EntityRef};


/// The world is a collection of entities
///
/// Allows for the creation, deletion, and retrieval of entities

/// Struct World that holds both entities and a RwLock to allow for many to one reading
/// and one to many writing. The RwLock will allow simultaneous getting and reading of the 
/// entities.  but only one writer. it will block all others.
pub struct World {
    entities: UnsafeCell<HopSlotMap<EntityId, RwLock<Entity>>>,
    outer: RwLock<()>,
}

impl World {
    /// Creates a new world, no parameters
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: UnsafeCell::new(HopSlotMap::with_key()),
            outer: RwLock::new(()),
        })
    }

    /// Creates a new component to by tracked
    /// by the world
    pub async fn insert(&self, entity: Entity) -> EntityId {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }.insert(RwLock::new(entity))
    }

    /// removes a component from the world,
    /// takes the id as a parameter
    pub async fn remove(&self, id: EntityId) -> Option<Entity> {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }
            .remove(id)
            .map(|e| e.into_inner())
    }

    /// get an immutable reference to the specified entity
    /// takes the id as a parameter
    pub async fn get(&self, id: EntityId) -> Option<EntityRef> {
        let _outer = self.outer.read().await;
        let inner = unsafe { &*self.entities.get() }.get(id)?;
        Some(EntityRef {
            _outer,
            inner: inner.read().await,
        })
    }

    /// get a mutable reference to the specified entity
    /// takes the id as a parameter
    pub async fn get_mut(&self, id: EntityId) -> Option<EntityMut> {
        let _outer = self.outer.read().await;
        let inner = unsafe { &*self.entities.get() }.get(id)?;
        Some(EntityMut {
            _outer,
            inner: inner.write().await,
        })
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}
