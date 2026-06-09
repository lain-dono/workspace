use crate::state::{AppState, InGame};
use bevy::prelude::*;
use bevy::{
    diagnostic,
    render::RenderPlugin,
    render::settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    window::{PresentMode, WindowResolution},
};
use bevy_asset_loader::prelude::*;

mod ai;
mod camera;
mod character;
mod dev;
mod grid;
mod material;
mod raycast;
mod render;
mod settings;
mod state;
mod time;
mod ui;

fn asset_path() -> String {
    use std::env::{current_dir, var};
    use std::path::PathBuf;

    (var("CARGO_MANIFEST_DIR").ok().map(PathBuf::from))
        .or(current_dir().ok())
        .map(|p| p.join("assets"))
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| String::from("./assets"))
}

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                file_path: asset_path(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Lifer"),
                    resolution: WindowResolution::new(1280, 720),
                    present_mode: PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            })
            //.disable::<bevy::log::LogPlugin>(),
            .set(bevy::log::LogPlugin {
                // `RUST_LOG=lifer=trace,thirst=trace cargo run --example thirst --features=trace` to see extra tracing output.
                #[cfg(feature = "dev_mode")]
                filter: "game=debug,wgpu=error,naga=warn".to_string(),
                #[cfg(not(feature = "dev_mode"))]
                filter: "game=warn,wgpu=error,naga=warn".to_string(),
                ..default()
            }),
        diagnostic::LogDiagnosticsPlugin::default(),
        // diagnostic::FrameTimeDiagnosticsPlugin::default(),
        //bevy::anti_alias::taa::TemporalAntiAliasPlugin,
        bevy_egui::EguiPlugin::default(),
        bevy::post_process::auto_exposure::AutoExposurePlugin,
        iyes_progress::ProgressPlugin::<AppState>::new()
            .with_state_transition(AppState::Loading, AppState::MainMenu),
        bevy_skein::SkeinPlugin::default(),
    ));

    app.add_plugins((
        #[cfg(feature = "dev_mode")]
        crate::dev::raycast::plugin,
        crate::time::plugin,
        crate::state::plugin,
        crate::render::plugin,
        crate::material::plugin,
        crate::raycast::plugin,
        crate::settings::SettingsPlugin::new("com", "Plague Automata", "Lifer"),
        crate::ui::plugin,
        //
        crate::grid::plugin,
        crate::camera::plugin,
        crate::character::plugin,
    ));

    // app.insert_resource(ClearColor(Color::BLACK));

    app.add_systems(Startup, (setup_scene,));
    app.add_systems(
        PreUpdate,
        (scene_load_check, scene_after_load)
            .chain()
            .run_if(in_state(InGame)),
    );

    app.add_loading_state(LoadingState::new(AppState::Loading).load_collection::<SceneHandle>());

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(ai::Actor {
        motives: vec![
            ai::Motive { current: 10.0 },
            ai::Motive { current: 20.0 },
            ai::Motive { current: 30.0 },
            ai::Motive { current: 60.0 },
            ai::Motive { current: 80.0 },
            ai::Motive { current: 100.0 },
        ],
    });
}

// #[derive(Resource)]
#[derive(AssetCollection, Resource)]
pub struct SceneHandle {
    #[asset(path = "models/furniture.glb")]
    pub handle: Handle<Gltf>,
    pub is_loaded: bool,
    pub instance_id: Option<bevy::scene::InstanceId>,
}

fn scene_load_check(
    asset_server: Res<AssetServer>,
    mut _scenes: ResMut<Assets<Scene>>,
    gltf: Res<Assets<Gltf>>,
    mut handle: ResMut<SceneHandle>,
    mut spawner: ResMut<SceneSpawner>,
) {
    if let Some(instance_id) = handle.instance_id {
        if !handle.is_loaded && spawner.instance_is_ready(instance_id) {
            info!("...done!");
            handle.is_loaded = true;
        }
    } else if asset_server.load_state(&handle.handle).is_loaded() {
        let gltf = gltf.get(&handle.handle).unwrap();
        let scene = gltf.scenes.first().unwrap();
        handle.instance_id = Some(spawner.spawn(scene.clone()));
        info!("Spawning scene...");
    }
}

fn scene_after_load(
    mut commands: Commands,
    mut setup: Local<bool>,
    scene_handle: Res<SceneHandle>,
) {
    if scene_handle.is_loaded && !*setup {
        *setup = true;

        // Spawn a default light if the scene does not have one
        info!("Spawning a directional light");

        commands.insert_resource(bevy::light::DirectionalLightShadowMap { size: 4096 });

        commands.spawn((
            DirectionalLight {
                shadows_enabled: true,

                illuminance: light_consts::lux::FULL_DAYLIGHT,
                // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                ..default()
            },
            bevy::light::CascadeShadowConfig::from(bevy::light::CascadeShadowConfigBuilder {
                minimum_distance: 0.1,
                maximum_distance: 150.0,
                first_cascade_far_bound: 20.0,
                overlap_proportion: 0.1,
                ..default()
            }),
            Transform::from_xyz(1.0, 3.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            // bevy::render::experimental::occlusion_culling::OcclusionCulling,
        ));
    }
}
