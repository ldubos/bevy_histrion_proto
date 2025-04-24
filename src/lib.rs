use bevy::{
    asset::AssetPath, ecs::system::SystemParam, platform::collections::HashMap, prelude::*,
};
use serde_json::{Map as JsonMap, Value as JsonValue, json};

mod identifier;
mod prototype;
mod registry;
mod schema;

pub use bevy_histrion_proto_derive::*;
pub use identifier::*;
pub use prototype::*;
pub use registry::*;
pub use schema::*;

pub mod prelude {
    pub use crate::{
        JsonSchema, PrototypeAppExt, PrototypeServer, identifier::*, prototype::*, registry::*,
    };
    pub use bevy_histrion_proto_derive::*;
}

pub struct PrototypesPlugin;

impl Plugin for PrototypesPlugin {
    fn build(&self, app: &mut App) {
        let app_prototype_type_registry = AppPrototypeTypeRegistry::default();

        app.register_type::<ErasedPrototypeId>()
            .register_type::<ErasedPrototypeName>()
            .init_resource::<PrototypeRegistries>()
            .init_resource::<LoadingPrototypesHandles>()
            .init_resource::<PrototypesSchemas>()
            .insert_resource(app_prototype_type_registry.clone());

        let type_registry = app.world().resource::<AppTypeRegistry>().0.clone();

        let prototypes_asset_loader = PrototypesAssetLoader {
            prototype_type_registry: app_prototype_type_registry.0.clone(),
            type_registry: type_registry.clone(),
        };

        app.init_asset::<PrototypesAsset>()
            .register_asset_loader(prototypes_asset_loader)
            .add_systems(Update, on_prototypes_asset_loaded);
    }
}

fn on_prototypes_asset_loaded(
    mut events_rx: EventReader<AssetEvent<PrototypesAsset>>,
    mut assets: ResMut<Assets<PrototypesAsset>>,
    mut registries: ResMut<PrototypeRegistries>,
    mut loading_prototypes_handles: ResMut<LoadingPrototypesHandles>,
    type_registry: Res<AppTypeRegistry>,
) {
    use bevy::reflect::DynamicStruct;

    let type_registry = type_registry.read();

    for event in events_rx.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };

        let Some(prototypes) = assets.remove(*id) else {
            warn!("Asset {id} not found");
            continue;
        };

        loading_prototypes_handles.remove(id);

        for (ty, DynamicPrototype { name, tags, proto }) in &*prototypes {
            let Some(proto_ty) = type_registry.get(*ty) else {
                error!("Type {:?} not found in registry", ty);
                continue;
            };

            let proto_data_short_path = proto_ty.type_info().type_path_table().short_path();
            let proto_short_path = format!("Prototype<{proto_data_short_path}>");

            // Get prototype type and check for errors
            let Some(proto_ty) = type_registry.get_with_short_type_path(&proto_short_path) else {
                error!("Failed to find prototype type {proto_short_path}");
                continue;
            };

            let Some(dyn_proto) = proto_ty.data::<ReflectDefault>() else {
                error!("Failed to find default for prototype type {proto_short_path}");
                continue;
            };

            let mut dyn_proto = dyn_proto.default();

            // Create dynamic structure for the prototype
            let mut dyn_struct = DynamicStruct::default();
            dyn_struct.insert("name", name.clone());
            dyn_struct.insert("tags", tags.clone());
            dyn_struct.insert_boxed("data", proto.to_dynamic());

            if let Err(err) = dyn_proto.try_apply(dyn_struct.as_partial_reflect()) {
                error!("Error applying dynamic prototype: {err}");
                continue;
            }

            registries.insert_dyn(ty, name.id(), dyn_proto);
        }
    }
}

mod private {
    pub trait Sealed {}
}

impl private::Sealed for App {}

pub trait PrototypeAppExt: private::Sealed {
    fn register_prototype<D: PrototypeData>(&mut self) -> &mut Self;
    fn get_prototypes_schemas(&self) -> String;
}

#[derive(Default, Resource)]
pub(crate) struct PrototypesSchemas {
    prototypes: HashMap<String, String>,
    refs: JsonMap<String, JsonValue>,
}

impl PrototypeAppExt for App {
    fn register_prototype<D: PrototypeData>(&mut self) -> &mut Self {
        self.register_type::<Prototype<D>>();

        if let Some(mut registries) = self.world_mut().get_resource_mut::<PrototypeRegistries>() {
            registries.new_registry::<D>();
        } else {
            error!("PrototypeRegistries resource not found");
            return self;
        }

        if let Some(mut schemas) = self.world_mut().get_resource_mut::<PrototypesSchemas>() {
            schemas.prototypes.insert(
                D::prototype_name().into(),
                <Prototype<D> as JsonSchema>::schema_ref(),
            );

            let schema = <Prototype<D> as JsonSchema>::json_schema(&mut schemas.refs);
            schemas
                .refs
                .insert(<Prototype<D> as JsonSchema>::schema_title(), schema);
        } else {
            error!("PrototypesSchemas resource not found");
            return self;
        }

        if let Some(prototypes) = self.world().get_resource::<AppPrototypeTypeRegistry>() {
            prototypes
                .0
                .write()
                .insert(D::prototype_name().into(), core::any::TypeId::of::<D>());
        } else {
            error!("AppPrototypeTypeRegistry resource not found");
            return self;
        }

        self
    }

    fn get_prototypes_schemas(&self) -> String {
        let PrototypesSchemas { prototypes, refs } = self.world().resource::<PrototypesSchemas>();
        let mut refs = refs.clone();

        refs.insert(
            "PrototypeAny".to_string(),
            json!({
                "type": "object",
                "required": ["type", "name"],
                "oneOf": prototypes.keys().map(|key| json!({
                    "type": "object",
                    "allOf": [
                        {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "enum": [key],
                                },
                            },
                        },
                        {
                            "$ref": prototypes.get(key).unwrap()
                        }
                    ]
                })).collect::<Vec<_>>(),
            }),
        );

        serde_json::to_string_pretty(&json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Prototype",
            "type": ["object", "array"],
            "oneOf": [
                {
                    "$ref": "#/definitions/PrototypeAny"
                },
                {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/PrototypeAny"
                    },
                }
            ],
            "definitions": refs,
        }))
        .unwrap()
    }
}

#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct LoadingPrototypesHandles(
    HashMap<AssetId<PrototypesAsset>, Handle<PrototypesAsset>>,
);

#[derive(SystemParam)]
pub struct PrototypeServer<'w> {
    asset_server: Res<'w, AssetServer>,
    loading_prototypes_handles: ResMut<'w, LoadingPrototypesHandles>,
}

impl PrototypeServer<'_> {
    /// Loads a prototypes file from the given path.
    pub fn load_prototypes(&mut self, path: &str) {
        let handle: Handle<PrototypesAsset> = self.asset_server.load(path);
        self.loading_prototypes_handles.insert(handle.id(), handle);
    }

    /// Loads all prototypes files from the given folder.
    pub fn load_prototypes_folder(&mut self, path: &str) {
        let files = {
            let path: AssetPath<'_> = path.into();
            let source = self.asset_server.get_source(path.source()).unwrap();
            let source = source.reader();

            bevy::tasks::block_on(async move {
                use bevy::tasks::futures_lite::StreamExt;

                let mut folder = source.read_directory(path.path()).await.unwrap();
                let mut files = Vec::new();

                while let Some(file) = folder.next().await {
                    if !source.is_directory(&file).await.unwrap() {
                        let file = file.to_string_lossy().to_string();
                        let asset_path: AssetPath<'_> = (&file).into();

                        let is_prototype_file = {
                            let Some(full_extension) = asset_path.get_full_extension() else {
                                continue;
                            };

                            PROTOTYPE_ASSET_EXTENSIONS.contains(&full_extension.as_str())
                        };

                        if is_prototype_file {
                            files.push(file);
                        }
                    }
                }

                files
            })
        };

        for file in files {
            self.load_prototypes(&file);
        }
    }
}

#[doc(hidden)]
pub mod _private {
    pub use serde_json;
}
