use std::ops::{Add, Mul, Sub};

#[derive(Default, Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl From<[f32; 2]> for Offset {
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl From<Offset> for [f32; 2] {
    fn from(Offset { x, y }: Offset) -> Self {
        [x, y]
    }
}

impl Add for Offset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.map2(Add::add, rhs)
    }
}

impl Sub for Offset {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.map2(Sub::sub, rhs)
    }
}

impl Mul<f32> for Offset {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(Mul::mul, rhs)
    }
}

impl Offset {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    #[inline(always)]
    fn map(self, map: impl Fn(f32, f32) -> f32, rhs: f32) -> Self {
        Self {
            x: map(self.x, rhs),
            y: map(self.y, rhs),
        }
    }

    #[inline(always)]
    fn map2(self, map: impl Fn(f32, f32) -> f32, rhs: Self) -> Self {
        Self {
            x: map(self.x, rhs.x),
            y: map(self.y, rhs.y),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Matrix {
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);

    pub const fn new(a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32) -> Self {
        Self { a, b, c, d, tx, ty }
    }

    #[inline]
    pub fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    #[inline]
    pub fn invert(&self) -> Self {
        let n = (self.a * self.d - self.b * self.c).recip();
        Self {
            a: self.d / n,
            b: -self.b / n,
            c: -self.c / n,
            d: self.a / n,
            tx: (self.c * self.ty - self.d * self.tx) / n,
            ty: (self.b * self.tx - self.a * self.ty) / n,
        }
    }

    /// Appends the given Matrix to this Matrix.
    #[inline]
    pub fn append(self, rhs: Self) -> Self {
        Self::concat(self, rhs)
    }

    /// Prepends the given Matrix to this Matrix.
    #[inline]
    pub fn prepend(self, lhs: Self) -> Self {
        Self::concat(lhs, self)
    }

    #[inline(always)]
    fn concat(lhs: Self, rhs: Self) -> Self {
        Self {
            a: lhs.a * rhs.a + lhs.b * rhs.c,
            b: lhs.a * rhs.b + lhs.b * rhs.d,
            c: lhs.c * rhs.a + lhs.d * rhs.c,
            d: lhs.c * rhs.b + lhs.d * rhs.d,
            tx: lhs.tx * rhs.a + lhs.ty * rhs.c + rhs.tx,
            ty: lhs.tx * rhs.b + lhs.ty * rhs.d + rhs.ty,
        }
    }
}
