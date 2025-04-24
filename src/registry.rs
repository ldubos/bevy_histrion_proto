use core::any::TypeId;

use bevy::prelude::*;
use bevy::{ecs::system::SystemParam, platform::collections::HashMap};

use crate::{ErasedPrototypeId, Prototype, PrototypeData, PrototypeId};

#[derive(Default, Debug, Resource)]
pub(crate) struct PrototypeRegistries {
    registries: HashMap<TypeId, HashMap<ErasedPrototypeId, Box<dyn Reflect>>>,
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync + 'static>() {}
    assert_send_sync::<PrototypeRegistries>();
    assert_send_sync::<Reg<'_, ()>>();
    assert_send_sync::<&str>();
};

impl PrototypeData for () {
    fn prototype_name() -> &'static str {
        "()"
    }
}

impl PrototypeRegistries {
    pub fn new_registry<P: PrototypeData>(&mut self) {
        self.registries.insert(TypeId::of::<P>(), HashMap::new());
    }

    pub fn insert<P: PrototypeData>(&mut self, proto: Prototype<P>) {
        let Some(registry) = self.registries.get_mut(&TypeId::of::<P>()) else {
            error!(
                "Attempted to insert prototype into unregistered registry {}",
                P::prototype_name()
            );
            return;
        };

        registry.insert(ErasedPrototypeId::from(*proto.id()), Box::new(proto));
    }

    pub fn insert_dyn(&mut self, type_id: &TypeId, id: ErasedPrototypeId, proto: Box<dyn Reflect>) {
        let Some(registry) = self.registries.get_mut(type_id) else {
            error!("Attempted to insert prototype into unregistered registry");
            return;
        };

        registry.insert(id, proto);
    }

    pub fn get<P: PrototypeData>(&self, id: &PrototypeId<P>) -> Option<&Prototype<P>> {
        self.registries
            .get(&TypeId::of::<P>())
            .and_then(|registry| registry.get(&(ErasedPrototypeId::from(*id))))
            .and_then(|proto| proto.downcast_ref::<Prototype<P>>())
    }
}

#[derive(SystemParam)]
pub struct Reg<'w, P: PrototypeData> {
    registries: Res<'w, PrototypeRegistries>,
    _marker: core::marker::PhantomData<P>,
}

impl<P: PrototypeData> Reg<'_, P> {
    /// Get a prototype instance with it's [`PrototypeId`]
    pub fn get(&self, id: impl Into<PrototypeId<P>>) -> Option<&Prototype<P>> {
        self.registries.get(&id.into())
    }
}

impl<P: PrototypeData> core::fmt::Debug for Reg<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Reg").finish()
    }
}

#[derive(SystemParam)]
pub struct RegMut<'w, P: PrototypeData> {
    registries: ResMut<'w, PrototypeRegistries>,
    _marker: core::marker::PhantomData<P>,
}

impl<P: PrototypeData> RegMut<'_, P> {
    /// Get a [`Prototype`] instance with it's [`PrototypeId`]
    pub fn get(&self, id: &PrototypeId<P>) -> Option<&Prototype<P>> {
        self.registries.get(id)
    }

    /// Insert a [`Prototype`] instance into the registry
    pub fn insert(&mut self, prototype: Prototype<P>) {
        self.registries.insert(prototype);
    }
}
