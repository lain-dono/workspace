#[inline(always)]
fn vadd([ax, ay, az]: [f32; 3], [bx, by, bz]: [f32; 3]) -> [f32; 3] {
    [ax + bx, ay + by, az + bz]
}

#[inline(always)]
fn vsub([ax, ay, az]: [f32; 3], [bx, by, bz]: [f32; 3]) -> [f32; 3] {
    [ax - bx, ay - by, az - bz]
}

fn normal(vertices: &[[f32; 3]]) -> Option<[f32; 3]> {
    // At least 3 vertices required
    if vertices.len() >= 3 {
        let init = ([0.0; 3], vertices[vertices.len() - 1]);
        let iter = vertices.iter();
        let ([x, y, z], _) = iter.fold(init, |(acc, prev), &v| {
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

pub fn project3d_to_2d(
    vertices: &[[f32; 3]],
    outer_count: usize,
    output: &mut Vec<[f32; 2]>,
) -> Option<[f32; 3]> {
    if let Some(normal) = normal(&vertices[0..outer_count]) {
        project_to_plane(vertices, normal, output);
        Some(normal)
    } else {
        None
    }
}

pub fn project_to_plane(vertices: &[[f32; 3]], [nx, ny, nz]: [f32; 3], output: &mut Vec<[f32; 2]>) {
    output.clear();

    let dd = (nx * nx + ny * ny).sqrt();
    if dd < 1e-15 {
        if nz > 0.0 {
            // do nothing
            output.extend(vertices.iter().map(|&[dx, dy, _]| [dx, dy]))
        } else {
            // flip
            output.extend(vertices.iter().map(|&[dx, dy, _]| [dy, dx]))
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
        output.extend(iter.map(|&[x, y, z]| [x * a + y * b + z * tx, x * c + y * d + z * ty]))
    }
}

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
