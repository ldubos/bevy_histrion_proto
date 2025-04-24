use core::any::{Any, TypeId};
use std::sync::{Arc, RwLock};

use bevy::platform::collections::HashMap;
use bevy::reflect::{DynamicEnum, DynamicStruct, DynamicTuple, GenericInfo, Reflectable};
use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader as AssetReader},
    prelude::*,
    reflect::{
        TypeRegistration, TypeRegistry, TypeRegistryArc,
        serde::{ReflectDeserializerProcessor, TypedReflectDeserializer},
    },
};
use serde::{Deserialize, de::DeserializeSeed};

use crate::{ErasedPrototypeName, JsonSchema, PrototypeId, PrototypeName};

#[derive(Default, Clone)]
pub(crate) struct PrototypeTypeRegistry {
    internal: Arc<RwLock<HashMap<Box<str>, TypeId>>>,
}

impl PrototypeTypeRegistry {
    /// Takes a read lock on the underlying [`PrototypeTypeRegistry`].
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<Box<str>, TypeId>> {
        self.internal
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Takes a write lock on the underlying [`PrototypeTypeRegistry`].
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<Box<str>, TypeId>> {
        self.internal
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[derive(Default, Resource, Clone)]
pub(crate) struct AppPrototypeTypeRegistry(pub PrototypeTypeRegistry);

#[derive(Clone, Deserialize)]
pub(crate) struct OnDiskPrototype {
    #[serde(rename = "type")]
    pub ty: Box<str>,
    pub name: ErasedPrototypeName,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub proto: serde_json::Value,
}

#[derive(Deref)]
pub(crate) struct OnDiskPrototypes(Box<[OnDiskPrototype]>);

impl<'de> Deserialize<'de> for OnDiskPrototypes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = <serde::__private::de::Content as Deserialize>::deserialize(deserializer)?;
        let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);

        if let Ok(prototypes) = <Box<[OnDiskPrototype]> as Deserialize>::deserialize(deserializer) {
            return Ok(OnDiskPrototypes(prototypes));
        }

        if let Ok(prototype) = <OnDiskPrototype as Deserialize>::deserialize(deserializer) {
            return Ok(OnDiskPrototypes(Box::new([prototype])));
        }

        Err(serde::de::Error::custom(
            "on disk prototypes must be a list or a single prototype",
        ))
    }
}

pub(crate) struct DynamicPrototype {
    pub name: ErasedPrototypeName,
    pub tags: Vec<String>,
    pub proto: Box<dyn PartialReflect>,
}

#[derive(Asset, TypePath, Deref)]
pub(crate) struct PrototypesAsset(Box<[(TypeId, DynamicPrototype)]>);

pub(crate) struct PrototypesAssetLoader {
    pub type_registry: TypeRegistryArc,
    pub prototype_type_registry: PrototypeTypeRegistry,
}

impl AssetLoader for PrototypesAssetLoader {
    type Asset = PrototypesAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn AssetReader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let on_disk_prototypes: OnDiskPrototypes = serde_json::from_slice(&bytes)?;

        // Helper for processing asset handles during deserialization
        struct HandleProcessor<'a, 'b> {
            load_context: &'a mut LoadContext<'b>,
        }

        impl ReflectDeserializerProcessor for HandleProcessor<'_, '_> {
            fn try_deserialize<'de, D>(
                &mut self,
                registration: &TypeRegistration,
                _registry: &TypeRegistry,
                deserializer: D,
            ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct AssetPathVisitor<'a, 'b> {
                    load_context: &'a mut LoadContext<'b>,
                }

                impl serde::de::Visitor<'_> for AssetPathVisitor<'_, '_> {
                    type Value = AssetPath<'static>;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("asset path")
                    }

                    fn visit_str<E>(self, relative_path: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(self
                            .load_context
                            .asset_path()
                            .parent()
                            .unwrap()
                            .resolve(relative_path)
                            .map_err(|err| serde::de::Error::custom(err.to_string()))?
                            .into_owned())
                    }
                }

                let type_info = registration.type_info();
                let type_path = type_info.type_path_table();

                if type_path.module_path() != Some("bevy_asset::handle")
                    || type_path.ident() != Some("Handle")
                {
                    return Ok(Err(deserializer));
                }

                let Some(reflect_default) = registration.data::<ReflectDefault>() else {
                    error!("Handle didn't have a ReflectDefault");
                    return Ok(Err(deserializer));
                };

                let generics = type_info.generics();
                let GenericInfo::Type(asset_type) = &generics[0] else {
                    error!("Handle didn't have a generic type parameter, why?");
                    return Ok(Err(deserializer));
                };

                let asset_path = deserializer.deserialize_str(AssetPathVisitor {
                    load_context: self.load_context,
                })?;

                // Load the asset and return an handle to it
                let handle = self
                    .load_context
                    .loader()
                    .with_dynamic_type(asset_type.type_id())
                    .load(asset_path);

                let mut dyn_handle = DynamicEnum::default();

                match handle {
                    UntypedHandle::Strong(strong_handle) => {
                        dyn_handle.set_variant("Strong", {
                            let mut dyn_tuple = DynamicTuple::default();
                            dyn_tuple.insert_boxed(strong_handle.to_dynamic());
                            dyn_tuple
                        });
                    }
                    UntypedHandle::Weak(untyped_asset_id) => {
                        dyn_handle.set_variant("Weak", {
                            let mut dyn_tuple = DynamicTuple::default();
                            dyn_tuple.insert_boxed({
                                let mut dyn_enum = DynamicEnum::default();

                                match untyped_asset_id {
                                    bevy::asset::UntypedAssetId::Index { index, .. } => {
                                        dyn_enum.set_variant("Index", {
                                            let mut dyn_struct = DynamicStruct::default();
                                            dyn_struct.insert_boxed("index", index.to_dynamic());
                                            dyn_struct
                                        });
                                    }
                                    bevy::asset::UntypedAssetId::Uuid { uuid, .. } => {
                                        dyn_enum.set_variant("Uuid", {
                                            let mut dyn_struct = DynamicStruct::default();
                                            dyn_struct.insert_boxed("uuid", uuid.to_dynamic());
                                            dyn_struct
                                        });
                                    }
                                }

                                dyn_enum.to_dynamic()
                            });
                            dyn_tuple
                        });
                    }
                }

                let mut typed_handle = reflect_default.default();
                typed_handle.apply(&dyn_handle);

                Ok(Ok(typed_handle.into_partial_reflect()))
            }
        }

        let registry = self.type_registry.read();
        let prototype_type_registry = self.prototype_type_registry.read();

        // Convert each on-disk prototype to a dynamic prototype
        let prototypes = (*on_disk_prototypes)
            .iter()
            .filter_map(|prototype| {
                // Look up the type ID for this prototype
                let Some(type_id) = prototype_type_registry.get(&prototype.ty) else {
                    error!("Unknown prototype type {}", prototype.ty);
                    return None;
                };

                let Some(type_registration) = registry.get(*type_id) else {
                    error!("Unknown prototype type id {:?}", type_id.type_id());
                    return None;
                };

                let mut handle_processor = HandleProcessor { load_context };
                let reflect_deserializer = TypedReflectDeserializer::with_processor(
                    type_registration,
                    &registry,
                    &mut handle_processor,
                );

                let proto = match reflect_deserializer.deserialize(&prototype.proto) {
                    Ok(proto) => proto,
                    Err(err) => {
                        error!("Failed to deserialize prototype: {}", err);
                        return None;
                    }
                };

                Some((
                    *type_id,
                    DynamicPrototype {
                        name: prototype.name.clone(),
                        tags: prototype.tags.clone(),
                        proto,
                    },
                ))
            })
            .collect::<Vec<_>>();

        Ok(PrototypesAsset(prototypes.into_boxed_slice()))
    }

    fn extensions(&self) -> &[&str] {
        PROTOTYPE_ASSET_EXTENSIONS
    }
}

pub(crate) const PROTOTYPE_ASSET_EXTENSIONS: &[&str] = &["proto", "proto.json"];

pub trait PrototypeData: Default + Clone + Reflectable + FromReflect + JsonSchema {
    fn prototype_name() -> &'static str;
}

#[derive(Debug, Clone, Reflect, Deref, DerefMut)]
#[reflect(Clone, Default)]
pub struct Prototype<P: PrototypeData> {
    name: PrototypeName<P>,
    tags: Vec<String>,
    #[deref]
    data: P,
}

impl<P: PrototypeData> Prototype<P> {
    /// Returns the [`PrototypeId`] of this prototype instance.
    #[inline(always)]
    pub fn id(&self) -> &PrototypeId<P> {
        self.name.id()
    }

    /// Returns the string name of this prototype instance.
    #[inline(always)]
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the list of tags associated with this prototype instance.
    #[inline(always)]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Returns a reference tothe [`PrototypeData`] of this prototype instance.
    #[inline(always)]
    pub fn data(&self) -> &P {
        &self.data
    }

    /// Returns a mutable reference to the [`PrototypeData`] of this prototype instance.
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut P {
        &mut self.data
    }

    /// Returns the prototype name of the [`PrototypeData`] type.
    #[inline(always)]
    pub fn prototype_name() -> &'static str {
        P::prototype_name()
    }
}

impl<P: PrototypeData> Default for Prototype<P> {
    fn default() -> Self {
        Self {
            name: PrototypeName::from_name(""),
            tags: Default::default(),
            data: Default::default(),
        }
    }
}

impl<P: PrototypeData> JsonSchema for Prototype<P> {
    fn json_schema(refs: &mut serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
        let ty_title = <PrototypeName<P> as JsonSchema>::schema_title();
        if !refs.contains_key(&ty_title) {
            let ty_schema = <PrototypeName<P> as JsonSchema>::json_schema(refs);
            refs.insert(ty_title, ty_schema);
        }
        let ty_title = <Vec<String> as JsonSchema>::schema_title();
        if !refs.contains_key(&ty_title) {
            let ty_schema = <Vec<String> as JsonSchema>::json_schema(refs);
            refs.insert(ty_title, ty_schema);
        }
        let ty_title = <P as JsonSchema>::schema_title();
        if !refs.contains_key(&ty_title) {
            let ty_schema = <P as JsonSchema>::json_schema(refs);
            refs.insert(ty_title, ty_schema);
        }

        serde_json::json!({
            "type":"object",
            "required": ["name"],
            "properties":{
                "name":{
                    "$ref": <PrototypeName<P>as JsonSchema> ::schema_ref()
                },
                "tags":{
                    "$ref": <Vec<String>as JsonSchema> ::schema_ref()
                }
            },
            "allOf": [{
                "$ref": <P as JsonSchema> ::schema_ref()
            }],
        })
    }
}
