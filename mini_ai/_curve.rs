use core::array::from_fn;
use core::f32::consts::PI;

#[derive(Clone, Copy)]
pub struct CurveFloat<const N: usize>(pub [f32; N]);

impl<const N: usize> CurveFloat<N> {
    pub fn from_fn<F: FnMut(usize) -> f32>(cb: F) -> Self {
        Self(from_fn::<f32, N, F>(cb))
    }

    pub fn linear(&self, t: f32) -> f32 {
        spline::<N, _, _>(t, |i| [key(i, N), self.0[i]], lerp)
    }
}

macro_rules! make_curve {
    ($name:ident, $ty:ty, $pack:ident, $unpack:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name<const N: usize>(pub [$ty; N]);

        impl<const N: usize> std::fmt::Debug for $name<N> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(&format!("$name<{N}>")).finish()
            }
        }

        impl<const N: usize> $name<N> {
            pub fn from_fn<F: FnMut(usize) -> $ty>(cb: F) -> Self {
                Self(from_fn::<$ty, N, F>(cb))
            }

            pub fn from_map<F: FnMut(f32) -> f32>(mut cb: F) -> Self {
                Self(from_fn::<f32, N, _>(|i| cb(key(i, N))).map($pack))
            }
        }
    };
}

make_curve!(CurveSnorm8, i8, pack_snorm8, unpack_snorm8);
make_curve!(CurveUnorm8, u8, pack_unorm8, unpack_unorm8);

#[inline]
fn key(i: usize, n: usize) -> f32 {
    i as f32 / (n - 1) as f32
}

#[inline]
pub fn keys<const N: usize>() -> [f32; N] {
    from_fn(|i| key(i, N))
}

#[inline(always)]
pub const fn pack_unorm8(f: f32) -> u8 {
    (f * 255.0) as u8
}

#[inline(always)]
pub const fn unpack_unorm8(u: u8) -> f32 {
    u as f32 / 255.0
}

#[inline(always)]
pub const fn pack_snorm8(f: f32) -> i8 {
    (f32::clamp(f, -1.0, 1.0) * 127.0) as i8
}

#[inline(always)]
pub const fn unpack_snorm8(i: i8) -> f32 {
    (i as f32) / 127.0
}

#[inline]
fn spline<const N: usize, F, I>(t: f32, cb: F, i: I) -> f32
where
    F: Fn(usize) -> [f32; 2],
    I: Fn(f32, f32, f32) -> f32,
{
    let points = from_fn::<_, N, _>(cb);
    let index = points.partition_point(|&[x, _]| x < t);

    if index == 0 {
        points[0][1]
    } else if index == N {
        points[index - 1][1]
    } else {
        let ([sx, sy], [ex, ey]) = (points[index - 1], points[index]);
        i(map_range01(t, sx, ex), sy, ey)
    }
}

#[inline(always)]
const fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[inline(always)]
fn cosine(t: f32, a: f32, b: f32) -> f32 {
    lerp((1.0 - (t * PI).cos()) * 0.5, a, b)
}

#[inline(always)]
const fn smoothstep(t: f32, a: f32, b: f32) -> f32 {
    lerp(t * t * (3.0 - 2.0 * t), a, b)
}

#[inline(always)]
const fn smootherstep(t: f32, a: f32, b: f32) -> f32 {
    lerp(t * t * t * (t * (6.0 * t - 15.0) + 10.0), a, b)
}

#[inline(always)]
const fn map_range(t: f32, from: [f32; 2], to: [f32; 2]) -> f32 {
    let [[a0, a1], [b0, b1]] = [from, to];
    b0 + (t - a0) * (b1 - b0) / (a1 - a0)
}

#[inline(always)]
const fn map_range01(s: f32, a0: f32, a1: f32) -> f32 {
    (s - a0) / (a1 - a0)
}

pub fn mono_curve_unorm8<const N: usize>(t: f32, y: [u8; N]) -> f32 {
    crate::ai::MotiveCurve::<N>::mono_curve(t, keys::<N>(), y.map(unpack_unorm8))
}

/// Monotone cubic interpolation of values y0 and y1 using x as the interpolation
/// parameter (assumed to be [0..1]). A modification of hermite cubic interpolation
/// that prevents overshoots (preserves monoticity). In order to both maintain
/// monotonicity and C1 continuity, two neighbouring samples to the left of y0 and
/// the right of y1 are also necessary
pub fn interpolate_cubic_monotonic_heckbert(
    x: f32,
    y_minus_1: f32,
    y0: f32,
    y1: f32,
    y2: f32,
) -> f32 {
    // Calculate secant line gradients for each successive pair of data points
    let s_minus_1 = y0 - y_minus_1;
    let s_0 = y1 - y0;
    let s_1 = y2 - y1;

    // Use central differences to calculate initial gradients at the end-points
    let mut m_0 = (s_minus_1 + s_0) * 0.5;
    let mut m_1 = (s_0 + s_1) * 0.5;

    // If the central curve (joining y0 and y1) is neither increasing or decreasing, we
    // should have a horizontal line, so immediately set gradients to zero here.
    if approx::relative_eq!(y0, y1) {
        m_0 = 0.0;
        m_1 = 0.0;
    } else {
        // If the curve to the left is horizontal, or the sign of the secants on either side
        // of the end-point are different, set the gradient to zero...
        if approx::relative_eq!(y_minus_1, y0)
            || s_minus_1 < 0.0 && s_0 >= 0.0
            || s_minus_1 > 0.0 && s_0 <= 0.0
        {
            m_0 = 0.0;
        } else {
            // ... otherwise, ensure the magnitude of the gradient is constrained to 3 times the
            // left secant, and 3 times the right secant (whatever is smaller)
            m_0 *= (3.0 * s_minus_1 / m_0).min(3.0 * s_0 / m_0).min(1.0);
        }

        // If the curve to the right is horizontal, or the sign of the secants on either side
        // of the end-point are different, set the gradient to zero...
        if approx::relative_eq!(y1, y2) || s_0 < 0.0 && s_1 >= 0.0 || s_0 > 0.0 && s_1 <= 0.0 {
            m_1 = 0.0;
        } else {
            // ... otherwise, ensure the magnitude of the gradient is constrained to 3 times the
            // left secant, and 3 times the right secant (whatever is smaller)
            m_1 *= (3.0 * s_0 / m_1).min(3.0 * s_1 / m_1).min(1.0);
        }
    }

    // Evaluate the cubic hermite spline
    let a = (m_0 + m_1 - 2.0 * s_0) * x;
    let b = 3.0 * s_0 - 2.0 * m_0 - m_1;
    let result = (((a + b) * x) + m_0) * x + y0;

    // The values at the end points (y0 and y1) define an interval that the curve passes
    // through. Since the curve between the end-points is now monotonic, all interpolated
    // values between these end points should be inside this interval. However, floating
    // point rounding error can still lead to values slightly outside this range.
    // Guard against this by clamping the interpolated result to this interval...
    let min = y0.min(y1);
    let max = y0.max(y1);
    result.min(max).max(min)
}
