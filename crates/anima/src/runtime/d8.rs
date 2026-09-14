// Your friendly neighbour https://en.wikipedia.org/wiki/Dihedral_group
//
// This file implements the dihedral group of order 16, also called
// of degree 8. That's why its called groupD8.

/// Implements the dihedral group D8, which is similar to
/// [group D4]{@link http://mathworld.wolfram.com/DihedralGroupD4.html};
/// D8 is the same but with diagonals, and it is used for texture
/// rotations.
//
/// The directions the U- and V- axes after rotation
/// of an angle of `a: GD8Constant` are the vectors `(uX(a), uY(a))`
/// and `(vX(a), vY(a))`. These aren't necessarily unit vectors.
//
/// **Origin:**
/// This is the small part of gameofbombs.com portal system. It works.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct D8(pub u8);

impl std::ops::Add for D8 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::compose(self, rhs)
    }
}

impl std::ops::Sub for D8 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::compose(self, rhs.inverse())
    }
}

impl D8 {
    /// East 0°
    pub const E: Self = Self(0b0000);

    /// Southeast 45°↻
    pub const SE: Self = Self(0b0001);

    /// South 90°↻
    pub const S: Self = Self(0b0010);

    /// Southwest 135°↻
    pub const SW: Self = Self(0b0011);

    /// West 180°
    pub const W: Self = Self(0b0100);

    /// Northwest -135°/225°↻
    pub const NW: Self = Self(0b0101);

    /// North -90°/270°↻
    pub const N: Self = Self(0b0110);

    /// Northeast -45°/315°↻
    pub const NE: Self = Self(0b0111);

    /// Reflection about Y-axis.
    pub const REFLECTION_Y: Self = Self(0b1000);

    /// Reflection about the main diagonal.
    pub const MAIN_DIAGONAL: Self = Self(0b1010);

    /// Reflection about X-axis.
    pub const REFLECTION_X: Self = Self(0b1100);

    /// Reflection about reverse diagonal.
    pub const REVERSE_DIAGONAL: Self = Self(0b1110);

    pub const UX: [i8; 16] = [1, 1, 0, -1, -1, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1];
    pub const UY: [i8; 16] = [0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 1, 1, 0, -1, -1, -1];
    pub const VX: [i8; 16] = [0, -1, -1, -1, 0, 1, 1, 1, 0, 1, 1, 1, 0, -1, -1, -1];
    pub const VY: [i8; 16] = [1, 1, 0, -1, -1, -1, 0, 1, -1, -1, 0, 1, 1, 1, 0, -1];

    /// [Cayley Table](https://en.wikipedia.org/wiki/Cayley_table)
    /// for the composition of each rotation in the dihederal group D8.
    pub const CAYLEY: [[u8; 16]; 16] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [1, 2, 3, 4, 5, 6, 7, 0, 9, 10, 11, 12, 13, 14, 15, 8],
        [2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9],
        [3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13, 14, 15, 8, 9, 10],
        [4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11],
        [5, 6, 7, 0, 1, 2, 3, 4, 13, 14, 15, 8, 9, 10, 11, 12],
        [6, 7, 0, 1, 2, 3, 4, 5, 14, 15, 8, 9, 10, 11, 12, 13],
        [7, 0, 1, 2, 3, 4, 5, 6, 15, 8, 9, 10, 11, 12, 13, 14],
        [8, 15, 14, 13, 12, 11, 10, 9, 0, 7, 6, 5, 4, 3, 2, 1],
        [9, 8, 15, 14, 13, 12, 11, 10, 1, 0, 7, 6, 5, 4, 3, 2],
        [10, 9, 8, 15, 14, 13, 12, 11, 2, 1, 0, 7, 6, 5, 4, 3],
        [11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4],
        [12, 11, 10, 9, 8, 15, 14, 13, 4, 3, 2, 1, 0, 7, 6, 5],
        [13, 12, 11, 10, 9, 8, 15, 14, 5, 4, 3, 2, 1, 0, 7, 6],
        [14, 13, 12, 11, 10, 9, 8, 15, 6, 5, 4, 3, 2, 1, 0, 7],
        [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
    ];

    /// Composes the two D8 operations.
    ///
    /// Taking `^` as reflection:
    ///
    /// |       | E=0 | S=2 | W=4 | N=6 | E^=8 | S^=10 | W^=12 | N^=14 |
    /// |-------|-----|-----|-----|-----|------|-------|-------|-------|
    /// | E=0   | E   | S   | W   | N   | E^   | S^    | W^    | N^    |
    /// | S=2   | S   | W   | N   | E   | S^   | W^    | N^    | E^    |
    /// | W=4   | W   | N   | E   | S   | W^   | N^    | E^    | S^    |
    /// | N=6   | N   | E   | S   | W   | N^   | E^    | S^    | W^    |
    /// | E^=8  | E^  | N^  | W^  | S^  | E    | N     | W     | S     |
    /// | S^=10 | S^  | E^  | N^  | W^  | S    | E     | N     | W     |
    /// | W^=12 | W^  | S^  | E^  | N^  | W    | S     | E     | N     |
    /// | N^=14 | N^  | W^  | S^  | E^  | N    | W     | S     | E     |
    pub const fn compose(self, rhs: Self) -> Self {
        Self(Self::CAYLEY[self.0 as usize][rhs.0 as usize])
    }

    /// Transform matrix for operation n is:
    /// `[ux, uy, vx, vy]`
    pub const fn matrix(self) -> [f32; 4] {
        [self.ux(), self.uy(), self.vx(), self.vy()]
    }

    /// The X-component of the U-axis after rotating the axes.
    pub const fn ux(self) -> f32 {
        Self::UX[self.0 as usize] as f32
    }

    /// The Y-component of the U-axis after rotating the axes.
    pub const fn uy(self) -> f32 {
        Self::UY[self.0 as usize] as f32
    }

    /// The X-component of the V-axis after rotating the axes.
    pub const fn vx(self) -> f32 {
        Self::VX[self.0 as usize] as f32
    }

    /// The Y-component of the V-axis after rotating the axes.
    pub const fn vy(self) -> f32 {
        Self::VY[self.0 as usize] as f32
    }

    /// The opposite symmetry of `rotation`
    pub const fn inverse(self) -> Self {
        // true only if between 8 & 15 (reflections)
        if self.0 & 8 != 0 {
            Self(self.0 % 16)
        } else {
            Self(8 - self.0 % 8)
        }
    }

    /// Adds 180 degrees to rotation, which is a commutative operation.
    pub const fn rotate180(self) -> Self {
        Self(self.0 ^ 4)
    }

    /// Checks if the rotation angle is vertical, i.e. south or north.
    ///
    /// It doesn't work for reflections.
    pub const fn is_vertical(self) -> bool {
        (self.0 % 4) == 2
    }

    /// Approximates the vector into one of the eight directions provided by [`GD8`].
    pub fn by_direction(dx: f32, dy: f32) -> Self {
        let (ax, ay) = (dx.abs(), dy.abs());

        let sn = ax * 2.0 <= ay;
        let ew = ay * 2.0 <= ax;

        match (dx >= 0.0, dy >= 0.0) {
            (_, true) if sn => Self::S,
            (_, false) if sn => Self::N,
            (true, _) if ew => Self::E,
            (false, _) if ew => Self::W,

            (true, true) => Self::SE,
            (false, true) => Self::SW,
            (true, false) => Self::NE,
            (false, false) => Self::NW,
        }
    }

    /// `(0 0) (1 0) (1 1) (0 1)`
    pub const fn default_uvs() -> [[f32; 2]; 4] {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    /// | Corner       | Coordinates |
    /// |--------------|-------------|
    /// | Top-Left     | `(x0, y0)`  |
    /// | Top-Right    | `(x1, y1)`  |
    /// | Bottom-Right | `(x2, y2)`  |
    /// | Bottom-Left  | `(x3, y3)`  |
    pub fn uvs(self, [x, y, w, h]: [f32; 4], [tw, th]: [f32; 2]) -> [[f32; 2]; 4] {
        let (dx, dy) = (w / 2.0 / tw, h / 2.0 / th);
        let (cx, cy) = (x / tw + dx, y / th + dy);

        let r0 = self + Self::NW; // top-left
        let r1 = self + Self::NE; // top-rigth
        let r2 = self + Self::SE; // bottom-right
        let r3 = self + Self::SW; // bottom-left

        [
            [cx + dx * r0.ux(), cy + dy * r0.uy()],
            [cx + dx * r1.ux(), cy + dy * r1.uy()],
            [cx + dx * r2.ux(), cy + dy * r2.uy()],
            [cx + dx * r3.ux(), cy + dy * r3.uy()],
        ]
    }
}

#[cfg(test)]
#[test]
#[allow(clippy::needless_range_loop)]
fn init_cayley() {
    let mut cayley: [[u8; 16]; 16] = [[0xFF; 16]; 16];

    for i in 0..16 {
        for j in 0..16 {
            let ux = (D8::UX[i] * D8::UX[j] + D8::VX[i] * D8::UY[j]).signum();
            let uy = (D8::UY[i] * D8::UX[j] + D8::VY[i] * D8::UY[j]).signum();
            let vx = (D8::UX[i] * D8::VX[j] + D8::VX[i] * D8::VY[j]).signum();
            let vy = (D8::UY[i] * D8::VX[j] + D8::VY[i] * D8::VY[j]).signum();

            for k in 0..16 {
                if D8::UX[k] == ux && D8::UY[k] == uy && D8::VX[k] == vx && D8::VY[k] == vy {
                    cayley[i][j] = k as u8;
                    break;
                }
            }
        }
    }

    assert_eq!(D8::CAYLEY, cayley);
}
