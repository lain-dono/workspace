use bevy::prelude::*;
use ecolor::{Color32, Rgba};
use epaint::{Mesh, PathStroke, Shape, Stroke, TessellationOptions, Tessellator, TextureId};
use std::f32::consts::TAU;

#[derive(Clone)]
pub struct Viewport {
    pub camera_transform: Mat4,
    pub clip_from_view: Mat4,
    pub ndc_to_world: Mat4,
    pub clip_from_world: Mat4,
    pub logical_viewport_size: Vec2,
}

impl Viewport {
    pub fn from_camera(camera: &Camera, camera_transform: &GlobalTransform) -> Option<Self> {
        Some(Self::new(
            camera_transform.compute_matrix(),
            camera.clip_from_view(),
            camera.logical_viewport_size()?,
        ))
    }

    pub fn new(camera_transform: Mat4, clip_from_view: Mat4, logical_viewport_size: Vec2) -> Self {
        Self {
            camera_transform,
            clip_from_view,
            ndc_to_world: camera_transform * clip_from_view.inverse(),
            clip_from_world: clip_from_view * camera_transform.inverse(),
            logical_viewport_size,
        }
    }

    #[inline]
    pub fn world_to_viewport(&self, world_position: Vec3) -> Option<Vec2> {
        let ndc_space_coords = self.world_to_ndc(world_position)?;
        // NDC z-values outside of 0 < z < 1 are outside the (implicit) camera frustum and are thus not in viewport-space
        if ndc_space_coords.z < 0.0 || ndc_space_coords.z > 1.0 {
            None
        } else {
            Some(self.ndc_to_viewport(ndc_space_coords.truncate()))
        }
    }

    #[inline]
    pub fn viewport_to_world(&self, viewport_position: Vec2) -> Option<Ray3d> {
        let ndc = self.viewport_to_ndc(viewport_position);

        // Using EPSILON because an ndc with Z = 0 returns NaNs.
        let world_near_plane = self.ndc_to_world.project_point3(ndc.extend(1.0));
        let world_far_plane = self.ndc_to_world.project_point3(ndc.extend(f32::EPSILON));

        // The fallible direction constructor ensures that world_near_plane and world_far_plane aren't NaN.
        let origin = world_near_plane;
        Dir3::new(world_far_plane - world_near_plane)
            .map_or(None, |direction| Some(Ray3d { origin, direction }))
    }

    #[inline]
    pub fn viewport_to_ndc(&self, mut viewport_position: Vec2) -> Vec2 {
        // Flip the Y co-ordinate origin from the top to the bottom.
        viewport_position.y = self.logical_viewport_size.y - viewport_position.y;
        viewport_position * 2.0 / self.logical_viewport_size - Vec2::ONE
    }

    #[inline]
    pub fn ndc_to_viewport(&self, ndc: Vec2) -> Vec2 {
        // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
        let mut viewport_position = (ndc + Vec2::ONE) / 2.0 * self.logical_viewport_size;
        // Flip the Y co-ordinate origin from the bottom to the top.
        viewport_position.y = self.logical_viewport_size.y - viewport_position.y;
        viewport_position
    }

    #[inline]
    pub fn world_to_ndc(&self, world_position: Vec3) -> Option<Vec3> {
        // Build a transformation matrix to convert from world space to NDC using camera data
        let ndc_space_coords = self.clip_from_world.project_point3(world_position);
        (!ndc_space_coords.is_nan()).then_some(ndc_space_coords)
    }

    #[inline]
    pub fn ndc_to_world(&self, ndc: Vec3) -> Option<Vec3> {
        // Build a transformation matrix to convert from NDC to world space using camera data
        let world_space_coords = self.ndc_to_world.project_point3(ndc);
        (!world_space_coords.is_nan()).then_some(world_space_coords)
    }
}

#[derive(Component, Clone)]
pub struct PainterHandle(pub Handle<PainterData>);

/// Data used to draw [`Gizmo`].
#[derive(Asset, Debug, Default, Clone, TypePath)]
pub struct PainterData {
    /// Vertices in viewport space.
    pub vertices: Vec<[f32; 2]>,
    /// Linear RGBA colors.
    pub colors: Vec<[f32; 4]>,
    /// Indices to the vertex data.
    pub indices: Vec<u32>,
}

#[derive(Resource)]
pub struct Painter<'a> {
    pub data: &'a mut PainterData,
    pub viewport: Viewport,
    pub scale_factor: f32,
}

impl Painter<'_> {
    pub fn clear(&mut self) {
        self.data.vertices.clear();
        self.data.colors.clear();
        self.data.indices.clear();
    }

    pub fn shape(&mut self, shape: Shape) {
        let options = TessellationOptions {
            feathering: true,
            ..Default::default()
        };

        let mut tessellator = Tessellator::new(
            self.scale_factor,
            options,
            Default::default(),
            Default::default(),
        );

        let mut mesh = Mesh::default();
        tessellator.tessellate_shape(shape, &mut mesh);
        mesh.texture_id = TextureId::default();

        let vertices = mesh.vertices.iter().map(|v| {
            let view_position = Vec2::new(v.pos.x, v.pos.y);
            self.viewport.viewport_to_ndc(view_position).to_array()
        });

        let colors = mesh.vertices.iter().map(|v| Rgba::from(v.color).to_array());

        let offset = self.data.vertices.len() as u32;
        let indices = mesh.indices.into_iter().map(|index| offset + index);
        self.data.vertices.extend(vertices);
        self.data.colors.extend(colors);
        self.data.indices.extend(indices);
    }

    fn points_to_screen(
        &self,
        transform: Mat4,
        points: impl IntoIterator<Item = Vec3>,
    ) -> impl Iterator<Item = emath::Pos2> {
        let viewport = self.viewport.clone();

        points
            .into_iter()
            .filter_map(move |point| viewport.world_to_viewport(transform.transform_point3(point)))
            .map(|p| emath::pos2(p.x, p.y))
    }

    fn point_to_screen(&self, transform: Mat4, point: Vec3) -> Option<emath::Pos2> {
        self.viewport
            .world_to_viewport(transform.transform_point3(point))
            .map(|Vec2 { x, y }| emath::pos2(x, y))
    }

    pub fn circle(&mut self, transform: Mat4, radius: f32, stroke: impl Into<PathStroke>) {
        let points = arc_points(radius, 0.0, TAU);
        let mut points: Vec<_> = self.points_to_screen(transform, points).collect();
        points.pop();
        self.shape(Shape::closed_line(points, stroke));
    }

    pub fn fill_circle(&mut self, transform: Mat4, radius: f32, fill: Color32) {
        if fill.a() != 0 {
            let stroke = (0.0, Color32::TRANSPARENT);
            let points = arc_points(radius, 0.0, TAU);
            let mut points: Vec<_> = self.points_to_screen(transform, points).collect();
            points.pop();
            self.shape(Shape::convex_polygon(points, fill, stroke));
        }
    }

    pub fn arc(
        &mut self,
        transform: Mat4,
        radius: f32,
        start: f32,
        end: f32,
        stroke: impl Into<PathStroke>,
    ) {
        let points = arc_points(radius, start, end);
        let mut points: Vec<_> = self.points_to_screen(transform, points).collect();

        let (first, last) = (points.first(), points.last());
        let closed = (first.zip(last)).filter(|(a, &b)| a.distance(b) < 1e-2);

        self.shape(if closed.is_some() {
            points.pop();
            Shape::closed_line(points, stroke)
        } else {
            Shape::line(points, stroke)
        });
    }

    pub fn polygon(
        &mut self,
        transform: Mat4,
        points: impl IntoIterator<Item = Vec3>,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) {
        let points: Vec<_> = self.points_to_screen(transform, points).collect();
        if points.len() > 2 {
            self.shape(Shape::convex_polygon(points, fill, stroke));
        }
    }

    pub fn polyline(
        &mut self,
        transform: Mat4,
        points: impl IntoIterator<Item = Vec3>,
        stroke: impl Into<PathStroke>,
    ) {
        let points: Vec<_> = self.points_to_screen(transform, points).collect();
        if points.len() > 1 {
            self.shape(Shape::line(points, stroke));
        }
    }

    pub fn sector(&mut self, transform: Mat4, radius: f32, start: f32, end: f32, fill: Color32) {
        let stroke = (0.0, Color32::TRANSPARENT);
        let angle_delta = end - start;
        let step_count = steps(angle_delta.abs());
        if step_count < 2 {
            return;
        }

        let step_size = angle_delta / (step_count - 1) as f32;

        if ((start - end).abs() - TAU).abs() < step_size.abs() {
            let points = arc_points(radius, 0.0, TAU);
            let mut points: Vec<_> = self.points_to_screen(transform, points).collect();
            points.pop();
            self.shape(Shape::convex_polygon(points, fill, stroke));
        } else {
            let mut points = Vec::with_capacity(step_count + 1);

            points.push(Vec3::new(0.0, 0.0, 0.0));

            let (sin_step, cos_step) = step_size.sin_cos();
            let (mut sin_angle, mut cos_angle) = start.sin_cos();

            for _ in 0..step_count {
                let x = cos_angle * radius;
                let z = sin_angle * radius;

                points.push(Vec3::new(x, 0.0, z));

                let new_sin = sin_angle * cos_step + cos_angle * sin_step;
                let new_cos = cos_angle * cos_step - sin_angle * sin_step;

                sin_angle = new_sin;
                cos_angle = new_cos;
            }

            let points = self.points_to_screen(transform, points).collect();
            self.shape(Shape::convex_polygon(points, fill, stroke));
        }
    }

    pub fn line(&mut self, transform: Mat4, from: Vec3, to: Vec3, stroke: impl Into<PathStroke>) {
        let stroke = stroke.into();
        let from = self.point_to_screen(transform, from);
        let to = self.point_to_screen(transform, to);

        if let Some(points) = from.zip(to).map(|(from, to)| [from, to]) {
            self.shape(Shape::LineSegment { points, stroke });
        }
    }

    pub fn dashed_line(
        &mut self,
        transform: Mat4,
        from: Vec3,
        to: Vec3,
        stroke: impl Into<Stroke>,

        dash_length: f32,
        gap_length: f32,
    ) {
        let from = self.point_to_screen(transform, from);
        let to = self.point_to_screen(transform, to);

        if let Some(points) = from.zip(to).map(|(from, to)| [from, to]) {
            dashes_from_line(
                &points,
                stroke.into(),
                &[dash_length],
                &[gap_length],
                |shape| self.shape(shape),
                0.,
            );
        }
    }

    pub fn arrow(&mut self, transform: Mat4, from: Vec3, to: Vec3, stroke: impl Into<Stroke>) {
        let Stroke { width, color } = stroke.into();
        let from = self.point_to_screen(transform, from);
        let to = self.point_to_screen(transform, to);

        if let Some((start, end)) = from.zip(to) {
            let cross = (end - start).normalized().rot90() * width / 2.0;
            let points = vec![start - cross, start + cross, end];
            self.shape(Shape::convex_polygon(points, color, PathStroke::NONE));
        }
    }
}

fn steps(angle: f32) -> usize {
    const STEPS_PER_RAD: f32 = 20.0;
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}

fn arc_points(radius: f32, start: f32, end: f32) -> impl Iterator<Item = Vec3> {
    let angle = (end - start).clamp(-TAU, TAU);
    let step_count = steps(angle);
    let step_size = angle / (step_count - 1) as f32;
    (0..step_count).map(move |i| {
        let step = step_size * i as f32;
        let angle = start + step;
        let (z, x) = angle.sin_cos();
        Vec3::new(x * radius, 0.0, z * radius)
    })
}

fn dashes_from_line(
    path: &[emath::Pos2],
    stroke: Stroke,
    dash_lengths: &[f32],
    gap_lengths: &[f32],
    mut shapes: impl FnMut(Shape),
    dash_offset: f32,
) {
    assert_eq!(dash_lengths.len(), gap_lengths.len());
    let mut position_on_segment = dash_offset;
    let mut drawing_dash = false;
    let mut step = 0;
    let steps = dash_lengths.len();
    path.windows(2).for_each(|window| {
        let (start, end) = (window[0], window[1]);
        let vector = end - start;
        let segment_length = vector.length();

        let mut start_point = start;
        while position_on_segment < segment_length {
            let new_point = start + vector * (position_on_segment / segment_length);
            if drawing_dash {
                // This is the end point.
                shapes(Shape::line_segment([start_point, new_point], stroke));
                position_on_segment += gap_lengths[step];
                // Increment step counter
                step += 1;
                if step >= steps {
                    step = 0;
                }
            } else {
                // Start a new dash.
                start_point = new_point;
                position_on_segment += dash_lengths[step];
            }
            drawing_dash = !drawing_dash;
        }

        // If the segment ends and the dash is not finished, add the segment's end point.
        if drawing_dash {
            shapes(Shape::line_segment([start_point, end], stroke));
        }

        position_on_segment -= segment_length;
    });
}
