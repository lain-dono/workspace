use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GridGizmosA;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GridGizmosB;

pub fn plugin(app: &mut App) {
    return;

    app.add_systems(Update, draw_grid)
        .insert_gizmo_config(
            GridGizmosA,
            GizmoConfig {
                enabled: true,
                line: GizmoLineConfig {
                    width: 2.,
                    perspective: true,
                    style: GizmoLineStyle::Solid,
                    joints: GizmoLineJoint::None,
                },
                depth_bias: -0.0002,
                render_layers: default(),
            },
        )
        .insert_gizmo_config(
            GridGizmosB,
            GizmoConfig {
                enabled: true,
                line: GizmoLineConfig {
                    width: 2.,
                    perspective: true,
                    style: GizmoLineStyle::Solid,
                    joints: GizmoLineJoint::None,
                },
                depth_bias: -0.0001,
                render_layers: default(),
            },
        );
}

fn draw_grid(mut gizmos1m: Gizmos<GridGizmosA>, mut gizmos20cm: Gizmos<GridGizmosB>) {
    let n = 80;

    gizmos1m.grid(
        Quat::from_rotation_x(FRAC_PI_2),
        UVec2::splat(n),
        Vec2::new(1.00, 1.00),
        LinearRgba::gray(0.75),
    );

    gizmos20cm.grid(
        Quat::from_rotation_x(FRAC_PI_2),
        UVec2::splat(n * 5),
        Vec2::new(0.20, 0.20),
        LinearRgba::gray(0.35),
    );
}
