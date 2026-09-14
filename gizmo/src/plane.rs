use crate::{
    config::GizmoDirection,
    draw::Painter,
    gizmo::{GizmoHandle, GizmoResult, State},
    math::{round_to_interval, DMat4, DQuat, DVec3, Ray},
    prepared::{PickResult, Prepared},
};
use ecolor::Color32;

impl Prepared {
    fn plane(&self, direction: GizmoDirection) -> Plane {
        let stroke_width = self.stroke_width;
        let gizmo_size = self.gizmo_size;

        let size = gizmo_size * 0.1 + stroke_width * 2.0;

        let bitangent = direction.plane_bitangent();
        let tangent = direction.plane_tangent();
        let origin = (bitangent + tangent) * gizmo_size * 0.5;

        Plane {
            bitangent,
            tangent,
            origin,
            normal: self.normal(direction),
            size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub origin: DVec3,
    pub bitangent: DVec3,
    pub tangent: DVec3,
    pub normal: DVec3,
    pub size: f64,
}

impl Plane {
    const FADE: (f64, f64) = (0.70, 0.86);

    pub fn pick(
        &self,
        ray: Ray,
        translation: DVec3,
        rotation: DQuat,
        eye_to_model_dir: DVec3,
    ) -> PickResult {
        let normal = self.normal;

        let plane_origin = translation + rotation * self.origin;
        let (t, dist_from_origin) = ray.to_plane_origin(normal, plane_origin);

        let dot = eye_to_model_dir.dot(normal).abs();
        let visibility = (1.0 - dot - Self::FADE.0) / (Self::FADE.1 - Self::FADE.0);
        let visibility = (1.0 - visibility).min(1.0);

        PickResult {
            point: ray.origin + ray.direction * t,
            visibility: visibility as f32,
            picked: visibility > 0.0 && dist_from_origin <= self.size,
            t,
        }
    }

    fn point_on_plane(&self, ray: Ray, translation: DVec3, rotation: DQuat) -> Option<DVec3> {
        let plane_origin = translation + rotation * self.origin;
        ray.intersect_plane(self.normal, plane_origin)
            .map(|t| ray.origin + ray.direction * t)
    }

    pub fn points(&self) -> [DVec3; 4] {
        let half_size = self.size * 0.5;
        let a = self.bitangent * half_size;
        let b = self.tangent * half_size;

        [
            self.origin - b - a,
            self.origin + b - a,
            self.origin + b + a,
            self.origin - b + a,
        ]
    }

    pub fn snap_delta(&self, rotation: DQuat, new_delta: DVec3, snap: f64) -> DVec3 {
        let bitangent = rotation * self.bitangent;
        let tangent = rotation * self.tangent;

        let cb = new_delta.cross(-bitangent);
        let lb = cb.length();

        let ct = new_delta.cross(tangent);
        let lt = ct.length();

        if lb > 1e-5 && lt > 1e-5 {
            let b = round_to_interval(lt, snap) * (ct / lt).dot(self.normal);
            let t = round_to_interval(lb, snap) * (cb / lb).dot(self.normal);
            bitangent * b + tangent * t
        } else {
            new_delta
        }
    }

    pub fn paint(&self, painter: &mut Painter, transform: DMat4, fill: Color32) {
        painter.fill_polygon(transform, self.points(), fill);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MovePlane {
    direction: GizmoDirection,
    start_view_dir: DVec3,
    start_point: DVec3,
    last_point: DVec3,
    opacity: f32,
}

impl MovePlane {
    pub fn new(direction: GizmoDirection) -> Self {
        Self {
            direction,
            start_view_dir: DVec3::default(),
            start_point: DVec3::default(),
            last_point: DVec3::default(),
            opacity: 0.0,
        }
    }
}

impl GizmoHandle for MovePlane {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let pick = match self.direction {
            GizmoDirection::View => config
                .circle(config.inner_circle_radius(), true)
                .pick(ray, config.focus_distance as f64),
            _ => config.plane(self.direction).pick(
                ray,
                config.transform.translation,
                config.local_space_rotation(),
                config.eye_to_model_dir,
            ),
        };

        self.start_view_dir = config.view_forward();
        self.start_point = pick.point;
        self.last_point = pick.point;
        self.opacity = pick.visibility;

        pick.picked.then_some(pick.t)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        if config.view_forward() != self.start_view_dir {
            // If the view_forward direction has changed, i.e. camera has rotated,
            // refresh the subgizmo state by calling pick. Feels a bit hacky, but
            // fixes the issue where the target starts flying away if camera is rotated
            // while view plane translation is active.
            self.pick(config, ray);
        }

        let plane = config.plane(self.direction);
        let rotation = config.local_space_rotation();

        let mut new_point = plane.point_on_plane(ray, config.transform.translation, rotation)?;
        let mut new_delta = new_point - self.start_point;

        if config.snapping {
            let snap = config.snap_distance as f64;
            new_delta = plane.snap_delta(rotation, new_delta, snap);
            new_point = self.start_point + new_delta;
        }

        let mut delta = new_point - self.last_point;
        let mut total = new_point - self.start_point;
        self.last_point = new_point;

        if config.local_space() {
            let inv_rotation = rotation.inverse();
            delta = inv_rotation * delta;
            total = inv_rotation * total;
        }

        Some(GizmoResult::Translation {
            delta: delta.into(),
            total: total.into(),
        })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        match self.direction {
            GizmoDirection::View => {
                let transform = config.view_transform();
                let color = config.color(state.focused, self.direction, None);
                let stroke = (config.visuals.stroke_width, color);
                painter.circle(transform, config.inner_circle_radius(), stroke);
            }
            _ => {
                let transform = config.local_space_transform();
                let fill = config.color(state.focused, self.direction, self.opacity);
                config.plane(self.direction).paint(painter, transform, fill);
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ScalePlane {
    direction: GizmoDirection,
    start_delta: f64,
    opacity: f32,
}

impl ScalePlane {
    pub fn new(direction: GizmoDirection) -> Self {
        Self {
            direction,
            start_delta: 0.0,
            opacity: 0.0,
        }
    }
}

impl GizmoHandle for ScalePlane {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let pick = match self.direction {
            GizmoDirection::View => config
                .circle(config.outer_circle_radius(), false)
                .pick(ray, config.focus_distance as f64),
            _ => config.plane(self.direction).pick(
                ray,
                config.transform.translation,
                config.local_space_rotation(),
                config.eye_to_model_dir,
            ),
        };

        let delta = config.model_to_screen(DVec3::ZERO);
        self.start_delta = delta.map(|pos| ray.pos.distance(pos) as f64)?;
        self.opacity = pick.visibility;
        pick.picked.then_some(pick.t)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        let delta = config.model_to_screen(DVec3::ZERO);
        let delta = delta.map(|pos| ray.pos.distance(pos) as f64)?;

        let mut delta = delta / self.start_delta;

        if config.snapping {
            delta = round_to_interval(delta, config.snap_scale as f64);
        }
        delta = delta.max(1e-4) - 1.0;

        let direction = if self.direction.is_view() {
            DVec3::ONE
        } else {
            (self.direction.plane_bitangent() + self.direction.plane_tangent()).normalize()
        };
        let total = (DVec3::ONE + (direction * delta)).into();
        Some(GizmoResult::Scale { total })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        match self.direction {
            GizmoDirection::View => {
                let transform = config.view_transform();
                let color = config.color(state.focused, self.direction, None);
                let stroke = (config.visuals.stroke_width, color);
                painter.circle(transform, config.outer_circle_radius(), stroke);
            }
            _ => {
                let transform = config.local_space_transform();
                let fill = config.color(state.focused, self.direction, self.opacity);
                config.plane(self.direction).paint(painter, transform, fill);
            }
        }
    }
}
