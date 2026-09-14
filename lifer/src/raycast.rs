use crate::state::InGame;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, plane_raycast.run_if(in_state(InGame)));

    #[cfg(debug_assertions)]
    {
        app.add_systems(OnEnter(InGame), spawn_raycast_marker);
        app.add_systems(Update, update_raycast_marker.after(plane_raycast));
    }
}

#[derive(Component)]
pub struct PlaneRaycast {
    pub plane_origin: Vec3,
    pub plane_normal: Vec3,
    pub result: Option<Vec3>,
}

impl PlaneRaycast {
    pub const Y: Self = Self {
        plane_origin: Vec3::ZERO,
        plane_normal: Vec3::Y,
        result: None,
    };
}

pub fn plane_raycast(
    windows: Query<&Window>,
    mut camera: Query<(Entity, &mut PlaneRaycast, &Camera)>,
    transform: TransformHelper,
) -> Result {
    let window = windows.single()?;
    let (entity, mut raycast, camera) = camera.single_mut()?;
    let camera_transform = transform.compute_global_transform(entity)?;

    raycast.result = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(&camera_transform, cursor).ok())
        .and_then(|ray| {
            let plane = InfinitePlane3d {
                normal: Dir3::new(raycast.plane_normal).unwrap(),
            };
            ray.intersect_plane(raycast.plane_origin, plane)
                .map(|distance| ray.get_point(distance))
        });

    Ok(())
}

#[derive(Component)]
pub struct RaycastMarker;

fn spawn_raycast_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        StateScoped(InGame),
        RaycastMarker,
        NotShadowCaster,
        NotShadowReceiver,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.1)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::Srgba(bevy::color::palettes::basic::RED),
            unlit: true,
            ..default()
        })),
        Visibility::Hidden,
    ));
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
