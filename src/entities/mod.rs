use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use slotmap::new_key_type;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::world::World;

/// A builder for creating entities and adding them to a world.
pub mod builder;

/// Error types for entity operations
pub mod errors {
    use std::{
        error::Error,
        fmt::{self, Display, Formatter},
    };

    /// Error type returned from [`Entity::add`]
    /// A component of this type is already a part of the specified entity.
    #[derive(Debug)]
    pub struct AlreadyExists;
    impl Display for AlreadyExists {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "component already exists")
        }
    }
    impl Error for AlreadyExists {}
}

new_key_type! {
    /// Unique identifier for an entity
    pub struct EntityId;
}

/// Entities are the base of ECS. An entity represents a single object in the world.
/// It is comprised of many components, which are just simple bits of data.
/// A component can be anything, so long as it satisfies
/// [`'static`](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound)
/// (tl;dr: it has no references) and [`Send`].
///
/// # Usage
/// ## Constructing an entity
/// In order to create an entity, an [`EntityBuilder`](builder::EntityBuilder) may be used like so:
/// ```rust
/// use jest::{world::World, entities::builder::EntityBuilder};
///
/// struct Foo {
///     bar: u32,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let world = World::new();
///
///     let mut builder = EntityBuilder::new();
///     builder.add(Foo {
///         bar: 42,
///     }).unwrap();
///     let entity_id = builder.build(&world).await;
///
///     assert!(world.get(entity_id).await.is_some());
/// }
/// ```
/// ## Accessing components
/// Components can be accessed using the [`Entity::get`] and [`Entity::get_mut`] methods.
/// They can also be added using the [`Entity::add`] method and removed using the
/// [`Entity::remove`] method. For example:
/// ```rust
/// use jest::{world::World, entities::{Entity, builder::EntityBuilder}};
///
/// struct Foo {
///    bar: u32,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let world = World::new();
///     // add a new, empty entity
///     let entity_id = EntityBuilder::new().build(&world).await;
///     
///     let mut entity = world
///         .get_mut(entity_id)
///         .await
///         .unwrap();
///
///     entity.add(Foo {
///         bar: 42,
///     }).unwrap();
///     let foo = entity.get::<Foo>().unwrap();
///     assert_eq!(foo.bar, 42);
///
///     entity.remove::<Foo>();
///     assert!(entity.get::<Foo>().is_none());
/// }
/// ```
pub struct Entity {
    components: HashMap<TypeId, Box<dyn Any + Send>>,
    // reference counter to the world
    _world: Arc<World>,
}
impl Entity {
    /// Adds a component of type `T` to the entity, returning [`AlreadyExists`](errors::AlreadyExists) if
    /// a component of the same type already exists.. `T` must satisfy
    /// [`'static`](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound)
    /// and [`Send`].
    pub fn add<T: Any + Send>(&mut self, component: T) -> Result<(), errors::AlreadyExists> {
        match self.components.entry(TypeId::of::<T>()) {
            Entry::Occupied(_) => Err(errors::AlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(Box::new(component));
                // TODO: notify world
                Ok(())
            }
        }
    }

    /// Removes a component of type `T` from the entity, returning it if it exists.
    pub fn remove<T: Any + Send>(&mut self) -> Option<T> {
        self.components
            .remove(&TypeId::of::<T>())
            .map(|c| *c.downcast::<T>().unwrap())
    }

    /// Get an immutable reference to the component of type `T` in this entity,
    /// if it exists.
    pub fn get<T: Any + Send>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|c| c.downcast_ref::<T>().unwrap())
    }

    /// Get a mutable reference to the component of type `T` in this entity,
    /// if it exists.
    pub fn get_mut<T: Any + Send>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|c| c.downcast_mut::<T>().unwrap())
    }
}

/// An immutable reference to an entity contained within a world.
/// This type implements `Deref` for usage as a normal reference.
///
/// Beware that holding this reference will block adding and removing
/// entities in the world, and block writing to this entity.
/// Be sure to drop it as soon as you're done with it.
pub struct EntityRef<'a> {
    pub(crate) _outer: RwLockReadGuard<'a, ()>,
    pub(crate) inner: RwLockReadGuard<'a, Entity>,
}
/// Get a reference to the underlying `Entity`.
impl Deref for EntityRef<'_> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A mutable reference to an entity contained within a world.
/// This type implements `Deref` and `DerefMut` for usage as a normal reference.
///
/// Beware that holding this reference will block adding and removing
/// entities in the world, and block accessing this entity.
/// Be sure to drop it as soon as you're done with it.
pub struct EntityMut<'a> {
    pub(crate) _outer: RwLockReadGuard<'a, ()>,
    pub(crate) inner: RwLockWriteGuard<'a, Entity>,
}
/// Get a reference to the underlying `Entity`.
impl Deref for EntityMut<'_> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
/// Get a mutable reference to the underlying `Entity`.
impl DerefMut for EntityMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
