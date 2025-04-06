use bevy::{
    asset::{Asset, AssetLoader, AssetServer},
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::identifier::Id;

/// A [`Prototype`] is a piece of data that can be stored inside a [`PrototypeRegistry`](crate::registry::PrototypeRegistry).
pub trait Prototype: core::fmt::Debug + Clone + Sized + Send + Sync + 'static {
    #[cfg(not(feature = "schemars"))]
    type Raw: Clone + for<'de> serde::Deserialize<'de>;
    #[cfg(feature = "schemars")]
    type Raw: Clone + for<'de> serde::Deserialize<'de> + schemars::JsonSchema;

    /// Returns the [`Id`] of this [`Prototype`].
    fn id(&self) -> Id<Self>;

    /// Creates a new instance of this [`Prototype`] from [`Prototype::Raw`].
    #[must_use]
    fn from_raw(raw: Self::Raw, asset_server: &AssetServer) -> Self;

    /// Returns the discriminant of this [`Prototype`].
    ///
    /// This is used to determine which [`Prototype`] to create when loading from a proto asset file.
    fn discriminant() -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawPrototype {
    #[serde(rename = "type")]
    pub discriminant: String,
    #[serde(flatten)]
    pub data: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath, Asset)]
#[serde(untagged)]
pub(crate) enum RawPrototypeAsset {
    List(Vec<RawPrototype>),
    Unit(RawPrototype),
}

impl RawPrototypeAsset {
    pub fn drain(&mut self) -> impl Iterator<Item = RawPrototype> + '_ {
        RawPrototypeDrainIterator::new(self)
    }
}

pub(crate) struct RawPrototypeAssetLoader;

impl AssetLoader for RawPrototypeAssetLoader {
    type Asset = RawPrototypeAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let raw_prototype = serde_json::from_slice(&bytes)?;
        Ok(raw_prototype)
    }

    fn extensions(&self) -> &[&str] {
        &[
            "proto.json",
            "protos.json",
            "prototype.json",
            "prototypes.json",
        ]
    }
}

pub(crate) struct RawPrototypeDrainIterator<'a> {
    asset: &'a mut RawPrototypeAsset,
    is_exhausted: bool,
}

impl<'a> RawPrototypeDrainIterator<'a> {
    pub fn new(asset: &'a mut RawPrototypeAsset) -> Self {
        Self {
            asset,
            is_exhausted: false,
        }
    }
}

impl Iterator for RawPrototypeDrainIterator<'_> {
    type Item = RawPrototype;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_exhausted {
            return None;
        }

        match self.asset {
            RawPrototypeAsset::Unit(unit) => {
                self.is_exhausted = true;
                Some(unit.clone())
            }
            RawPrototypeAsset::List(list) => list.pop(),
        }
    }
}
