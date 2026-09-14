#[derive(Debug, Default)]
pub struct Polygon<T> {
    pub vertices: Vec<(T, [f32; 2])>,
}

impl<T: Copy> Polygon<T> {
    /*
    pub fn triangulate(&mut self, triangles: &mut Vec<T>) {
        if !self.is_ccw() {
            self.vertices.reverse();
        }
        self.triangulate_impl(triangles);
    }
    */

    pub fn triangulate(&mut self, triangles: &mut Vec<T>) {
        while self.vertices.len() > 3 {
            let len = self.vertices.len();
            let mut found_ear = false;

            for index in 0..len {
                let indices = [(index + len - 1) % len, index, (index + 1) % len];
                let positions = indices.map(|index| self.vertices[index].1);
                let mut iter = self.vertices.iter().map(|&(_, p)| p).enumerate();

                let is_ear = is_convex(positions)
                    && iter.all(|(j, p)| indices.contains(&j) || !is_point_inside(positions, p));

                if is_ear {
                    triangles.extend(indices.map(|index| self.vertices[index].0));
                    self.vertices.remove(index);
                    found_ear = true;
                    break;
                }
            }

            if !found_ear {
                break;
            }
        }

        // Add the last triangle
        if let &[(a, _), (b, _), (c, _)] = self.vertices.as_slice() {
            triangles.extend([a, b, c]);
        }
    }

    pub fn is_ccw(&self) -> bool {
        is_ccw(&self.vertices, |p| p.1)
    }

    pub fn project(&mut self, vertices: &[(T, [f32; 3])], outer: usize) -> Option<[f32; 3]> {
        if let Some(normal) = normal(&vertices[0..outer]) {
            self.project_to(vertices, normal);
            Some(normal)
        } else {
            None
        }
    }

    pub fn project_to(&mut self, vertices: &[(T, [f32; 3])], [nx, ny, nz]: [f32; 3]) {
        self.vertices.clear();

        let dd = (nx * nx + ny * ny).sqrt();
        if dd < 1e-15 {
            let iter = vertices.iter();
            if nz > 0.0 {
                // do nothing
                self.vertices
                    .extend(iter.map(|&(data, [dx, dy, _])| (data, [dx, dy])))
            } else {
                // flip
                self.vertices
                    .extend(iter.map(|&(data, [dx, dy, _])| (data, [dy, dx])))
            }
        } else {
            // rotation
            let [ax, ay] = [ny / dd, nx / dd];
            let theta = nz.acos();
            let sin_t = theta.sin();
            let cos_t = theta.cos();
            let inv_cos_t = 1.0 - cos_t;
            let s = -ax * ay * inv_cos_t;
            let [a, b, tx] = [ax * ax * inv_cos_t + cos_t, s, -ay * sin_t];
            let [c, d, ty] = [s, ay * ay * inv_cos_t + cos_t, -ax * sin_t];

            let iter = vertices.iter();
            self.vertices.extend(
                iter.map(|&(data, [x, y, z])| {
                    (data, [x * a + y * b + z * tx, x * c + y * d + z * ty])
                }),
            )
        }
    }
}

#[inline(always)]
fn vadd([ax, ay, az]: [f32; 3], [bx, by, bz]: [f32; 3]) -> [f32; 3] {
    [ax + bx, ay + by, az + bz]
}

#[inline(always)]
fn vsub([ax, ay, az]: [f32; 3], [bx, by, bz]: [f32; 3]) -> [f32; 3] {
    [ax - bx, ay - by, az - bz]
}

fn normal<T>(vertices: &[(T, [f32; 3])]) -> Option<[f32; 3]> {
    // At least 3 vertices required
    if vertices.len() >= 3 {
        let init = ([0.0; 3], vertices[vertices.len() - 1].1);
        let iter = vertices.iter();
        let ([x, y, z], _) = iter.fold(init, |(acc, prev), &(_, v)| {
            let [nx, ny, nz] = vsub(prev, v);
            let [px, py, pz] = vadd(prev, v);
            let p = [ny * pz - nz * py, nz * px - nx * pz, nx * py - ny * px];
            (vadd(acc, p), v)
        });

        let length = (x * x + y * y + z * z).sqrt();
        (length >= 1e-30).then(|| [x, y, z].map(|v| v / length))
    } else {
        None
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_do_nothing() {
        let mut buf = Vec::new();
        let vertices = &[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0]];
        assert!(project3d_to_2d(vertices, 3, &mut buf).is_some());
        assert_eq!(buf, &[[0., 0.], [2., 0.], [2., 2.0]]);
    }

    #[test]
    fn test_flip() {
        let mut buf = Vec::new();
        let vertices = &[[0.0, 0.0, 0.0], [2.0, 2.0, 0.0], [2.0, 0.0, 0.0]];
        assert!(project3d_to_2d(vertices, 3, &mut buf).is_some());
        assert_eq!(buf, &[[0., 0.], [2., 2.], [0., 2.]]);
    }

    #[test]
    fn test_rotate() {
        let mut buf = Vec::new();
        let vertices = &[[0.0, 0.0, 0.0], [0.0, 0.0, 2.0], [0.0, 2.0, 2.0]];
        assert!(project3d_to_2d(vertices, 3, &mut buf).is_some());
        assert_eq!(buf, &[[0., 0.], [2., 0.], [2., 1.9999999]]);
    }

    #[test]
    fn test_invalid_input1() {
        let mut buf = Vec::new();
        let vertices: &[[f32; 3]; 0] = &[];
        assert!(project3d_to_2d(vertices, 0, &mut buf).is_none());
    }

    #[test]
    fn test_invalid_input2() {
        // when normal is zero vector
        let vertices = &[
            [0., 0., 0.],
            [0., 1., 0.],
            [0., 0., 0.],
            [0., 0., 1.],
            [0., 0., 0.],
        ];
        assert!(normal(vertices).is_none());
    }
}
*/

// #[test]
// fn polyfil_main() {
//     let positions = [
//         (0, [0.0, 0.0, 0.0]),
//         (1, [1.0, 0.0, 0.0]),
//         (2, [1.0, 1.0, 0.0]),
//         (3, [0.0, 1.0, 0.0]),
//     ];

//     let mut polygon = PolygonEarClipping::default();
//     polygon.project(&positions, positions.len());

//     let mut triangles = vec![];
//     polygon.triangulate(&mut triangles);
//     for [a, b, c] in triangles {
//         println!("{:?} {:?} {:?}", positions[a], positions[b], positions[c]);
//     }

//     panic!();
// }

pub fn is_ccw<T>(vertices: &[T], map: impl Fn(&T) -> [f32; 2]) -> bool {
    let mut prev_index = vertices.len() - 1;
    let mut sum = 0.0;
    for next_index in 0..vertices.len() {
        let [prev_x, prev_y] = map(&vertices[prev_index]);
        let [next_x, next_y] = map(&vertices[next_index]);
        sum += (next_x - prev_x) * (next_y + prev_y);
        prev_index = next_index;
    }
    sum < 0.0
}

#[inline(always)]
fn is_convex([[ax, ay], [bx, by], [cx, cy]]: [[f32; 2]; 3]) -> bool {
    (bx - ax) * (cy - ay) - (by - ay) * (cx - ax) >= 0.0
}

#[inline(always)]
fn is_point_inside([a, b, c]: [[f32; 2]; 3], p: [f32; 2]) -> bool {
    fn area([ax, ay]: [f32; 2], [bx, by]: [f32; 2], [cx, cy]: [f32; 2]) -> f32 {
        (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
    }

    let ab = area(p, a, b);
    let bc = area(p, b, c);
    let ca = area(p, c, a);

    (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
}
