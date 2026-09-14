pub mod asset;
pub mod component;
pub mod context;
pub mod filebrowser;
pub mod gizmo;
pub mod hierarchy;
pub mod inspector;
pub mod tab;

pub use self::{
    asset::{
        ReflectEntity, ReflectScene, ReflectSceneLoader, ReflectSceneSpawnError,
        ReflectSceneSpawner, SceneMapping,
    },
    filebrowser::FileBrowser,
    gizmo::{GizmoPlugin, GIZMO_DRIVER},
    hierarchy::Hierarchy,
    inspector::{Inspector, InspectorState},
    tab::SceneTab,
};

use bevy::ecs::event::Events;
use bevy::prelude::*;

#[derive(Default)]
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ReflectScene>()
            .init_resource::<SceneMapping>()
            .init_resource::<InspectorState>()
            .init_asset_loader::<ReflectSceneLoader>()
            .init_resource::<ReflectSceneSpawner>()
            //.add_systems(PreUpdate, scene_spawner_system.exclusive_system().at_end());
            .add_systems(PreUpdate, scene_spawner_system);
    }
}

pub fn scene_spawner_system(world: &mut World) {
    world.resource_scope(|world, mut spawner: Mut<ReflectSceneSpawner>| {
        let scene_asset_events = world.resource::<Events<AssetEvent<ReflectScene>>>();

        let mut handles = Vec::new();
        let spawner = &mut *spawner;
        for event in spawner.event_reader.read(scene_asset_events) {
            if let AssetEvent::Modified { id } = event {
                if spawner.scenes.contains_key(id) {
                    handles.push(id.clone_weak());
                }
            }
        }

        spawner.despawn_queued(world).unwrap();
        spawner.spawn_queued(world).unwrap();
        spawner.update_spawned(world, &handles).unwrap();
        spawner.set_scene_instance_parent_sync(world);
    });
}
