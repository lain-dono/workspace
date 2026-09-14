use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Curve {
    Step,
    Linear,
    Cubic,
}

impl Curve {
    #[inline]
    pub fn lerp_to<T: Lerp>(self, t: f32, p0: T, p1: T, p2: T, p3: T) -> T {
        match self {
            Self::Step => Lerp::step(p0, p3, t),
            Self::Linear => Lerp::linear(p0, p3, t),
            Self::Cubic => Lerp::cubic(p0, p1, p2, p3, t),
        }
    }
}

pub trait Lerp: Default + Copy + Mul<f32, Output = Self> + Add<Output = Self> {
    #[inline]
    fn step(p0: Self, p1: Self, t: f32) -> Self {
        if t < 1.0 {
            p0
        } else {
            p1
        }
    }

    #[inline]
    fn linear(p0: Self, p1: Self, t: f32) -> Self {
        p0 * (1.0 - t) + p1 * t
    }

    #[inline]
    fn cubic(p0: Self, p1: Self, p2: Self, p3: Self, t: f32) -> Self {
        let h = 1.0 - t;
        let p0 = p0 * (h * h * h);
        let p1 = p1 * (t * h * h * 3.0);
        let p2 = p2 * (t * t * h * 3.0);
        let p3 = p3 * (t * t * t);
        p0 + p1 + p2 + p3
    }
}

impl<T> Lerp for T where T: Default + Copy + Mul<f32, Output = T> + Add<Output = T> {}
