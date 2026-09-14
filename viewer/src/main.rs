//! A simple glTF scene viewer made with Bevy.
//!
//! Just run `cargo run --release --example scene_viewer /path/to/model.gltf`,
//! replacing the path as appropriate.
//! In case of multiple scenes, you can select which to display by adapting the file path: `/path/to/model.gltf#Scene1`.
//! With no arguments it will load the `FlightHelmet` glTF model from the repository assets subdirectory.
//! Pass `--help` to see all the supported arguments.
//!
//! If you want to hot reload asset changes, enable the `file_watcher` cargo feature.

#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use argh::FromArgs;
use bevy::{
    camera::primitives::{Aabb, Sphere},
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass},
    pbr::DefaultOpaqueRendererMethod,
    prelude::*,
    render::{experimental::occlusion_culling::OcclusionCulling, view::Hdr},
};
use bevy_egui::EguiPlugin;
use bevy_skein::SkeinPlugin;

mod camera;
mod material;
mod ui;

#[cfg(feature = "animation")]
mod animation;
mod morph;
mod scene;

mod raycast;

use self::camera::CameraController;
use self::scene::SceneHandle;

/// A simple glTF scene viewer made with Bevy
#[derive(FromArgs, Resource)]
struct Args {
    /// the path to the glTF scene
    #[argh(positional, default = "String::from(\"assets/models/furniture.glb\")")]
    scene_path: String,
    /// enable a depth prepass
    #[argh(switch)]
    depth_prepass: Option<bool>,
    /// enable occlusion culling
    #[argh(switch)]
    occlusion_culling: Option<bool>,
    /// enable deferred shading
    #[argh(switch)]
    deferred: Option<bool>,
    /// spawn a light even if the scene already has one
    #[argh(switch)]
    add_light: Option<bool>,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let mut args: Args = argh::from_env();
    #[cfg(target_arch = "wasm32")]
    let mut args: Args = Args::from_args(&[], &[]).unwrap();

    args.deferred = Some(true);
    args.occlusion_culling = Some(true);

    let deferred = args.deferred;

    let title = String::from("scene viewer");
    let file_path = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { title, ..default() }),
                ..default()
            })
            .set(AssetPlugin {
                file_path,
                ..default()
            }),
        SkeinPlugin::default(),
        EguiPlugin::default(),
        self::camera::plugin,
        self::scene::plugin,
        self::morph::plugin,
        self::material::plugin,
        self::ui::plugin,
    ))
    .insert_resource(args)
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, setup_scene_after_load);

    // If deferred shading was requested, turn it on.
    if deferred == Some(true) {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred());
    }

    #[cfg(feature = "animation")]
    app.add_plugins(self::animation::plugin);

    app.run();
}

fn parse_scene(scene_path: String) -> (String, usize) {
    if scene_path.contains('#') {
        let gltf_and_scene = scene_path.split('#').collect::<Vec<_>>();
        if let Some((last, path)) = gltf_and_scene.split_last()
            && let Some(index) = last
                .strip_prefix("Scene")
                .and_then(|index| index.parse::<usize>().ok())
        {
            return (path.join("#"), index);
        }
    }
    (scene_path, 0)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, args: Res<Args>) {
    let scene_path = &args.scene_path;
    info!("Loading {scene_path}");
    let (file_path, scene_index) = parse_scene((*scene_path).clone());

    commands.insert_resource(SceneHandle::new(asset_server.load(file_path), scene_index));
}

fn setup_scene_after_load(
    mut commands: Commands,
    mut setup: Local<bool>,
    mut scene_handle: ResMut<SceneHandle>,
    asset_server: Res<AssetServer>,
    args: Res<Args>,
    meshes: Query<(&GlobalTransform, Option<&Aabb>), With<Mesh3d>>,
) {
    if scene_handle.is_loaded && !*setup {
        *setup = true;
        // Find an approximate bounding box of the scene from its meshes
        if meshes.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
            return;
        }

        let mut min = Vec3A::splat(f32::MAX);
        let mut max = Vec3A::splat(f32::MIN);
        for (transform, maybe_aabb) in &meshes {
            let aabb = maybe_aabb.unwrap();
            // If the Aabb had not been rotated, applying the non-uniform scale would produce the
            // correct bounds. However, it could very well be rotated and so we first convert to
            // a Sphere, and then back to an Aabb to find the conservative min and max points.
            let sphere = Sphere {
                center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
                radius: transform.radius_vec3a(aabb.half_extents),
            };
            let aabb = Aabb::from(sphere);
            min = min.min(aabb.min());
            max = max.max(aabb.max());
        }

        let size = (max - min).length();
        let aabb = Aabb::from_min_max(Vec3::from(min), Vec3::from(max));

        info!("Spawning a controllable 3D perspective camera");
        let mut projection = PerspectiveProjection::default();
        projection.far = projection.far.max(size * 10.0);
        projection.fov = core::f32::consts::PI / 6.0;

        // let walk_speed = dbg!(size * 3.0);
        let walk_speed = dbg!(size);
        let camera_controller = CameraController {
            walk_speed,
            run_speed: 3.0 * walk_speed,
            ..default()
        };

        // Display the controls of the scene viewer
        info!("{camera_controller}");
        info!("{}", *scene_handle);

        let mut camera = commands.spawn((
            Camera3d::default(),
            Projection::from(projection),
            Transform::from_translation(Vec3::from(aabb.center) + size * Vec3::new(0.5, 0.25, 0.5))
                .looking_at(Vec3::from(aabb.center), Vec3::Y),
            Hdr,
            Camera {
                is_active: false,
                ..default()
            },
            EnvironmentMapLight {
                diffuse_map: asset_server
                    .load("assets/environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server
                    .load("assets/environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
                intensity: 150.0,
                ..default()
            },
            bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
            bevy::post_process::bloom::Bloom {
                // max_mip_dimension: 1024,
                // scale: Vec2::new(4.0, 4.0),
                // ..bevy::core_pipeline::bloom::Bloom::OLD_SCHOOL
                prefilter: bevy::post_process::bloom::BloomPrefilter {
                    threshold: 2.1, //0.6,
                    threshold_softness: 0.2,
                },
                ..bevy::post_process::bloom::Bloom::NATURAL
            },
            // bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
            // bevy::core_pipeline::experimental::taa::TemporalAntiAliasing::default(),
            bevy::anti_alias::smaa::Smaa {
                preset: bevy::anti_alias::smaa::SmaaPreset::Ultra,
            },
            camera_controller,
        ));

        // If occlusion culling was requested, include the relevant components.
        // The Z-prepass is currently required.
        if args.occlusion_culling == Some(true) {
            camera.insert((DepthPrepass, OcclusionCulling));
        }

        // If the depth prepass was requested, include it.
        if args.depth_prepass == Some(true) {
            camera.insert(DepthPrepass);
        }

        // If deferred shading was requested, include the prepass.
        if args.deferred == Some(true) {
            camera
                .insert(Msaa::Off)
                .insert(DepthPrepass)
                .insert(DeferredPrepass);
        }

        // Spawn a default light if the scene does not have one
        if !scene_handle.has_light || args.add_light == Some(true) {
            info!("Spawning a directional light");

            commands.insert_resource(bevy::light::DirectionalLightShadowMap { size: 4096 });

            let mut light = commands.spawn((
                DirectionalLight {
                    shadows_enabled: true,

                    illuminance: light_consts::lux::FULL_DAYLIGHT,
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
            ));
            if args.occlusion_culling == Some(true) {
                light.insert(OcclusionCulling);
            }

            scene_handle.has_light = true;
        }
    }
}
