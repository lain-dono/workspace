use crate::math::{DMat4, DVec3, Rect};
use ecolor::Color32;
use enumset::{enum_set, EnumSet, EnumSetType};

/// The default snapping distance for rotation in radians
pub const DEFAULT_SNAP_ANGLE: f32 = std::f32::consts::PI / 32.0;
/// The default snapping distance for translation
pub const DEFAULT_SNAP_DISTANCE: f32 = 0.1;
/// The default snapping distance for scale
pub const DEFAULT_SNAP_SCALE: f32 = 0.1;

/// Configuration of a gizmo.
///
/// Defines how the gizmo is drawn to the screen and
/// how it can be interacted with.
#[derive(Debug, Copy, Clone)]
pub struct GizmoConfig {
    /// View matrix for the gizmo, aligning it with the camera's viewpoint.
    pub view_matrix: mint::RowMatrix4<f64>,
    /// Projection matrix for the gizmo, determining how it is projected onto the screen.
    pub projection_matrix: mint::RowMatrix4<f64>,
    /// Screen area where the gizmo is displayed.
    pub viewport_rect: Rect,

    /// The gizmo's operation modes.
    pub modes: EnumSet<GizmoMode>,
    /// If set, this mode is forced active and other modes are disabled
    pub mode_override: Option<GizmoMode>,

    /// Determines the gizmo's orientation relative to global or local axes.
    pub orientation: GizmoOrientation,
    /// Pivot point for transformations
    pub pivot_point: TransformPivotPoint,

    /// Toggles snapping to predefined increments during transformations for precision.
    pub snapping: bool,
    /// Angle increment for snapping rotations, in radians.
    pub snap_angle: f32,
    /// Distance increment for snapping translations.
    pub snap_distance: f32,
    /// Scale increment for snapping scalings.
    pub snap_scale: f32,

    /// Visual settings for the gizmo, affecting appearance and visibility.
    pub visuals: GizmoVisuals,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            view_matrix: DMat4::IDENTITY.into(),
            projection_matrix: DMat4::IDENTITY.into(),
            viewport_rect: Rect::NOTHING,

            modes: GizmoMode::all(),
            mode_override: None,

            orientation: GizmoOrientation::default(),
            pivot_point: TransformPivotPoint::default(),

            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
            snap_scale: DEFAULT_SNAP_SCALE,

            visuals: GizmoVisuals::default(),
        }
    }
}

/// Operation mode of a gizmo.
#[derive(Debug, EnumSetType, Hash)]
#[repr(u8)]
pub enum GizmoMode {
    /// Rotate around the X axis
    RotateX,
    /// Rotate around the Y axis
    RotateY,
    /// Rotate around the Z axis
    RotateZ,
    /// Rotate around the view forward axis
    RotateView,
    /// Rotate using an arcball (trackball)
    Arcball,

    /// Translate along the X axis
    TranslateX,
    /// Translate along the Y axis
    TranslateY,
    /// Translate along the Z axis
    TranslateZ,
    /// Translate along the XY plane
    TranslateYZ,
    /// Translate along the XZ plane
    TranslateXZ,
    /// Translate along the YZ plane
    TranslateXY,
    /// Translate along the view forward axis
    TranslateView,

    /// Scale along the X axis
    ScaleX,
    /// Scale along the Y axis
    ScaleY,
    /// Scale along the Z axis
    ScaleZ,
    /// Scale along the XY plane
    ScaleYZ,
    /// Scale along the XZ plane
    ScaleXZ,
    /// Scale along the YZ plane
    ScaleXY,
    /// Scale uniformly in all directions
    ScaleUniform,
}

impl GizmoMode {
    /// All modes
    pub fn all() -> EnumSet<Self> {
        EnumSet::all()
    }

    /// All rotation modes
    pub const fn all_rotate() -> EnumSet<Self> {
        enum_set!(Self::RotateX | Self::RotateY | Self::RotateZ | Self::RotateView)
    }

    /// All translation modes
    pub const fn all_translate() -> EnumSet<Self> {
        enum_set!(
            Self::TranslateX
                | Self::TranslateY
                | Self::TranslateZ
                | Self::TranslateYZ
                | Self::TranslateXZ
                | Self::TranslateXY
                | Self::TranslateView
        )
    }

    /// All scaling modes
    pub const fn all_scale() -> EnumSet<Self> {
        enum_set!(
            Self::ScaleX
                | Self::ScaleY
                | Self::ScaleZ
                | Self::ScaleYZ
                | Self::ScaleXZ
                | Self::ScaleXY
                | Self::ScaleUniform
        )
    }

    /// Is this mode for rotation
    pub fn is_rotate(&self) -> bool {
        matches!(self.kind(), GizmoModeKind::Rotate)
    }

    /// Is this mode for translation
    pub fn is_translate(&self) -> bool {
        matches!(self.kind(), GizmoModeKind::Translate)
    }

    /// Is this mode for scaling
    pub fn is_scale(&self) -> bool {
        matches!(self.kind(), GizmoModeKind::Scale)
    }

    /// Axes this mode acts on
    pub fn axes(&self) -> EnumSet<GizmoDirection> {
        match self {
            Self::RotateX | Self::TranslateX | Self::ScaleX => enum_set!(GizmoDirection::X),
            Self::RotateY | Self::TranslateY | Self::ScaleY => enum_set!(GizmoDirection::Y),
            Self::RotateZ | Self::TranslateZ | Self::ScaleZ => enum_set!(GizmoDirection::Z),

            Self::TranslateXY | Self::ScaleXY => enum_set!(GizmoDirection::X | GizmoDirection::Y),
            Self::TranslateYZ | Self::ScaleYZ => enum_set!(GizmoDirection::Y | GizmoDirection::Z),
            Self::TranslateXZ | Self::ScaleXZ => enum_set!(GizmoDirection::X | GizmoDirection::Z),

            Self::RotateView | Self::TranslateView => enum_set!(GizmoDirection::View),
            Self::ScaleUniform | Self::Arcball => {
                enum_set!(GizmoDirection::X | GizmoDirection::Y | GizmoDirection::Z)
            }
        }
    }

    pub(crate) fn direction(&self) -> GizmoDirection {
        match self {
            Self::RotateX | Self::TranslateX | Self::ScaleX => GizmoDirection::X,
            Self::RotateY | Self::TranslateY | Self::ScaleY => GizmoDirection::Y,
            Self::RotateZ | Self::TranslateZ | Self::ScaleZ => GizmoDirection::Z,

            Self::TranslateXY | Self::ScaleXY => GizmoDirection::Z,
            Self::TranslateXZ | Self::ScaleXZ => GizmoDirection::Y,
            Self::TranslateYZ | Self::ScaleYZ => GizmoDirection::X,

            Self::RotateView | Self::TranslateView => GizmoDirection::View,
            Self::ScaleUniform | Self::Arcball => GizmoDirection::View,
        }
    }

    /// Returns the modes that match to given axes exactly
    pub fn all_from_axes(axes: EnumSet<GizmoDirection>) -> EnumSet<Self> {
        EnumSet::<Self>::all()
            .iter()
            .filter(|mode| mode.axes() == axes)
            .collect()
    }

    pub fn kind(&self) -> GizmoModeKind {
        match self {
            Self::RotateX | Self::RotateY | Self::RotateZ => GizmoModeKind::Rotate,
            Self::RotateView => GizmoModeKind::Rotate,

            Self::TranslateX | Self::TranslateY | Self::TranslateZ => GizmoModeKind::Translate,
            Self::TranslateYZ | Self::TranslateXZ | Self::TranslateXY => GizmoModeKind::Translate,
            Self::TranslateView => GizmoModeKind::Translate,

            Self::ScaleX | Self::ScaleY | Self::ScaleZ => GizmoModeKind::Scale,
            Self::ScaleYZ | Self::ScaleXZ | Self::ScaleXY => GizmoModeKind::Scale,
            Self::ScaleUniform => GizmoModeKind::Scale,

            Self::Arcball => GizmoModeKind::Arcball,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum GizmoModeKind {
    Rotate,
    Translate,
    Scale,
    Arcball,
}

/// The point in space around which all rotations are centered.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum TransformPivotPoint {
    /// Pivot around the median point of targets
    #[default]
    MedianPoint,
    /// Pivot around each target's own origin
    IndividualOrigins,
}

/// Orientation of a gizmo.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum GizmoOrientation {
    /// Transformation axes are aligned to world space.
    #[default]
    Global,
    /// Transformation axes are aligned to the last target's orientation.
    Local,
}

#[derive(Debug, EnumSetType, Hash)]
pub enum GizmoDirection {
    /// Gizmo points in the X-direction
    X,
    /// Gizmo points in the Y-direction
    Y,
    /// Gizmo points in the Z-direction
    Z,
    /// Gizmo points in the view direction
    View,
}

impl GizmoDirection {
    pub const fn is_view(&self) -> bool {
        matches!(self, Self::View)
    }

    pub const fn plane_bitangent(self) -> DVec3 {
        match self {
            Self::X => DVec3::Y,
            Self::Y => DVec3::Z,
            Self::Z => DVec3::X,
            Self::View => DVec3::ZERO, // Unused
        }
    }

    pub const fn plane_tangent(self) -> DVec3 {
        match self {
            Self::X => DVec3::Z,
            Self::Y => DVec3::X,
            Self::Z => DVec3::Y,
            Self::View => DVec3::ZERO, // Unused
        }
    }

    pub const fn local_normal(self, view: DVec3) -> DVec3 {
        match self {
            Self::X => DVec3::X,
            Self::Y => DVec3::Y,
            Self::Z => DVec3::Z,
            Self::View => view,
        }
    }
}

/// Controls the visual style of the gizmo
#[derive(Debug, Copy, Clone)]
pub struct GizmoVisuals {
    /// Color of the x axis
    pub x_color: Color32,
    /// Color of the y axis
    pub y_color: Color32,
    /// Color of the z axis
    pub z_color: Color32,
    /// Color of the forward axis
    pub s_color: Color32,
    /// Alpha of the gizmo color when inactive
    pub inactive_alpha: f32,
    /// Alpha of the gizmo color when highlighted/active
    pub highlight_alpha: f32,
    /// Color to use for highlighted and active axes. By default, the axis color is used with `highlight_alpha`
    pub highlight_color: Option<Color32>,
    /// Width (thickness) of the gizmo strokes
    pub stroke_width: f32,
    /// Gizmo size in pixels
    pub gizmo_size: f32,
}

impl Default for GizmoVisuals {
    fn default() -> Self {
        Self {
            x_color: Color32::from_rgb(255, 0, 125),
            y_color: Color32::from_rgb(0, 255, 125),
            z_color: Color32::from_rgb(0, 125, 255),
            s_color: Color32::from_rgb(255, 255, 255),
            inactive_alpha: 0.7,
            highlight_alpha: 1.0,
            highlight_color: None,
            stroke_width: 4.0,
            gizmo_size: 75.0,
        }
    }
}
