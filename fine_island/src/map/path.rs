use super::Vector;
use std::f32::consts::TAU;

#[derive(Clone, Default)]
pub struct Path {
    pub points: Vec<Vector>,
}

impl Path {
    pub const fn new(points: Vec<Vector>) -> Self {
        Self { points }
    }

    /// Pushes a point onto the end of the path
    pub fn push(&mut self, point: Vector) {
        self.points.push(point);
    }

    /// Returns a new path with the points in reverse order
    pub fn reverse(mut self) -> Self {
        self.points.reverse();
        self
    }

    pub fn map(mut self, mut f: impl FnMut(Vector) -> Vector) -> Self {
        self.points.iter_mut().for_each(|p| *p = f(*p));
        self
    }

    /// Translates a given path
    pub fn translate(self, dx: f32, dy: f32, dz: f32) -> Self {
        self.map(|p| p + Vector::new(dx, dy, dz))
    }

    pub fn translate_to(self, delta: Vector) -> Self {
        self.map(|p| p + delta)
    }

    /// Returns a new path rotated along the X axis by a given origin
    pub fn rotate_x(self, origin: Vector, angle: f32) -> Self {
        self.map(|p| p.rotate_x(origin, angle))
    }
    /// Returns a new path rotated along the Y axis by a given origin
    pub fn rotate_y(self, origin: Vector, angle: f32) -> Self {
        self.map(|p| p.rotate_y(origin, angle))
    }
    /// Returns a new path rotated along the Z axis by a given origin
    pub fn rotate_z(self, origin: Vector, angle: f32) -> Self {
        self.map(|p| p.rotate_z(origin, angle))
    }
    /// Scales a path about a given origin
    pub fn scale(self, anchor: Vector, dx: f32, dy: f32, dz: f32) -> Self {
        self.map(|p| p.scale(anchor, dx, dy, dz))
    }

    /// The estimated depth of a path as defined by the average depth of its points
    pub fn depth(&self) -> f32 {
        self.points.iter().fold(0.0, |acc, p| acc + p.depth()) / self.points.len().max(1) as f32
    }

    // Some paths to play with

    /// A rectangle with the bottom-left corner in the origin
    pub fn rectangle(origin: Vector, width: f32, height: f32) -> Self {
        Self {
            points: vec![
                origin,
                Vector::new(origin.x + width, origin.y, origin.z),
                Vector::new(origin.x + width, origin.y + height, origin.z),
                Vector::new(origin.x, origin.y + height, origin.z),
            ],
        }
    }

    /// A circle centered at origin with a given radius and number of vertices
    pub fn circle(origin: Vector, radius: f32, vertices: usize) -> Self {
        Self {
            points: (0..vertices)
                .map(|i| (i as f32 * TAU / vertices as f32).sin_cos())
                .map(|(sn, cs)| origin + Vector::new(radius * cs, radius * sn, 0.0))
                .collect(),
        }
    }

    /*
    /// A star centered at origin with a given outer radius, inner
    /// radius, and number of points
    ///
    /// Buggy - concave polygons are difficult to draw with our method
    Path.Star (origin, outerRadius, innerRadius, points) {
      var i, r, path = new Path();

      for (i = 0; i < points * 2; i++) {
        r = (i % 2 === 0) ? outerRadius : innerRadius;

        path.push(new Point(
          r * Math.cos(i * Math.PI / points),
          r * Math.sin(i * Math.PI / points),
          0));
      }

      return path.translate(origin.x, origin.y, origin.z);
    };

    */
}
