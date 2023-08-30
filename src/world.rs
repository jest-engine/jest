use std::{cell::UnsafeCell, sync::Arc};

use slotmap::HopSlotMap;
use tokio::sync::RwLock;

use crate::entities::{Entity, EntityId, EntityMut, EntityRef};

pub struct World {
    entities: UnsafeCell<HopSlotMap<EntityId, RwLock<Entity>>>,
    outer: RwLock<()>,
}
impl World {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: UnsafeCell::new(HopSlotMap::with_key()),
            outer: RwLock::new(()),
        })
    }

    pub async fn insert(&self, entity: Entity) -> EntityId {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }.insert(RwLock::new(entity))
    }
    pub async fn remove(&self, id: EntityId) -> Option<Entity> {
        let _outer = self.outer.write().await;
        unsafe { &mut *self.entities.get() }
            .remove(id)
            .map(|e| e.into_inner())
    }

    pub async fn get(&self, id: EntityId) -> Option<EntityRef> {
        let _outer = self.outer.read().await;
        let inner = unsafe { &*self.entities.get() }.get(id)?;
        Some(EntityRef {
            _outer,
            inner: inner.read().await,
        })
    }
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
