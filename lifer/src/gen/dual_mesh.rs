use bevy::{platform::collections::hash_map::HashMap, prelude::*};
use delaunator::Point;
use std::ops::{Index, IndexMut};

pub struct IndexMap<K, V> {
    data: Vec<V>,
    marker: std::marker::PhantomData<K>,
}

impl<K, V: Clone> IndexMap<K, V> {
    pub const fn empty() -> Self {
        Self {
            data: vec![],
            marker: std::marker::PhantomData,
        }
    }

    pub fn new(default: V, len: usize) -> Self {
        Self {
            data: vec![default; len],
            marker: std::marker::PhantomData,
        }
    }

    pub const fn from_vec(data: Vec<V>) -> Self {
        Self {
            data,
            marker: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, k: K) -> Option<&V>
    where
        K: Into<usize>,
    {
        self.data.get(k.into())
    }
}

impl<K: Into<usize>, V> Index<K> for IndexMap<K, V> {
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        &self.data[index.into()]
    }
}

impl<K: Into<usize>, V> IndexMut<K> for IndexMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.data[index.into()]
    }
}

/// Each initial point generates one region, where
/// points.slice(0, numBoundaryPoints) are considered to be
/// boundary points/regions.
///
/// As created from Delaunator, the mesh has some sides without pairs.
/// Optionally use TriangleMesh.addGhostStructure() to add "ghost"
/// sides, triangles, and region to complete the mesh. Elements that
/// aren't "ghost" are called "solid".
pub struct MeshInitializer {
    points: Vec<Vec2>,
    halfedges: IndexMap<Side, Side>,
    triangles: IndexMap<Side, Region>,
    num_boundary_points: usize,
    num_solid_sides: usize,
}

impl MeshInitializer {
    pub fn new(points: Vec<Point>, triangles: Vec<usize>, halfedges: Vec<usize>) -> Self {
        let points = points
            .into_iter()
            .map(|Point { x, y }| Vec2::new(x as f32, y as f32))
            .collect::<Vec<_>>();
        let triangles = triangles.into_iter().map(Region).collect::<Vec<_>>();
        let halfedges = halfedges.into_iter().map(Side).collect::<Vec<_>>();

        let triangles = IndexMap::<Side, Region>::from_vec(triangles);
        let halfedges = IndexMap::<Side, Side>::from_vec(halfedges);

        Self {
            points,
            triangles,
            halfedges,

            num_boundary_points: 0,
            num_solid_sides: 0,
        }
    }

    /// Construct ghost elements to complete the graph.
    pub fn with_ghost(
        points: Vec<Point>,
        triangles: Vec<usize>,
        halfedges: Vec<usize>,
        num_boundary_points: usize,
    ) -> Self {
        let points = points
            .into_iter()
            .map(|Point { x, y }| Vec2::new(x as f32, y as f32))
            .collect::<Vec<_>>();
        let triangles = triangles.into_iter().map(Region).collect::<Vec<_>>();
        let halfedges = halfedges.into_iter().map(Side).collect::<Vec<_>>();

        let triangles = IndexMap::<Side, Region>::from_vec(triangles);
        let halfedges = IndexMap::<Side, Side>::from_vec(halfedges);

        let num_solid_sides = triangles.len();

        // let num_regions = 0;

        let mut num_unpaired_sides = 0;
        let mut first_unpaired_edge = Side(usize::MAX);
        let mut s_unpaired_r = HashMap::new(); // seed to side
        for s in (0..num_solid_sides).map(Side) {
            if halfedges[s].is_empty() {
                num_unpaired_sides += 1;
                s_unpaired_r.insert(triangles[s], s);
                first_unpaired_edge = s;
            }
        }

        let r_ghost = Region(points.len());
        let points = {
            let mut newpoints = points;
            newpoints.push(Vec2::new(f32::NAN, f32::NAN));
            newpoints
        };

        let mut triangles = {
            let mut r_newstart_s = IndexMap::<Side, Region>::new(
                Region(usize::MAX),
                num_solid_sides + 3 * num_unpaired_sides,
            );
            r_newstart_s.data[0..triangles.len()].copy_from_slice(&triangles.data);
            r_newstart_s
        };

        let mut halfedges = {
            let mut s_newopposite_s = IndexMap::<Side, Side>::new(
                Side(usize::MAX),
                num_solid_sides + 3 * num_unpaired_sides,
            );
            s_newopposite_s.data[0..halfedges.len()].copy_from_slice(&halfedges.data);
            s_newopposite_s
        };

        let mut s_started = first_unpaired_edge;
        for i in 0..num_unpaired_sides {
            let s_ghost_a = Side(num_solid_sides + 3 * i);
            let s_ghost_b = Side(num_solid_sides + 3 * i + 1);
            let s_ghost_c = Side(num_solid_sides + 3 * i + 2);
            let s_ghost_k = Side(num_solid_sides + (3 * i + 4) % (3 * num_unpaired_sides));

            // Construct a ghost side for s
            halfedges[s_started] = s_ghost_a;
            halfedges[s_ghost_a] = s_started;
            triangles[s_ghost_a] = triangles[s_started.next()];

            // Construct the rest of the ghost triangle
            triangles[s_ghost_b] = triangles[s_started];
            triangles[s_ghost_c] = r_ghost;

            halfedges[s_ghost_c] = s_ghost_k;
            halfedges[s_ghost_k] = s_ghost_c;

            s_started = s_unpaired_r[&triangles[s_started.next()]];
        }

        Self {
            num_solid_sides,
            num_boundary_points,
            points,
            triangles,
            halfedges,
        }
    }
}

// Represent a triangle-polygon dual mesh with:
//   - Regions (r)
//   - Sides (s)
//   - Triangles (t)
//
// Each element has an id:
//   - 0 <= r < num_regions
//   - 0 <= s < num_sides
//   - 0 <= t < num_triangles
//
// Naming convention:
//   y_name_x takes x (r, s, t) as input and produces y (r, s, t) as output.
//
// A side is directed. If two triangles t0, t1 are adjacent, there will
// be two sides representing the boundary, one for t0 and one for t1.
// These can be accessed with t_inner_s and t_outer_s.
//
// A side also represents the boundary between two regions.
// If two regions r0, r1 are adjacent,
// there will be two sides representing the boundary,
// r_begin_s and r_end_s.
//
// A side from p-->q will have a pair q-->p, at index
// s_opposite_s. It will be -1 if the side doesn't have a pair.
// Use addGhostStructure() to add ghost pairs to all sides.
pub struct DualMesh {
    // public data
    pub num_sides: usize,
    pub num_solid_sides: usize,
    pub num_regions: usize,
    pub num_solid_regions: usize,
    pub num_triangles: usize,
    pub num_solid_triangles: usize,
    pub num_boundary_regions: usize,

    // internal data that has accessors
    pub halfedges: IndexMap<Side, Side>,
    pub triangles: IndexMap<Side, Region>,
    pub s_of_r: IndexMap<Region, Side>,
    pub vertex_t: IndexMap<Triangle, Vec2>,
    pub vertex_r: IndexMap<Region, Vec2>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Triangle(pub usize);

impl From<Triangle> for usize {
    fn from(Triangle(index): Triangle) -> Self {
        index
    }
}

impl From<Side> for Triangle {
    fn from(Side(s): Side) -> Self {
        Self(s / 3)
    }
}

impl Triangle {
    pub fn s_around(self) -> [Side; 3] {
        let Self(t) = self;
        [Side(3 * t), Side(3 * t + 1), Side(3 * t + 2)]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Side(pub usize);

impl From<Side> for usize {
    fn from(Side(index): Side) -> Self {
        index
    }
}

impl From<Triangle> for Side {
    fn from(Triangle(t): Triangle) -> Self {
        Self(t * 3)
    }
}

impl Side {
    pub fn is_empty(&self) -> bool {
        self.0 == usize::MAX
    }

    pub fn prev(self) -> Self {
        let Self(s) = self;
        Self(if s % 3 == 0 { s + 2 } else { s - 1 })
    }

    pub fn next(self) -> Self {
        let Self(s) = self;
        Self(if s % 3 == 2 { s - 2 } else { s + 1 })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Region(pub usize);

impl From<Region> for usize {
    fn from(Region(index): Region) -> Self {
        index
    }
}

impl DualMesh {
    // pub fn t_from_s(s: usize) -> usize {
    //     s / 3
    // }
    // pub fn s_prev_s(s: usize) -> usize {
    //     if s % 3 == 0 {
    //         s + 2
    //     } else {
    //         s - 1
    //     }
    // }
    // pub fn s_next_s(s: usize) -> usize {
    //     if s % 3 == 2 {
    //         s - 2
    //     } else {
    //         s + 1
    //     }
    // }

    /*


    // Constructor takes partial mesh information from Delaunator and
    // constructs the rest.
    constructor (init: MeshInitializer | TriangleMesh) {
        if ('points' in init) {
            // Construct a new TriangleMesh from points + delaunator data
            self.numBoundaryRegions = init.numBoundaryPoints ?? 0;
            self.numSolidSides = init.numSolidSides ?? 0;
            self._vertex_t = [];
            self.update(init);
        } else {
            // Shallow copy an existing TriangleMesh data
            Object.assign(self, init);
        }
    }
    */

    // Update internal data structures from Delaunator
    // Update internal data structures to match the input mesh.
    // Use if you have updated the triangles/halfedges with Delaunator
    // and want the dual mesh to match the updated data. Note that
    // self DOES not update boundary regions or ghost elements.
    pub fn new(init: MeshInitializer) -> Self {
        let MeshInitializer {
            points,
            halfedges,
            triangles,
            num_boundary_points: num_boundary_regions,
            num_solid_sides,
        } = init;
        // let points = points
        //     .into_iter()
        //     .map(|Point { x, y }| Vec2::new(x as f32, y as f32))
        //     .collect::<Vec<_>>();
        // let triangles = triangles.into_iter().map(Region).collect::<Vec<_>>();
        // let halfedges = halfedges.into_iter().map(Side).collect::<Vec<_>>();

        // let triangles = IndexMap::<Side, Region>::from_vec(triangles);
        // let halfedges = IndexMap::<Side, Side>::from_vec(halfedges);

        let vertex_r = IndexMap::<Region, Vec2>::from_vec(points);

        let num_sides = triangles.len();
        let num_regions = vertex_r.len();
        let num_solid_regions = num_regions - 1; // TODO: only if there are ghosts
        let num_triangles = num_sides / 3;
        let num_solid_triangles = num_solid_sides / 3;

        let mut vertex_t = IndexMap::<Triangle, Vec2>::empty();
        if vertex_t.len() < num_triangles {
            // Extend self array to be big enough
            let old = vertex_t.len();
            let _new = num_triangles - old;
            for _ in old..num_triangles {
                vertex_t.data.push(Vec2::ZERO);
            }
        }

        // Construct an index for finding sides connected to a region
        let mut s_of_r = IndexMap::<Region, Side>::new(Side(0), num_regions);
        for s in 0..triangles.len() {
            let s = Side(s);
            let endpoint = triangles[s.next()];
            if s_of_r[endpoint] == Side(0) || halfedges[s].is_empty() {
                s_of_r[endpoint] = s;
            }
        }

        // Construct triangle coordinates
        for t in 0..triangles.len() / 3 {
            let t = Triangle(t);
            let s = Side::from(t);
            let [a, b, c] = t.s_around();
            let a = vertex_r[triangles[a]];
            let b = vertex_r[triangles[b]];
            let c = vertex_r[triangles[c]];
            vertex_t[t] = if s.0 >= num_solid_sides {
                // ghost triangle center is just outside the unpaired side
                let d = b - a;
                let scale = 10.0 / (d.x * d.x + d.y * d.y).sqrt(); // go 10units away from side
                let d = d * Vec2::new(scale, -scale);
                (a + b) * 0.5 + d
            } else {
                // solid triangle center is at the centroid
                (a + b + c) / 3.0
            }
        }

        Self {
            num_sides,
            num_solid_sides,
            num_regions,
            num_solid_regions,
            num_triangles,
            num_solid_triangles,
            num_boundary_regions,

            halfedges,
            triangles,
            s_of_r,
            vertex_t,
            vertex_r,
        }
    }

    // Accessors

    pub fn pos_of_r(&self, r: Region) -> Vec2 {
        self.vertex_r[r]
    }
    pub fn pos_of_t(&self, t: Triangle) -> Vec2 {
        self.vertex_t[t]
    }

    pub fn r_begin_s(&self, s: Side) -> Region {
        self.triangles[s]
    }
    pub fn r_end_s(&self, s: Side) -> Region {
        self.r_begin_s(s.next())
    }

    pub fn t_inner_s(&self, s: Side) -> Triangle {
        Triangle::from(s)
    }
    pub fn t_outer_s(&self, s: Side) -> Triangle {
        Triangle::from(self.halfedges[s])
    }

    pub fn s_opposite_s(&self, s: Side) -> Side {
        self.halfedges[s]
    }

    pub fn s_around_r(&self, r: Region) -> impl Iterator<Item = Side> + '_ {
        self.around(self.s_of_r[r]).map(|s| self.s_opposite_s(s))
    }

    pub fn s_around_s(&self, s: Side) -> impl Iterator<Item = Side> + '_ {
        self.around(s).map(|s| self.s_opposite_s(s))
    }

    pub fn t_around_t(&self, t: Triangle) -> [Triangle; 3] {
        let [a, b, c] = t.s_around();
        [self.t_outer_s(a), self.t_outer_s(b), self.t_outer_s(c)]
    }

    pub fn t_around_r(&self, r: Region) -> impl Iterator<Item = Triangle> + '_ {
        self.around(self.s_of_r[r]).map(Triangle::from)
    }

    pub fn t_around_s(&self, s: Side) -> impl Iterator<Item = Triangle> + '_ {
        self.around(s).map(Triangle::from)
    }

    pub fn r_around_t(&self, t: Triangle) -> [Region; 3] {
        let [a, b, c] = t.s_around();
        [self.triangles[a], self.triangles[b], self.triangles[c]]
    }

    pub fn r_around_r(&self, r: Region) -> impl Iterator<Item = Region> + '_ {
        self.around(self.s_of_r[r]).map(|s| self.r_begin_s(s))
    }

    pub fn around(&self, start: Side) -> AroundIter {
        AroundIter {
            start,
            incoming: None,
            halfedges: &self.halfedges.data,
        }
    }

    pub fn r_ghost(&self) -> usize {
        self.num_regions - 1
    }
    pub fn is_ghost_s(&self, Side(s): Side) -> bool {
        s >= self.num_solid_sides
    }
    pub fn is_ghost_r(&self, Region(r): Region) -> bool {
        r == self.num_regions - 1
    }
    pub fn is_ghost_t(&self, t: Triangle) -> bool {
        self.is_ghost_s(Side::from(t))
    }
    pub fn is_boundary_s(&self, s: Side) -> bool {
        self.is_ghost_s(s) && (s.0 % 3 == 0)
    }
    pub fn is_boundary_r(&self, Region(r): Region) -> bool {
        r < self.num_boundary_regions
    }
}

pub struct AroundIter<'a> {
    start: Side,
    incoming: Option<Side>,
    halfedges: &'a [Side],
}

impl<'a> Iterator for AroundIter<'a> {
    type Item = Side;

    fn next(&mut self) -> Option<Self::Item> {
        self.incoming = match self.incoming {
            None => Some(self.start),
            Some(incoming) => {
                let next = self.halfedges[incoming.next().0];
                if next.is_empty() || next == self.start {
                    return None;
                } else {
                    Some(next)
                }
            }
        };

        self.incoming
    }
}

/*
* Helper functions for building a TriangleMesh.
*
* The TriangleMesh constructor takes points, delaunator output,
* and a count of the number of boundary points. The boundary points
* must be the prefix of the points array.
*
* To have equally spaced points added around a rectangular boundary,
* pass in a boundary with the rectangle size and the boundary
* spacing. If using Poisson disc points, I recommend √2 times the
* spacing used for Poisson disc.
*
* Recommended code structure:

  import {generateInteriorBoundaryPoints} from "dual-mesh/create.js";
  import {TriangleMesh} from "dual-mesh/index.js";

  const bounds = {left: 0, top: 0, width: 1000, height: 1000};
  const spacing = 50;
  let points = generateInteriorBoundaryPoints(bounds, spacing);
  let numBoundaryPoints = points.length;
  let generator = new Poisson({
    shape: [bounds.width, bounds.height],
    minDistance: spacing / Math.sqrt(2),
  });
  for (let p of points) { generator.addPoint(p); }
  points = generator.fill();

  let init = {points, delaunator: Delaunator.from(points), numBoundaryPoints};
  init = TriangleMesh.addGhostStructure(init);
  let mesh = new TriangleMesh(init);

*/

/// Check for skinny triangles, indicating bad point selection
pub fn check_triangle_inequality(points: &[Vec3], triangles: Vec<Region>, halfedges: &[Side]) {
    /*
    const badAngleLimit = 30;
    let summary = new Array(badAngleLimit).fill(0);
    let count = 0;
    for (let s = 0; s < triangles.length; s++) {
        let r0 = triangles[s],
            r1 = triangles[TriangleMesh.s_next_s(s)],
            r2 = triangles[TriangleMesh.s_next_s(TriangleMesh.s_next_s(s))];
        let p0 = points[r0],
            p1 = points[r1],
            p2 = points[r2];
        let d0 = [p0[0]-p1[0], p0[1]-p1[1]];
        let d2 = [p2[0]-p1[0], p2[1]-p1[1]];
        let dotProduct = d0[0] * d2[0] + d0[1] + d2[1];
        let angleDegrees = 180 / Math.PI * Math.acos(dotProduct);
        if (angleDegrees < badAngleLimit) {
            summary[angleDegrees|0]++;
            count++;
        }
    }
    // NOTE: a much faster test would be the ratio of the inradius to
    // the circumradius, but as I'm generating these offline, I'm not
    // worried about speed right now

    // TODO: consider adding circumcenters of skinny triangles to the point set
    if (count > 0) {
        console.log('  bad angles:', summary.join(" "));
    }
    */
}

/// Add vertices evenly along the boundary of the mesh just barely
/// inside the given boundary rectangle.
///
/// The boundarySpacing parameter should be roughly √2 times the
/// poisson disk minDistance spacing or √½ the maxDistance spacing.
///
/// They need to be inside and not outside so that these points can be
/// used with the poisson disk libraries I commonly use. The libraries
/// require that all points be inside the range.
///
/// Since these points are slightly inside the boundary, the triangle
/// mesh will not fill the boundary. Generate exterior boundary points
/// if you need to fill the boundary.
///
/// I use a *slight* curve so that the Delaunay triangulation doesn't
/// make long thin triangles along the boundary.
pub fn generate_interior_boundary_points(
    width: f64,
    height: f64,
    boundary_spacing: f64,
) -> Vec<Point> {
    // https://www.redblobgames.com/x/2314-poisson-with-boundary/

    let epsilon = 1e-4;
    let curvature = 1.0;

    let w = ((width - 2.0 * curvature) / boundary_spacing).ceil();
    let h = ((height - 2.0 * curvature) / boundary_spacing).ceil();

    let mut points = vec![];

    // top and bottom
    for q in 0..w as usize {
        let t = q as f64 / w;
        let dx = (width - 2.0 * curvature) * t;
        let dy = epsilon + curvature * 4.0 * (t - 0.5).powi(2);
        points.push(Point {
            x: curvature + dx,
            y: dy,
        });
        points.push(Point {
            x: width - curvature - dx,
            y: height - dy,
        });
    }

    // left and right
    for r in 0..h as usize {
        let t = r as f64 / h;
        let dy = (height - 2.0 * curvature) * t;
        let dx = epsilon + curvature * 4.0 * (t - 0.5).powi(2);
        points.push(Point {
            x: dx,
            y: height - curvature - dy,
        });
        points.push(Point {
            x: width - dx,
            y: curvature + dy,
        });
    }

    points
}

/// Add vertices evenly along the boundary of the mesh
/// outside the given boundary rectangle.
///
/// The boundarySpacing parameter should be roughly √2 times the
/// poisson disk minDistance spacing or √½ the maxDistance spacing.
///
/// If using poisson disc selection, the interior boundary points will
/// be to keep the points separated and the exterior boundary points
/// will be to make sure the entire map area is filled.
pub fn generate_exterior_boundary_points(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    boundary_spacing: f32,
) -> Vec<Vec2> {
    // https://www.redblobgames.com/x/2314-poisson-with-boundary/

    let curvature = 1.0;
    let diagonal = boundary_spacing / f32::sqrt(2.0);

    let mut points = vec![];

    let w = ((width - 2.0 * curvature) / boundary_spacing).ceil();
    let h = ((height - 2.0 * curvature) / boundary_spacing).ceil();

    // top and bottom
    for q in 0..w as usize {
        let t = q as f32 / w;
        let dx = (width - 2.0 * curvature) * t + boundary_spacing / 2.0;
        points.push(Vec2::new(left + dx, top - diagonal));
        points.push(Vec2::new(left + width - dx, top + height + diagonal));
    }

    // Left and right
    for r in 0..h as usize {
        let t = r as f32 / h;
        let dy = (height - 2.0 * curvature) * t + boundary_spacing / 2.0;
        points.push(Vec2::new(left - diagonal, top + height - dy));
        points.push(Vec2::new(left + width + diagonal, top + dy));
    }

    // Corners
    points.push(Vec2::new(left - diagonal, top - diagonal));
    points.push(Vec2::new(left + width + diagonal, top - diagonal));
    points.push(Vec2::new(left - diagonal, top + height + diagonal));
    points.push(Vec2::new(left + width + diagonal, top + height + diagonal));

    points
}
