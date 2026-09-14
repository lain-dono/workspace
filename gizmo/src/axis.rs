use crate::{
    config::{GizmoDirection, GizmoMode},
    draw::Painter,
    gizmo::{GizmoHandle, GizmoResult, State},
    math::{round_to_interval, segment_to_segment, DMat4, DVec3, Ray},
    prepared::{PickResult, Prepared},
};
use ecolor::Color32;
use enumset::EnumSet;

impl GizmoDirection {
    fn to_scale(self) -> GizmoMode {
        match self {
            Self::X => GizmoMode::ScaleX,
            Self::Y => GizmoMode::ScaleY,
            Self::Z => GizmoMode::ScaleZ,
            _ => unreachable!(),
        }
    }
    fn to_translate(self) -> GizmoMode {
        match self {
            Self::X => GizmoMode::TranslateX,
            Self::Y => GizmoMode::TranslateY,
            Self::Z => GizmoMode::TranslateZ,
            _ => unreachable!(),
        }
    }
}

impl GizmoMode {
    fn arrow_modes_overlapping(self, other: EnumSet<Self>) -> bool {
        (self == Self::TranslateX && other.contains(Self::ScaleX))
            || (self == Self::TranslateY && other.contains(Self::ScaleY))
            || (self == Self::TranslateZ && other.contains(Self::ScaleZ))
            || (self == Self::ScaleX && other.contains(Self::TranslateX))
            || (self == Self::ScaleY && other.contains(Self::TranslateY))
            || (self == Self::ScaleZ && other.contains(Self::TranslateZ))
    }
}

impl Prepared {
    fn axis(&self, direction: DVec3, mode: GizmoMode, cap: AxisCap) -> Axis {
        let stroke_width = self.stroke_width;
        let gizmo_size = self.gizmo_size;

        let many_modes = self.modes.len() > 1;
        let overlap_translate = mode.is_translate() && mode.arrow_modes_overlapping(self.modes);

        // Modes contain both translate and scale. Use a bit different translate arrow, so the modes do not overlap.
        let (start, length) = if overlap_translate {
            let start = direction * (gizmo_size + (stroke_width * 3.0));
            let length = gizmo_size * 0.2 + stroke_width;
            (start, length)
        } else {
            let fix = stroke_width * 2.0 * many_modes as usize as f64;
            let start = direction * (stroke_width * 0.5 + self.inner_circle_radius());
            let length = gizmo_size - start.length() - fix;
            (start, length)
        };

        Axis::new(start, direction, length, cap)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisCap {
    Arrow,
    Cube,
}

#[derive(Debug, Clone, Copy)]
pub struct Axis {
    pub start: DVec3,
    pub end: DVec3,
    pub direction: DVec3,
    pub length: f64,
    pub cap: AxisCap,
}

impl Axis {
    const FADE: (f64, f64) = (0.95, 0.99);

    pub fn new(start: DVec3, direction: DVec3, length: f64, cap: AxisCap) -> Self {
        Self {
            start,
            end: start + direction * length,
            direction,
            length,
            cap,
        }
    }

    pub fn pick(
        self,
        translation: DVec3,
        ray: Ray,
        eye_to_model_dir: DVec3,
        focus_distance: f64,
    ) -> PickResult {
        let start = self.start + translation;
        let end = self.end + translation;

        let ray_length = 1e+14;

        let (t, sub_t) = segment_to_segment(
            ray.origin,
            ray.origin + ray.direction * ray_length,
            start,
            end,
        );

        let ray_point = ray.origin + ray.direction * ray_length * t;
        let point = start + self.direction * self.length * sub_t;
        let dist = (ray_point - point).length();
        let dot = eye_to_model_dir.dot(self.direction).abs();
        let visibility = (dot - Self::FADE.0) / (Self::FADE.1 - Self::FADE.0);
        let visibility = (1.0 - visibility).min(1.0);

        PickResult {
            point,
            visibility: visibility as f32,
            picked: visibility > 0.0 && dist <= focus_distance,
            t,
        }
    }

    pub fn paint(
        &self,
        painter: &mut Painter,
        transform: DMat4,
        stroke_width: f32,
        scale_factor: f32,
        color: Color32,
    ) {
        let tip_stroke_width = 2.4 * stroke_width;
        let tip_length = (tip_stroke_width * scale_factor) as f64;
        let tip_start = self.end - self.direction * tip_length;

        let stroke = (stroke_width, color);
        painter.line(transform, self.start, tip_start, stroke);

        let stroke = (tip_stroke_width, color);
        match self.cap {
            AxisCap::Arrow => painter.arrow(transform, tip_start, self.end, stroke),
            AxisCap::Cube => painter.line(transform, tip_start, self.end, stroke),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MoveAxis {
    direction: GizmoDirection,
    start_view_dir: DVec3,
    start_point: DVec3,
    last_point: DVec3,
    opacity: f32,
}

impl MoveAxis {
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

impl GizmoHandle for MoveAxis {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let direction = config.normal(self.direction);
        let mode = self.direction.to_translate();
        let pick = config.axis(direction, mode, AxisCap::Arrow).pick(
            config.transform.translation,
            ray,
            config.eye_to_model_dir,
            config.focus_distance as f64,
        );

        self.start_view_dir = config.view_forward();
        self.start_point = pick.point;
        self.last_point = pick.point;
        self.opacity = pick.visibility;

        pick.picked.then_some(pick.t)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        if config.view_forward() != self.start_view_dir {
            // If the view_forward direction has changed, i.e. camera has rotated,
            // refresh the handle state by calling pick. Feels a bit hacky, but
            // fixes the issue where the target starts flying away if camera is rotated
            // while view plane translation is active.
            self.pick(config, ray);
        }

        // Finds the nearest point on line that points in translation handle direction
        let mut new_point = {
            let origin = config.transform.translation;
            let direction = config.normal(self.direction);
            let (_ray_t, handle_t) = ray.to_ray(origin, direction);
            origin + direction * handle_t
        };
        let mut new_delta = new_point - self.start_point;

        if config.snapping {
            let snap = config.snap_distance as f64;
            let len = new_delta.length();
            new_delta = if len > 1e-5 {
                new_delta / len * round_to_interval(len, snap)
            } else {
                new_delta
            };

            new_point = self.start_point + new_delta;
        }

        let mut delta = new_point - self.last_point;
        let mut total = new_point - self.start_point;
        self.last_point = new_point;

        if config.local_space() {
            let inverse_rotation = config.transform.rotation.inverse();
            delta = inverse_rotation * delta;
            total = inverse_rotation * total;
        }

        Some(GizmoResult::Translation {
            delta: delta.into(),
            total: total.into(),
        })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        let transform = config.local_space_transform();
        let direction = self.direction.local_normal(-config.view_forward());
        let mode = self.direction.to_translate();
        config.axis(direction, mode, AxisCap::Arrow).paint(
            painter,
            transform,
            config.visuals.stroke_width,
            config.scale_factor,
            config.color(state.focused, self.direction, self.opacity),
        );
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ScaleAxis {
    direction: GizmoDirection,
    start_delta: f64,
    opacity: f32,
}

impl ScaleAxis {
    pub fn new(direction: GizmoDirection) -> Self {
        Self {
            direction,
            start_delta: 0.0,
            opacity: 0.0,
        }
    }
}

impl GizmoHandle for ScaleAxis {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64> {
        let direction = config.normal(self.direction);
        let mode = self.direction.to_scale();
        let pick = config.axis(direction, mode, AxisCap::Cube).pick(
            config.transform.translation,
            ray,
            config.eye_to_model_dir,
            config.focus_distance as f64,
        );

        self.opacity = pick.visibility;
        self.start_delta = config
            .model_to_screen(DVec3::ZERO)
            .map(|pos| ray.pos.distance(pos) as f64)?;

        pick.picked.then_some(pick.t)
    }

    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult> {
        let delta = config
            .model_to_screen(DVec3::ZERO)
            .map(|pos| ray.pos.distance(pos) as f64)?;

        let mut delta = delta / self.start_delta;
        if config.snapping {
            delta = round_to_interval(delta, config.snap_scale as f64);
        }
        delta = delta.max(1e-4) - 1.0;

        let direction = self.direction.local_normal(-config.view_forward());
        let total = (DVec3::ONE + (direction * delta)).into();
        Some(GizmoResult::Scale { total })
    }

    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter) {
        let transform = config.local_space_transform();
        let direction = self.direction.local_normal(-config.view_forward());
        let mode = self.direction.to_scale();

        config.axis(direction, mode, AxisCap::Cube).paint(
            painter,
            transform,
            config.visuals.stroke_width,
            config.scale_factor,
            config.color(state.focused, self.direction, self.opacity),
        );
    }
}
