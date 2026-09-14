#[derive(Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl std::ops::Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul for Vector {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl std::ops::Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Vector {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn cross(self, rhs: Self) -> Self {
        let x = self.y * rhs.z - rhs.y * self.z;
        let y = self.x * rhs.z - rhs.x * self.z;
        let z = self.x * rhs.y - rhs.x * self.y;
        Self::new(x, -y, z)
    }

    pub const fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub const fn magnitude_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn magnitude(&self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    pub fn normalize(self) -> Self {
        let magnitude = self.magnitude();
        // If the magnitude is 0 then return the zero vector instead of dividing by 0
        if magnitude.is_finite() && magnitude != 0.0 {
            self / magnitude
        } else {
            Self::new(0.0, 0.0, 0.0)
        }
    }

    /// Scale a point about a given origin
    pub fn scale(self, anchor: Self, dx: f32, dy: f32, dz: f32) -> Self {
        (self - anchor) * Self::new(dx, dy, dz) + anchor
    }

    /// Rotate about origin on the X axis
    pub fn rotate_x(self, origin: Self, angle: f32) -> Self {
        let p = self - origin;
        let [z, y] = rotate(angle, p.z, p.y);
        Self::new(p.x, y, z) + origin
    }

    /// Rotate about origin on the Y axis
    pub fn rotate_y(self, origin: Self, angle: f32) -> Self {
        let p = self - origin;
        let [x, z] = rotate(angle, p.x, p.z);
        Self::new(x, p.y, z) + origin
    }

    /// Rotate about origin on the Z axis
    pub fn rotate_z(self, origin: Self, angle: f32) -> Self {
        let p = self - origin;
        let [x, y] = rotate(angle, p.x, p.y);
        Self::new(x, y, p.z) + origin
    }

    /// The depth of a point in the isometric plane
    pub fn depth(&self) -> f32 {
        // z is weighted slightly to accomodate |_ arrangements
        self.x + self.y - 2.0 * self.z
    }

    /// Distance between two points
    pub fn distance(p1: Self, p2: Self) -> f32 {
        (p2 - p1).magnitude()
    }
}

fn rotate(angle: f32, a: f32, b: f32) -> [f32; 2] {
    let (sn, cs) = angle.sin_cos();
    [a * cs - b * sn, a * sn + b * cs]
}
