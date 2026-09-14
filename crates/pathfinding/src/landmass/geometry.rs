use bevy::math::Vec3;

pub(crate) fn edge_intersection(
    edge1: [Vec3; 2],
    edge2: [Vec3; 2],
    intersection_distance_squared: f32,
) -> Option<[Vec3; 2]> {
    let dir1 = edge1[1] - edge1[0];
    let t21 = [edge2[0] - edge1[0], edge2[1] - edge1[0]]
        .map(|v| dir1.dot(v) / dir1.length_squared())
        .map(|v| v.clamp(0.0, 1.0));

    if t21[0] <= t21[1] {
        return None;
    }

    let dir2 = edge2[1] - edge2[0];
    let t12 = [edge1[0] - edge2[0], edge1[1] - edge2[0]]
        .map(|v| dir2.dot(v) / dir2.length_squared())
        .map(|v| v.clamp(0.0, 1.0));
    if t12[0] <= t12[1] {
        return None;
    }

    let pts21 = t21.map(|v| v * dir1 + edge1[0]);
    let pts12 = t12.map(|v| v * dir2 + edge2[0]);

    if pts21[1].distance_squared(pts12[0]) > intersection_distance_squared
        || pts21[0].distance_squared(pts12[1]) > intersection_distance_squared
    {
        None
    } else {
        Some([(pts21[1] + pts12[0]) * 0.5, (pts21[0] + pts12[1]) * 0.5])
    }
}
