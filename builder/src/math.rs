use bevy::prelude::*;

pub fn resample_line(start: Vec3, end: Vec3, count: usize) -> impl Iterator<Item = Vec3> {
    let delta = (end - start) / (count - 1) as f32;
    (0..count).map(move |index| start + delta * index as f32)
}

pub struct QuadCurve {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
}

impl QuadCurve {
    pub fn sample(&self, t: f32) -> Vec2 {
        let h = 1.0 - t;
        self.p0 * (h * h) + self.p1 * (2.0 * t * h) + self.p2 * (t * t)
    }

    pub fn derivative(&self, t: f32) -> Vec2 {
        let mt = 1.0 - t;
        let x = 2.0 * (mt * (self.p1.x - self.p0.x) + t * (self.p2.x - self.p1.x));
        let y = 2.0 * (mt * (self.p1.y - self.p0.y) + t * (self.p2.y - self.p1.y));
        Vec2::new(x, y)
    }

    pub fn normal(&self, t: f32) -> Vec2 {
        self.derivative(t).perp().normalize()
    }

    pub fn approx_length(&self, steps: usize) -> f32 {
        pairs(self.lut(steps)).map(|(a, b)| a.distance(b)).sum()
    }

    fn lut(&self, steps: usize) -> impl Iterator<Item = Vec2> + '_ {
        Self::lut_steps(steps).map(|t| self.sample(t))
    }

    fn lut_steps(steps: usize) -> impl Iterator<Item = f32> {
        (0..steps).map(move |i| (i / (steps - 1)) as f32)
    }

    pub fn intervals(&self, interval: f32) -> impl Iterator<Item = (f32, Vec2, Vec2)> {
        assert!(interval.is_finite());
        assert!(interval > 0.0);

        let step = remap(interval, (0.0, self.arclen()), (0.0, 1.0));

        let mut prev_iter = 0.0;
        let mut intervals = vec![(prev_iter, self.sample(prev_iter), self.normal(prev_iter))];

        loop {
            let mut next_iter = prev_iter;
            let mut next_len = 0.0;

            let mut prev_len = 0.0;
            while next_len <= interval {
                prev_iter = next_iter;
                prev_len = next_len;

                next_iter = prev_iter + step;
                next_len += self.sample(prev_iter).distance(self.sample(next_iter));
            }

            prev_iter = remap(interval, (prev_len, next_len), (prev_iter, next_iter));
            intervals.push((prev_iter, self.sample(prev_iter), self.normal(prev_iter)));

            if next_iter >= 1.0 {
                break intervals.into_iter();
            }
        }
    }

    pub fn _intervals(&self, interval: f32) -> Vec<(f32, Vec2, Vec2)> {
        assert!(interval.is_finite());
        assert!(interval > 0.0);

        let step = remap(interval, (0.0, self.arclen()), (0.0, 1.0));

        let mut prev_iter = 0.0;
        let mut prev_base = 0.0;
        let mut intervals = vec![(prev_iter, self.sample(prev_iter), self.normal(prev_iter))];

        loop {
            let next_iter = prev_iter + step;
            let next_base = prev_base + self.sample(prev_iter).distance(self.sample(next_iter));

            if next_base <= interval {
                prev_iter = next_iter;
                prev_base = next_base;
            } else {
                prev_iter = remap(interval, (prev_base, next_base), (prev_iter, next_iter));
                prev_base = 0.0;

                intervals.push((prev_iter, self.sample(prev_iter), self.normal(prev_iter)));

                if next_iter >= 1.0 {
                    break intervals;
                }
            }
        }
    }

    /// Arclength of a quadratic Bézier segment.
    ///
    /// This computation is based on an analytical formula. Since that formula suffers
    /// from numerical instability when the curve is very close to a straight line, we
    /// detect that case and fall back to Legendre-Gauss quadrature.
    ///
    /// Adapted from <http://www.malczak.linuxpl.com/blog/quadratic-bezier-curve-length/>
    /// with permission.
    pub fn arclen(&self) -> f32 {
        let &Self { p0, p1, p2 } = self;

        let d2 = p0 - 2.0 * p1 + p2;
        let a = d2.length_squared();
        let d1 = p1 - p0;
        let c = d1.length_squared();
        if a < 5e-4 * c {
            // This case happens for nearly straight Béziers.
            //
            // Calculate arclength using Legendre-Gauss quadrature using formula from Behdad
            // in https://github.com/Pomax/BezierInfo-2/issues/77
            let v0 = -0.492_943_53 * p0 + 0.430_331_47 * p1 + 0.062_612_034 * p2;
            let v1 = (p2 - p0) * 0.444_444_44;
            let v2 = -0.062_612_034 * p0 - 0.430_331_47 * p1 + 0.492_943_53 * p2;
            v0.length_squared() + v1.length() + v2.length_squared()
        } else {
            let b = 2.0 * d2.dot(d1);

            let sabc = (a + b + c).sqrt();
            let a2 = a.powf(-0.5);
            let a32 = a2.powi(3);
            let c2 = 2.0 * c.sqrt();
            let ba_c2 = b * a2 + c2;

            let v0 = 0.25 * a2 * a2 * b * (2.0 * sabc - c2) + sabc;
            // TODO: justify and fine-tune this exact constant.
            if ba_c2 < 1e-13 {
                // This case happens for Béziers with a sharp kink.
                v0
            } else {
                v0 + 0.25
                    * a32
                    * (4.0 * c * a - b * b)
                    * (((2.0 * a + b) * a2 + 2.0 * sabc) / ba_c2).ln()
            }
        }
    }
}

fn pairs<T: Copy, I: Iterator<Item = T>>(mut iter: I) -> impl Iterator<Item = (T, T)> {
    let mut prev = iter.next();
    std::iter::from_fn(move || {
        let next = iter.next();
        prev.replace(next?).zip(next)
    })
}

pub fn remap(value: f32, old: (f32, f32), new: (f32, f32)) -> f32 {
    ((value - old.0) * (new.1 - new.0)) / (old.1 - old.0) + new.0
}

pub fn select<T>(f: T, t: T, cond: bool) -> T {
    if cond {
        t
    } else {
        f
    }
}

pub trait InputSelect<Input> {
    fn map_pressed<T>(&self, f: T, t: T, input: Input) -> T;
}

impl<Input> InputSelect<Input> for ButtonInput<Input>
where
    Input: Copy + Eq + std::hash::Hash + Send + Sync + 'static,
{
    fn map_pressed<T>(&self, f: T, t: T, input: Input) -> T {
        crate::math::select(f, t, self.pressed(input))
    }
}

#[allow(clippy::upper_case_acronyms)]
pub enum AxisMode {
    XYZ,
    YZX,
    ZXY,

    XZY,
    YXZ,
    ZYX,
}

pub fn axes_to_rotation(mode: AxisMode, a: Dir3, b: Dir3) -> Quat {
    let c = Dir3::new(Vec3::cross(*a, *b)).unwrap();
    let b = Dir3::new(Vec3::cross(*c, *a)).unwrap();
    let (x, y, z, w) = (a, b, c, -c);

    let [a, b, c] = match mode {
        AxisMode::XYZ => [x, y, w],
        AxisMode::YZX => [y, w, x],
        AxisMode::ZXY => [w, x, y],

        AxisMode::XZY => [x, z, y],
        AxisMode::YXZ => [y, x, z],
        AxisMode::ZYX => [z, y, x],
    };

    let m = [a.to_array(), b.to_array(), c.to_array()];
    Quat::from_mat3(&Mat3::from_cols_array_2d(&m))
}
