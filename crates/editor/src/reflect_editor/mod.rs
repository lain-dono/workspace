use bevy::ecs::change_detection::MutUntyped;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::CommandQueue;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::ecs::world::Mut;
use bevy::reflect::{
    std_traits::ReflectDefault, Reflect, ReflectMut, ReflectRef, TypeInfo, TypeRegistry,
};
use bevy::reflect::{ReflectFromPtr, TypeRegistration};
use std::any::{Any, TypeId};

mod edit_array;
mod edit_enum;
mod edit_list;
mod edit_map;
mod edit_tuple_and_struct;

pub mod errors;

mod icon_btn;
mod options;
mod vtable;

pub use self::errors::Error;
pub use self::options::{AnyOptions, Options, OptionsType, ReflectOptions};
pub use self::vtable::{Arg, EditorVTable, EditorVTableBuilder, FieldEditor, ShortCircuit};

pub(crate) fn iter_all_eq<T: Copy + PartialEq>(mut iter: impl Iterator<Item = T>) -> Option<T> {
    let first = iter.next()?;
    iter.all(|elem| elem == first).then_some(first)
}

pub struct ReflectEditor<'a, 'q, 'w> {
    pub registry: &'a TypeRegistry,
    pub world: UnsafeWorldCell<'w>,
    pub queue: &'q mut CommandQueue,
    vtable: ShortCircuit,
}

impl<'a, 'q, 'w> ReflectEditor<'a, 'q, 'w> {
    pub fn new(
        registry: &'a TypeRegistry,
        world: UnsafeWorldCell<'w>,
        queue: &'q mut CommandQueue,
    ) -> Self {
        Self {
            registry,
            world,
            queue,
            vtable: ShortCircuit::default(),
        }
    }

    pub fn with_short_circuit(self, vtable: ShortCircuit) -> Self {
        Self { vtable, ..self }
    }

    /// [`InspectorUi`] with short circuiting methods able to display `bevy_asset` [`Handle`](bevy_asset::Handle)s
    pub fn for_bevy(
        registry: &'a TypeRegistry,
        world: UnsafeWorldCell<'w>,
        queue: &'q mut CommandQueue,
    ) -> Self {
        Self {
            registry,
            world,
            queue,
            vtable: ShortCircuit::for_bevy_assets(),
        }
    }

    pub(crate) fn reborrow(&mut self) -> ReflectEditor<'_, '_, 'w> {
        ReflectEditor {
            registry: self.registry,
            world: self.world,
            queue: self.queue,
            vtable: self.vtable,
        }
    }

    /// Draws the inspector UI for the given value with some options in a read-only way.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn reflect_ref(&mut self, Arg { ui, id, mut opt }: Arg, value: &dyn Reflect) {
        let type_id = Any::type_id(value);

        if opt.is_empty() {
            if let Some(data) = self.registry.get_type_data::<ReflectOptions>(type_id) {
                opt.0 = &data.0;
            }
        }

        let args = Arg { ui, id, opt };
        if let Some(s) = self.registry.get_type_data::<EditorVTable>(type_id) {
            return s.ui_ref(self.reborrow(), args, value);
        }
        if (self.vtable.fn_ref)(self.reborrow(), args, value).is_some() {
            return;
        }

        let args = Arg { ui, id, opt };
        match value.reflect_ref() {
            ReflectRef::Struct(value) => self.struct_ref(args, value),
            ReflectRef::TupleStruct(value) => self.tuple_struct_ref(args, value),
            ReflectRef::Tuple(value) => self.tuple_ref(args, value),
            ReflectRef::List(value) => self.list_ref(args, value),
            ReflectRef::Array(value) => self.array_ref(args, value),
            ReflectRef::Map(value) => self.map_ref(args, value),
            ReflectRef::Enum(value) => self.enum_ref(args, value),
            ReflectRef::Value(value) => {
                errors::reflect_value_no_impl(args.ui, value.reflect_short_type_path())
            }
        }
    }

    /// Draws the inspector UI for the given value with some options.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn reflect_mut(&mut self, Arg { ui, id, mut opt }: Arg, value: &mut dyn Reflect) -> bool {
        let type_id = Any::type_id(value);

        if opt.is_empty() {
            if let Some(data) = self.registry.get_type_data::<ReflectOptions>(type_id) {
                opt.0 = &data.0;
            }
        }

        let args = Arg { ui, id, opt };
        if let Some(s) = self.registry.get_type_data::<EditorVTable>(type_id) {
            return s.ui_mut(self.reborrow(), args, value);
        }
        if let Some(changed) = (self.vtable.fn_mut)(self.reborrow(), args, value) {
            return changed;
        }

        let args = Arg { ui, id, opt };
        match value.reflect_mut() {
            ReflectMut::Struct(value) => self.struct_mut(args, value),
            ReflectMut::TupleStruct(value) => self.tuple_struct_mut(args, value),
            ReflectMut::Tuple(value) => self.tuple_mut(args, value),
            ReflectMut::List(value) => self.list_mut(args, value),
            ReflectMut::Array(value) => self.array_mut(args, value),
            ReflectMut::Map(value) => self.map_mut(args, value),
            ReflectMut::Enum(value) => self.enum_mut(args, value),
            ReflectMut::Value(value) => {
                errors::reflect_value_no_impl(args.ui, value.reflect_short_type_path());
                false
            }
        }
    }

    pub fn reflect_many(
        &mut self,
        type_id: TypeId,
        name: &str,
        Arg { ui, id, opt }: Arg,
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let Some(info) = self.registry.get(type_id).map(TypeRegistration::type_info) else {
            errors::Error::NoTypeRegistration(type_id).show(ui, name);
            return false;
        };

        let mut opt = opt;
        if opt.is_empty() {
            if let Some(data) = self.registry.get_type_data::<ReflectOptions>(type_id) {
                opt.0 = &data.0;
            }
        }

        let args = Arg { ui, id, opt };
        if let Some(s) = self.registry.get_type_data::<EditorVTable>(type_id) {
            return s.ui_many(self.reborrow(), args, values, projector);
        }
        if let Some(changed) =
            (self.vtable.fn_many)(self.reborrow(), args, type_id, name, values, projector)
        {
            return changed;
        }

        let args = Arg { ui, id, opt };
        match info {
            TypeInfo::Struct(info) => self.struct_many(args, info, values, projector),
            TypeInfo::TupleStruct(info) => self.tuple_struct_many(args, info, values, projector),
            TypeInfo::Tuple(info) => self.tuple_many(args, info, values, projector),
            TypeInfo::List(info) => self.list_many(args, info, values, projector),
            TypeInfo::Array(info) => self.array_many(args, info, values, projector),
            TypeInfo::Map(info) => self.map_many(args, info, values, projector),
            TypeInfo::Enum(info) => self.enum_many(args, info, values, projector),
            TypeInfo::Value(info) => {
                errors::reflect_value_no_impl(args.ui, info.type_path());
                false
            }
        }
    }
}

pub trait DefaultFor {
    fn default_for(&self, type_id: TypeId) -> Option<Box<dyn Reflect>>;
}

impl DefaultFor for TypeRegistry {
    fn default_for(&self, type_id: TypeId) -> Option<Box<dyn Reflect>> {
        self.get_type_data::<ReflectDefault>(type_id)
            .map(ReflectDefault::default)
    }
}

use bevy::ecs::system::Resource;

impl ReflectEditor<'_, '_, '_> {
    /// # Safety
    pub unsafe fn get_resource_mut<R: Resource>(&self) -> Result<Mut<'_, R>, Error> {
        self.world.get_resource_mut::<R>().ok_or_else(|| {
            let type_id = std::any::TypeId::of::<R>();
            Error::ResourceDoesNotExist(type_id)
        })
    }
}

/// Gets a mutable reference in form of a [`&mut dyn Reflect`](bevy::reflect::Reflect) to a component at an entity.
///
/// Returns an error if the type does not register [`Reflect`].
///
/// # Safety
///
/// - this only accesses the component ID and doesn't keep any references
/// - we have access to (entity, component)
pub unsafe fn get_component_reflect<'w>(
    registry: &TypeRegistry,
    world: UnsafeWorldCell<'w>,
    entity: Entity,
    type_id: TypeId,
) -> Result<Mut<'w, dyn Reflect>, Error> {
    let err = Error::NoComponentId(type_id);
    let component_id = world.components().get_id(type_id).ok_or(err)?;
    let err = Error::ComponentDoesNotExist(entity, type_id);
    let cell = world.get_entity(entity).ok_or(err)?;
    let value = cell.get_mut_by_id(component_id).ok_or(err)?;
    reflect_from_mut_untyped(registry, value, type_id)
}

/// Gets a mutable reference in form of a [`Mut<dyn Reflect>`](bevy::reflect::Reflect) to the resource given by `type_id`.
///
/// Returns an error if the type does not register [`Reflect`].
///
/// # Safety
/// - we have access to `type_id`
/// - value is of type type_id
pub unsafe fn get_resource_reflect<'w>(
    registry: &TypeRegistry,
    world: UnsafeWorldCell<'w>,
    type_id: TypeId,
) -> Result<Mut<'w, dyn Reflect>, Error> {
    let err = Error::ResourceDoesNotExist(type_id);
    let component_id = world.components().get_resource_id(type_id).ok_or(err)?;
    let err = Error::ResourceDoesNotExist(type_id);
    let value = world.get_resource_mut_by_id(component_id).ok_or(err)?;
    reflect_from_mut_untyped(registry, value, type_id)
}

// SAFETY: MutUntyped is of type with `type_id`
unsafe fn reflect_from_mut_untyped<'a>(
    registry: &TypeRegistry,
    value: MutUntyped<'a>,
    type_id: TypeId,
) -> Result<Mut<'a, dyn Reflect>, Error> {
    let err = Error::NoTypeRegistration(type_id);
    let registration = registry.get(type_id).ok_or(err)?;
    let err = Error::NoTypeData(type_id, "ReflectFromPtr");
    let from_ptr = registration.data::<ReflectFromPtr>().ok_or(err)?;
    Ok(value.map_unchanged(|val| from_ptr.as_reflect_mut(val)))
}
