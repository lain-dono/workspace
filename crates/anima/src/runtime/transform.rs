#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AffineMatrix {
    pub sx: f32,
    pub shy: f32,
    pub shx: f32,
    pub sy: f32,

    pub tx: f32,
    pub ty: f32,
}

impl Default for AffineMatrix {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul<Self> for AffineMatrix {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        Self::multiply(&self, &other)
    }
}

impl std::ops::MulAssign<Self> for AffineMatrix {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = Self::multiply(self, &other);
    }
}

impl AffineMatrix {
    pub const fn new(sx: f32, shx: f32, tx: f32, shy: f32, sy: f32, ty: f32) -> Self {
        Self {
            sx,
            shx,
            tx,

            shy,
            sy,
            ty,
        }
    }

    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0)
    }

    pub const fn translate(tx: f32, ty: f32) -> Self {
        Self::new(1.0, 0.0, tx, 0.0, 1.0, ty)
    }

    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self::new(sx, 0.0, 0.0, 0.0, sy, 0.0)
    }

    pub const fn shear(shx: f32, shy: f32) -> Self {
        Self::new(1.0, shx, 0.0, shy, 1.0, 0.0)
    }

    pub const fn from_mat2x2([sx, shy, shx, sy]: [f32; 4]) -> Self {
        Self::new(sx, shx, 0.0, shy, sy, 0.0)
    }

    #[inline]
    pub fn skew_x(theta: f32) -> Self {
        Self::shear(theta.tan(), 0.0)
    }

    #[inline]
    pub fn skew_y(theta: f32) -> Self {
        Self::shear(0.0, theta.tan())
    }

    #[inline]
    pub fn rotate(theta: f32) -> Self {
        let (sn, cs) = theta.sin_cos();
        Self::new(cs, -sn, 0.0, sn, cs, 0.0)
    }

    #[inline(always)]
    fn multiply(lhs: &Self, rhs: &Self) -> Self {
        Self {
            sx: lhs.sx * rhs.sx + lhs.shx * rhs.shy,
            shy: lhs.shy * rhs.sx + lhs.sy * rhs.shy,
            shx: lhs.sx * rhs.shx + lhs.shx * rhs.sy,
            sy: lhs.shy * rhs.shx + lhs.sy * rhs.sy,
            tx: lhs.sx * rhs.tx + lhs.shx * rhs.ty + lhs.tx,
            ty: lhs.shy * rhs.tx + lhs.sy * rhs.ty + lhs.ty,
        }
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        self.adjugate().mul_scalar(self.determinant().recip())
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.sx * self.sy - self.shy * self.shx
    }

    #[inline]
    pub fn adjugate(&self) -> Self {
        Self {
            sx: self.sy,
            shy: -self.shy,
            shx: -self.shx,
            sy: self.sx,
            tx: self.shx * self.ty - self.sy * self.tx,
            ty: self.shy * self.tx - self.sx * self.ty,
        }
    }

    #[inline]
    fn mul_scalar(&self, n: f32) -> Self {
        Self {
            sx: self.sx * n,
            shy: self.shy * n,
            shx: self.shx * n,
            sy: self.sy * n,
            tx: self.tx * n,
            ty: self.ty * n,
        }
    }

    #[inline]
    pub fn apply<I: Into<[f32; 2]>, F: From<[f32; 2]>>(&self, coord: I) -> F {
        self.apply_impl(coord.into()).into()
    }

    #[inline]
    pub fn apply_impl(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let xp = y * self.shx + x * self.sx + self.tx;
        let yp = x * self.shy + y * self.sy + self.ty;
        [xp, yp]
    }

    pub fn combine(rotate: f32, scale: [f32; 2], skew: [f32; 2], translate: [f32; 2]) -> Self {
        let (sn_x, cs_x) = (rotate + skew[0]).sin_cos();
        let (sn_y, cn_y) = (rotate + skew[1]).sin_cos();

        Self {
            sx: cn_y * scale[0],
            shy: sn_y * scale[0],
            shx: -sn_x * scale[1],
            sy: cs_x * scale[1],
            tx: translate[0],
            ty: translate[1],
        }
    }
}
