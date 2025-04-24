use bevy::{ecs::component::Component, reflect::prelude::*};
use const_fnv1a_hash::fnv1a_hash_str_64;
use serde::{Deserialize, Serialize};

/// A unique identifier for a prototype.
///
/// This is either used to retrieve a prototype from a registry,
/// or to reference a prototype from another prototype.
///
/// e.g. a recipe may reference an item prototype as an ingredient.
#[derive(Component, Reflect)]
#[reflect(Clone, Serialize, Deserialize)]
pub struct PrototypeId<T> {
    hash: u64,
    #[reflect(ignore)]
    _marker: core::marker::PhantomData<T>,
}

impl<T> PrototypeId<T> {
    /// Creates a new prototype id from a string.
    #[must_use]
    pub const fn from_name(name: &str) -> Self {
        Self {
            hash: fnv1a_hash_str_64(name),
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates a new prototype id from a raw hash.
    #[must_use]
    pub const fn from_raw(hash: u64) -> Self {
        Self {
            hash,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T> PartialEq for PrototypeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T> Eq for PrototypeId<T> {}

impl<T> Copy for PrototypeId<T> {}

impl<T> Clone for PrototypeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> core::fmt::Debug for PrototypeId<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PrototypeId").field(&self.hash).finish()
    }
}

impl<T> core::fmt::Display for PrototypeId<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.hash)
    }
}

impl<T> core::hash::Hash for PrototypeId<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

unsafe impl<T> Send for PrototypeId<T> {}
unsafe impl<T> Sync for PrototypeId<T> {}

impl<T> From<&str> for PrototypeId<T> {
    fn from(value: &str) -> Self {
        Self::from_name(value)
    }
}

impl<T> From<Box<str>> for PrototypeId<T> {
    fn from(value: Box<str>) -> Self {
        Self::from_name(&value)
    }
}

impl<T> From<String> for PrototypeId<T> {
    fn from(value: String) -> Self {
        Self::from_name(&value)
    }
}

impl<T> Serialize for PrototypeId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.hash)
    }
}

impl<'de, T> Deserialize<'de> for PrototypeId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Default)]
        struct PrototypeIdVisitor<T>(core::marker::PhantomData<T>);

        impl<T> serde::de::Visitor<'_> for PrototypeIdVisitor<T> {
            type Value = PrototypeId<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string or a u64")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrototypeId::from_name(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrototypeId::from_raw(v))
            }
        }

        deserializer.deserialize_any(PrototypeIdVisitor(core::marker::PhantomData))
    }
}

/// A prototype name.
/// This is a wrapper around a `PrototypeId` that also stores the name as a string.
#[derive(Component, Reflect)]
#[reflect(Clone, Serialize, Deserialize)]
pub struct PrototypeName<T> {
    id: PrototypeId<T>,
    name: String,
}

impl<T> PrototypeName<T> {
    /// Creates a new prototype name from a string.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        Self {
            id: PrototypeId::from_name(name),
            name: name.to_string(),
        }
    }

    pub fn id(&self) -> &PrototypeId<T> {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> PartialEq for PrototypeName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for PrototypeName<T> {}

impl<T> Clone for PrototypeName<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

impl<T> core::fmt::Debug for PrototypeName<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PrototypeName")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl<T> core::fmt::Display for PrototypeName<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<T> core::hash::Hash for PrototypeName<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

unsafe impl<T> Send for PrototypeName<T> {}
unsafe impl<T> Sync for PrototypeName<T> {}

impl<T> From<&str> for PrototypeName<T> {
    fn from(value: &str) -> Self {
        Self::from_name(value)
    }
}

impl<T> From<Box<str>> for PrototypeName<T> {
    fn from(value: Box<str>) -> Self {
        Self::from_name(&value)
    }
}

impl<T> From<String> for PrototypeName<T> {
    fn from(value: String) -> Self {
        Self::from_name(&value)
    }
}

impl<T> Serialize for PrototypeName<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

impl<'de, T> Deserialize<'de> for PrototypeName<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(Self::from_name(&name))
    }
}

/// A type erased version of [`PrototypeId`].
#[derive(Component, Reflect)]
#[reflect(Serialize, Deserialize)]
pub struct ErasedPrototypeId {
    hash: u64,
}

impl ErasedPrototypeId {
    /// Creates a new erased prototype id from a string.
    #[must_use]
    pub const fn from_name(name: &str) -> Self {
        Self {
            hash: fnv1a_hash_str_64(name),
        }
    }

    /// Creates a new erased prototype id from a raw hash.
    #[must_use]
    pub const fn from_raw(hash: u64) -> Self {
        Self { hash }
    }
}

impl PartialEq for ErasedPrototypeId {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for ErasedPrototypeId {}

impl Copy for ErasedPrototypeId {}

impl Clone for ErasedPrototypeId {
    fn clone(&self) -> Self {
        *self
    }
}

impl core::fmt::Debug for ErasedPrototypeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ErasedPrototypeId")
            .field(&self.hash)
            .finish()
    }
}

impl core::fmt::Display for ErasedPrototypeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.hash)
    }
}

impl core::hash::Hash for ErasedPrototypeId {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl From<&str> for ErasedPrototypeId {
    fn from(name: &str) -> Self {
        Self::from_name(name)
    }
}

impl From<Box<str>> for ErasedPrototypeId {
    fn from(name: Box<str>) -> Self {
        Self::from_name(&name)
    }
}

impl From<String> for ErasedPrototypeId {
    fn from(name: String) -> Self {
        Self::from_name(&name)
    }
}

impl From<u64> for ErasedPrototypeId {
    fn from(hash: u64) -> Self {
        Self::from_raw(hash)
    }
}

impl<T> From<PrototypeId<T>> for ErasedPrototypeId {
    fn from(id: PrototypeId<T>) -> Self {
        Self::from_raw(id.hash)
    }
}

impl<T> From<ErasedPrototypeId> for PrototypeId<T> {
    fn from(id: ErasedPrototypeId) -> Self {
        Self::from_raw(id.hash)
    }
}

impl From<ErasedPrototypeName> for ErasedPrototypeId {
    fn from(name: ErasedPrototypeName) -> Self {
        name.id
    }
}

impl<T> From<PrototypeName<T>> for ErasedPrototypeId {
    fn from(name: PrototypeName<T>) -> Self {
        name.id.into()
    }
}

impl Serialize for ErasedPrototypeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.hash)
    }
}

impl<'de> Deserialize<'de> for ErasedPrototypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Default)]
        struct ErasedPrototypeIdVisitor;

        impl serde::de::Visitor<'_> for ErasedPrototypeIdVisitor {
            type Value = ErasedPrototypeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string or a u64")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ErasedPrototypeId::from_name(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ErasedPrototypeId::from_raw(v))
            }
        }

        deserializer.deserialize_any(ErasedPrototypeIdVisitor)
    }
}

/// A type erased version of [`PrototypeName`].
#[derive(Component, Reflect)]
#[reflect(Clone, Serialize, Deserialize)]
pub struct ErasedPrototypeName {
    id: ErasedPrototypeId,
    name: String,
}

impl ErasedPrototypeName {
    /// Creates a new name from a string.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        Self {
            id: ErasedPrototypeId::from_name(name),
            name: String::from(name),
        }
    }

    pub fn id(&self) -> ErasedPrototypeId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for ErasedPrototypeName {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ErasedPrototypeName {}

impl Clone for ErasedPrototypeName {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

impl core::fmt::Debug for ErasedPrototypeName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ErasedPrototypeName")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl core::fmt::Display for ErasedPrototypeName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl core::hash::Hash for ErasedPrototypeName {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<&str> for ErasedPrototypeName {
    fn from(name: &str) -> Self {
        Self::from_name(name)
    }
}

impl From<Box<str>> for ErasedPrototypeName {
    fn from(name: Box<str>) -> Self {
        Self::from_name(&name)
    }
}

impl From<String> for ErasedPrototypeName {
    fn from(name: String) -> Self {
        Self::from_name(&name)
    }
}

impl<T> From<PrototypeName<T>> for ErasedPrototypeName {
    fn from(name: PrototypeName<T>) -> Self {
        Self {
            id: name.id.into(),
            name: name.name,
        }
    }
}

impl<T> From<ErasedPrototypeName> for PrototypeName<T> {
    fn from(name: ErasedPrototypeName) -> Self {
        Self {
            id: name.id.into(),
            name: name.name,
        }
    }
}

impl Serialize for ErasedPrototypeName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

impl<'de> Deserialize<'de> for ErasedPrototypeName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(Self::from_name(&name))
    }
}
