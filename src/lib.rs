#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use bevy::{ecs::system::SystemId, platform_support::collections::HashMap, prelude::*};

pub mod identifier;
pub mod prototype;
pub mod registry;

pub use bevy_histrion_proto_derive::*;
use prototype::*;
use registry::*;
use serde_json::Value as JsonValue;

pub mod prelude {
    pub use super::RegisterPrototype;
    pub use crate::identifier::{Id, NamedId};
    pub use crate::prototype::Prototype;
    pub use crate::registry::{Reg, RegMut, RegistryError, RegistryEvent};
    pub use bevy_histrion_proto_derive::*;
}

mod private {
    pub trait Sealed {}
}

impl private::Sealed for App {}

pub trait RegisterPrototype: private::Sealed {
    fn register_prototype<T: Prototype>(&mut self) -> &mut Self;

    #[cfg(feature = "schemars")]
    fn get_prototypes_schema(&self) -> schemars::schema::RootSchema;
}

impl RegisterPrototype for App {
    fn register_prototype<P: Prototype>(&mut self) -> &mut Self {
        let system_id = self.register_system(load_prototype::<P>);

        self.world_mut()
            .resource_mut::<PrototypesLoaders>()
            .insert(P::discriminant().to_owned(), system_id);

        self.init_resource::<PrototypeRegistry<P>>()
            .add_event::<RegistryEvent<P>>();

        #[cfg(feature = "schemars")]
        {
            let mut prototypes_schemas = self.world_mut().resource_mut::<PrototypesSchemas>();
            let mut generator = schemars::r#gen::SchemaGenerator::default();

            prototypes_schemas.prototypes.insert(
                <P::Raw as schemars::JsonSchema>::schema_name(),
                <P::Raw as schemars::JsonSchema>::json_schema(&mut generator),
            );
            prototypes_schemas.discriminants.insert(
                <P::Raw as schemars::JsonSchema>::schema_name(),
                P::discriminant().to_owned(),
            );
            prototypes_schemas
                .schemas
                .extend(generator.definitions().clone());
        }

        self
    }

    #[cfg(feature = "schemars")]
    fn get_prototypes_schema(&self) -> schemars::schema::RootSchema {
        let prototypes_schemas = self.world().resource::<PrototypesSchemas>();

        let mut definitions = prototypes_schemas.schemas.clone();
        definitions.extend(prototypes_schemas.schemas.clone());

        let any_prototype_schema =
            schemars::schema::Schema::Object(schemars::schema::SchemaObject {
                metadata: Some(Box::new(schemars::schema::Metadata {
                    title: Some("PrototypeAny".to_string()),
                    ..Default::default()
                })),
                subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                    any_of: Some(
                        prototypes_schemas
                            .discriminants
                            .iter()
                            .map(|(key, discriminant)| {
                                schemars::_private::new_internally_tagged_enum(
                                    "type",
                                    discriminant.as_str(),
                                    false,
                                )
                                .flatten(prototypes_schemas.prototypes.get(key).unwrap().clone())
                            })
                            .collect::<Vec<_>>(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            });
        definitions.insert("PrototypeAny".to_owned(), any_prototype_schema);

        let draft_settings = schemars::r#gen::SchemaSettings::draft07();

        schemars::schema::RootSchema {
            meta_schema: draft_settings.meta_schema,
            schema: schemars::schema::SchemaObject {
                metadata: Some(Box::new(schemars::schema::Metadata {
                    title: Some("Prototype".to_owned()),
                    ..Default::default()
                })),
                subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                    any_of: Some(vec![
                        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
                            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                                schemars::schema::InstanceType::Array,
                            ))),
                            array: Some(Box::new(schemars::schema::ArrayValidation {
                                items: Some(schemars::schema::SingleOrVec::Single(Box::new(
                                    schemars::schema::Schema::new_ref(
                                        "#/definitions/PrototypeAny".to_owned(),
                                    ),
                                ))),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                        schemars::schema::Schema::new_ref("#/definitions/PrototypeAny".to_owned()),
                    ]),
                    ..Default::default()
                })),
                ..Default::default()
            },
            definitions,
        }
    }
}

fn load_prototype<P: Prototype>(
    In(raw): In<JsonValue>,
    mut registry: RegMut<P>,
    asset_server: Res<AssetServer>,
) {
    let raw = match serde_json::from_value::<P::Raw>(raw) {
        Ok(raw) => raw,
        Err(err) => {
            error!("Failed to load prototype: {}", err);
            return;
        }
    };

    let prototype = P::from_raw(raw, &asset_server);

    if let Err(err) = registry.insert(prototype) {
        error!("Failed to load prototype: {}", err);
    }
}

#[derive(Debug, Default, Clone, Resource, Deref, DerefMut)]
struct PrototypesLoaders(HashMap<String, SystemId<In<JsonValue>>>);

#[cfg(feature = "schemars")]
#[derive(Default, Resource)]
struct PrototypesSchemas {
    discriminants: schemars::Map<String, String>,
    prototypes: schemars::Map<String, schemars::schema::Schema>,
    schemas: schemars::Map<String, schemars::schema::Schema>,
}

pub struct HistrionProtoPlugin;

impl Plugin for HistrionProtoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrototypesLoaders>()
            .init_asset::<prototype::RawPrototypeAsset>()
            .register_asset_loader(prototype::RawPrototypeAssetLoader)
            .add_systems(
                Update,
                on_raw_asset_loaded.run_if(on_event::<AssetEvent<prototype::RawPrototypeAsset>>),
            );

        #[cfg(feature = "schemars")]
        app.init_resource::<PrototypesSchemas>();
    }
}

fn on_raw_asset_loaded(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<prototype::RawPrototypeAsset>>,
    mut assets: ResMut<Assets<prototype::RawPrototypeAsset>>,
    loaders: Res<PrototypesLoaders>,
) {
    for mut asset in events.read().filter_map(|event| match event {
        AssetEvent::LoadedWithDependencies { id } => assets.remove(*id),
        _ => None,
    }) {
        for raw in asset.drain() {
            let Some(loader) = loaders.get(&raw.discriminant) else {
                warn!("No loader found for prototype type: {}", raw.discriminant);
                continue;
            };
            commands.run_system_with(*loader, raw.data);
        }
    }
}
