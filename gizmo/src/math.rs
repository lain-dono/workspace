pub use emath::{Pos2, Rect, Vec2};
pub use glam::{DMat3, DMat4, DQuat, DVec2, DVec3, DVec4, Mat4, Quat, Vec3, Vec4Swizzles};

/// Creates a matrix that represents rotation between two 3d vectors
///
/// Credit: <https://www.iquilezles.org/www/articles/noacos/noacos.htm>
pub fn rotation_align(from: DVec3, to: DVec3) -> DMat3 {
    let v = from.cross(to);
    let c = from.dot(to);
    let k = 1.0 / (1.0 + c);

    DMat3::from_cols_array(&[
        v.x * v.x * k + c,
        v.x * v.y * k + v.z,
        v.x * v.z * k - v.y,
        v.y * v.x * k - v.z,
        v.y * v.y * k + c,
        v.y * v.z * k + v.x,
        v.z * v.x * k + v.y,
        v.z * v.y * k - v.x,
        v.z * v.z * k + c,
    ])
}

/// Finds points on two segments that are closest to each other.
/// This can be used to determine the shortest distance between those two segments.
///
/// Credit: Practical Geometry Algorithms by Daniel Sunday: <http://geomalgorithms.com/code.html>
pub fn segment_to_segment(a1: DVec3, a2: DVec3, b1: DVec3, b2: DVec3) -> (f64, f64) {
    let delta_1 = a1 - b1;
    let a = a2 - a1;
    let b = b2 - b1;

    let (a_len, a_d1) = (a.length_squared(), a.dot(delta_1));
    let (b_len, b_d1) = (b.length_squared(), b.dot(delta_1));

    let a_dot_b = a.dot(b);
    let n = a_len * b_len - a_dot_b * a_dot_b;

    let mut sn;
    let mut tn;
    let mut sd = n;
    let mut td = n;

    if n < 1e-8 {
        sn = 0.0;
        sd = 1.0;
        tn = b_d1;
        td = b_len;
    } else {
        sn = a_dot_b * b_d1 - b_len * a_d1;
        tn = a_len * b_d1 - a_dot_b * a_d1;
        if sn < 0.0 {
            sn = 0.0;
            tn = b_d1;
            td = b_len;
        } else if sn > sd {
            sn = sd;
            tn = b_d1 + a_dot_b;
            td = b_len;
        }
    }

    if tn < 0.0 {
        tn = 0.0;
        if -a_d1 < 0.0 {
            sn = 0.0;
        } else if -a_d1 > a_len {
            sn = sd;
        } else {
            sn = -a_d1;
            sd = a_len;
        }
    } else if tn > td {
        tn = td;
        if (-a_d1 + a_dot_b) < 0.0 {
            sn = 0.0;
        } else if (-a_d1 + a_dot_b) > a_len {
            sn = sd;
        } else {
            sn = -a_d1 + a_dot_b;
            sd = a_len;
        }
    }

    let ta = if sn.abs() < 1e-8 { 0.0 } else { sn / sd };
    let tb = if tn.abs() < 1e-8 { 0.0 } else { tn / td };

    (ta, tb)
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub pos: Pos2,
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray {
    /// Calculate a world space ray from given screen space position
    pub fn from_pointer(view_projection: DMat4, viewport_rect: Rect, pos: Pos2) -> Self {
        let mat = view_projection.inverse();
        // let origin = screen_to_world(viewport_rect, mat, pos, -1.0);
        let origin = screen_to_world(viewport_rect, mat, pos, f64::EPSILON);
        let target = screen_to_world(viewport_rect, mat, pos, 1.0);

        let direction = (target - origin).normalize();

        Self {
            pos,
            origin,
            direction,
        }
    }
}

impl Ray {
    /// Finds the intersection point of a ray and a plane
    /// and distance from the intersection to the plane origin
    pub fn to_plane_origin(&self, disc_normal: DVec3, disc_origin: DVec3) -> (f64, f64) {
        if let Some(t) = self.intersect_plane(disc_normal, disc_origin) {
            (t, (self.origin + self.direction * t - disc_origin).length())
        } else {
            (0.0, f64::MAX)
        }
    }

    /// Finds the intersection point of a ray and a plane
    pub fn intersect_plane(&self, plane_normal: DVec3, plane_origin: DVec3) -> Option<f64> {
        let denom = plane_normal.dot(self.direction);
        (denom.abs() >= 10e-8)
            .then(|| (plane_origin - self.origin).dot(plane_normal) / denom)
            .filter(|&t| t >= 0.0)
    }

    pub fn to_ray(&self, b1: DVec3, bdir: DVec3) -> (f64, f64) {
        Self::ray_to_ray(self.origin, self.direction, b1, bdir)
    }

    /// Finds points on two rays that are closest to each other.
    /// This can be used to determine the shortest distance between those two rays.
    ///
    /// Credit: Practical Geometry Algorithms by Daniel Sunday: <http://geomalgorithms.com/code.html>
    pub fn ray_to_ray(a1: DVec3, adir: DVec3, b1: DVec3, bdir: DVec3) -> (f64, f64) {
        let b = adir.dot(bdir);
        let w = a1 - b1;
        let d = adir.dot(w);
        let e = bdir.dot(w);
        let dot = 1.0 - b * b;
        let ta;
        let tb;

        if dot < 1e-8 {
            ta = 0.0;
            tb = e;
        } else {
            ta = (b * e - d) / dot;
            tb = (e - b * d) / dot;
        }

        (ta, tb)
    }
}

/// Rounds given value to the nearest interval
pub fn round_to_interval(value: f64, interval: f64) -> f64 {
    (value / interval).round() * interval
}

/// Calculates 2d screen coordinates from 3d world coordinates
pub fn world_to_screen(viewport: Rect, mvp: DMat4, pos: DVec3) -> Option<Pos2> {
    let p = mvp * DVec4::from((pos, 1.0));
    (p.w >= 1e-10).then(|| {
        let (x, y) = (p.x / p.w, -p.y / p.w);
        viewport.center() + Vec2::new(x as f32, y as f32) * viewport.size() / 2.0
    })
}

/// Calculates 3d world coordinates from 2d screen coordinates
pub fn screen_to_world(viewport: Rect, mat: DMat4, pos: Pos2, z: f64) -> DVec3 {
    let pos = ((pos - viewport.min) / viewport.size()) * 2.0 - Vec2::new(1.0, 1.0);
    let world_pos = mat * DVec4::new(pos.x as f64, -pos.y as f64, z, 1.0);

    // w is zero when far plane is set to infinity
    if world_pos.w.abs() < 1e-7 {
        (world_pos / 1e-7).xyz()
    } else {
        (world_pos / world_pos.w).xyz()
    }
}
