use std::{cell::UnsafeCell, sync::Arc};

use slotmap::HopSlotMap;
use tokio::sync::RwLock;

use crate::entities::{Entity, EntityId, EntityMut, EntityRef};

/// A world is a collection of [entities](Entity). It manages important
/// ECS functions, such as queries and systems, and it is the center of your game.
/// # Usage
/// Check the docs of [`Entity`] for some examples on how to use a world.
///
/// # Performance
/// Our `World` implementation is designed to be O(1) in every aspect.
/// It is also designed to scale well to multiple threads.
pub struct World {
    entities: UnsafeCell<HopSlotMap<EntityId, RwLock<Entity>>>,
    outer: RwLock<()>,
}
impl World {
    /// Creates a new, empty world.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: UnsafeCell::new(HopSlotMap::with_key()),
            outer: RwLock::new(()),
        })
    }

    /// Inserts an entity into the world. Use this if you already have an [`Entity`] object.
    /// Otherwise, use [`EntityBuilder`](crate::entities::builder::EntityBuilder) to create one.
    pub async fn insert(&self, entity: Entity) -> EntityId {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }.insert(RwLock::new(entity))
    }

    /// Removes an entity from the world by ID. Returns the entity if it existed.
    pub async fn remove(&self, id: EntityId) -> Option<Entity> {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }
            .remove(id)
            .map(|e| e.into_inner())
    }

    /// Gets an immutable reference to the entity specified by `id`.
    /// See the docs of [`EntityRef`] for more information.
    pub async fn get(&self, id: EntityId) -> Option<EntityRef> {
        let _outer = self.outer.read().await;
        let inner = unsafe { &*self.entities.get() }.get(id)?;
        Some(EntityRef {
            _outer,
            inner: inner.read().await,
        })
    }

    /// Gets a mutable reference to the entity specified by `id`.
    /// See the docs of [`EntityMut`] for more information.
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
/// [`World`] uses a system of [`RwLock`]s for sync interior mutability.
/// An interested reader can browse the source to understand the implementation
/// details.
unsafe impl Sync for World {}
