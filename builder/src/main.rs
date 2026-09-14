use bevy::{
    camera::{Exposure, PhysicalCameraParameters},
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    winit::WinitSettings,
};
use bevy_egui::EguiPlugin;
use std::f32::consts::PI;

// mod giz;
// mod math;
//mod roof;
// mod player;

mod object;
pub mod pan_orbit_camera;
pub mod post;
pub mod scheme;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgba(0.2, 0.2, 0.2, 0.0)));
    app.insert_resource(WinitSettings::desktop_app());

    app.add_plugins((DefaultPlugins, MeshPickingPlugin, EguiPlugin::default()));

    app.add_systems(Startup, setup_environment);

    app.add_plugins(crate::scheme::plugin);
    app.add_plugins(crate::pan_orbit_camera::plugin);
    app.add_plugins(crate::post::plugin);

    app.add_plugins(crate::object::plugin);

    app.add_systems(
        PreUpdate,
        absorb_egui_inputs
            .after(bevy_egui::input::write_egui_input_system)
            .before(bevy_egui::begin_pass_system),
    );

    app.run();
}

fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,

    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_wheel: ResMut<Messages<bevy::input::mouse::MouseWheel>>,
    mut acc_wheel: ResMut<bevy::input::mouse::AccumulatedMouseScroll>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    if !(ctx.egui_wants_pointer_input() || ctx.is_pointer_over_egui()) {
        return;
    }
    let modifiers = [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ];

    let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));

    keyboard.reset_all();
    mouse.reset_all();
    mouse_wheel.clear();
    *acc_wheel = bevy::input::mouse::AccumulatedMouseScroll::default();

    for key in pressed.into_iter().flatten() {
        keyboard.press(key);
    }
}

fn setup_environment(mut commands: Commands) {
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    // directional 'sun' light
    commands.spawn((
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_y(PI / 4.0) * Quat::from_rotation_x(-PI / 4.0),
            ..default()
        },
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            minimum_distance: 1.0,
            maximum_distance: 100.0,
            first_cascade_far_bound: 10.0,
            overlap_proportion: 0.4,
            ..default()
        }
        .build(),
    ));

    // Camera

    let e = Exposure::from_physical_camera(PhysicalCameraParameters {
        aperture_f_stops: 1.0,
        shutter_speed_s: 1.0 / 125.0,
        sensitivity_iso: 100.0,
        sensor_height: 0.01866,
    });
    dbg!(e.ev100);

    commands.spawn((
        Camera3d::default(),
        bevy::camera::Hdr,
        Camera::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        self::pan_orbit_camera::PanOrbitSettings::default(),
        self::pan_orbit_camera::PanOrbitState {
            center: Vec3::new(1.0, 2.0, 3.0),
            radius: 50.0,
            pitch: -f32::to_radians(15.0),
            yaw: -f32::to_radians(30.0),
            upside_down: false,
        },
        Msaa::Sample4,
        Tonemapping::TonyMcMapface,
        //Smaa::default(),
        //ScreenSpaceAmbientOcclusion::default(),
        // Exposure::INDOOR,
        Exposure::OVERCAST,
        // Exposure::from_physical_camera(PhysicalCameraParameters {
        //     aperture_f_stops: 1.0,
        //     shutter_speed_s: 1.0 / 125.0,
        //     sensitivity_iso: 100.0,
        //     sensor_height: 0.01866,
        // }),

        // ambient light
        AmbientLight {
            color: bevy::color::palettes::css::ORANGE_RED.into(),
            // brightness: 0.02,
            brightness: 200.0,
            affects_lightmapped_meshes: true,
        },
    ));
}
