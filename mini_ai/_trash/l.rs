/// Monotone quadratic spline interpolation
///
/// This implementation uses the Fritsch-Carlson method to ensure monotonicity.
pub struct MonotoneQuadraticSpline {
    x: Vec<f32>,
    y: Vec<f32>,
    m: Vec<f32>,
}

impl MonotoneQuadraticSpline {
    /// Create a new monotone quadratic spline from a set of points
    pub fn new(mut x: Vec<f32>, mut y: Vec<f64>) -> Self {
        assert_eq!(x.len(), y.len());
        x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        y.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = x.len();
        let mut m = vec![0.0; n];

        // Calculate the slopes
        for i in 1..n - 1 {
            let d1 = (y[i] - y[i - 1]) / (x[i] - x[i - 1]);
            let d2 = (y[i + 1] - y[i]) / (x[i + 1] - x[i]);
            m[i] = (d1 + d2) / 2.0;
        }

        // Apply the Fritsch-Carlson method to ensure monotonicity
        for i in 1..n - 1 {
            let d1 = (y[i] - y[i - 1]) / (x[i] - x[i - 1]);
            let d2 = (y[i + 1] - y[i]) / (x[i + 1] - x[i]);
            if (d1 * d2) < 0.0 {
                m[i] = 0.0;
            } else if d1.abs() < d2.abs() {
                m[i] = d1;
            } else {
                m[i] = d2;
            }
        }

        // Handle the boundary cases
        m[0] = (y[1] - y[0]) / (x[1] - x[0]);
        m[n - 1] = (y[n - 1] - y[n - 2]) / (x[n - 1] - x[n - 2]);

        MonotoneQuadraticSpline { x, y, m }
    }

    /// Evaluate the spline at a given point
    pub fn evaluate(&self, x: f32) -> f64 {
        let n = self.x.len();
        let mut i = 0;
        while i < n - 1 && self.x[i + 1] < x {
            i += 1;
        }

        if x <= self.x[0] {
            return self.y[0] + self.m[0] * (x - self.x[0]);
        } else if x >= self.x[n - 1] {
            return self.y[n - 1] + self.m[n - 1] * (x - self.x[n - 1]);
        }

        let h = self.x[i + 1] - self.x[i];
        let a = (self.x[i + 1] - x) / h;
        let b = (x - self.x[i]) / h;
        let c = (a * a * a - a) * h * h / 6.0;
        let d = (b * b * b - b) * h * h / 6.0;

        a * self.y[i] + b * self.y[i + 1] + c * self.m[i] + d * self.m[i + 1]
    }
}

fn main() {
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let y = vec![0.0, 2.0, 3.0, 2.0, 0.0];
    let spline = MonotoneQuadraticSpline::new(x, y);

    println!("Spline evaluated at 1.5: {}", spline.evaluate(1.5));
    println!("Spline evaluated at 2.5: {}", spline.evaluate(2.5));
}
