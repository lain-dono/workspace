use crate::state::InGame;
use bevy::{
    anti_alias::{contrast_adaptive_sharpening, fxaa, smaa},
    camera,
    core_pipeline::{prepass, tonemapping},
    input::mouse::MouseWheel,
    pbr,
    post_process::{auto_exposure, bloom},
    prelude::*,
    render::view,
};
use std::f32::consts::FRAC_2_PI;

pub fn plugin(app: &mut App) {
    app.init_resource::<InputConfig>()
        .add_systems(Startup, setup_camera_system)
        .add_systems(
            Update,
            (update_player_camera.before(crate::raycast::plane_raycast)).run_if(in_state(InGame)),
        );
}

#[derive(Component)]
pub struct CameraController;

fn setup_camera_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    compensation_curves: ResMut<Assets<auto_exposure::AutoExposureCompensationCurve>>,
) {
    let projection = PerspectiveProjection {
        fov: core::f32::consts::PI / 6.0,
        near: 0.1,
        far: 1000.0,
        aspect_ratio: 1.0,
    };

    let metering_mask = asset_server.load("basic_metering_mask.png");

    commands
        .spawn((
            Transform::from_xyz(-2.5, 0.0, 0.0).with_rotation(Quat::from_rotation_y(FRAC_2_PI)),
            Visibility::Inherited,
            CameraController,
        ))
        .with_child((
            crate::raycast::PlaneRaycast::Y,
            Camera3d::default(),
            Projection::from(projection),
            Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            Camera::default(),
            EnvironmentMapLight {
                diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
                intensity: 150.0,
                ..default()
            },
            camera_exposure(metering_mask, compensation_curves),
            (
                // view::Hdr,
                tonemapping::Tonemapping::TonyMcMapface,
                tonemapping::DebandDither::Enabled,
                prepass::DepthPrepass,
                // prepass::NormalPrepass,
                // prepass::MotionVectorPrepass,
                // prepass::DeferredPrepass,
            ),
            (
                // pbr::ScreenSpaceAmbientOcclusion::default(),
                view::ColorGrading::default(),
                /*
                bloom::Bloom {
                    // max_mip_dimension: 1024,
                    // scale: Vec2::new(4.0, 4.0),
                    // ..bevy::core_pipeline::bloom::Bloom::OLD_SCHOOL
                    // prefilter: bloom::BloomPrefilter {
                    //     threshold: 2.1, //0.6,
                    //     threshold_softness: 0.2,
                    // },
                    ..bloom::Bloom::NATURAL
                },
                */
            ),
            (
                Msaa::Sample4,
                contrast_adaptive_sharpening::ContrastAdaptiveSharpening {
                    sharpening_strength: 0.7,
                    enabled: true,
                    ..default()
                },
                /*
                fxaa::Fxaa::default(),
                smaa::Smaa {
                    preset: smaa::SmaaPreset::Ultra,
                },
                */
            ),
        ));

    // camera.insert(bevy::render::experimental::occlusion_culling::OcclusionCulling);
}

fn camera_exposure(
    metering_mask: Handle<Image>,
    mut compensation_curves: ResMut<Assets<auto_exposure::AutoExposureCompensationCurve>>,
) -> impl Bundle {
    let curve = bevy::math::cubic_splines::LinearSpline::new([
        Vec2::new(-4.0, -2.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(4.0, 2.0),
    ]);

    (
        camera::Exposure::OVERCAST,
        auto_exposure::AutoExposure {
            // range: -camera::Exposure::EV100_SUNLIGHT..=camera::Exposure::EV100_SUNLIGHT,
            range: -camera::Exposure::EV100_OVERCAST..=camera::Exposure::EV100_OVERCAST,

            metering_mask,
            compensation_curve: compensation_curves
                .add(auto_exposure::AutoExposureCompensationCurve::from_curve(curve).unwrap()),

            speed_brighten: 3.0 * 2.0,
            speed_darken: 1.0 * 2.0,

            ..default()
        },
    )
}

#[derive(Resource)]
pub struct InputConfig {
    pub movement_speed: f32,
    pub rotation_speed: f32,

    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,

    pub rotate_left: KeyCode,
    pub rotate_right: KeyCode,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            movement_speed: 20.0,
            rotation_speed: 3.0,

            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,

            rotate_left: KeyCode::KeyQ,
            rotate_right: KeyCode::KeyE,
        }
    }
}

type FilterPair<A, B> = (With<A>, Without<B>);

pub fn update_player_camera(
    input: Res<InputConfig>,
    mut controller: Query<(&mut Transform, &Children), FilterPair<CameraController, Camera>>,
    mut camera: Query<&mut Transform, FilterPair<Camera, CameraController>>,

    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    time: Res<Time<Real>>,
) -> Result {
    let wheel = mouse_wheel.read().fold(0.0, |acc, event| acc + event.y);

    let (mut controller_tx, children) = controller.single_mut()?;

    let Some(camera_entity) = children.first().copied() else {
        return Ok(());
    };
    if let Ok(mut transform) = camera.get_mut(camera_entity)
        && wheel != 0.0
    {
        let old_len = transform.translation.length();
        let new_len = (old_len - wheel * 2.0).clamp(20.0, 200.0);
        transform.translation = transform.translation.normalize() * new_len;
    }

    let mut diff = Vec3::new(0.0, 0.0, 0.0);

    if keyboard.pressed(input.move_left) {
        diff.x += 1.0;
    }
    if keyboard.pressed(input.move_right) {
        diff.x -= 1.0;
    }
    if keyboard.pressed(input.move_forward) {
        diff.z += 1.0;
    }
    if keyboard.pressed(input.move_backward) {
        diff.z -= 1.0;
    }

    let base_forward = controller_tx.forward();

    let forward = Vec3::new(base_forward.x, 0.0, base_forward.z);
    let right = Vec3::new(base_forward.z, 0.0, -base_forward.x);

    let movement = (right * diff.x + forward * diff.z).normalize_or_zero();

    controller_tx.translation += movement * time.delta_secs() * input.movement_speed;

    let mut angle = 0.0;
    if keyboard.pressed(input.rotate_right) {
        angle += 1.0;
    }
    if keyboard.pressed(input.rotate_left) {
        angle -= 1.0;
    }
    controller_tx.rotate_y(angle * time.delta_secs() * input.rotation_speed);

    Ok(())
}
