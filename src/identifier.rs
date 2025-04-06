use bevy::{ecs::component::Component, reflect::Reflect};
use const_fnv1a_hash::fnv1a_hash_str_64;
use serde::{Deserialize, Serialize};

/// The unique identifier of type `T`.
#[derive(Component, Reflect, Serialize)]
pub struct Id<T> {
    hash: u64,
    #[reflect(ignore)]
    #[serde(skip)]
    _marker: core::marker::PhantomData<T>,
}

impl<T> Id<T> {
    /// Creates a new [`Id`] from human-readable string identifier.
    #[must_use]
    pub const fn from_name(name: &str) -> Self {
        Self {
            hash: fnv1a_hash_str_64(name),
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates a new [`Id`] from a raw value.
    #[must_use]
    pub const fn from_raw(value: u64) -> Self {
        Self {
            hash: value,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns the raw value of the [`Id`].
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.hash
    }
}

impl<T> core::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Id").field("value", &self.hash).finish()
    }
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T> Eq for Id<T> {}

impl<T> core::hash::Hash for Id<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor<T> {
            _marker: core::marker::PhantomData<T>,
        }

        impl<T> serde::de::Visitor<'_> for IdVisitor<T> {
            type Value = Id<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string or a u64")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::from_name(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::from_name(&v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Id::from_raw(v))
            }
        }

        deserializer.deserialize_any(IdVisitor {
            _marker: core::marker::PhantomData::<T>,
        })
    }
}

impl<T> From<NamedId<T>> for Id<T> {
    fn from(value: NamedId<T>) -> Self {
        value.id()
    }
}

impl<T> From<String> for Id<T> {
    fn from(value: String) -> Self {
        Self::from_name(&value)
    }
}

impl<T> From<&str> for Id<T> {
    fn from(value: &str) -> Self {
        Self::from_name(value)
    }
}

/// The unique identifier of type `T`.
///
/// Unlike [`Id<T>`](Id) it preserves the original [`String`] used to construct the [`Id`]'s hash.
#[derive(Debug, Component, Reflect)]
pub struct NamedId<T> {
    name: String,
    id: Id<T>,
}

impl<T> NamedId<T> {
    /// Creates a new [`NamedId`] from human-readable string identifier.
    pub fn from_name(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            id: Id::from_name(name),
        }
    }

    /// Returns the human-readable string identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the [`Id`] of the [`NamedId`].
    pub fn id(&self) -> Id<T> {
        self.id
    }
}

impl<T> core::fmt::Display for NamedId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{:X}", self.name, self.id.raw())
    }
}

impl<T> Clone for NamedId<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: self.id,
        }
    }
}

impl<T> PartialEq for NamedId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for NamedId<T> {}

impl<T> core::hash::Hash for NamedId<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.id.hash);
    }
}

impl<T> Serialize for NamedId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

impl<'de, T> Deserialize<'de> for NamedId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|name| Self::from_name(&name))
    }
}

impl<T> From<String> for NamedId<T> {
    fn from(value: String) -> Self {
        Self::from_name(&value)
    }
}

impl<T> From<&str> for NamedId<T> {
    fn from(value: &str) -> Self {
        Self::from_name(value)
    }
}
