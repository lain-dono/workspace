use crate::{
    config::GizmoDirection,
    draw::Painter,
    gizmo::{GizmoHandle, GizmoResult, State},
    math::{
        rotation_align, round_to_interval, screen_to_world, DMat4, DQuat, DVec2, DVec3, Pos2, Ray,
    },
    prepared::Prepared,
};
use ecolor::Color32;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

#[derive(Debug, Copy, Clone)]
pub struct Rotate {
    direction: GizmoDirection,

    start_axis_angle: f64,
    start_rotation_angle: f64,
    last_rotation_angle: f64,
    current_delta: f64,
}

impl Rotate {
    pub fn new(direction: GizmoDirection) -> Self {
        Self {
            direction,

            start_axis_angle: 0.0,
            start_rotation_angle: 0.0,
            last_rotation_angle: 0.0,
            current_delta: 0.0,
        }
    }
}

impl GizmoHandle for Rotate {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let radius = config.arc_radius(self.direction);
        let origin = config.transform.translation;
        let normal = config.normal(self.direction);
        let tangent = config.tangent(self.direction);

        let (t, dist_from_gizmo_origin) = ray.to_plane_origin(normal, origin);
        let dist_from_gizmo_edge = (dist_from_gizmo_origin - radius).abs();

        let hit_pos = ray.origin + ray.direction * t;
        let dir_to_origin = (origin - hit_pos).normalize();
        let nearest_circle_pos = hit_pos + dir_to_origin * (dist_from_gizmo_origin - radius);

        let offset = (nearest_circle_pos - origin).normalize();

        let (y, x) = if self.direction.is_view() {
            (tangent.cross(normal).dot(offset), tangent.dot(offset))
        } else {
            let forward = config.view_forward_handled();
            (offset.cross(forward).dot(normal), offset.dot(forward))
        };
        let angle = DVec2::new(x, y).to_angle();

        let cursor_pos = ray.pos;
        let rotation_angle = config
            .rotation_angle(self.direction, cursor_pos)
            .unwrap_or(0.0);
        self.start_axis_angle = angle;
        self.start_rotation_angle = rotation_angle;
        self.last_rotation_angle = rotation_angle;
        self.current_delta = 0.0;

        let dist = dist_from_gizmo_edge <= config.focus_distance as f64;
        (dist && angle.abs() < config.arc_angle(self.direction)).then_some(t)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        let rotation_angle = config.rotation_angle(self.direction, ray.pos)?;
        let rotation_angle = if config.snapping {
            let value = rotation_angle - self.start_rotation_angle;
            round_to_interval(value, config.snap_angle as f64) + self.start_rotation_angle
        } else {
            rotation_angle
        };

        let mut angle_delta = rotation_angle - self.last_rotation_angle;

        // Always take the smallest angle, e.g. -10° instead of 350°
        if angle_delta > PI {
            angle_delta -= TAU;
        } else if angle_delta < -PI {
            angle_delta += TAU;
        }

        self.last_rotation_angle = rotation_angle;
        self.current_delta += angle_delta;

        let normal = self.direction.local_normal(-config.view_forward());

        Some(GizmoResult::Rotation {
            axis: normal.into(),
            delta: -angle_delta,
            total: self.current_delta,
            is_view_axis: self.direction == GizmoDirection::View,
        })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        let transform = config.rotation_matrix(self.direction);

        let color = config.color(state.focused, self.direction, None);
        let stroke = (config.visuals.stroke_width, color);

        let radius = config.arc_radius(self.direction);

        if !state.active {
            let angle = config.arc_angle(self.direction);
            let (start, end) = (FRAC_PI_2 - angle, FRAC_PI_2 + angle);
            painter.arc(transform, radius, start, end, stroke);
            return;
        }

        let mut start_1 = self.start_axis_angle + FRAC_PI_2;
        let mut end_1 = start_1 + self.current_delta;

        if start_1 > end_1 {
            // First make it so that end angle is always greater than start angle
            (start_1, end_1) = (end_1, start_1);
        }

        // The polyline does not get rendered correctly if
        // the start and end lines are exactly the same
        end_1 += 1e-5;

        let total_angle = end_1 - start_1;

        let full_circles = (total_angle / TAU).abs() as u32;

        end_1 -= TAU * full_circles as f64;

        let mut start_2 = end_1;
        let mut end_2 = start_1 + TAU;

        if config.view_forward().dot(config.normal(self.direction)) < 0.0 {
            // Swap start and end angles based on the view direction relative to gizmo normal.
            // Otherwise the filled sector gets drawn incorrectly.
            (start_1, end_1) = (end_1, start_1);
            (start_2, end_2) = (end_2, start_2);
        }

        let points = [
            DVec3::new(start_1.cos() * radius, 0.0, start_1.sin() * radius),
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(end_1.cos() * radius, 0.0, end_1.sin() * radius),
        ];
        painter.polyline(transform, points, stroke);

        if full_circles > 0 {
            let fill = color.linear_multiply((0.25 * full_circles as f32).min(1.0));
            painter.fill_sector(transform, radius, start_2, end_2, fill);
        }

        {
            let fill = color.linear_multiply((0.25 * (full_circles + 1) as f32).min(1.0));
            painter.fill_sector(transform, radius, start_1, end_1, fill);
        }

        painter.circle(transform, radius, stroke);

        // Draw snapping ticks
        if config.snapping {
            let stroke_width = stroke.0 / 2.0;
            for i in 0..((TAU / config.snap_angle as f64) as usize + 1) {
                let angle = i as f64 * config.snap_angle as f64 + end_1;
                let pos = DVec3::new(angle.cos(), 0.0, angle.sin());
                let stroke = (stroke_width, stroke.1);

                let from = pos * radius * 1.1;
                let to = pos * radius * 1.2;
                painter.line(transform, from, to, stroke);
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Arcball {
    last_pos: Pos2,
    total_rotation: DQuat,
}

impl Arcball {
    pub fn new() -> Self {
        Self {
            last_pos: Pos2::default(),
            total_rotation: DQuat::default(),
        }
    }
}

impl GizmoHandle for Arcball {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let pick = config
            .circle(config.arcball_radius(), true)
            .pick(ray, config.focus_distance as f64);
        self.last_pos = ray.pos;
        pick.picked.then_some(f64::MAX)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        let dir = ray.pos - self.last_pos;

        let rotation_delta = if dir.length_sq() > f32::EPSILON {
            let mat = config.view_proj.inverse();
            let a = screen_to_world(config.viewport_rect, mat, ray.pos, 0.0);
            let b = screen_to_world(config.viewport_rect, mat, self.last_pos, 0.0);

            let origin = config.view_forward();
            let a = (a - origin).normalize();
            let b = (b - origin).normalize();

            DQuat::from_axis_angle(a.cross(b).normalize(), a.dot(b).acos() * 10.0)
        } else {
            DQuat::IDENTITY
        };

        self.last_pos = ray.pos;
        self.total_rotation = rotation_delta * self.total_rotation;

        Some(GizmoResult::Arcball {
            delta: rotation_delta.into(),
            total: self.total_rotation.into(),
        })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        let color = Color32::WHITE.gamma_multiply(if state.focused { 0.10 } else { 0.0 });
        let rotation = config.view_rotation();
        let transform = DMat4::from_rotation_translation(rotation, config.transform.translation);
        painter.filled_circle(transform, color, config.arcball_radius())
    }
}

impl Prepared {
    /// Calculates angle of the rotation axis arc.
    /// The arc is a semicircle, which turns into a full circle when viewed
    /// directly from the front.
    fn arc_angle(&self, direction: GizmoDirection) -> f64 {
        let dot = self.normal(direction).dot(self.view_forward()).abs();
        let min_dot = 0.990;
        let max_dot = 0.995;
        let diff = max_dot - min_dot;

        let angle = ((dot - min_dot) / diff).clamp(0.0, 1.0) * FRAC_PI_2 + FRAC_PI_2;
        if (angle - PI).abs() < 1e-2 {
            PI
        } else {
            angle
        }
    }

    /// Calculates a matrix used when rendering the rotation axis.
    fn rotation_matrix(&self, direction: GizmoDirection) -> DMat4 {
        if direction.is_view() {
            self.view_transform()
        } else {
            let local_tangent = match direction {
                GizmoDirection::X => DVec3::Z,
                GizmoDirection::Y => DVec3::Z,
                GizmoDirection::Z => -DVec3::Y,
                GizmoDirection::View => unreachable!(),
            };
            let local_normal = direction.local_normal(-self.view_forward());

            let (tangent, normal) = if self.local_space() {
                let rotation = self.transform.rotation;
                (rotation * local_tangent, rotation * local_normal)
            } else {
                (local_tangent, local_normal)
            };

            // First rotate towards the gizmo normal
            let rotation = DQuat::from_mat3(&rotation_align(DVec3::Y, local_normal));
            let rotation = if self.local_space() {
                self.transform.rotation * rotation
            } else {
                rotation
            };

            // Rotate towards the camera, along the rotation axis.
            let rotation = {
                let forward = self.view_forward_handled();
                let y = tangent.cross(forward).dot(normal);
                let x = tangent.dot(forward);
                let angle = DVec2::new(x, y).to_angle();
                DQuat::from_axis_angle(normal, angle) * rotation
            };

            DMat4::from_rotation_translation(rotation, self.transform.translation)
        }
    }

    fn rotation_angle(&self, direction: GizmoDirection, cursor_pos: Pos2) -> Option<f64> {
        let gizmo_pos = self.model_to_screen(DVec3::ZERO)?;
        let x = cursor_pos.x as f64 - gizmo_pos.x as f64;
        let y = cursor_pos.y as f64 - gizmo_pos.y as f64;
        let delta = DVec2::new(x, y).try_normalize()?;
        let dot = self.view_forward().dot(self.normal(direction));
        Some(delta.to_angle() * dot.signum())
    }
}
