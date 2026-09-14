#![allow(clippy::type_complexity)]

use crate::character::SpawnCharacter;
use crate::state::InGame;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseWheel,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        view::ColorGrading,
        RenderPlugin,
    },
    window::PresentMode,
};

pub mod gen;

pub mod character;
pub mod debug;
pub mod loading;
pub mod main_menu;
pub mod mechanics;
pub mod player;
pub mod raycast;
pub mod render;
pub mod settings;
pub mod splash_screen;
pub mod state;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Lifer"),
                    resolution: (1280.0, 720.0).into(),
                    present_mode: PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),

                synchronous_pipeline_compilation: false,

                ..default()
            })
            .set(bevy::log::LogPlugin {
                // `RUST_LOG=lifer=trace,thirst=trace cargo run --example thirst --features=trace` to see extra tracing output.
                // filter: "lifer=debug,wgpu=error,naga=warn".to_string(),
                filter: "lifer=warn,wgpu=error,naga=warn".to_string(),
                ..default()
            }),
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
        bevy_egui::EguiPlugin::default(),
        bevy::core_pipeline::auto_exposure::AutoExposurePlugin,
    ));

    // see: https://github.com/mvlabat/bevy_egui/issues/47
    app.insert_resource(bevy_egui::EguiGlobalSettings {
        auto_create_primary_context: true,
        enable_focused_non_window_context_updates: true,
        input_system_settings: default(),
        enable_absorb_bevy_input_system: true,
        enable_cursor_icon_updates: true,
    });

    // for SSR
    // app.insert_resource(bevy::pbr::DefaultOpaqueRendererMethod::deferred());

    app.add_plugins((
        crate::state::plugin,
        crate::settings::SettingsPlugin::new("com", "Plague Automata", "Lifer"),
        crate::splash_screen::plugin,
        crate::loading::plugin,
        crate::player::plugin,
        crate::main_menu::plugin,
        crate::mechanics::plugin,
        crate::character::plugin,
        crate::raycast::plugin,
        crate::render::plugin,
        crate::debug::plugin,
    ));

    {
        use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

        app.add_plugins(WireframePlugin::default())
            .insert_resource(WireframeConfig {
                // The global wireframe config enables drawing of wireframes on every mesh,
                // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
                // regardless of the global configuration.
                global: false,
                // Controls the default color of all wireframes. Used as the default color for global wireframes.
                // Can be changed per mesh using the `WireframeColor` component.
                default_color: Color::WHITE,
            });
    }

    app.add_systems(Startup, setup_camera);
    app.add_systems(OnEnter(InGame), init_scene);

    app.add_systems(bevy_egui::EguiPrimaryContextPass, cascade_config_ui);
    app.add_systems(PostUpdate, cascade_config_update);

    app.run();
}

use bevy_egui::{egui, EguiContexts};

pub fn menu_style<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    use egui::FontFamily::Proportional;
    use egui::FontId;
    use egui::TextStyle::*;

    ui.scope(|ui| {
        ui.style_mut().text_styles = [
            (Heading, FontId::new(50.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(24.0, Proportional)),
            (Monospace, FontId::new(24.0, Proportional)),
            (Button, FontId::new(24.0, Proportional)),
            (Small, FontId::new(18.0, Proportional)),
        ]
        .into();

        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
        ui.spacing_mut().button_padding = egui::vec2(0.0, 32.0);

        contents(ui)
    })
}

fn setup_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut compensation_curves: ResMut<
        Assets<bevy::core_pipeline::auto_exposure::AutoExposureCompensationCurve>,
    >,
) {
    let metering_mask = asset_server.load("basic_metering_mask.png");

    let compensation_curve = bevy::math::cubic_splines::LinearSpline::new([
        bevy::math::vec2(-4.0, -2.0),
        bevy::math::vec2(0.0, 0.0),
        bevy::math::vec2(2.0, 0.0),
        bevy::math::vec2(4.0, 2.0),
    ]);

    let compensation_curve = compensation_curves.add(
        bevy::core_pipeline::auto_exposure::AutoExposureCompensationCurve::from_curve(
            compensation_curve,
        )
        .unwrap(),
    );

    // commands.insert_resource(Msaa::Sample4);

    commands
        .spawn((
            Transform::from_xyz(-2.5, 0.0, 0.0),
            Visibility::Inherited,
            crate::player::CameraController,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Msaa::Off,
                    Camera {
                        hdr: true,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
                    bevy::render::camera::CameraRenderGraph::new(
                        bevy::core_pipeline::core_3d::graph::Core3d,
                    ),
                    Projection::Perspective(PerspectiveProjection {
                        fov: std::f32::consts::FRAC_PI_4,
                        // near: 0.1,
                        // far: 1000.0,
                        near: 1.0,
                        far: 1000.0,

                        aspect_ratio: 1.0,
                    }),
                    Camera3d::default(),
                    /*
                    Camera3dBundle {

                        // tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
                        // deband_dither: bevy::core_pipeline::tonemapping::DebandDither::default(),
                        // visible_entities: bevy::render::view::VisibleEntities::default(),
                        // frustum: bevy::render::primitives::Frustum::default(),
                        // color_grading: bevy::render::view::ColorGrading::default(),
                        // exposure: bevy::render::camera::Exposure::default(),
                        // main_texture_usages: bevy::render::camera::CameraMainTextureUsages::default(),
                        ..default()
                    },
                        */
                ))
                .insert((
                    // bevy::render::view::GpuCulling,
                    bevy::render::view::NoCpuCulling,
                ))
                .insert(crate::raycast::PlaneRaycast::Y)
                .insert(camera(metering_mask, compensation_curve))
                .insert(ColorGrading::default())
                .insert(EnvironmentMapLight {
                    diffuse_map: asset_server
                        .load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                    specular_map: asset_server
                        .load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
                    intensity: 2000.0,
                    rotation: Quat::IDENTITY,
                    affects_lightmapped_mesh_diffuse: true,
                });
        });
}

fn camera(
    metering_mask: Handle<Image>,
    compensation_curve: Handle<bevy::core_pipeline::auto_exposure::AutoExposureCompensationCurve>,
) -> impl Bundle {
    use bevy::core_pipeline::{
        auto_exposure::AutoExposure,
        bloom::Bloom,
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
        experimental::taa::TemporalAntiAliasing,
        motion_blur::MotionBlur,
        post_process::ChromaticAberration,
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass, NormalPrepass},
        smaa::Smaa,
        tonemapping::DebandDither,
        tonemapping::Tonemapping,
    };

    let render = (
        bevy::render::primitives::Frustum::default(),
        bevy::render::view::VisibleEntities::default(),
        bevy::render::view::ColorGrading::default(),
        bevy::render::camera::TemporalJitter::default(),
        bevy::render::camera::Exposure::OVERCAST,
        bevy::render::camera::CameraMainTextureUsages::default(),
    );

    let pbr = (
        bevy::pbr::ShadowFilteringMethod::Temporal,
        // bevy::pbr::ShadowFilteringMethod::Gaussian,
        // bevy::pbr::VolumetricFogSettings::default(),
        bevy::pbr::ScreenSpaceReflections::default(),
        bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
        /*
        bevy::pbr::FogSettings {
            color: Color::srgba_u8(43, 44, 47, 255),
            falloff: FogFalloff::from_visibility_colors(
                // distance in world units up to which objects retain visibility (>= 5% contrast)
                75.0,
                // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::srgb(0.35, 0.5, 0.66),
                // atmospheric inscattering color (light gained due to scattering from the sun)
                Color::srgb(0.8, 0.844, 1.0),
            ),
            ..default()
        },
        */
        // bevy::pbr::VolumetricFog {
        //     // This value is explicitly set to 0 since we have no environment map light.
        //     ambient_intensity: 0.002,
        //     ..default()
        // },
    );

    let core = (
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        // bevy::core_pipeline::dof::DepthOfFieldSettings::default(),
        (
            DepthPrepass,
            NormalPrepass,
            // MotionVectorPrepass,
            // DeferredPrepass,
        ),
        // ChromaticAberration {
        //     intensity: 0.005,
        //     //max_samples,
        //     ..default()
        // },
        Bloom::NATURAL,
        ContrastAdaptiveSharpening {
            sharpening_strength: 0.7,
            ..default()
        },
        // TemporalAntiAliasSettings::default(),
        Smaa::default(),
        // MotionBlur::default(),
        AutoExposure {
            // range: -bevy::render::camera::Exposure::EV100_OVERCAST
            //     ..=bevy::render::camera::Exposure::EV100_OVERCAST,
            range: -bevy::render::camera::Exposure::EV100_SUNLIGHT
                ..=bevy::render::camera::Exposure::EV100_SUNLIGHT,

            metering_mask,
            compensation_curve,

            speed_brighten: 3.0 / 4.0,
            speed_darken: 1.0 / 4.0,

            ..default()
        },
    );

    (render, pbr, core)
}

fn init_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawner: EventWriter<SpawnCharacter>,
) {
    // circular base
    commands.spawn((
        StateScoped(InGame),
        Mesh3d(meshes.add(Circle::new(204.0))),
        // material: cache.get_material(
        //     &mut materials,
        //     Color::Srgba(bevy::color::palettes::css::DARK_GREEN),
        // ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::Srgba(bevy::color::palettes::css::DARK_GREEN),
            // perceptual_roughness: 0.099,
            perceptual_roughness: 1.00,
            ..default()
        })),
        // perceptual_roughness_threshold
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ..default()
        },
    ));

    {
        // some characters

        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};
        let mut rng = SmallRng::from_entropy();

        let positions = fast_poisson::Poisson2D::new()
            .with_dimensions([100.0; 2], 5.0)
            .with_seed(rng.gen());

        for [x, z] in positions {
            spawner.send(SpawnCharacter {
                transform: Transform::from_xyz((x - 100.0) as f32, 0.0, (z - 100.0) as f32),
                ..default()
            });
        }
    }

    // spawn player
    spawner.send(SpawnCharacter {
        player: true,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        // color: Color::ORANGE_RED,
        // brightness: 0.02,
        color: Color::WHITE,
        brightness: 80.0,
        // brightness: 800.0,
        affects_lightmapped_meshes: true,
    });

    // commands.insert_resource(DirectionalLightShadowMap { size: 2048 });
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    if false {
        let mesh = crate::gen::gen_map();

        commands.spawn((
            StateScoped(InGame),
            Mesh3d(meshes.add(mesh)),
            // material: cache.get_material(&mut materials, Color::WHITE),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 1.0,
                unlit: true,
                fog_enabled: false,
                ..default()
            })),
        ));
    }

    let builder = CascadeShadowConfigBuilder {
        num_cascades: 4,
        // minimum_distance: 0.1,
        minimum_distance: 5.0,
        // first_cascade_far_bound: 5.0,
        first_cascade_far_bound: 20.0,
        // maximum_distance: 1000.0,
        maximum_distance: 200.0,
        overlap_proportion: 0.2,
    };
    let builder = CascadeShadowConfigBuilderComponent::from(builder);

    // directional 'sun' light
    commands.spawn((
        StateScoped(InGame),
        // builder.clone(),
        bevy::pbr::VolumetricLight,
        DirectionalLight {
            shadows_enabled: true,
            // illuminance: light_consts::lux::FULL_DAYLIGHT,
            illuminance: light_consts::lux::DIRECT_SUNLIGHT,

            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        builder.build(),
    ));
}

#[derive(Component, Clone)]
pub struct CascadeShadowConfigBuilderComponent {
    pub num_cascades: usize,
    pub minimum_distance: f32,
    pub maximum_distance: f32,
    pub first_cascade_far_bound: f32,
    pub overlap_proportion: f32,
}

impl From<CascadeShadowConfigBuilderComponent> for CascadeShadowConfigBuilder {
    fn from(val: CascadeShadowConfigBuilderComponent) -> Self {
        CascadeShadowConfigBuilder {
            num_cascades: val.num_cascades,
            minimum_distance: val.minimum_distance,
            maximum_distance: val.maximum_distance,
            first_cascade_far_bound: val.first_cascade_far_bound,
            overlap_proportion: val.overlap_proportion,
        }
    }
}

impl From<CascadeShadowConfigBuilder> for CascadeShadowConfigBuilderComponent {
    fn from(value: CascadeShadowConfigBuilder) -> Self {
        Self {
            num_cascades: value.num_cascades,
            minimum_distance: value.minimum_distance,
            maximum_distance: value.maximum_distance,
            first_cascade_far_bound: value.first_cascade_far_bound,
            overlap_proportion: value.overlap_proportion,
        }
    }
}

impl CascadeShadowConfigBuilderComponent {
    fn build(self) -> CascadeShadowConfig {
        CascadeShadowConfigBuilder::from(self).build()
    }
}

fn cascade_config_update(
    mut query: Query<(
        &mut CascadeShadowConfig,
        Ref<CascadeShadowConfigBuilderComponent>,
    )>,
) {
    for (mut config, cascade) in &mut query {
        if cascade.is_changed() {
            *config = cascade.clone().build();
        }
    }
}

fn cascade_config_ui(
    mut contexts: EguiContexts,
    mut query: Query<Mut<CascadeShadowConfigBuilderComponent>>,
) -> Result {
    let Ok(config) = query.single_mut() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::from("#CASCADE"))
        .anchor(egui::Align2::LEFT_BOTTOM, [0.0; 2])
        .show(ctx, |ui| {
            egui::Frame::side_top_panel(ui.style()).show(ui, |ui| {
                let _ = ui.allocate_space(egui::vec2(250.0, 0.0));

                let mut config = config.map_unchanged(|c| c);

                let min = config.bypass_change_detection().minimum_distance;
                let max = config.bypass_change_detection().maximum_distance;

                ui.label("minimum_distance");
                let widget =
                    egui::DragValue::new(&mut config.bypass_change_detection().minimum_distance);
                if ui.add(widget.range(1.0..=max - 1.0).speed(1.0)).changed() {
                    config.set_changed();
                }

                ui.label("maximum_distance");
                let widget =
                    egui::DragValue::new(&mut config.bypass_change_detection().maximum_distance);
                if ui.add(widget.range(min + 1.0..=200.0).speed(1.0)).changed() {
                    config.set_changed();
                }

                ui.label("first_cascade_far_bound");
                let widget = egui::DragValue::new(
                    &mut config.bypass_change_detection().first_cascade_far_bound,
                );
                if ui
                    .add(widget.range(min + 1.0..=max - 1.0).speed(1.0))
                    .changed()
                {
                    config.set_changed();
                }
            });
        });

    Ok(())
}
