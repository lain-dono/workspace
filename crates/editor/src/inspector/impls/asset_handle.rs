use crate::reflect_editor::{errors, Arg, Error, ReflectEditor, ShortCircuit};
use bevy::asset::{ReflectAsset, ReflectHandle};
use bevy::reflect::Reflect;
use std::any::{Any, TypeId};

impl ShortCircuit {
    pub fn for_bevy_assets() -> Self {
        Self {
            fn_ref: asset_ref,
            fn_mut: asset_mut,
            fn_many: asset_many,
        }
    }
}

pub fn asset_ref(mut editor: ReflectEditor, arg: Arg, value: &dyn Reflect) -> Option<()> {
    let registry = editor.registry;

    let reflect_handle = registry.get_type_data::<ReflectHandle>(Any::type_id(value))?;
    let asset_type_id = reflect_handle.asset_type_id();

    let Some(reflect_asset) = registry.get_type_data::<ReflectAsset>(asset_type_id) else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        Error::NoTypeData(asset_type_id, "ReflectAsset").show(arg.ui, &type_name);
        return Some(());
    };

    let handle = value.as_any();
    let handle = reflect_handle.downcast_handle_untyped(handle).unwrap();
    let handle_id = handle.id();

    // SAFETY: the following code only accesses a resources it has access to, `Assets<T>`
    let Some(asset_value) = reflect_asset.get(unsafe { editor.world.world() }, handle) else {
        errors::dead_asset_handle(arg.ui, handle_id);
        return Some(());
    };

    editor.reflect_ref(arg.with("asset"), asset_value);
    Some(())
}

pub fn asset_mut(mut editor: ReflectEditor, arg: Arg, value: &mut dyn Reflect) -> Option<bool> {
    let registry = editor.registry;

    let reflect_handle = registry.get_type_data::<ReflectHandle>(Any::type_id(value))?;
    let asset_type_id = reflect_handle.asset_type_id();

    let Some(reflect_asset) = registry.get_type_data::<ReflectAsset>(asset_type_id) else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        Error::NoTypeData(asset_type_id, "ReflectAsset").show(arg.ui, &type_name);
        return Some(false);
    };

    let handle = value.as_any();
    let handle = reflect_handle.downcast_handle_untyped(handle).unwrap();
    let handle_id = handle.id();

    // SAFETY: the world allows mutable access to `Assets<T>`
    let Some(asset_value) = (unsafe { reflect_asset.get_unchecked_mut(editor.world, handle) })
    else {
        errors::dead_asset_handle(arg.ui, handle_id);
        return Some(false);
    };

    Some(editor.reflect_mut(arg.with("asset"), asset_value))
}

pub fn asset_many(
    mut editor: ReflectEditor,
    arg: Arg,
    type_id: TypeId,
    _type_name: &str,
    values: &mut [&mut dyn Reflect],
    projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> Option<bool> {
    let registry = editor.registry;

    let reflect_handle = registry.get_type_data::<ReflectHandle>(type_id)?;
    let asset_type_id = reflect_handle.asset_type_id();

    let Some(reflect_asset) = registry.get_type_data::<ReflectAsset>(asset_type_id) else {
        let type_name = errors::name_of_type(asset_type_id, editor.registry);
        Error::NoTypeData(asset_type_id, "ReflectAsset").show(arg.ui, &type_name);
        return Some(false);
    };

    let mut new_values = Vec::with_capacity(values.len());
    let mut used_handles = Vec::with_capacity(values.len());

    for value in values {
        let handle = projector(*value).as_any();
        let handle = reflect_handle.downcast_handle_untyped(handle).unwrap();
        let handle_id = handle.id();

        if used_handles.contains(&handle_id) {
            continue;
        }
        used_handles.push(handle_id);

        // SAFETY: the world allows mutable access to `Assets<T>`
        let Some(asset_value) = (unsafe { reflect_asset.get_unchecked_mut(editor.world, handle) })
        else {
            errors::dead_asset_handle(arg.ui, handle_id);
            return Some(false);
        };

        new_values.push(asset_value);
    }

    let values = new_values.as_mut_slice();
    Some(editor.reflect_many(asset_type_id, "", arg.with("asset"), values, &|a| a))
}
