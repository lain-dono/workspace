use bevy::prelude::*;

pub trait ExtVec2 {
    fn extend_y(self, y: f32) -> Vec3;
}

impl ExtVec2 for Vec2 {
    #[inline(always)]
    fn extend_y(self, y: f32) -> Vec3 {
        self.extend(y).xzy()
    }
}

#[inline]
pub fn distance_to_segment([a, b]: [egui::Pos2; 2], p: egui::Pos2) -> f32 {
    let len2 = a.distance_sq(b);
    if len2 <= 0.0 {
        p.distance(a)
    } else {
        let t = ((p - a).dot(b - a) / len2).clamp(0.0, 1.0);
        p.distance(a + t * (b - a))
    }
}

#[inline]
pub fn clamp_projection([a, b]: [Vec2; 2], p: Vec2) -> Vec2 {
    let pa = (p - a).length_squared();
    let pb = (p - b).length_squared();
    let ab = (a - b).length_squared();
    if pa <= ab && pb <= ab {
        p
    } else if pa <= pb {
        a
    } else {
        b
    }
}
