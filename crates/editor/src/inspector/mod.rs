pub mod impls;
pub mod integration;

/// [`bevy_app::Plugin`] used to register default [`struct@InspectorOptions`] and [`InspectorEguiImpl`](crate::inspector_egui_impls::InspectorEguiImpl)s
pub struct DefaultInspectorConfigPlugin;

impl bevy::prelude::Plugin for DefaultInspectorConfigPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if app.is_plugin_added::<Self>() {
            return;
        }

        let type_registry = app.world.resource::<bevy::ecs::prelude::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        self::impls::register_default_options(&mut type_registry);
        self::impls::register_all(&mut type_registry);
    }
}

use bevy::core::Name;
use bevy::ecs::prelude::*;
use bevy::ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell};

/// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
pub fn guess_entity_name(world: UnsafeWorldCell, entity: Entity) -> String {
    let name = unsafe { name_for_entity(world, entity).unwrap_or("Entity") };
    format!("{name} ({entity:?})")
}

unsafe fn name_for_entity(world: UnsafeWorldCell<'_>, entity: Entity) -> Option<&str> {
    world.get_entity(entity).and_then(|entity| {
        Option::or_else(entity.get::<Name>().map(Name::as_str), || {
            name_from_components(entity)
        })
    })
}

fn name_from_components(entity: UnsafeEntityCell<'_>) -> Option<&str> {
    macro_rules! _naming {
        ( $entity:ident: [ $(< $ty:ty > : $name:literal ,)+ ]) => {
            $( if $entity.contains::<$ty>() { return Some($name); } )+
        };
    }

    _naming!(entity: [
        <crate::ui::EditorPanel>: "Editor Panel",
        <bevy::window::PrimaryWindow>: "Primary Window",
        <bevy::window::Window>: "Window",
        <bevy::core_pipeline::core_3d::Camera3d>: "Camera 3D",
        <bevy::core_pipeline::core_2d::Camera2d>: "Camera 2D",
        <bevy::pbr::PointLight>: "Point Light",
        <bevy::pbr::SpotLight>: "Spot Light",
        <bevy::pbr::DirectionalLight>: "Directional Light",
        <bevy::text::Text>: "Text",
        <bevy::ui::Node>: "UI Node",
        <bevy::asset::Handle<bevy::pbr::StandardMaterial>>: "PBR Mesh",
    ]);

    None
}
