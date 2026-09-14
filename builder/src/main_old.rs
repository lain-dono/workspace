// use ::gizmo::plugin::{GizmoHotkeys, GizmoOptions, GizmoOrientation, TransformGizmoPlugin};
use bevy::{color::palettes::css::*, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin};
//use bevy_mod_outline::{OutlineBundle, OutlineVolume};
//use bevy_mod_picking::{prelude::PickSelection, PickableBundle};
// use gizmo::GizmoMode;
use std::f32::consts::PI;

//mod geo;
mod math;
mod player;
mod roof;

pub mod bmesh;
pub mod editor;

//pub mod giz;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .init_gizmo_group::<MyGizmos>()
        .add_systems(Startup, setup);
    // .add_systems(
    //     Update,
    //     (
    //         // draw_example_collection,
    //         // draw_example_collection_my,
    //         update_config,
    //     ),
    // );

    // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
    // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
    app.add_plugins(EguiPlugin);
    app.add_systems(Update, ui_example_system);

    app.add_plugins(self::player::plugin);
    app.add_plugins(self::roof::plugin);

    //app.add_plugins(self::giz::plugin);

    // if false {
    //     app.add_plugins(TransformGizmoPlugin)
    //         .add_plugins(::gizmo::picking::picking_plugin)
    //         .insert_resource(GizmoOptions {
    //             // hotkeys: Some(GizmoHotkeys::default()),
    //             hotkeys: None,
    //             // gizmo_modes: GizmoMode::all_rotate(),
    //             // gizmo_modes: GizmoMode::all_translate(),
    //             // gizmo_modes: GizmoMode::TranslateX.into(),
    //             //gizmo_modes: GizmoMode::all_scale(),
    //             gizmo_orientation: GizmoOrientation::Local,

    //             ..default()
    //         });
    // }

    app.run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyGizmos;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    // cube
    commands.spawn((
        Transform::from_xyz(0.0, 0.5, 0.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        PickableBundle {
            selection: PickSelection { is_selected: true },
            ..default()
        },
        OutlineBundle {
            outline: OutlineVolume {
                visible: false,
                colour: Color::WHITE,
                width: 2.0,
            },
            ..default()
        },
    ));
    // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            // first_cascade_far_bound: 4.0,
            maximum_distance: 250.0,
            ..default()
        }
        .into(),
        ..default()
    });

    //  if false {
    //      // example instructions
    //      commands.spawn(
    //          TextBundle::from_section(
    //              "Press 'D' to toggle drawing gizmos on top of everything else in the scene\n\
    //          Press 'P' to toggle perspective for line gizmos\n\
    //          Hold 'Left' or 'Right' to change the line width of straight gizmos\n\
    //          Hold 'Up' or 'Down' to change the line width of round gizmos\n\
    //          Press '1' or '2' to toggle the visibility of straight gizmos or round gizmos\n\
    //          Press 'B' to show all AABB boxes\n\
    //          Press 'U' or 'I' to cycle through line styles for straight or round gizmos\n\
    //          Press 'J' or 'K' to cycle through line joins for straight or round gizmos",
    //              TextStyle::default(),
    //          )
    //          .with_style(Node {
    //              position_type: PositionType::Absolute,
    //              top: Val::Px(12.0),
    //              left: Val::Px(12.0),
    //              ..default()
    //          }),
    //      );
    //  }
}

fn draw_example_collection(mut gizmos: Gizmos, time: Res<Time>) {
    /*
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(20),
        Vec2::new(2., 2.),
        // Light gray
        LinearRgba::gray(0.65),
    );

    gizmos.cuboid(
        Transform::from_translation(Vec3::Y * 0.5).with_scale(Vec3::splat(1.25)),
        BLACK,
    );
    gizmos.rect(
        Vec3::new(time.elapsed_secs().cos() * 2.5, 1., 0.),
        Quat::from_rotation_y(PI / 2.),
        Vec2::splat(2.),
        LIME,
    );

    for y in [0., 0.5, 1.] {
        gizmos.ray(
            Vec3::new(1., y, 0.),
            Vec3::new(-3., (time.elapsed_secs() * 3.).sin(), 0.),
            BLUE,
        );
    }

    gizmos.arrow(Vec3::ZERO, Vec3::ONE * 1.5, YELLOW);

    // You can create more complex arrows using the arrow builder.
    gizmos
        .arrow(Vec3::new(2., 0., 2.), Vec3::new(2., 2., 2.), ORANGE_RED)
        .with_double_end()
        .with_tip_length(0.5);

    */
}

fn draw_example_collection_my(mut gizmos: Gizmos<MyGizmos>) {
    /*
    gizmos.sphere(Vec3::new(1., 0.5, 0.), Quat::IDENTITY, 0.5, RED);

    gizmos
        .rounded_cuboid(
            Vec3::new(-2.0, 0.75, -0.75),
            Quat::IDENTITY,
            Vec3::splat(0.9),
            TURQUOISE,
        )
        .edge_radius(0.1)
        .arc_resolution(4);

    gizmos
        .arc_3d(
            180.0_f32.to_radians(),
            0.2,
            Vec3::ONE,
            Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize()),
            ORANGE,
        )
        .resolution(10);

    // Circles have 32 line-segments by default.
    gizmos.circle(Vec3::ZERO, Dir3::Y, 3., BLACK);
    // You may want to increase this for larger circles or spheres.
    gizmos.circle(Vec3::ZERO, Dir3::Y, 3.1, NAVY).resolution(64);
    gizmos
        .sphere(Vec3::ZERO, Quat::IDENTITY, 3.2, BLACK)
        .resolution(64);

    */
}

fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        for (_, config, _) in config_store.iter_mut() {
            config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
        }
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        for (_, config, _) in config_store.iter_mut() {
            // Toggle line_perspective
            config.line_perspective ^= true;
            // Increase the line width when line_perspective is on
            config.line_width *= if config.line_perspective { 5. } else { 1. / 5. };
        }
    }

    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    //config.line_perspective = true;

    if keyboard.pressed(KeyCode::ArrowRight) {
        config.line_width += 5. * time.delta_secs();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        config.line_width -= 5. * time.delta_secs();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        config.enabled ^= true;
    }
    if keyboard.just_pressed(KeyCode::KeyU) {
        config.line_style = match config.line_style {
            GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
            _ => GizmoLineStyle::Solid,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyJ) {
        config.line_joints = match config.line_joints {
            GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
            GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
            GizmoLineJoint::Round(_) => GizmoLineJoint::None,
            GizmoLineJoint::None => GizmoLineJoint::Bevel,
        };
    }

    let (my_config, _) = config_store.config_mut::<MyGizmos>();
    if keyboard.pressed(KeyCode::ArrowUp) {
        my_config.line_width += 5. * time.delta_secs();
        my_config.line_width = my_config.line_width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        my_config.line_width -= 5. * time.delta_secs();
        my_config.line_width = my_config.line_width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        my_config.enabled ^= true;
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        my_config.line_style = match my_config.line_style {
            GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
            _ => GizmoLineStyle::Solid,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        my_config.line_joints = match my_config.line_joints {
            GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
            GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
            GizmoLineJoint::Round(_) => GizmoLineJoint::None,
            GizmoLineJoint::None => GizmoLineJoint::Bevel,
        };
    }

    if keyboard.just_pressed(KeyCode::KeyB) {
        // AABB gizmos are normally only drawn on entities with a ShowAabbGizmo component
        // We can change this behaviour in the configuration of AabbGizmoGroup
        config_store.config_mut::<AabbGizmoConfigGroup>().1.draw_all ^= true;
    }
}
