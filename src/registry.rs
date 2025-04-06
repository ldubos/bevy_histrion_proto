use core::borrow::Borrow;

use crate::{identifier::Id, prototype::Prototype};
use bevy::{
    ecs::{
        resource::Resource,
        system::{Res, ResMut, SystemParam},
    },
    platform_support::collections::HashMap,
    prelude::{Deref, DerefMut},
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

#[derive(SystemParam, Deref, DerefMut)]
pub struct RegMut<'w, P: Prototype> {
    registry: ResMut<'w, PrototypeRegistry<P>>,
}
