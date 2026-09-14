mod axis;
mod painter;
mod render;

use core::f32;

pub use self::axis::Axis;
pub use self::painter::{Painter, PainterData, Viewport};
pub use self::render::{GpuPainter, PainterHandles, PainterRenderPlugin};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn plugin(app: &mut App) {
    app.init_asset::<PainterData>()
        .add_plugins(PainterRenderPlugin)
        .add_systems(Last, update);
}

/// Marker used to specify which camera to use for painter.
#[derive(Component)]
pub struct PainterCamera;

#[allow(clippy::too_many_arguments)]
pub fn update(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<PainterCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,

    mut gizmo_state: Local<GizmoState>,
    mut input_state: Local<InputState>,

    // mut last_scaled_cursor_pos: Local<Vec2>,
    mut assets: ResMut<Assets<PainterData>>,
    mut store: ResMut<PainterHandles>,

    mut house_state: Query<(&mut crate::roof::HouseState, &Transform)>,
) {
    let Ok(window) = windows.get_single() else {
        // No primary window found.
        return;
    };

    let input_state = {
        if let Some(new_cursor) = window.cursor_position() {
            input_state.cursor = new_cursor;
        }

        input_state.started = None;
        if mouse.just_pressed(MouseButton::Left) {
            input_state.dragging = Some(input_state.cursor);
            input_state.started = Some(input_state.cursor);
        }
        if !mouse.pressed(MouseButton::Left) {
            input_state.dragging = None;
            input_state.started = None;
        }
        *input_state
    };

    let (camera, camera_transform) = {
        let mut active_camera = None;

        for camera in cameras.iter() {
            if !camera.0.is_active {
                continue;
            }
            if active_camera.is_some() {
                // multiple active cameras found, warn and skip
                bevy::log::warn!("Only one camera with a GizmoCamera component is supported.");
                return;
            }
            active_camera = Some(camera);
        }

        match active_camera {
            Some(camera) => camera,
            None => return, // no active cameras in the scene
        }
    };

    let Some(viewport) = Viewport::from_camera(camera, camera_transform) else {
        return;
    };

    let handle = store
        .handle
        .get_or_insert_with(|| assets.add(PainterData::default()));

    let mut painter = Painter {
        data: assets.get_mut(handle).unwrap(),
        viewport,
        scale_factor: window.scale_factor(),
    };

    painter.clear();

    paint(&mut painter, input_state, &mut gizmo_state);

    for (mut house_state, _transform) in &mut house_state {
        crate::roof::paint_house_gizmo(&mut painter, input_state, &mut house_state);
    }
}

#[derive(Clone)]
pub struct AxisGroupState<T> {
    pub active: Option<T>,
    pub last: Option<Vec3>,
}

impl<T: PartialEq> AxisGroupState<T> {
    pub fn is_active(&self, mode: T) -> bool {
        self.active == Some(mode)
    }

    pub fn update(&mut self, hover: Option<T>, input: &InputState) {
        if self.active.is_none() && input.started.is_some() {
            self.active = hover;
            self.last = None;
        } else if input.dragging.is_none() {
            self.active = None;
            self.last = None;
        }
    }

    pub fn movement_delta(&mut self, origin: Vec3, direction: Dir3, cursor: Ray3d) -> Vec3 {
        let handle = Ray3d { origin, direction };
        let (_, handle_t) = ray_to_ray(cursor, handle);
        let new = handle.origin + handle.direction * handle_t;
        self.last.replace(new).map_or(Vec3::ZERO, |last| new - last)
    }
}

#[derive(Clone)]
pub struct GizmoState {
    axis: AxisGroupState<Mode>,
    translation: Vec3,
    rotation: Quat,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            axis: AxisGroupState {
                active: None,
                last: None,
            },
            rotation: Quat::from_axis_angle(Vec3::ONE.normalize(), 0.5),
            translation: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub cursor: Vec2,
    pub started: Option<Vec2>,
    pub dragging: Option<Vec2>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    MoveX,
    MoveY,
    MoveZ,

    ScaleX,
    ScaleY,
    ScaleZ,
}

fn paint(painter: &mut Painter, input: InputState, gizmo: &mut GizmoState) {
    // let highlight_color = None;
    let size = 75.0;

    let visuals = Visuals {
        // inactive_alpha: 0.7,
        // inactive_alpha: 0.27,
        inactive_alpha: 0.5,
        highlight_alpha: 1.0,
        stroke_width: 4.0,
        fill_opacity: 0.2,
    };

    let transform = Mat4::from_rotation_translation(gizmo.rotation, gizmo.translation);

    let scale_length = screen_scale(&painter.viewport, transform);

    // let focus_distance = scale * (visuals.stroke_width / 2.0 + 5.0);
    // let focus_distance = scale * (visuals.stroke_width * 0.5);

    if false {
        let stroke = visuals.stroke(Y_COLOR, false);
        let radius = 2.0 * scale_length * size;
        painter.circle(transform, radius, stroke);
        painter.fill_circle(transform, radius, visuals.fill(S_COLOR, false));
    }

    let focus_distance = visuals.stroke_width * 0.5 + 5.0;

    let scale_start = 0.0;
    let scale_end = scale_start + size;
    let move_start = scale_end + size * 0.2;
    let move_end = move_start + size * 0.3;

    let x_move = Axis::arrow_px(transform, scale_length, move_start, move_end);
    let y_move = Axis::arrow_py(transform, scale_length, move_start, move_end);
    let z_move = Axis::arrow_pz(transform, scale_length, move_start, move_end);

    let x_scale = Axis::cube_px(transform, scale_length, scale_start, scale_end);
    let y_scale = Axis::cube_py(transform, scale_length, scale_start, scale_end);
    let z_scale = Axis::cube_pz(transform, scale_length, scale_start, scale_end);

    let items = [
        (Mode::MoveX, x_move, X_COLOR),
        (Mode::MoveY, y_move, Y_COLOR),
        (Mode::MoveZ, z_move, Z_COLOR),
        (Mode::ScaleX, x_scale, X_COLOR),
        (Mode::ScaleY, y_scale, Y_COLOR),
        (Mode::ScaleZ, z_scale, Z_COLOR),
    ];

    let hover = find_hover(&painter.viewport, &input, focus_distance, items.iter());

    gizmo.axis.update(hover, &input);

    for (mode, mut axis, color) in items {
        let is_active = gizmo.axis.is_active(mode);
        let is_hovered = hover == Some(mode);

        if is_active {
            axis.start = 0.0;

            let count = 100;
            let step = Vec3::from(axis.ray.direction);
            let mut start = axis.ray.origin - step * count as f32;
            for _ in -count..count {
                let next = start + step;
                painter.line(axis.transform, start, next, (0.25, color));
                start = next;
            }
        }

        if gizmo.axis.active.is_none() || is_active {
            axis.paint(painter, visuals.stroke(color, is_hovered || is_active));
        }
    }

    let Some(cursor) = painter.viewport.viewport_to_world(input.cursor) else {
        return;
    };

    let move_dir = match gizmo.axis.active {
        Some(Mode::MoveX) => Some(Dir3::X),
        Some(Mode::MoveY) => Some(Dir3::Y),
        Some(Mode::MoveZ) => Some(Dir3::Z),
        _ => None,
    };

    if let Some(dir) = move_dir {
        let origin = gizmo.translation;
        let direction = gizmo.rotation * dir;
        gizmo.translation += gizmo.axis.movement_delta(origin, direction, cursor);
    }
}

pub fn find_hover<'a, A: Copy + 'static, B: 'static>(
    viewport: &Viewport,
    input: &InputState,
    focus_distance: f32,
    items: impl Iterator<Item = &'a (A, Axis, B)>,
) -> Option<A> {
    items
        .filter_map(|(mode, axis, _)| Some(*mode).zip(axis.distance2d(viewport, input.cursor)))
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .and_then(|(mode, distance)| (distance <= focus_distance).then_some(mode))
}

#[derive(Clone, Copy, Debug)]
pub struct Visuals {
    pub inactive_alpha: f32,
    pub highlight_alpha: f32,
    pub fill_opacity: f32,
    pub stroke_width: f32,
}

impl Visuals {
    pub fn fill(&self, color: epaint::Color32, focused: bool) -> epaint::Color32 {
        self.color(color, focused).gamma_multiply(self.fill_opacity)
    }

    pub fn stroke(&self, color: epaint::Color32, focused: bool) -> (f32, epaint::Color32) {
        (self.stroke_width, self.color(color, focused))
    }

    pub fn color(&self, color: epaint::Color32, focused: bool) -> epaint::Color32 {
        let (highlight, inactive) = (self.highlight_alpha, self.inactive_alpha);
        color.linear_multiply(if focused { highlight } else { inactive })
    }
}

pub fn ray_to_ray(a: Ray3d, b: Ray3d) -> (f32, f32) {
    let a_direction = a.direction.as_vec3();
    let b_direction = b.direction.as_vec3();

    let ab = a_direction.dot(b_direction);
    let diff = a.origin - b.origin;
    let aw = a_direction.dot(diff);
    let bw = b_direction.dot(diff);
    let dot = 1.0 - ab * ab;

    let (ta, tb);
    if dot < 1e-8 {
        ta = 0.0;
        tb = bw;
    } else {
        ta = (ab * bw - aw) / dot;
        tb = (bw - ab * aw) / dot;
    }

    (ta, tb)
}

/// Finds the intersection point of a ray and a plane
/// and distance from the intersection to the plane origin
pub fn plane_origin(ray: Ray3d, plane_origin: Vec3, plane: InfinitePlane3d) -> Option<(f32, f32)> {
    let distance = ray.intersect_plane(plane_origin, plane)?;
    Some((distance, (ray.get_point(distance) - plane_origin).length()))
}

pub fn screen_scale(viewport: &Viewport, transform: Mat4) -> f32 {
    let clip_from_view = viewport.clip_from_view;
    let clip_from_world = viewport.clip_from_world;
    let viewport_size = viewport.logical_viewport_size;

    let mvp = clip_from_world * transform;
    mvp.w_axis.w / clip_from_view.x_axis.x / viewport_size.x * 2.0
}

pub const X_COLOR: epaint::Color32 = epaint::Color32::from_rgb(255, 0, 125);
pub const Y_COLOR: epaint::Color32 = epaint::Color32::from_rgb(0, 255, 125);
pub const Z_COLOR: epaint::Color32 = epaint::Color32::from_rgb(0, 125, 255);
pub const S_COLOR: epaint::Color32 = epaint::Color32::from_rgb(255, 255, 255);

pub const _X_COLOR: Color = Color::srgb(1.0, 0.0, 0.5);
pub const _Y_COLOR: Color = Color::srgb(0.0, 1.0, 0.5);
pub const _Z_COLOR: Color = Color::srgb(0.0, 0.5, 1.0);
pub const _S_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
