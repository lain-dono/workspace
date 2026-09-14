//! Methods for displaying `bevy` resources, assets and entities
//!
//! # Example
//!
//! ```rust
//! use bevy_inspector_egui::bevy_inspector;
//! # use bevy_ecs::prelude::*;
//! # use bevy_reflect::Reflect;
//! # use bevy_render::prelude::Msaa;
//! # use bevy_math::Vec3;
//!
//! #[derive(States, Debug, Clone, Eq, PartialEq, Hash, Reflect, Default)]
//! enum AppState { #[default] A, B, C }
//!
//! fn show_ui(world: &mut World, ui: &mut egui::Ui) {
//!     let mut any_reflect_value = Vec3::new(1.0, 2.0, 3.0);
//!     bevy_inspector::ui_for_value(&mut any_reflect_value, ui, world);
//!
//!     ui.heading("Msaa resource");
//!     bevy_inspector::ui_for_resource::<Msaa>(world, ui);
//!
//!     ui.heading("App State");
//!     bevy_inspector::ui_for_state::<AppState>(world, ui);
//!
//!     egui::CollapsingHeader::new("Entities")
//!         .default_open(true)
//!         .show(ui, |ui| {
//!             bevy_inspector::ui_for_world_entities(world, ui);
//!         });
//!     egui::CollapsingHeader::new("Resources").show(ui, |ui| {
//!         bevy_inspector::ui_for_resources(world, ui);
//!     });
//!     egui::CollapsingHeader::new("Assets").show(ui, |ui| {
//!         bevy_inspector::ui_for_all_assets(world, ui);
//!     });
//! }
//! ```

use super::guess_entity_name;
use crate::reflect_editor::{
    errors, get_component_reflect, get_resource_reflect, Arg, ReflectEditor,
};
use bevy::asset::{
    Asset, AssetServer, Assets, ReflectAsset, ReflectHandle, UntypedAssetId, UntypedHandle,
};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::ecs::{
    component::ComponentId, prelude::*, query::ReadOnlyWorldQuery, reflect::AppTypeRegistry,
    system::CommandQueue,
};
use bevy::hierarchy::{Children, Parent};
use bevy::reflect::{Reflect, TypeRegistry};
use pretty_type_name::pretty_type_name;
use std::any::TypeId;

/// Display a single [`&mut dyn Reflect`](bevy_reflect::Reflect).
///
/// If you are wondering why this function takes in a [`&mut World`](bevy_ecs::world::World), it's so that if the value contains e.g. a
/// `Handle<StandardMaterial>` it can look up the corresponding asset resource and display the asset value inline.
///
/// If all you're displaying is a simple value without any references into the bevy world, consider just using
/// [`reflect_inspector::ui_for_value`](crate::reflect_inspector::ui_for_value).
fn ui_for_value(value: &mut dyn Reflect, ui: &mut egui::Ui, world: &mut World) -> bool {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut queue = CommandQueue::default();
    let mut env = ReflectEditor::for_bevy(&registry, world.as_unsafe_world_cell(), &mut queue);
    let changed = env.reflect_mut(Arg::null(ui), value);
    queue.apply(world);
    changed
}

/// Display all reflectable resources in the world
fn ui_for_resources(world: &mut World, ui: &mut egui::Ui) {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut resources: Vec<_> = registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    let mut queue = CommandQueue::default();

    for (name, type_id) in resources {
        ui.collapsing(name, |ui| {
            ui_for_resource_internal(
                world.as_unsafe_world_cell(),
                &mut queue,
                type_id,
                ui,
                name,
                &registry,
            );
        });
    }

    queue.apply(world);
}

/// Display the resource `R`
fn ui_for_resource<R: Resource + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let mut queue = CommandQueue::default();

    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    unsafe {
        let world = world.as_unsafe_world_cell();
        // create a context with access to the world except for the `R` resource
        let Some(mut resource) = world.get_resource_mut::<R>() else {
            return errors::Error::ResourceDoesNotExist(std::any::TypeId::of::<R>())
                .show_typed::<R>(ui);
        };

        let mut env = ReflectEditor::for_bevy(&registry, world, &mut queue);

        if env.reflect_mut(Arg::null(ui), resource.bypass_change_detection()) {
            resource.set_changed();
        }
    }
    queue.apply(world);
}

/// Display all reflectable assets
fn ui_for_all_assets(world: &mut World, ui: &mut egui::Ui) {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut assets: Vec<_> = registry
        .iter()
        .filter(|registration| registration.data::<ReflectAsset>().is_some())
        .map(|registration| {
            let name = registration.type_info().type_path_table().short_path();
            (name, registration.type_id())
        })
        .collect();

    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (name, asset_type_id) in assets {
        ui.collapsing(name, |ui| {
            ui_for_assets_internal(world, asset_type_id, ui, &registry);
        });
    }
}

/// Display all assets of the specified asset type `A`
fn ui_for_assets<A: Asset + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let asset_server = world.get_resource::<AssetServer>().cloned();

    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut queue = CommandQueue::default();
    unsafe {
        let world = world.as_unsafe_world_cell();

        // create a context with access to the world except for the `R` resource
        let Some(mut assets) = world.get_resource_mut::<Assets<A>>() else {
            return errors::Error::ResourceDoesNotExist(std::any::TypeId::of::<Assets<A>>())
                .show_typed::<Assets<A>>(ui);
        };

        let mut assets: Vec<_> = assets.iter_mut().collect();
        assets.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (handle_id, asset) in assets {
            let id = egui::Id::new(handle_id);

            let title = handle_name(handle_id.untyped(), asset_server.as_ref());
            egui::CollapsingHeader::new(title)
                .id_source(id)
                .show(ui, |ui| {
                    let mut env = ReflectEditor::for_bevy(&registry, world, &mut queue);
                    env.reflect_mut(Arg::id(id, ui), asset);
                });
        }
    }

    queue.apply(world);
}

/// Display state `T` and change state on edit
fn ui_for_state<T: States + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut queue = CommandQueue::default();
    unsafe {
        let world = world.as_unsafe_world_cell();

        // create a context with access to the world except for the `State<T>` resource

        let Some(state) = world.get_resource::<State<T>>() else {
            errors::state_does_not_exist(ui, &pretty_type_name::<T>());
            return;
        };

        let Some(mut next_state) = world.get_resource_mut::<NextState<T>>() else {
            errors::state_does_not_exist(ui, &pretty_type_name::<T>());
            return;
        };

        let mut env = ReflectEditor::for_bevy(&registry, world, &mut queue);

        let mut current = state.get().clone();
        let changed = env.reflect_mut(Arg::null(ui), &mut current);
        if changed {
            next_state.0 = Some(current);
        }
    }
    queue.apply(world);
}

/// Display all entities and their components
fn ui_for_world_entities(world: &mut World, ui: &mut egui::Ui) {
    ui_for_world_entities_filtered::<Without<Parent>>(world, ui, true);
}

/// Display all entities matching the given filter
fn ui_for_world_entities_filtered<F: ReadOnlyWorldQuery>(
    world: &mut World,
    ui: &mut egui::Ui,
    with_children: bool,
) {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();

    let mut root_entities = world.query_filtered::<Entity, F>();
    let mut entities = root_entities.iter(world).collect::<Vec<_>>();
    entities.sort();

    let mut queue = CommandQueue::default();

    let id = egui::Id::new("world ui");
    for entity in entities {
        let id = id.with(entity);

        let world = world.as_unsafe_world_cell();
        let entity_name = guess_entity_name(world, entity);

        egui::CollapsingHeader::new(&entity_name)
            .id_source(id)
            .show(ui, |ui| {
                if with_children {
                    ui_for_entity_with_children_inner(world, &mut queue, entity, ui, id, &registry);
                } else {
                    ui_for_entity_components(world, &mut queue, entity, ui, id, &registry);
                }
            });
    }

    queue.apply(world);
}

/// Display the given entity with all its components and children
fn ui_for_entity_with_children(
    world: UnsafeWorldCell<'_>,
    queue: &mut CommandQueue,
    entity: Entity,
    ui: &mut egui::Ui,
) {
    let registry = unsafe { world.get_resource::<AppTypeRegistry>().cloned().unwrap() };
    let registry = registry.read();

    let entity_name = guess_entity_name(world, entity);
    ui.label(entity_name);

    ui_for_entity_with_children_inner(world, queue, entity, ui, egui::Id::new(entity), &registry)
}

fn ui_for_entity_with_children_inner(
    world: UnsafeWorldCell<'_>,
    queue: &mut CommandQueue,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    registry: &TypeRegistry,
) {
    ui_for_entity_components(world, queue, entity, ui, id, registry);

    let children = world
        .get_entity(entity)
        .and_then(|entity| unsafe { entity.get::<Children>() })
        .map(|children| children.iter().copied().collect::<Vec<_>>());
    if let Some(children) = children {
        if !children.is_empty() {
            ui.label("Children");
            for &child in children.iter() {
                let id = id.with(child);

                let child_entity_name = guess_entity_name(world, child);
                egui::CollapsingHeader::new(&child_entity_name)
                    .id_source(id)
                    .show(ui, |ui| {
                        ui.label(&child_entity_name);

                        ui_for_entity_with_children_inner(world, queue, child, ui, id, registry);
                    });
            }
        }
    }
}

/// Display the components of the given entity
fn ui_for_entity(world: &mut World, entity: Entity, ui: &mut egui::Ui) {
    let mut queue = CommandQueue::default();

    let registry = world.resource::<AppTypeRegistry>().0.clone();
    let registry = registry.read();
    {
        let world = world.as_unsafe_world_cell();

        let entity_name = guess_entity_name(world, entity);
        ui.label(entity_name);

        ui_for_entity_components(
            world,
            &mut queue,
            entity,
            ui,
            egui::Id::new(entity),
            &registry,
        );
    }
    queue.apply(world);
}

/// Display the components of the given entity
pub(crate) fn ui_for_entity_components(
    world: UnsafeWorldCell<'_>,
    queue: &mut CommandQueue,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    registry: &TypeRegistry,
) {
    let Some(components) = components_of_entity(world, entity) else {
        errors::entity_does_not_exist(ui, entity);
        return;
    };

    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);

        let header = egui::CollapsingHeader::new(&name).id_source(id);

        let Some(component_type_id) = component_type_id else {
            header.show(ui, |ui| errors::no_type_id(ui, &name));
            continue;
        };

        if size == 0 {
            header.show(ui, |_| {});
            continue;
        }

        let mut value =
            match unsafe { get_component_reflect(registry, world, entity, component_type_id) } {
                Ok(value) => value,
                Err(error) => {
                    header.show(ui, |ui| error.show(ui, &name));
                    continue;
                }
            };

        if value.is_changed() {
            set_highlight_style(ui);
        }

        header.show(ui, |ui| {
            ui.reset_style();

            let id = id.with(component_id);
            let value_mut = value.bypass_change_detection();
            if ReflectEditor::for_bevy(registry, world, queue)
                .reflect_mut(Arg::id(id, ui), value_mut)
            {
                value.set_changed();
            }
        });
        ui.reset_style();
    }
}

fn set_highlight_style(ui: &mut egui::Ui) {
    let stroke = egui::Stroke::new(1.0, egui::Color32::GOLD);

    let visuals = &mut ui.style_mut().visuals;
    visuals.collapsing_header_frame = true;
    visuals.widgets.inactive.bg_stroke = stroke;
    visuals.widgets.active.bg_stroke = stroke;
    visuals.widgets.hovered.bg_stroke = stroke;
    visuals.widgets.noninteractive.bg_stroke = stroke;
}

fn components_of_entity(
    world: UnsafeWorldCell<'_>,
    entity: Entity,
) -> Option<Vec<(String, ComponentId, Option<TypeId>, usize)>> {
    let entity_ref = world.get_entity(entity)?;

    let archetype = entity_ref.archetype();
    let mut components: Vec<_> = archetype
        .components()
        .map(|component_id| {
            let info = world.components().get_info(component_id).unwrap();
            let name = pretty_type_name::pretty_type_name_str(info.name());
            (name, component_id, info.type_id(), info.layout().size())
        })
        .collect();
    components.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    Some(components)
}

/// Display the given entity with all its components and children
pub fn ui_for_entities_shared_components(
    world: UnsafeWorldCell<'_>,
    queue: &mut CommandQueue,
    entities: &[Entity],
    ui: &mut egui::Ui,
) {
    let registry = unsafe { world.get_resource::<AppTypeRegistry>().cloned().unwrap() };
    let registry = registry.read();

    let Some(&first) = entities.first() else {
        return;
    };

    let Some(mut components) = components_of_entity(world, first) else {
        return errors::entity_does_not_exist(ui, first);
    };

    for &entity in entities.iter().skip(1) {
        components.retain(|(_, id, _, _)| {
            world
                .get_entity(entity)
                .is_none_or(|entity| entity.contains_id(*id))
        })
    }

    let mut env = ReflectEditor::for_bevy(&registry, world, queue);

    let id = egui::Id::null();
    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);
        egui::CollapsingHeader::new(&name)
            .id_source(id)
            .show(ui, |ui| {
                if size == 0 {
                    return;
                }
                let Some(component_type_id) = component_type_id else {
                    return errors::no_type_id(ui, &name);
                };

                let mut values = Vec::with_capacity(entities.len());
                let mut bypass = Vec::with_capacity(entities.len());

                for (i, &entity) in entities.iter().enumerate() {
                    // skip duplicate entities
                    if entities[0..i].contains(&entity) {
                        continue;
                    }

                    // SAFETY: entities are distinct, env has a context with just resources
                    match unsafe {
                        get_component_reflect(&registry, world, entity, component_type_id)
                    } {
                        Ok(mut value) => {
                            bypass.push(unsafe {
                                crate::util::fuck_mut(value.bypass_change_detection())
                            });
                            values.push(value);
                        }
                        Err(error) => {
                            return error.show(ui, &name);
                        }
                    }
                }

                let id = id.with(component_id);
                let bypass = bypass.as_mut_slice();
                let args = Arg::id(id, ui);
                let changed = env.reflect_many(component_type_id, &name, args, bypass, &|a| a);
                if changed {
                    values.into_iter().for_each(|mut value| value.set_changed());
                }
            });
    }
}

pub fn handle_name(handle: UntypedAssetId, asset_server: Option<&AssetServer>) -> String {
    match asset_server.and_then(|server| server.get_path(handle)) {
        Some(path) => path.to_string(),
        None => match handle {
            UntypedAssetId::Index { index, .. } => format!("{:?}", egui::Id::new(index)),
            UntypedAssetId::Uuid { uuid, .. } => format!("{}", uuid),
        },
    }
}

/// Display the resource with the given [`TypeId`]
pub fn ui_for_resource_internal(
    world: UnsafeWorldCell<'_>,
    queue: &mut CommandQueue,
    resource_type_id: TypeId,
    ui: &mut egui::Ui,
    name_of_type: &str,
    registry: &TypeRegistry,
) {
    let mut env = ReflectEditor::for_bevy(registry, world, queue);

    let mut resource = match unsafe { get_resource_reflect(registry, world, resource_type_id) } {
        Ok(resource) => resource,
        Err(error) => return error.show(ui, name_of_type),
    };

    let bypass = resource.bypass_change_detection();
    if env.reflect_mut(Arg::null(ui), bypass) {
        resource.set_changed();
    }
}

/// Display all assets of the given asset [`TypeId`]
pub fn ui_for_assets_internal(
    world: &mut World,
    asset_type_id: TypeId,
    ui: &mut egui::Ui,
    registry: &TypeRegistry,
) {
    let asset_server = world.get_resource::<AssetServer>().cloned();

    let Some(registration) = registry.get(asset_type_id) else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        return errors::Error::NoTypeRegistration(asset_type_id).show(ui, &type_name);
    };
    let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        return errors::no_type_data(ui, &type_name, "ReflectAsset");
    };
    let Some(reflect_handle) =
        registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id())
    else {
        let type_name = errors::name_of_type(reflect_asset.handle_type_id(), registry);
        return errors::no_type_data(ui, &type_name, "ReflectHandle");
    };

    let ids: Vec<_> = reflect_asset.ids(world).collect();

    // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
    // displaying the T with a world view excluding Assets<T>.
    let mut queue = CommandQueue::default();

    for handle_id in ids {
        let id = egui::Id::new(handle_id);
        let mut handle = reflect_handle.typed(UntypedHandle::Weak(handle_id));

        egui::CollapsingHeader::new(handle_name(handle_id, asset_server.as_ref()))
            .id_source(id)
            .show(ui, |ui| {
                let world = world.as_unsafe_world_cell();
                let mut env = ReflectEditor::for_bevy(registry, world, &mut queue);
                env.reflect_mut(Arg::id(id, ui), &mut *handle);
            });
    }

    queue.apply(world)
}

/// Display a given asset by handle and asset [`TypeId`]
pub fn ui_for_asset(
    world: &mut World,
    asset_type_id: TypeId,
    handle: UntypedAssetId,
    ui: &mut egui::Ui,
    registry: &TypeRegistry,
) -> bool {
    let Some(registration) = registry.get(asset_type_id) else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        errors::Error::NoTypeRegistration(asset_type_id).show(ui, &type_name);
        return false;
    };
    let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
        let type_name = errors::name_of_type(asset_type_id, registry);
        errors::no_type_data(ui, &type_name, "ReflectAsset");
        return false;
    };
    let Some(reflect_handle) =
        registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id())
    else {
        let type_name = errors::name_of_type(reflect_asset.handle_type_id(), registry);
        errors::no_type_data(ui, &type_name, "ReflectHandle");
        return false;
    };

    let _: Vec<_> = reflect_asset.ids(world).collect();

    // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
    // displaying the T with a world view excluding Assets<T>.
    let mut queue = CommandQueue::default();

    let id = egui::Id::new(handle);
    let mut handle = reflect_handle.typed(UntypedHandle::Weak(handle));

    let mut env = ReflectEditor::for_bevy(registry, world.as_unsafe_world_cell(), &mut queue);
    let changed = env.reflect_mut(Arg::id(id, ui), &mut *handle);

    queue.apply(world);

    changed
}

use crate::panel::run_panel_scrollbar;
use crate::ui::EditorStage;
use bevy::prelude::*;
use std::marker::PhantomData;

// "World Inspector"
#[derive(Component, Default)]
pub struct WorldInspectorTab;

impl WorldInspectorTab {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    /// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            egui::CollapsingHeader::new("Entities").show(ui, |ui| ui_for_world_entities(world, ui));
            egui::CollapsingHeader::new("Resources").show(ui, |ui| ui_for_resources(world, ui));
            egui::CollapsingHeader::new("Assets").show(ui, |ui| ui_for_all_assets(world, ui));
            ui.allocate_space(ui.available_size());
        });
    }
}

// "World Inspector"
#[derive(Component, Default)]
pub struct AllAssetsTab;

impl AllAssetsTab {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    /// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            ui_for_all_assets(world, ui);
            ui.allocate_space(ui.available_size());
        });
    }
}

#[derive(Component, Default)]
pub struct ResourceTab<T: Resource + Reflect>(pub PhantomData<fn() -> T>);

impl<T: Resource + Reflect> ResourceTab<T> {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            ui.heading(pretty_type_name::<T>());
            ui_for_resource::<T>(world, ui);
            ui.allocate_space(ui.available_size());
        });
    }
}

#[derive(Component, Default)]
pub struct StateTab<T: States + Reflect>(pub PhantomData<fn() -> T>);

impl<T: States + Reflect> StateTab<T> {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            ui.heading(std::any::type_name::<T>());
            ui.heading(pretty_type_name::<T>());
            ui_for_state::<T>(world, ui);
        });
    }
}

#[derive(Component, Default)]
pub struct AssetTab<A: Asset + Reflect>(pub PhantomData<fn() -> A>);

impl<A: Asset + Reflect> AssetTab<A> {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            ui.heading(pretty_type_name::<A>());
            ui_for_assets::<A>(world, ui);
            ui.allocate_space(ui.available_size());
        });
    }
}

#[derive(Component)]
pub struct EntityQueryTab<F: ReadOnlyWorldQuery>(pub PhantomData<fn() -> F>);

impl<F: ReadOnlyWorldQuery> Default for EntityQueryTab<F> {
    fn default() -> Self {
        Self(PhantomData::<fn() -> F>)
    }
}

impl<F: ReadOnlyWorldQuery + 'static> EntityQueryTab<F> {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            //ui.heading(pretty_type_name::<F>());
            ui_for_world_entities_filtered::<F>(world, ui, false);
            ui.allocate_space(ui.available_size());
        });
    }
}
