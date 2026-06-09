use crate::state::InGame;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, plane_raycast.run_if(in_state(InGame)));
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
