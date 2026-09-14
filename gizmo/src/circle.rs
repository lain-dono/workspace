use crate::{
    math::{DVec3, Ray},
    prepared::{PickResult, Prepared},
};

impl Prepared {
    pub fn circle(&self, radius: f64, filled: bool) -> Circle {
        Circle {
            origin: self.transform.translation,
            normal: -self.view_forward(),
            radius,
            filled,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub origin: DVec3,
    pub normal: DVec3,
    pub radius: f64,
    pub filled: bool,
}

impl Circle {
    pub fn pick(&self, ray: Ray, focus_distance: f64) -> PickResult {
        let (t, dist_from_gizmo_origin) = ray.to_plane_origin(self.normal, self.origin);

        let point = ray.origin + ray.direction * t;

        let picked = if self.filled {
            dist_from_gizmo_origin <= self.radius + focus_distance
        } else {
            (dist_from_gizmo_origin - self.radius).abs() <= focus_distance
        };

        PickResult {
            point,
            visibility: 1.0,
            picked,
            t,
        }
    }
}
