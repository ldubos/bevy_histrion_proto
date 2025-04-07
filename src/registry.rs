use core::borrow::Borrow;

use crate::{RawPrototypeAsset, identifier::Id, prototype::Prototype};
use bevy::{
    asset::{AssetId, AssetPath, AssetServer, Handle},
    ecs::{
        event::{Event, EventWriter},
        resource::Resource,
        system::{Res, ResMut, SystemParam},
    },
    platform_support::collections::HashMap,
    prelude::Deref,
    reflect::*,
};
use thiserror::Error;

/// An error that can occur when modifying a [`PrototypeRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegistryError<P: Prototype> {
    /// The Id of the item is already in use.
    #[error("The ID {:?} is already in use.", _0)]
    DuplicateId(Id<P>),
    /// The item with the given Id was not found.
    #[error("The item with ID {:?} was not found.", _0)]
    NotFound(Id<P>),
}

#[derive(Debug, Clone, Resource, Reflect)]
pub struct PrototypeRegistry<P: Prototype> {
    prototypes: HashMap<Id<P>, P>,
}

impl<P: Prototype> PrototypeRegistry<P> {
    /// Inserts a new prototype into the registry.
    ///
    /// If a duplicate entry is found, returns
    /// [`Err(RegistryError::DuplicateId(id))`](RegistryError::DuplicateId).
    pub fn insert(&mut self, prototype: P) -> Result<(), RegistryError<P>> {
        use bevy::platform_support::collections::hash_map::Entry;

        let id = prototype.id();

        match self.prototypes.entry(id) {
            Entry::Occupied(_) => Err(RegistryError::DuplicateId(id)),
            Entry::Vacant(v) => {
                v.insert(prototype);
                Ok(())
            }
        }
    }

    /// Inserts multiple prototypes into the registry.
    ///
    /// If a duplicate entry is found, returns
    /// [`Err(RegistryError::DuplicateId(id))`](RegistryError::DuplicateId).
    #[inline]
    pub fn insert_many(&mut self, prototypes: &[P]) -> Result<(), RegistryError<P>> {
        prototypes
            .iter()
            .try_for_each(|prototype| self.insert(prototype.clone()))
    }

    /// Gets a reference to a prototype from the registry by its [`Id`].
    ///
    /// Returns [`None`] if no prototype with the given [`Id`] is found.
    #[must_use]
    pub fn get(&self, id: &Id<P>) -> Option<&P> {
        self.prototypes.get(id)
    }

    /// Gets a reference to a prototype from the registry by its name.
    ///
    /// Returns [`None`] if no prototype with the given name is found.
    #[inline]
    #[must_use]
    pub fn get_by_name(&self, name: impl Borrow<str>) -> Option<&P> {
        self.get(&Id::from_name(name.borrow()))
    }

    /// Gets a mutable reference to a prototype from the registry by its [`Id`].
    ///
    /// Returns [`None`] if no prototype with the given [`Id`] is found.
    #[must_use]
    pub fn get_mut(&mut self, id: &Id<P>) -> Option<&mut P> {
        self.prototypes.get_mut(id)
    }

    /// Gets a mutable reference to a prototype from the registry by its name.
    ///
    /// Returns [`None`] if no prototype with the given name is found.
    #[inline]
    #[must_use]
    pub fn get_mut_by_name(&mut self, name: impl Borrow<str>) -> Option<&mut P> {
        self.get_mut(&Id::from_name(name.borrow()))
    }

    /// Removes a prototype from the registry.
    ///
    /// The prototype removed is returned, if it was found.
    ///
    /// If the entry can't be found, returns
    /// [`Err(RegistryError::NotFound(id))`](RegistryError::NotFound)
    pub fn remove(&mut self, id: &Id<P>) -> Result<P, RegistryError<P>> {
        self.prototypes
            .remove(id)
            .ok_or(RegistryError::NotFound(*id))
    }

    /// Removes a prototype from the registry by name.
    ///
    /// The prototype removed is returned, if it was found.
    ///
    /// If the entry can't be found, returns
    /// [`Err(RegistryError::NotFound(id))`](RegistryError::NotFound)
    pub fn remove_by_name(&mut self, name: impl Borrow<str>) -> Result<P, RegistryError<P>> {
        self.remove(&Id::from_name(name.borrow()))
    }

    /// Returns an iterator over the ids of the prototypes in the registry.
    pub fn ids(&self) -> impl Iterator<Item = &Id<P>> {
        self.prototypes.keys()
    }

    /// Returns an iterator over the prototypes in the registry.
    pub fn prototypes(&self) -> impl Iterator<Item = &P> {
        self.prototypes.values()
    }

    /// Clears the registry of all prototypes.
    pub fn clear(&mut self) {
        self.prototypes.clear();
    }
}

impl<P: Prototype> Default for PrototypeRegistry<P> {
    fn default() -> Self {
        Self {
            prototypes: HashMap::default(),
        }
    }
}

#[derive(SystemParam, Deref)]
pub struct Reg<'w, P: Prototype> {
    registry: Res<'w, PrototypeRegistry<P>>,
}

#[derive(SystemParam, Deref)]
pub struct RegMut<'w, P: Prototype> {
    #[deref]
    registry: ResMut<'w, PrototypeRegistry<P>>,
    events: EventWriter<'w, RegistryEvent<P>>,
}

impl<P: Prototype> RegMut<'_, P> {
    /// Inserts a new prototype into the registry.
    ///
    /// If a duplicate entry is found, returns
    /// [`Err(RegistryError::DuplicateId(id))`](RegistryError::DuplicateId).
    pub fn insert(&mut self, prototype: P) -> Result<(), RegistryError<P>> {
        let id = prototype.id();
        match self.registry.insert(prototype) {
            Ok(()) => {
                self.events.write(RegistryEvent::Added(id));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Removes a prototype from the registry.
    ///
    /// If the prototype is not found, returns
    /// [`Err(RegistryError::NotFound(id))`](RegistryError::NotFound).
    pub fn remove(&mut self, id: Id<P>) -> Result<(), RegistryError<P>> {
        match self.registry.remove(&id) {
            Ok(prototype) => {
                self.events.write(RegistryEvent::Removed(prototype));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Removes a prototype from the registry by name.
    ///
    /// The prototype removed is returned, if it was found.
    ///
    /// If the entry can't be found, returns
    /// [`Err(RegistryError::NotFound(id))`](RegistryError::NotFound)
    pub fn remove_by_name(&mut self, name: &str) -> Result<(), RegistryError<P>> {
        match self.registry.remove_by_name(name) {
            Ok(prototype) => {
                self.events.write(RegistryEvent::Removed(prototype));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub enum RegistryEvent<P: Prototype> {
    Added(Id<P>),
    Removed(P),
}

// TODO: find a better way to avoid dropping assets in case of "indivual" asset loading
#[derive(Debug, Default, Clone, Resource)]
pub(crate) struct LoadingPrototypes {
    pub handles: HashMap<AssetId<RawPrototypeAsset>, Handle<RawPrototypeAsset>>,
}

#[derive(SystemParam)]
pub struct PrototypeServer<'w> {
    asset_server: Res<'w, AssetServer>,
    loading: ResMut<'w, LoadingPrototypes>,
}

impl<'w> PrototypeServer<'w> {
    // TODO: find a way to load only prototypes inside a folder
    pub fn load_prototypes(&mut self, path: impl Into<AssetPath<'w>>) {
        let handle: Handle<RawPrototypeAsset> = self.asset_server.load(path);
        self.loading.handles.insert(handle.id(), handle);
    }

    pub fn load_prototypes_folder(&mut self, path: impl Into<AssetPath<'w>>) {
        let _ = self.asset_server.load_folder(path);
    }
}
