use super::{Painter, Viewport};
use bevy::prelude::*;

fn segment2d_distance(a: Vec2, b: Vec2, point: Vec2) -> f32 {
    let len = a.distance_squared(b);
    point.distance(if len == 0.0 {
        a
    } else {
        a + (b - a) * ((point - a).dot(b - a) / len).clamp(0.0, 1.0)
    })
}

const fn ray(direction: Dir3) -> Ray3d {
    Ray3d {
        origin: Vec3::ZERO,
        direction,
    }
}

pub const RAY_PX: Ray3d = ray(Dir3::X);
pub const RAY_PY: Ray3d = ray(Dir3::Y);
pub const RAY_PZ: Ray3d = ray(Dir3::Z);

pub const RAY_NX: Ray3d = ray(Dir3::NEG_X);
pub const RAY_NY: Ray3d = ray(Dir3::NEG_Y);
pub const RAY_NZ: Ray3d = ray(Dir3::NEG_Z);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxisCap {
    Arrow,
    Cube,
}

#[derive(Clone, Debug)]
pub struct Axis {
    pub transform: Mat4,
    pub ray: Ray3d,
    pub scale: f32,
    pub start: f32,
    pub end: f32,
    pub cap: Option<AxisCap>,
}

impl Axis {
    const AXIS_TIP: f32 = 2.4;

    pub const fn arrow(transform: Mat4, ray: Ray3d, scale: f32, start: f32, end: f32) -> Self {
        Self::new(transform, ray, scale, start, end, Some(AxisCap::Arrow))
    }
    pub const fn arrow_px(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_PX, scale, start, end)
    }
    pub const fn arrow_py(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_PY, scale, start, end)
    }
    pub const fn arrow_pz(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_PZ, scale, start, end)
    }

    pub const fn arrow_nx(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_NX, scale, start, end)
    }
    pub const fn arrow_ny(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_NY, scale, start, end)
    }
    pub const fn arrow_nz(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::arrow(transform, RAY_NZ, scale, start, end)
    }

    pub const fn cube(transform: Mat4, ray: Ray3d, scale: f32, start: f32, end: f32) -> Self {
        Self::new(transform, ray, scale, start, end, Some(AxisCap::Cube))
    }
    pub const fn cube_px(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_PX, scale, start, end)
    }
    pub const fn cube_py(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_PY, scale, start, end)
    }
    pub const fn cube_pz(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_PZ, scale, start, end)
    }

    pub const fn cube_nx(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_NX, scale, start, end)
    }
    pub const fn cube_ny(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_NY, scale, start, end)
    }
    pub const fn cube_nz(transform: Mat4, scale: f32, start: f32, end: f32) -> Self {
        Self::cube(transform, RAY_NZ, scale, start, end)
    }

    pub const fn origin(mut self, origin: Vec3) -> Self {
        self.ray.origin = origin;
        self
    }

    pub const fn new(
        transform: Mat4,
        ray: Ray3d,
        scale: f32,
        start: f32,
        end: f32,
        cap: Option<AxisCap>,
    ) -> Self {
        Self {
            transform,
            ray,
            scale,
            start,
            end,
            cap,
        }
    }

    pub fn distance2d(&self, viewport: &Viewport, point: Vec2) -> Option<f32> {
        let a = self.ray.get_point(self.scale * self.start);
        let b = self.ray.get_point(self.scale * self.end);
        let a = viewport.world_to_viewport(self.transform.transform_point3(a))?;
        let b = viewport.world_to_viewport(self.transform.transform_point3(b))?;
        Some(segment2d_distance(a, b, point))
    }

    pub fn paint(&self, painter: &mut Painter, stroke: impl Into<epaint::Stroke>) {
        let stroke = stroke.into();
        let tip_stroke = epaint::Stroke::new(stroke.width * Self::AXIS_TIP, stroke.color);

        let (start, end) = (self.scale * self.start, self.scale * self.end);
        let p0 = self.ray.get_point(start);
        let p1 = self.ray.get_point(end - self.scale * tip_stroke.width);
        let p2 = self.ray.get_point(end);

        match self.cap {
            Some(AxisCap::Cube) => {
                painter.line(self.transform, p0, p1, stroke);
                painter.line(self.transform, p1, p2, tip_stroke);
            }
            Some(AxisCap::Arrow) => {
                painter.line(self.transform, p0, p1, stroke);
                painter.arrow(self.transform, p1, p2, tip_stroke);
            }
            None => painter.line(self.transform, p0, p2, stroke),
        }
    }
}
