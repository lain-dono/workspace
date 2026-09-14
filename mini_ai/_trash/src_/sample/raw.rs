
use bevy::{
    ecs::component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
    prelude::*,
};
use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

unsafe fn create_array_component<T: Send + Sync>(
    world: &mut World,
    name: impl Into<std::borrow::Cow<'static, str>>,
    n: usize,
) -> ComponentId {
    let layout = Layout::array::<T>(n).unwrap();
    let clone = ComponentCloneBehavior::Default;
    world.register_component_with_descriptor(unsafe {
        ComponentDescriptor::new_with_layout(name, StorageType::Table, layout, None, true, clone)
    })
}

unsafe fn insert_array_component<T: Send + Sync>(
    mut entity: EntityWorldMut,
    id: ComponentId,
    mut component: Vec<T>,
) {
    if let Some(info) = entity.world().components().get_info(id) {
        unsafe {
            entity.insert_by_id(id, to_owning_ptr(&mut component));
        }
    }
}

pub fn sample(world: &mut World) {
    let id = unsafe { create_array_component::<u64>(world, "sample", 5) };

    let Some(info) = world.components().get_info(id) else {
        return;
    };

    let len = info.layout().size() / size_of::<u64>();

    let entity = world.spawn_empty();
    unsafe { insert_array_component(entity, id, vec![7u64; len]) }

    /*
    let to_insert_ids = vec![id];
    let mut to_insert_data = vec![values];
    let iter_components = to_owning_ptrs(&mut to_insert_data).into_iter();

    let mut entity = world.spawn_empty();

    // SAFETY:
    // - Component ids have been taken from the same world
    // - Each array is created to the layout specified in the world
    unsafe {
        entity.insert_by_ids(&to_insert_ids, iter_components);
    }
    */
}

// Constructs `OwningPtr` for each item in `components`
// By sharing the lifetime of `components` with the resulting ptrs we ensure we don't drop the data before use
fn to_owning_ptrs(components: &mut [Vec<u64>]) -> Vec<bevy::ptr::OwningPtr<bevy::ptr::Aligned>> {
    components
        .iter_mut()
        .map(|data| to_owning_ptr(data))
        .collect()
}

fn to_owning_ptr<T>(data: &mut Vec<T>) -> bevy::ptr::OwningPtr<bevy::ptr::Aligned> {
    let ptr = data.as_mut_ptr();
    // SAFETY:
    // - Pointers are guaranteed to be non-null
    // - Memory pointed to won't be dropped until `components` is dropped
    unsafe { bevy::ptr::OwningPtr::new(NonNull::new_unchecked(ptr.cast())) }
}
