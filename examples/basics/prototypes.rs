#![allow(dead_code)]

use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use bevy_histrion_proto::prelude::*;

#[derive(Debug, Clone, Prototype)]
#[proto(discriminant = "sword")]
pub struct Sword {
    #[proto(id)]
    pub id: NamedId<Self>,
    #[proto(default(1.0))]
    pub damage: f32,
    pub level: u32,
    pub effects: Vec<Id<Effect>>,
    #[proto(asset)]
    pub icon: Handle<Icon>,
}

#[derive(Debug, Clone, Prototype)]
#[proto(discriminant = "effect")]
pub struct Effect {
    #[proto(id)]
    pub id: NamedId<Self>,
    pub damage_multiplier: Option<f32>,
    pub slow_factor: Option<f32>,
    pub slow_duration: Option<f32>,
    #[proto(asset)]
    pub icon: Handle<Icon>,
}

#[derive(Debug, Clone, Reflect, Asset, Deref)]
pub struct Icon(char);

impl core::fmt::Display for Icon {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct IconLoader;

impl AssetLoader for IconLoader {
    type Asset = Icon;
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

        Ok(Icon(text.chars().next().unwrap_or('❓')))
    }

    fn extensions(&self) -> &[&str] {
        &["icon"]
    }
}

pub struct PrototypesPlugin;

impl Plugin for PrototypesPlugin {
    fn build(&self, app: &mut App) {
        app.register_prototype::<Sword>()
            .register_prototype::<Effect>()
            .init_asset::<Icon>()
            .register_asset_loader(IconLoader);
    }
}
