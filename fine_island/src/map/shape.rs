use super::{Path, Vector};
use std::f32::consts::PI;

#[derive(Clone, Default)]
pub struct Shape {
    pub paths: Vec<Path>,
}

impl Shape {
    /// Pushes a path onto the end of the Shape
    pub fn push(&mut self, path: Path) {
        self.paths.push(path);
    }

    fn map(&self, f: impl FnMut(Path) -> Path) -> Self {
        Self {
            paths: self.paths.iter().cloned().map(f).collect(),
        }
    }

    /*
    /// Translates a given shape
    pub fn translate(&self, dx: f32, dy: f32, dz: f32) -> Self {
        self.map(|p| p.translate(dx, dy, dz))
    }
    /// Returns a new shape rotated along the X axis by a given origin
    pub fn rotate_x(&self, origin: Point, angle: f32) -> Self {
        self.map(|p| p.rotate_x(origin, angle))
    }
    /// Returns a new shape rotated along the Y axis by a given origin
    pub fn rotate_y(&self, origin: Point, angle: f32) -> Self {
        self.map(|p| p.rotate_y(origin, angle))
    }
    /// Returns a new shape rotated along the Z axis by a given origin
    pub fn rotate_z(&self, origin: Point, angle: f32) -> Self {
        self.map(|p| p.rotate_z(origin, angle))
    }
    /// Scales a shape about a given origin
    pub fn scale(&self, origin: Point, dx: f32, dy: f32, dz: f32) -> Self {
        self.map(|p| p.scale(origin, dx, dy, dz))
    }
    */

    /// Utility function to create a 3D object by raising a 2D path along the z-axis
    pub fn extrude(path: Path, height: f32) -> Self {
        let top = path.clone().translate(0.0, 0.0, height);

        let mut paths = Vec::with_capacity(2 + path.points.len());

        // Push the top and bottom faces, top face must be oriented correctly
        paths.push(path.clone().reverse());
        paths.push(top.clone());

        // Push each side face
        for i in 0..path.points.len() {
            let j = (i + 1) % path.points.len();
            let points = vec![top.points[i], path.points[i], path.points[j], top.points[j]];
            paths.push(Path { points });
        }

        Self { paths }
    }

    // Some shapes to play with

    /// A prism located at origin with dimensions dx, dy, dz
    pub fn prism(origin: Vector, dx: f32, dy: f32, dz: f32) -> Shape {
        // The shape we will return
        let mut paths = vec![];

        // Squares parallel to the x-axis
        let x_face = Path::new(vec![
            Vector::ZERO,
            Vector::new(dx, 0.0, 0.0),
            Vector::new(dx, 0.0, dz),
            Vector::new(0.0, 0.0, dz),
        ]);

        // Square parallel to the y-axis
        let y_face = Path::new(vec![
            Vector::ZERO,
            Vector::new(0.0, 0.0, dz),
            Vector::new(0.0, dy, dz),
            Vector::new(0.0, dy, 0.0),
        ]);

        // Square parallel to the xy-plane
        let xy_face = Path::new(vec![
            Vector::ZERO,
            Vector::new(dx, 0.0, 0.0),
            Vector::new(dx, dy, 0.0),
            Vector::new(0.0, dy, 0.0),
        ]);

        let [x_face, y_face, xy_face] = [x_face, y_face, xy_face].map(|p| p.translate_to(origin));

        // Push this face and its opposite
        paths.push(x_face.clone());
        paths.push(x_face.reverse().translate(0.0, dy, 0.0));

        paths.push(y_face.clone());
        paths.push(y_face.reverse().translate(dx, 0.0, 0.0));

        // This surface is oriented backwards, so we need to reverse the points
        paths.push(xy_face.clone().reverse());
        paths.push(xy_face.translate(0.0, 0.0, dz));

        Self { paths }
    }

    pub fn pyramid(origin: Vector, dx: f32, dy: f32, dz: f32) -> Self {
        let mut paths = vec![];

        let [dx2, dy2] = [dx, dy].map(|p| p * 0.5);
        let anchor = origin + Vector::new(dx2, dy2, 0.0);
        let p = Vector::new(dx2, dy2, dz);

        // Path parallel to the x-axis
        let x_face = Path::new(vec![Vector::ZERO, Vector::new(dx, 0.0, 0.0), p]);
        // Path parallel to the y-axis
        let y_face = Path::new(vec![Vector::ZERO, p, Vector::new(0.0, dy, 0.0)]);

        let [x_face, y_face] = [x_face, y_face].map(|p| p.translate_to(origin));

        // Push the face, and its opposite face, by rotating around the Z-axis
        paths.push(x_face.clone());
        paths.push(x_face.rotate_z(anchor, PI));

        paths.push(y_face.clone());
        paths.push(y_face.rotate_z(anchor, PI));

        Self { paths }
    }

    pub fn cylinder(origin: Vector, radius: f32, vertices: usize, height: f32) -> Self {
        Self::extrude(Path::circle(origin, radius, vertices), height)
    }
}
