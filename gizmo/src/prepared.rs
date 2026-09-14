use crate::{
    config::{GizmoConfig, GizmoDirection},
    math::{screen_to_world, world_to_screen, DMat3, DMat4, DQuat, DVec3, Pos2},
    GizmoMode, GizmoOrientation,
};
use ecolor::Color32;
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct PickResult {
    pub point: DVec3,
    pub visibility: f32,
    pub picked: bool,
    pub t: f64,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GizmoTransform {
    /// Rotation of the gizmo
    pub rotation: DQuat,
    /// Translation of the gizmo
    pub translation: DVec3,
    /// Scale of the gizmo
    pub scale: DVec3,
}

impl GizmoTransform {
    pub fn as_mint(&self) -> crate::Transform {
        crate::Transform {
            scale: self.scale.into(),
            rotation: self.rotation.into(),
            translation: self.translation.into(),
        }
    }

    pub fn as_dmat4(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Prepared {
    config: GizmoConfig,

    pub transform: GizmoTransform,

    view: DMat4,
    proj: DMat4,

    /// Combined view-projection matrix
    pub view_proj: DMat4,

    /// Combined model-view-projection matrix
    mvp: DMat4,

    /// Scale factor for the gizmo rendering
    pub scale_factor: f32,
    /// How close the mouse pointer needs to be to a handle before it is focused
    pub focus_distance: f32,
    /// Direction from the camera to the gizmo in world space
    pub eye_to_model_dir: DVec3,

    pub gizmo_size: f64,
    pub stroke_width: f64,
}

impl Deref for Prepared {
    type Target = GizmoConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl Prepared {
    /// Whether local orientation is used
    pub fn local_space(&self) -> bool {
        // Scaling currently only works in local orientation,
        // so the configured orientation is ignored.
        let is_scaling = (self.config.mode_override.is_none()
            && !self.config.modes.is_disjoint(GizmoMode::all_scale()))
            || self
                .config
                .mode_override
                .filter(GizmoMode::is_scale)
                .is_some();

        is_scaling || matches!(self.config.orientation, GizmoOrientation::Local)
    }

    /// Forward vector of the view camera
    pub fn view_forward(&self) -> DVec3 {
        DVec3::new(self.view.x_axis.z, self.view.y_axis.z, self.view.z_axis.z)
    }

    pub(crate) fn view_forward_handled(&self) -> DVec3 {
        let left_handed = if self.proj.z_axis.w == 0.0 {
            self.proj.z_axis.z > 0.0
        } else {
            self.proj.z_axis.w > 0.0
        };

        self.view_forward() * if left_handed { -1.0 } else { 1.0 }
    }

    /// Up vector of the view camera
    pub fn view_up(&self) -> DVec3 {
        DVec3::new(self.view.x_axis.y, self.view.y_axis.y, self.view.z_axis.y)
    }

    /// Right vector of the view camera
    pub fn view_right(&self) -> DVec3 {
        DVec3::new(self.view.x_axis.x, self.view.y_axis.x, self.view.z_axis.x)
    }

    pub fn model_to_screen(&self, pos: DVec3) -> Option<Pos2> {
        world_to_screen(self.config.viewport_rect, self.mvp, pos)
    }

    pub fn local_space_rotation(&self) -> DQuat {
        if self.local_space() {
            self.transform.rotation
        } else {
            DQuat::IDENTITY
        }
    }

    pub fn local_space_transform(&self) -> DMat4 {
        let rotation = self.local_space_rotation();
        DMat4::from_rotation_translation(rotation, self.transform.translation)
    }

    pub fn view_transform(&self) -> DMat4 {
        let rotation = self.view_rotation();
        DMat4::from_rotation_translation(rotation, self.transform.translation)
    }

    pub fn view_rotation(&self) -> DQuat {
        let x_axis = self.view_up();
        let y_axis = -self.view_forward();
        let z_axis = -self.view_right();
        DQuat::from_mat3(&DMat3::from_cols(x_axis, y_axis, z_axis))
    }

    /// Radius to use for inner circle handles
    pub fn inner_circle_radius(&self) -> f64 {
        self.gizmo_size * 0.2
    }

    /// Radius to use for outer circle handles
    pub fn outer_circle_radius(&self) -> f64 {
        self.gizmo_size + self.stroke_width + (self.scale_factor * 5.0) as f64
    }

    /// Radius to use for outer circle subgizmos
    pub fn arcball_radius(&self) -> f64 {
        self.gizmo_size + self.stroke_width - (self.scale_factor * 5.0) as f64
    }

    pub fn arc_radius(&self, direction: GizmoDirection) -> f64 {
        if direction.is_view() {
            self.outer_circle_radius()
        } else {
            self.gizmo_size
        }
    }

    pub fn normal(&self, direction: GizmoDirection) -> DVec3 {
        let normal = direction.local_normal(-self.view_forward());
        if self.local_space() && !direction.is_view() {
            self.transform.rotation * normal
        } else {
            normal
        }
    }

    pub fn tangent(&self, direction: GizmoDirection) -> DVec3 {
        let tangent = match direction {
            GizmoDirection::X | GizmoDirection::Y => DVec3::Z,
            GizmoDirection::Z => -DVec3::Y,
            GizmoDirection::View => -self.view_right(),
        };
        if self.local_space() && !direction.is_view() {
            self.transform.rotation * tangent
        } else {
            tangent
        }
    }

    pub fn color(
        &self,
        focused: bool,
        direction: GizmoDirection,
        opacity: impl Into<Option<f32>>,
    ) -> Color32 {
        let visuals = self.config.visuals;
        let color = match direction {
            GizmoDirection::X => visuals.x_color,
            GizmoDirection::Y => visuals.y_color,
            GizmoDirection::Z => visuals.z_color,
            GizmoDirection::View => visuals.s_color,
        };

        let color = visuals.highlight_color.filter(|_| focused).unwrap_or(color);
        let color = color.linear_multiply(if focused {
            visuals.highlight_alpha
        } else {
            visuals.inactive_alpha
        });

        if let Some(opacity) = opacity.into() {
            color.gamma_multiply(opacity)
        } else {
            color
        }
    }
}

impl Prepared {
    pub fn update_for_config(&mut self, config: GizmoConfig) {
        self.proj = DMat4::from(config.projection_matrix);
        self.view = DMat4::from(config.view_matrix);

        self.view_proj = self.proj * self.view;

        self.config = config;

        self.update_transform(self.transform);
    }

    pub fn update_for_targets(&mut self, targets: &[crate::Transform]) {
        let mut transform = GizmoTransform {
            scale: DVec3::ZERO,
            translation: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
        };

        for target in targets {
            let target = target.as_gizmo();
            transform.scale += target.scale;
            transform.translation += target.translation;
            transform.rotation = target.rotation;
        }

        if targets.is_empty() {
            transform.scale = DVec3::ONE;
        } else {
            transform.translation /= targets.len() as f64;
            transform.scale /= targets.len() as f64;
        }

        self.update_transform(transform);
    }

    pub fn update_transform(&mut self, transform: GizmoTransform) {
        self.transform = transform;
        self.mvp = self.view_proj * transform.as_dmat4();

        let viewport = self.config.viewport_rect;

        self.scale_factor =
            self.mvp.w_axis.w as f32 / self.proj.x_axis.x as f32 / viewport.width() * 2.0;

        let pos = transform.translation;
        let screen_pos = self.model_to_screen(pos);
        let screen_pos = screen_pos.unwrap_or_default();

        let inv_view_proj = self.view_proj.inverse();
        let gizmo_view_near = screen_to_world(viewport, inv_view_proj, screen_pos, -1.0);

        self.focus_distance = self.scale_factor * (self.config.visuals.stroke_width / 2.0 + 5.0);
        self.eye_to_model_dir = (gizmo_view_near - transform.translation).normalize_or_zero();

        self.gizmo_size = (self.scale_factor * self.config.visuals.gizmo_size) as f64;
        self.stroke_width = (self.scale_factor * self.config.visuals.stroke_width) as f64;
    }
}
