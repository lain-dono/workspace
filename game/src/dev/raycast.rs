use crate::raycast::{PlaneRaycast, plane_raycast};
use bevy::{ecs::system::SystemParam, light, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_raycast_marker);
    app.add_systems(Update, update_raycast_marker.after(plane_raycast));
}

#[derive(Component)]
pub struct RaycastMarker;

fn spawn_raycast_marker(mut commands: Commands, mut builder: MarkerBuilder) {
    commands.spawn((
        RaycastMarker,
        Visibility::Hidden,
        builder.unlit_cube(0.25, bevy::color::palettes::basic::RED),
    ));
}

#[derive(SystemParam)]
struct MarkerBuilder<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

impl MarkerBuilder<'_> {
    fn unlit_cube(&mut self, length: f32, color: impl Into<Color>) -> impl Bundle {
        (
            light::NotShadowCaster,
            light::NotShadowReceiver,
            Mesh3d(self.meshes.add(Mesh::from(Cuboid::from_length(length)))),
            MeshMaterial3d(self.materials.add(StandardMaterial {
                base_color: color.into(),
                unlit: true,
                ..default()
            })),
        )
    }
}

fn update_raycast_marker(
    raycast: Query<&PlaneRaycast>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<RaycastMarker>>,
) {
    if let Ok((mut transform, mut visibility)) = marker.single_mut() {
        let point = raycast.single().ok().and_then(|raycast| raycast.result);
        *visibility = if let Some(point) = point {
            transform.translation = point;
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
