use crate::math::{world_to_screen, DMat4, DVec3, Pos2, Rect};
use ecolor::{Color32, Rgba};
use epaint::{Mesh, PathStroke, Shape, Stroke, TessellationOptions, Tessellator, TextureId};
use std::f64::consts::TAU;

/// Data used to draw [`Gizmo`].
#[derive(Default, Clone, Debug)]
pub struct DrawData {
    /// Vertices in viewport space.
    pub vertices: Vec<[f32; 2]>,
    /// Linear RGBA colors.
    pub colors: Vec<[f32; 4]>,
    /// Indices to the vertex data.
    pub indices: Vec<u32>,
}

pub struct Painter {
    pub data: DrawData,
    pub view_proj: DMat4,
    pub viewport: Rect,
    pub pixels_per_point: f32,
}

impl Painter {
    pub fn finish(self) -> DrawData {
        self.data
    }

    fn shape(&mut self, shape: Shape) {
        let ppi = self.pixels_per_point;
        let options = TessellationOptions {
            feathering: true,
            ..Default::default()
        };

        let mut tessellator =
            Tessellator::new(ppi, options, Default::default(), Default::default());

        let mut mesh = Mesh::default();
        tessellator.tessellate_shape(shape, &mut mesh);
        mesh.texture_id = TextureId::default();

        let (vertices, colors): (Vec<_>, Vec<_>) = mesh
            .vertices
            .iter()
            .map(|v| ([v.pos.x, v.pos.y], Rgba::from(v.color).to_array()))
            .unzip();

        let index_offset = self.data.vertices.len() as u32;
        self.data.vertices.extend(vertices);
        self.data.colors.extend(colors);
        self.data
            .indices
            .extend(mesh.indices.into_iter().map(|idx| index_offset + idx));
    }

    pub fn circle(&mut self, transform: DMat4, radius: f64, stroke: impl Into<PathStroke>) {
        let mvp = self.view_proj * transform;
        self.shape(arc(self.viewport, mvp, radius, 0.0, TAU, stroke));
    }

    pub fn filled_circle(&mut self, transform: DMat4, fill: Color32, radius: f64) {
        if fill.a() != 0 {
            let mvp = self.view_proj * transform;
            let stroke = (0.0, Color32::TRANSPARENT);
            self.shape(filled_circle(self.viewport, mvp, radius, fill, stroke));
        }
    }

    pub fn arc(
        &mut self,
        transform: DMat4,
        radius: f64,
        start: f64,
        end: f64,
        stroke: impl Into<PathStroke>,
    ) {
        let mvp = self.view_proj * transform;
        self.shape(arc(self.viewport, mvp, radius, start, end, stroke));
    }

    pub fn fill_polygon(
        &mut self,
        transform: DMat4,
        points: impl IntoIterator<Item = DVec3>,
        fill: Color32,
    ) {
        if fill.a() != 0 {
            let mvp = self.view_proj * transform;
            let stroke = (0.0, Color32::TRANSPARENT);
            self.shape(polygon(self.viewport, mvp, points, fill, stroke));
        }
    }

    pub fn polyline(
        &mut self,
        transform: DMat4,
        points: impl IntoIterator<Item = DVec3>,
        stroke: impl Into<PathStroke>,
    ) {
        let mvp = self.view_proj * transform;
        self.shape(polyline(self.viewport, mvp, points, stroke));
    }

    pub fn fill_sector(
        &mut self,
        transform: DMat4,
        radius: f64,
        start: f64,
        end: f64,
        fill: Color32,
    ) {
        let stroke = (0.0, Color32::TRANSPARENT);
        let mvp = self.view_proj * transform;
        self.shape(sector(self.viewport, mvp, radius, start, end, fill, stroke));
    }

    pub fn line(
        &mut self,
        transform: DMat4,
        from: DVec3,
        to: DVec3,
        stroke: impl Into<PathStroke>,
    ) {
        let mvp = self.view_proj * transform;
        self.shape(line_segment(self.viewport, mvp, from, to, stroke));
    }

    pub fn arrow(&mut self, transform: DMat4, from: DVec3, to: DVec3, stroke: impl Into<Stroke>) {
        let mvp = self.view_proj * transform;
        self.shape(arrow(self.viewport, mvp, from, to, stroke));
    }
}

fn sector(
    viewport: Rect,
    mvp: DMat4,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    fill: impl Into<Color32>,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let angle_delta = end_angle - start_angle;
    let step_count = steps(angle_delta.abs());

    if step_count < 2 {
        return Shape::Noop;
    }

    let step_size = angle_delta / (step_count - 1) as f64;

    if ((start_angle - end_angle).abs() - TAU).abs() < step_size.abs() {
        filled_circle(viewport, mvp, radius, fill.into(), stroke)
    } else {
        let mut points = Vec::with_capacity(step_count + 1);

        points.push(DVec3::new(0.0, 0.0, 0.0));

        let (sin_step, cos_step) = step_size.sin_cos();
        let (mut sin_angle, mut cos_angle) = start_angle.sin_cos();

        for _ in 0..step_count {
            let x = cos_angle * radius;
            let z = sin_angle * radius;

            points.push(DVec3::new(x, 0.0, z));

            let new_sin = sin_angle * cos_step + cos_angle * sin_step;
            let new_cos = cos_angle * cos_step - sin_angle * sin_step;

            sin_angle = new_sin;
            cos_angle = new_cos;
        }

        let points = points_to_screen(points, viewport, mvp).collect();
        Shape::convex_polygon(points, fill, stroke)
    }
}

fn filled_circle(
    viewport: Rect,
    mvp: DMat4,
    radius: f64,
    fill: impl Into<Color32>,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let points = arc_points(radius, 0.0, TAU);
    let mut points: Vec<_> = points_to_screen(points, viewport, mvp).collect();
    points.pop();
    Shape::convex_polygon(points, fill, stroke.into())
}

fn arc(
    viewport: Rect,
    mvp: DMat4,
    radius: f64,
    start: f64,
    end: f64,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let points = arc_points(radius, start, end);
    let mut points: Vec<_> = points_to_screen(points, viewport, mvp).collect();

    let (first, last) = (points.first(), points.last());
    let closed = (first.zip(last)).filter(|(a, &b)| a.distance(b) < 1e-2);

    if closed.is_some() {
        points.pop();
        Shape::closed_line(points, stroke.into())
    } else {
        Shape::line(points, stroke.into())
    }
}

fn arrow(viewport: Rect, mvp: DMat4, from: DVec3, to: DVec3, stroke: impl Into<Stroke>) -> Shape {
    let stroke = stroke.into();
    let arrow_start = world_to_screen(viewport, mvp, from);
    let arrow_end = world_to_screen(viewport, mvp, to);

    if let Some((start, end)) = arrow_start.zip(arrow_end) {
        let cross = (end - start).normalized().rot90() * stroke.width / 2.0;
        let points = vec![start - cross, start + cross, end];
        Shape::convex_polygon(points, stroke.color, PathStroke::NONE)
    } else {
        Shape::Noop
    }
}

fn line_segment(
    viewport: Rect,
    mvp: DMat4,
    from: DVec3,
    to: DVec3,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let Some(from) = world_to_screen(viewport, mvp, from) else {
        return Shape::Noop;
    };
    let Some(to) = world_to_screen(viewport, mvp, to) else {
        return Shape::Noop;
    };

    let points = [from, to];
    let stroke = stroke.into();
    Shape::LineSegment { points, stroke }
}

fn polygon(
    viewport: Rect,
    mvp: DMat4,
    points: impl IntoIterator<Item = DVec3>,
    fill: impl Into<Color32>,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let points: Vec<_> = points_to_screen(points, viewport, mvp).collect();
    if points.len() > 2 {
        Shape::convex_polygon(points, fill, stroke)
    } else {
        Shape::Noop
    }
}

fn polyline(
    viewport: Rect,
    mvp: DMat4,
    points: impl IntoIterator<Item = DVec3>,
    stroke: impl Into<PathStroke>,
) -> Shape {
    let points: Vec<_> = points_to_screen(points, viewport, mvp).collect();
    if points.len() > 1 {
        Shape::line(points, stroke)
    } else {
        Shape::Noop
    }
}

fn points_to_screen(
    points: impl IntoIterator<Item = DVec3>,
    viewport: Rect,
    mvp: DMat4,
) -> impl Iterator<Item = Pos2> {
    let iter = points.into_iter();
    iter.filter_map(move |point| world_to_screen(viewport, mvp, point))
}

fn steps(angle: f64) -> usize {
    const STEPS_PER_RAD: f64 = 20.0;
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}

fn arc_points(radius: f64, start: f64, end: f64) -> impl Iterator<Item = DVec3> {
    let angle = (end - start).clamp(-TAU, TAU);
    let step_count = steps(angle);
    let step_size = angle / (step_count - 1) as f64;
    (0..step_count).map(move |i| {
        let step = step_size * i as f64;
        let angle = start + step;
        let (z, x) = angle.sin_cos();
        DVec3::new(x * radius, 0.0, z * radius)
    })
}
