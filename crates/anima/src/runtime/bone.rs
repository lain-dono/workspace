use super::{util, Color, Matrix};
use std::ops::{Add, Mul, MulAssign};

pub struct Bone {
    pub transform: Transform,
    pub color: Color,
    pub length: f32,
    pub parent: u32,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            color: Color::default(),
            length: 0.0,
            parent: u32::MAX,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename = "Bone")]
pub struct BoneData {
    pub name: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parent: String,

    #[serde(default, skip_serializing_if = "util::is_default")]
    pub skin: bool,

    #[serde(default, skip_serializing_if = "util::is_zero_f32")]
    pub length: f32,
    #[serde(default, skip_serializing_if = "util::is_zero_f32")]
    pub rotate: f32,
    #[serde(default, skip_serializing_if = "util::is_zero_f32x2")]
    pub translate: [f32; 2],
    #[serde(default, skip_serializing_if = "util::is_one_f32x2")]
    pub scale: [f32; 2],
    #[serde(default, skip_serializing_if = "util::is_zero_f32x2")]
    pub shear: [f32; 2],

    #[serde(default, skip_serializing_if = "util::is_default")]
    pub color: Color,
}

impl BoneData {
    pub fn transform(&self) -> Transform {
        Transform {
            rotate: self.rotate,
            translate: self.translate,
            scale: self.scale,
            shear: self.shear,
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.rotate = transform.rotate;
        self.translate = transform.translate;
        self.scale = transform.scale;
        self.shear = transform.shear;
    }
}

impl Default for BoneData {
    fn default() -> Self {
        Self {
            name: String::default(),
            parent: String::default(),
            skin: false,

            length: 0.0,
            rotate: 0.0,
            translate: [0.0; 2],
            scale: [1.0; 2],
            shear: [0.0; 2],
            //TODO inherit: String::default(),
            color: Color(0x989898FF),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub rotate: f32,
    pub translate: [f32; 2],
    pub scale: [f32; 2],
    pub shear: [f32; 2],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            rotate: 0.0,
            translate: [0.0; 2],
            scale: [1.0; 2],
            shear: [0.0; 2],
        }
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_transform(rhs)
    }
}

impl MulAssign for Transform {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul_transform(rhs)
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix {
        let (sx, cx) = (self.rotate - self.shear[0]).sin_cos();
        let (sy, cy) = (self.rotate + self.shear[1]).sin_cos();
        let sx = -sx;

        Matrix {
            a: cy * self.scale[0],
            b: sy * self.scale[0],
            c: sx * self.scale[1],
            d: cx * self.scale[1],
            tx: self.translate[0],
            ty: self.translate[1],
        }
    }

    pub fn inverse(&self) -> Self {
        Self {
            translate: [-self.translate[0], -self.translate[1]],
            rotate: -self.rotate,
            scale: [self.scale[0].recip(), self.scale[1].recip()],
            shear: [-self.shear[0], -self.shear[1]],
        }
    }

    pub fn mul_transform(&self, transform: Self) -> Self {
        fn map(a: [f32; 2], map: impl Fn(f32, f32) -> f32, b: [f32; 2]) -> [f32; 2] {
            [map(a[0], b[0]), map(a[1], b[1])]
        }

        Self {
            translate: self.apply_impl(transform.translate),
            rotate: self.rotate + transform.rotate,
            scale: map(self.scale, Mul::mul, transform.scale),
            shear: map(self.shear, Add::add, transform.shear),
        }
    }

    pub fn apply<I: Into<[f32; 2]>, F: From<[f32; 2]>>(&self, coord: I) -> F {
        self.apply_impl(coord.into()).into()
    }

    pub fn apply_impl(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        self.to_matrix().apply(x, y)
    }
}
