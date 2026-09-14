use bevy::{prelude::*, render::view::Hdr};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, camera_control_system).add_systems(
        PostUpdate,
        crate::label::sync_labels.before(bevy::ui::ui_layout_system),
    );
}

fn camera_control_system(
    mut commands: Commands,
    camera: Single<(Entity, &mut Transform, Has<Hdr>), With<Camera3d>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let (entity, mut transform, has_hdr) = camera.into_inner();

    if input.just_pressed(KeyCode::KeyH) {
        if has_hdr {
            commands.entity(entity).remove::<Hdr>();
        } else {
            commands.entity(entity).insert(Hdr);
        }
    }

    let dt = time.delta_secs();

    let l = input.pressed(KeyCode::ArrowLeft);
    let r = input.pressed(KeyCode::ArrowRight);

    // let l = input.pressed(KeyCode::KeyA);
    // let r = input.pressed(KeyCode::KeyD);

    let angle_y = if r { dt } else { 0.0 } - if l { dt } else { 0.0 };
    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(angle_y));

    // let t = input.pressed(KeyCode::KeyW);
    // let b = input.pressed(KeyCode::KeyS);
    // let angle_x = if b { dt } else { 0.0 } - if t { dt } else { 0.0 };
    // camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(angle_x));
}
