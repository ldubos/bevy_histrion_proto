#![allow(dead_code)]

use bevy::{
    asset::{Asset, AssetLoader, AsyncReadExt, Handle},
    reflect::Reflect,
};
use bevy_histrion_proto::prelude::*;

#[derive(Debug, Clone, Prototype)]
#[prototype(discriminant = "my_proto_1")]
pub struct MyPrototype1 {
    #[prototype(id)]
    pub id: NamedId<Self>,
    #[prototype(default = "String::new")]
    pub foo: String,
}

#[derive(Debug, Clone, Prototype)]
#[prototype(discriminant = "my_proto_2")]
pub struct MyPrototype2 {
    #[prototype(id)]
    pub id: Id<Self>,
    #[prototype(default(32))]
    pub bar: i32,
}

#[derive(Debug, Clone, Prototype)]
#[prototype(discriminant = "proto_with_assets")]
pub struct ProtoWithAssets {
    #[prototype(id)]
    pub id: Id<Self>,
    #[prototype(asset)]
    pub asset1: Handle<MyAsset>,
    #[prototype(asset)]
    pub asset2: Handle<MyAsset>,
}

#[derive(Debug, Clone, Reflect, Asset, serde::Deserialize)]
pub struct MyAsset(String);

pub struct MyAssetLoader;

impl AssetLoader for MyAssetLoader {
    type Asset = MyAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut text = String::new();
        reader.read_to_string(&mut text).await?;

        Ok(MyAsset(text))
    }

    fn extensions(&self) -> &[&str] {
        &["my_asset"]
    }
}
