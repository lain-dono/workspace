use crate::{
    axis::{MoveAxis, ScaleAxis},
    config::{GizmoConfig, GizmoDirection, GizmoMode, TransformPivotPoint},
    draw::{DrawData, Painter},
    math::{DQuat, DVec3, Pos2, Ray},
    plane::{MovePlane, ScalePlane},
    prepared::{GizmoTransform, Prepared},
    rotation::{Arcball, Rotate},
    Transform,
};
use enumset::EnumSet;
use std::fmt::Debug;

pub trait GizmoHandle: Send + Sync {
    fn pick(&mut self, config: &Prepared, ray: Ray) -> Option<f64>;
    fn update(&mut self, config: &Prepared, ray: Ray) -> Option<GizmoResult>;
    fn draw(&self, config: &Prepared, state: &State, painter: &mut Painter);
}

#[derive(Clone, Debug, Default)]
pub struct State {
    /// Whether this handle is focused this frame
    pub focused: bool,
    /// Whether this handle is active this frame
    pub active: bool,
}

struct Wrapper {
    mode: GizmoMode,
    state: State,
    handle: Box<dyn GizmoHandle>,
}

/// A 3D transformation gizmo.
#[derive(Default)]
pub struct Gizmo {
    /// Prepared configuration of the gizmo.
    /// Includes the original [`GizmoConfig`] as well as
    /// various other values calculated from it, used for
    /// interaction and drawing the gizmo.
    config: Prepared,
    /// Handles used in the gizmo.
    handles: Vec<Wrapper>,
    active_id: Option<GizmoMode>,
    target_start_transforms: Vec<Transform>,
    gizmo_start_transform: GizmoTransform,
}

impl Gizmo {
    /// Creates a new gizmo from given configuration
    pub fn new(config: GizmoConfig) -> Self {
        let mut gizmo = Self::default();
        gizmo.update_config(config);
        gizmo
    }

    /// Current configuration used by the gizmo.
    pub fn config(&self) -> &GizmoConfig {
        &self.config
    }

    /// Updates the configuration used by the gizmo.
    pub fn update_config(&mut self, config: GizmoConfig) {
        let is_changed = {
            let other: &GizmoConfig = &self.config;
            (config.modes != other.modes && config.mode_override.is_none())
                || (config.mode_override != other.mode_override)
        };

        if is_changed {
            self.handles.clear();
            self.active_id = None;
        }

        self.config.update_for_config(config);

        if self.handles.is_empty() {
            let config = self.config;

            // Get all modes that are currently enabled
            let modes = config.modes;
            let modes = config.mode_override.map_or(modes, EnumSet::only);

            self.add(modes, GizmoMode::RotateX, Rotate::new);
            self.add(modes, GizmoMode::RotateY, Rotate::new);
            self.add(modes, GizmoMode::RotateZ, Rotate::new);
            self.add(modes, GizmoMode::RotateView, Rotate::new);
            self.add(modes, GizmoMode::Arcball, |_| Arcball::new());

            self.add(modes, GizmoMode::TranslateX, MoveAxis::new);
            self.add(modes, GizmoMode::TranslateY, MoveAxis::new);
            self.add(modes, GizmoMode::TranslateZ, MoveAxis::new);

            self.add(modes, GizmoMode::TranslateView, MovePlane::new);
            self.add(modes, GizmoMode::TranslateXY, MovePlane::new);
            self.add(modes, GizmoMode::TranslateXZ, MovePlane::new);
            self.add(modes, GizmoMode::TranslateYZ, MovePlane::new);

            self.add(modes, GizmoMode::ScaleX, ScaleAxis::new);
            self.add(modes, GizmoMode::ScaleY, ScaleAxis::new);
            self.add(modes, GizmoMode::ScaleZ, ScaleAxis::new);

            if !modes.contains(GizmoMode::RotateView) {
                self.add(modes, GizmoMode::ScaleUniform, ScalePlane::new);
            }
            if !modes.contains(GizmoMode::TranslateXY) {
                self.add(modes, GizmoMode::ScaleXY, ScalePlane::new);
            }
            if !modes.contains(GizmoMode::TranslateXZ) {
                self.add(modes, GizmoMode::ScaleXZ, ScalePlane::new);
            }
            if !modes.contains(GizmoMode::TranslateYZ) {
                self.add(modes, GizmoMode::ScaleXY, ScalePlane::new);
            }
        }
    }

    fn add<T: GizmoHandle + 'static>(
        &mut self,
        modes: EnumSet<GizmoMode>,
        mode: GizmoMode,
        new: impl FnOnce(GizmoDirection) -> T,
    ) {
        if modes.contains(mode) {
            self.handles.push(Wrapper {
                mode,
                state: State::default(),
                handle: Box::new(new(mode.direction())),
            });
        }
    }

    /// Was this gizmo focused after the latest [`Gizmo::update`] call.
    pub fn is_focused(&self) -> bool {
        self.handles.iter().any(|handle| handle.state.focused)
    }

    /// Updates the gizmo based on given interaction information.
    ///
    /// # Examples
    ///
    /// ```
    /// # // Dummy values
    /// # use transform_gizmo::GizmoInteraction;
    /// # let mut gizmo = transform_gizmo::Gizmo::default();
    /// # let cursor_pos = Default::default();
    /// # let drag_started = true;
    /// # let dragging = true;
    /// # let mut transforms = vec![];
    ///
    /// let interaction = GizmoInteraction {
    ///     cursor_pos,
    ///     drag_started,
    ///     dragging
    /// };
    ///
    /// if let Some((_result, new_transforms)) = gizmo.update(interaction, &transforms) {
    ///                 for (new_transform, transform) in
    ///     // Update transforms
    ///     new_transforms.iter().zip(&mut transforms)
    ///     {
    ///         *transform = *new_transform;
    ///     }
    /// }
    /// ```
    ///
    /// Returns the result of the interaction with the updated transformation.
    ///
    /// [`Some`] is returned when any of the handles is being dragged, [`None`] otherwise.
    pub fn update(
        &mut self,
        interaction: GizmoInteraction,
        targets: &[Transform],
    ) -> Option<(GizmoResult, Vec<Transform>)> {
        if !self.config.viewport_rect.is_finite() {
            return None;
        }

        // Update the gizmo based on the given target transforms,
        // unless the gizmo is currently being interacted with.
        if self.active_id.is_none() {
            self.config.update_for_targets(targets);
        }

        // All handles are initially considered unfocused.
        for Wrapper { state, .. } in &mut self.handles {
            state.focused = false;
        }

        let force_active = self.config.mode_override.is_some();

        let pointer_ray = Ray::from_pointer(
            self.config.view_proj,
            self.config.viewport_rect,
            Pos2::from(interaction.cursor_pos),
        );

        // If there is no active handle, find which one of them
        // is under the mouse pointer, if any.
        if self.active_id.is_none() {
            // Picks the handlesthat is closest to the given world space ray.
            let handle = if force_active {
                // If mode is overridden, assume we only have that mode, and choose it.
                self.handles.first_mut().map(|sub| {
                    sub.handle.pick(&self.config, pointer_ray);
                    sub
                })
            } else {
                self.handles
                    .iter_mut()
                    .filter_map(|sub| sub.handle.pick(&self.config, pointer_ray).map(|t| (t, sub)))
                    .min_by(|(first, _), (second, _)| first.total_cmp(second))
                    .map(|(_, sub)| sub)
            };

            if let Some(Wrapper { state, mode, .. }) = handle {
                state.focused = true;

                // If we started dragging from one of the handles, mark it as active.
                if interaction.drag_started || force_active {
                    self.active_id = Some(*mode);
                    self.target_start_transforms = targets.to_vec();
                    self.gizmo_start_transform = self.config.transform;
                }
            }
        }

        let mut result = None;

        if let Some(Wrapper { handle, state, .. }) = self
            .active_id
            .and_then(|mode| self.handles.iter_mut().find(|sub| sub.mode == mode))
        {
            let is_active_and_focused = interaction.dragging || force_active;
            state.active = is_active_and_focused;
            state.focused = is_active_and_focused;

            if is_active_and_focused {
                result = handle.update(&self.config, pointer_ray);
            } else {
                self.active_id = None;
            }
        }

        let Some(result) = result else {
            // No interaction, no result.
            self.config.update_for_targets(targets);
            return None;
        };

        self.config.update_transform(Self::combine_transform(
            self.config,
            result,
            self.config.transform,
            self.gizmo_start_transform,
        ));

        let updated_targets = targets
            .iter()
            .zip(&self.target_start_transforms)
            .map(|(transform, start)| {
                Self::combine_transform(self.config, result, transform.as_gizmo(), start.as_gizmo())
                    .as_mint()
            })
            .collect();

        Some((result, updated_targets))
    }

    /// Return all the necessary data to draw the latest gizmo interaction.
    ///
    /// The gizmo draw data consists of vertices in viewport coordinates.
    pub fn draw(&self, pixels_per_point: f32) -> DrawData {
        let mut painter = Painter {
            data: DrawData::default(),
            view_proj: self.config.view_proj,
            viewport: self.config.viewport_rect,
            pixels_per_point,
        };

        if self.config.viewport_rect.is_finite() {
            for Wrapper { state, handle, .. } in &self.handles {
                if self.active_id.is_none() || state.active {
                    handle.draw(&self.config, state, &mut painter);
                }
            }
        }

        painter.finish()
    }

    fn combine_transform(
        config: Prepared,
        result: GizmoResult,
        transform: GizmoTransform,
        start: GizmoTransform,
    ) -> GizmoTransform {
        let pivot = match config.pivot_point {
            TransformPivotPoint::MedianPoint => Some(config.transform.translation),
            TransformPivotPoint::IndividualOrigins => None,
        };

        match result {
            GizmoResult::Rotation {
                axis,
                delta,
                total: _,
                is_view_axis,
            } => {
                let axis = DVec3::from(axis);
                let axis = if config.local_space() && !is_view_axis {
                    transform.rotation * axis
                } else {
                    axis
                };
                let delta = DQuat::from_axis_angle(axis, delta);

                Self::update_rotation_quat(transform, delta, pivot)
            }
            GizmoResult::Translation { delta, total: _ } => {
                let delta = DVec3::from(delta);
                let delta = if config.local_space() {
                    start.rotation * delta
                } else {
                    delta
                };

                GizmoTransform {
                    scale: start.scale,
                    rotation: start.rotation,
                    translation: transform.translation + delta,
                }
            }
            GizmoResult::Scale { total } => GizmoTransform {
                scale: start.scale * DVec3::from(total),
                rotation: transform.rotation,
                translation: transform.translation,
            },
            GizmoResult::Arcball { delta, total: _ } => {
                Self::update_rotation_quat(transform, delta.into(), pivot)
            }
        }
    }

    fn update_rotation_quat(
        transform: GizmoTransform,
        delta: DQuat,
        pivot: Option<DVec3>,
    ) -> GizmoTransform {
        GizmoTransform {
            scale: transform.scale,
            rotation: delta * transform.rotation,
            translation: if let Some(pivot) = pivot {
                pivot + delta * (transform.translation - pivot)
            } else {
                transform.translation
            },
        }
    }
}

/// Information needed for interacting with the gizmo.
#[derive(Default, Clone, Copy, Debug)]
pub struct GizmoInteraction {
    /// Current cursor position in window coordinates.
    pub cursor_pos: (f32, f32),
    /// Whether dragging was started this frame.
    /// Usually this is set to true if the primary mouse
    /// button was just pressed.
    pub drag_started: bool,
    /// Whether the user is currently dragging.
    /// Usually this is set to true whenever the primary mouse
    /// button is being pressed.
    pub dragging: bool,
}

/// Result of a gizmo transformation
#[derive(Debug, Copy, Clone)]
pub enum GizmoResult {
    Rotation {
        /// The rotation axis,
        axis: mint::Vector3<f64>,
        /// The latest rotation angle delta
        delta: f64,
        /// Total rotation angle of the gizmo interaction
        total: f64,
        /// Whether we are rotating along the view axis
        is_view_axis: bool,
    },
    Translation {
        /// The latest translation delta
        delta: mint::Vector3<f64>,
        /// Total translation of the gizmo interaction
        total: mint::Vector3<f64>,
    },
    Scale {
        /// Total scale of the gizmo interaction
        total: mint::Vector3<f64>,
    },
    Arcball {
        /// The latest rotation delta
        delta: mint::Quaternion<f64>,
        /// Total rotation of the gizmo interaction
        total: mint::Quaternion<f64>,
    },
}
