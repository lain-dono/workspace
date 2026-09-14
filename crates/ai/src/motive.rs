use bevy::time::Time;
use std::array::from_fn;

#[derive(Clone, Copy, Debug)]
pub struct Motive {
    pub current: f32,
    pub weight: f32,
    pub curve: Curve<6>,
}

impl Motive {
    #[must_use]
    pub fn new(current: f32, curve: Curve<6>) -> Self {
        Self {
            current,
            weight: 1.0,
            curve,
        }
    }

    #[must_use]
    pub fn with(current: f32, weight: f32, curve: Curve<6>) -> Self {
        Self {
            current,
            weight,
            curve,
        }
    }

    #[must_use]
    pub fn with_keys(current: f32, weight: f32, curve_y: [i8; 6]) -> Self {
        Self {
            current,
            weight,
            curve: Curve::new_keys(curve_y),
        }
    }

    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(0.0, 100.0);
    }

    #[must_use]
    pub fn score(&self, reward: f32) -> f32 {
        self.scorer(self.current, reward)
    }

    #[must_use]
    pub fn scorer(&self, need: f32, reward: f32) -> f32 {
        self.weight * (self.curve.get(need) - self.curve.get(need + reward))
    }
}

impl Motive {
    pub fn tick<T: Default>(&mut self, time: &Time<T>, speed: f32, until: f32) -> bool {
        let speed = speed * time.delta_secs();
        let value: &mut f32 = &mut self.current;
        let next = *value + speed;
        if speed > 0.0 && *value > until || speed < 0.0 && *value < until {
            true
        } else if speed > 0.0 && next > until || speed < 0.0 && next < until {
            *value = until;
            true
        } else {
            *value = next;
            false
        }
    }
}

impl AsRef<f32> for Motive {
    fn as_ref(&self) -> &f32 {
        &self.current
    }
}

impl AsMut<f32> for Motive {
    fn as_mut(&mut self) -> &mut f32 {
        &mut self.current
    }
}

impl std::ops::AddAssign<f32> for Motive {
    fn add_assign(&mut self, rhs: f32) {
        self.set(self.current + rhs);
    }
}

impl std::ops::SubAssign<f32> for Motive {
    fn sub_assign(&mut self, rhs: f32) {
        self.set(self.current - rhs);
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Curve<const N: usize> {
    #[serde(with = "super::arrays")]
    pub x: [u8; N],
    #[serde(with = "super::arrays")]
    pub y: [i8; N],
}

impl<const N: usize> Curve<N> {
    #[must_use]
    pub const fn new(x: [u8; N], y: [i8; N]) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn new_keys(y: [i8; N]) -> Self {
        let x = Self::keys().map(Self::pack_unorm8);
        Self { x, y }
    }

    #[must_use]
    pub fn new_random(mut rng: impl rand::Rng) -> Self {
        Self::new_keys(from_fn(|_| rng.random_range(i8::MIN..0)))
    }

    #[must_use]
    pub fn point(&self, index: usize) -> [f32; 2] {
        let (x, y) = (self.x[index], self.y[index]);
        [Self::unpack_unorm8(x) * 100.0, Self::unpack_snorm8(y)]
    }

    #[must_use]
    pub fn get(&self, value: f32) -> f32 {
        let t = value / 100.0;
        let x = self.x.map(Self::unpack_unorm8);
        let y = self.y.map(Self::unpack_snorm8);
        Self::mono_curve(t, x, y)
    }

    pub fn set_point(&mut self, index: usize, x: f32, y: f32) {
        let m = N - 1;

        let cx = self.x;
        let min = if index == 0 { 0x00 } else { cx[index - 1] + 1 };
        let max = if index == m { 0xFF } else { cx[index + 1] - 1 };

        self.x[index] = Self::pack_unorm8(x / 100.0).clamp(min, max);
        self.y[index] = Self::pack_snorm8(y);
    }

    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn keys() -> [f32; N] {
        from_fn(|i| i as f32 / (N - 1) as f32)
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    const fn pack_unorm8(f: f32) -> u8 {
        (f * 255.0) as u8
    }

    #[inline]
    const fn unpack_unorm8(u: u8) -> f32 {
        u as f32 / 255.0
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    const fn pack_snorm8(f: f32) -> i8 {
        (f.clamp(-1.0, 1.0) * 127.0) as i8
    }

    #[inline]
    const fn unpack_snorm8(i: i8) -> f32 {
        (i as f32) / 127.0
    }

    #[must_use]
    pub fn mono_curve(t: f32, curve_x: [f32; N], curve_y: [f32; N]) -> f32 {
        assert!(N >= 2, "Must have at least 2 control points.");

        let t = t.clamp(0.0, 1.0);

        if t <= curve_x[0] {
            return curve_y[0];
        }
        if t >= curve_x[N - 1] {
            return curve_y[N - 1];
        }

        let mut i = 0;
        while t >= curve_x[i + 1] {
            i += 1;
            if approx::relative_eq!(t, curve_x[i]) {
                return curve_y[i];
            }
        }

        let mut delta = [0.0; N];
        let mut slopes = [0.0; N];

        for i in 0..N - 1 {
            delta[i] = (curve_y[i + 1] - curve_y[i]) / (curve_x[i + 1] - curve_x[i]);
        }

        for i in 1..N - 1 {
            slopes[i] = (delta[i - 1] + delta[i]) * 0.5;
        }

        slopes[0] = delta[0];
        slopes[N - 1] = delta[N - 2];

        for i in 0..N - 1 {
            if delta[i] == 0.0 {
                slopes[i] = 0.0;
                slopes[i + 1] = 0.0;
            } else {
                let alpha = slopes[i] / delta[i];
                let beta = slopes[i + 1] / delta[i];
                let hypot = alpha.hypot(beta);
                if hypot > 9.0 {
                    let t = 3.0 / hypot;
                    slopes[i] = t * alpha * delta[i];
                    slopes[i + 1] = t * beta * delta[i];
                }
            }
        }

        let [ax, bx] = [curve_x[i], curve_x[i + 1]];
        let [ay, by] = [curve_y[i], curve_y[i + 1]];
        let [am, bm] = [slopes[i], slopes[i + 1]];

        let dx = bx - ax;
        let t1 = (t - ax) / dx;
        let t2 = 1.0 - t1;

        let a = (ay * (1.0 + 2.0 * t1) + am * dx * t1) * t2 * t2;
        let b = (by * (3.0 - 2.0 * t1) - bm * dx * t2) * t1 * t1;
        a + b
    }
}
