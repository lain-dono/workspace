use crate::{giz::PainterCamera, math::InputSelect};
// use ::gizmo::plugin::GizmoCamera;
use bevy::{input::mouse::MouseMotion, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_player)
        .add_systems(Update, move_player);
}

#[derive(Debug, Component)]
pub struct Player {
    forward: KeyCode,
    backward: KeyCode,
    left: KeyCode,
    right: KeyCode,

    move_speed: f32,
}

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player {
                forward: KeyCode::KeyW,
                backward: KeyCode::KeyS,
                right: KeyCode::KeyD,
                left: KeyCode::KeyA,

                move_speed: 10.0,
            },
            Visibility::Inherited,
            Transform::from_xyz(0., 1.5, 6.).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .with_children(|parent| {
            // parent.spawn(Camera2dBundle::default());
            parent.spawn((Camera3d::default(), PainterCamera))
                // .insert(GizmoCamera, )
            ;
        });
}

pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    btns: Res<ButtonInput<MouseButton>>,
    mut mouse: EventReader<MouseMotion>,
    mut player: Query<(&mut Transform, &Player)>,
    time: Res<Time>,
) {
    let (mut transform, player) = player.single_mut();

    if btns.pressed(MouseButton::Right) {
        for motion in mouse.read() {
            let yaw = -motion.delta.x * 0.003;
            let pitch = -motion.delta.y * 0.002;
            // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
            transform.rotate_y(yaw);
            transform.rotate_local_x(pitch);
        }
    }

    let forward = keys.map_pressed(0.0, 1.0, player.forward);
    let backward = keys.map_pressed(0.0, -1.0, player.backward);
    let right = keys.map_pressed(0.0, 1.0, player.right);
    let left = keys.map_pressed(0.0, -1.0, player.left);

    let a = transform.forward() * (forward + backward);
    let b = transform.right() * (right + left);

    transform.translation += player.move_speed * (a + b) * time.delta_secs();
}
