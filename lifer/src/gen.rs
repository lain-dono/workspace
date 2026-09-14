use bevy::platform::collections::hash_set::HashSet;
use bevy::prelude::*;
use delaunator::Point;
use fast_poisson::Poisson2D;
use noise::NoiseFn;

pub mod dual_mesh;
pub mod mesh_builder;

pub use self::dual_mesh::{DualMesh, IndexMap, MeshInitializer, Region, Side, Triangle};
pub use self::mesh_builder::MeshBuilder;

const RADIUS: f64 = 1.0;
const GRIDSIZE_X: f64 = 250.0 * RADIUS;
const GRIDSIZE_Y: f64 = 250.0 * RADIUS;
const WAVELENGTH: f64 = 0.5;
const GEN_SCALE_X: f64 = 1.0 / GRIDSIZE_X;
const GEN_SCALE_Y: f64 = 1.0 / GRIDSIZE_Y;

pub fn gen_map() -> Mesh {
    let map = Map::new(GRIDSIZE_X, GRIDSIZE_Y, RADIUS, 12345);

    let mut seen = HashSet::new();

    let mut builder = MeshBuilder::default();

    let offset = Vec2::new(GRIDSIZE_X as f32, GRIDSIZE_Y as f32) * 0.5;

    for side in (0..map.mesh.halfedges.len()).map(Side) {
        let region = map.mesh.triangles[side.next()];
        // if map.mesh.is_ghost_r(region) || seen.contains(&region) {
        if seen.contains(&region) {
            continue;
        }

        seen.insert(region);

        let tris = map.mesh.t_around_s(side).collect::<Vec<_>>();

        let s = map.centroids[Triangle::from(side)] - offset;

        for pair in tris[1..].windows(2) {
            if let &[a, b] = pair {
                let a = map.centroids[a] - offset;
                let b = map.centroids[b] - offset;

                // let elevation = (elevation[region] - 0.5) * 2.0;
                let elevation = map.elevation.get(region).copied().unwrap_or(0.0);
                let elevation = elevation * 4.0;

                let position = [
                    [a.x, elevation, a.y],
                    [s.x, elevation, s.y],
                    [b.x, elevation, b.y],
                ];

                builder.add(position, [Vec3::Y.into(); 3], [map.biome_color(region); 3]);
            }
        }
    }

    builder.build()
}

pub struct Map {
    mesh: DualMesh,
    centroids: IndexMap<Triangle, Vec2>,
    elevation: IndexMap<Region, f32>,
    moisture: IndexMap<Region, f32>,
}

impl Map {
    pub fn new(gridsize_x: f64, gridsize_y: f64, radius: f64, seed: u64) -> Self {
        let mut points =
            self::dual_mesh::generate_interior_boundary_points(gridsize_x, gridsize_y, 10.0);

        let num_boundary_points = points.len();

        let noise = Poisson2D::new()
            .with_seed(seed)
            .with_dimensions([gridsize_x, gridsize_y], radius)
            .iter()
            .map(|[x, y]| Point { x, y });

        points.extend(noise);

        // let points: Vec<_> = points.a
        // .collect();

        let elevation = assign_elevation(&points, seed as u32);
        let moisture = assign_moisture(&points, seed as u32);
        let delaunay = delaunator::triangulate(&points);

        let init = MeshInitializer::with_ghost(
            points,
            delaunay.triangles,
            delaunay.halfedges,
            num_boundary_points,
        );
        let mesh = DualMesh::new(init);

        let centroids = calculate_centroids(&mesh);

        Self {
            mesh,
            centroids,
            elevation,
            moisture,
        }
    }

    fn biome_color(&self, region: Region) -> [f32; 4] {
        let e = self.elevation.get(region).copied();
        let m = self.moisture.get(region).copied();
        let Some((e, m)) = e.zip(m) else {
            return [1.0, 0.0, 0.0, 1.0];
        };

        // return [e, e, e, 1.0];

        let e = (e - 0.5) * 2.0;

        // if !(0.0..=1.0).contains(&e) {
        //     return [1.0, 0.0, 0.0, 1.0];
        // }

        if e < 0.0 {
            let r = 48.0 + 48.0 * e;
            let g = 64.0 + 64.0 * e;
            let b = 127.0 + 127.0 * e;
            [r / 255.0, g / 255.0, b / 255.0, 1.0]
        } else {
            let e = (e * 3.0).round() as usize;
            let m = (m * 5.0).round() as usize;

            // println!("E: {:?}, M: {:?}", e, m);

            let (r, g, b) = match (e, m) {
                (3, 3..=5) => (255, 255, 255), // snow
                (3, 2) => (184, 184, 100),     // tundra
                (3, 1) => (100, 100, 100),     // bare
                (3, 0) => (0, 0, 0),           // scorched

                (2, 4 | 5) => (144, 163, 100), // taiga
                (2, 2 | 3) => (124, 144, 100), // shrubland
                (2, 0 | 1) => (200, 209, 139), // temperature desert

                (1, 5) => (33, 124, 46),      // temperature rain forest
                (1, 3 | 4) => (81, 136, 49),  // temperature deciduous forest
                (1, 1 | 2) => (124, 163, 51), // grassland
                (1, 0) => (200, 209, 139),    // temperature desert

                (0, 4 | 5) => (0, 100, 49),  // tropical rain forest
                (0, 2 | 3) => (49, 144, 33), // tropical seasonal forest
                (0, 1) => (124, 163, 51),    // grassland
                (0, 0) => (212, 184, 131),   // subtropical desert

                _ => (255, 0, 0),
            };

            let gamma = 2.2;

            let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            let (r, g, b) = (r.powf(gamma), g.powf(gamma), b.powf(gamma));

            [r, g, b, 1.0]

            /*
            // tweaks
            let m = m * (1.0 - e);
            let e = e * e * e * e;

            r = 210.0 - 100.0 * m;
            g = 185.0 - 45.0 * m;
            b = 139.0 - 45.0 * m;

            r = 255.0 * e + r * (1.0 - e);
            g = 255.0 * e + g * (1.0 - e);
            b = 255.0 * e + b * (1.0 - e);
            [r / 255.0, g / 255.0, b / 255.0, 1.0]
            */
        }
    }
}

fn remap(&Point { x, y }: &Point) -> Point {
    Point {
        x: x * GEN_SCALE_X - 0.5,
        y: y * GEN_SCALE_Y - 0.5,
    }
}

fn noise_map(points: &[Point], seed: u32) -> impl Iterator<Item = f32> + '_ {
    let source = noise::Fbm::<noise::OpenSimplex>::new(seed);
    points.iter().map(move |&Point { x, y }| {
        let x = x * GEN_SCALE_X - 0.5;
        let y = y * GEN_SCALE_Y - 0.5;
        let value = source.get([x / WAVELENGTH, y / WAVELENGTH]) as f32;
        (1.0 + value) / 2.0
    })
}

fn assign_elevation(points: &[Point], seed: u32) -> IndexMap<Region, f32> {
    // let simplex = noise::OpenSimplex::new(seed);

    // let data = noise_map(points, seed)
    // .map(|value| {
    //     // start with noise:
    //     // let value = (1.0 + simplex.get([x / WAVELENGTH, y / WAVELENGTH])) / 2.0;
    //     let value = simplex.get([x / WAVELENGTH, y / WAVELENGTH]);
    //     // modify noise to make islands:
    //     // let d = 2.0 * f64::max(x.abs(), y.abs()); // should be 0-1
    //     // ((1.0 + value - d) / 2.0) as f32
    //     value as f32
    // })
    // .collect();

    IndexMap::from_vec(noise_map(points, seed + 1).collect())
}

fn assign_moisture(points: &[Point], seed: u32) -> IndexMap<Region, f32> {
    IndexMap::from_vec(noise_map(points, seed + 2).collect())

    // // let simplex = noise::OpenSimplex::new(seed);
    // let source = noise::Fbm::<noise::OpenSimplex>::new(seed);

    // // let data = points
    // //     .iter()
    // //     .map(remap)
    // //     // .map(|p| (1.0 + simplex.get([p.x / WAVELENGTH, p.y / WAVELENGTH])) as f32 / 2.0)
    // //     // .map(|p| simplex.get([p.x / WAVELENGTH, p.y / WAVELENGTH]) as f32)
    // //     .map(|p| source.get([p.x, p.y]) as f32)
    // //     .collect();

    // IndexMap::from_vec(data)
}

fn calculate_centroids(mesh: &DualMesh) -> IndexMap<Triangle, Vec2> {
    let num_triangles = mesh.halfedges.len() / 3;
    let data = (0..num_triangles)
        .map(|t| {
            let [a, b, c] = mesh.r_around_t(Triangle(t));

            let a = mesh.pos_of_r(a);
            let b = mesh.pos_of_r(b);
            let c = mesh.pos_of_r(c);

            (a + b + c) / 3.0
        })
        .collect();

    IndexMap::from_vec(data)
}
