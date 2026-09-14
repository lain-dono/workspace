use super::coords::{CoordinateSystem, PointSampleDistance};
use crate::bvh::BoundingBox;
use crate::disjoint_set::DisjointSet;
use bevy::{
    math::{
        Vec3,
        bounding::{Aabb3d, IntersectsVolume},
        swizzles::Vec3Swizzles,
    },
    platform::collections::{HashMap, HashSet},
};
use std::{cmp::Ordering, marker::PhantomData};
use thiserror::Error;

/// A navigation mesh.
pub struct NavMeshBuilder<T: CoordinateSystem> {
    /// The vertices that make up the polygons.
    pub vertices: Vec<T::Coord>,
    /// The polygons of the mesh. Polygons are indices to the `vertices` that
    /// make up the polygon. Polygons must be convex, and oriented
    /// counterclockwise (using the right hand rule). Polygons are assumed to be
    /// not self-intersecting.
    pub polygons: Vec<Vec<u32>>,
    /// The type index of each polygon. This type index is translated into a real
    /// [`crate::NodeType`] when assigned to an [`crate::Archipelago`]. Must be
    /// the same length as [`Self::polygons`].
    pub polygon_type_indices: Vec<u32>,
}

impl<T: CoordinateSystem> Clone for NavMeshBuilder<T> {
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            polygons: self.polygons.clone(),
            polygon_type_indices: self.polygon_type_indices.clone(),
        }
    }
}

/// An error when validating a navigation mesh.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Stores the number of polygons and the number of type indices.
    #[error(
        "The polygon type indices do not have the same length as the polygons. There are {0} polygons, but {1} type indices."
    )]
    TypeIndicesHaveWrongLength(usize, usize),
    /// Stores the index of the polygon.
    #[error("The polygon at index {0} is concave or has edges in clockwise order.")]
    ConcavePolygon(usize),
    /// Stores the index of the polygon.
    #[error("The polygon at index {0} does not have at least 3 vertices.")]
    NotEnoughVerticesInPolygon(usize),
    /// Stores the index of the polygon.
    #[error("The polygon at index {0} references an out-of-bounds vertex.")]
    InvalidVertexIndexInPolygon(usize),
    /// Stores the index of the polygon.
    #[error("The polygon at index {0} contains a degenerate edge (an edge with zero length).")]
    DegenerateEdgeInPolygon(usize),
    /// Stores the indices of the two vertices that make up the edge.
    #[error("The edge made from vertices {0} and {1} is used by more than two polygons.")]
    DoublyConnectedEdge(u32, u32),
}

impl<T: CoordinateSystem> NavMeshBuilder<T> {
    #[must_use]
    pub fn new(vertices: impl Into<Vec<T::Coord>>) -> Self {
        Self {
            vertices: vertices.into(),
            polygons: Vec::new(),
            polygon_type_indices: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_polygon(mut self, ty: u32, poly: impl Into<Vec<u32>>) -> Self {
        self.polygons.push(poly.into());
        self.polygon_type_indices.push(ty);
        self
    }

    /// Ensures required invariants of the navigation mesh, and computes
    /// additional derived properties to produce and optimized and validated
    /// navigation mesh. Returns an error if the navigation mesh is invalid in
    /// some way.
    pub fn validate(self) -> Result<NavMesh<T>, ValidationError> {
        #[derive(Clone, Copy)]
        enum State {
            Disconnected,
            Boundary {
                poly: usize,
                edge: usize,
            },
            Connected {
                poly_1: usize,
                edge_1: usize,

                poly_2: usize,
                edge_2: usize,
            },
        }

        let Self {
            mut polygons,
            vertices,
            polygon_type_indices,
        } = self;

        if polygons.len() != polygon_type_indices.len() {
            return Err(ValidationError::TypeIndicesHaveWrongLength(
                polygons.len(),
                polygon_type_indices.len(),
            ));
        }

        let vertices = vertices
            .iter()
            .copied()
            .map(T::to_landmass)
            .collect::<Vec<_>>();

        let bounds = BoundingBox::from_point_cloud(vertices.iter().copied());
        let mut region_sets = DisjointSet::with_len(polygons.len());
        let mut connectivity_set = HashMap::new();

        for (polygon_index, polygon) in polygons.iter().enumerate() {
            if polygon.len() < 3 {
                return Err(ValidationError::NotEnoughVerticesInPolygon(polygon_index));
            }

            for &vertex_index in polygon {
                if vertex_index >= vertices.len() as u32 {
                    return Err(ValidationError::InvalidVertexIndexInPolygon(polygon_index));
                }
            }

            //          if !is_convex(polygon, &vertices) {
            //              return dbg!(Err(ValidationError::ConcavePolygon(polygon_index)));
            //          }

            for index in 0..polygon.len() {
                let left_vertex = polygon[(index + polygon.len() - 1) % polygon.len()];
                let center_vertex = polygon[index];
                let right_vertex = polygon[(index + 1) % polygon.len()];

                // Check if the edge is degenerate.

                let edge = if center_vertex < right_vertex {
                    (center_vertex, right_vertex)
                } else {
                    (right_vertex, center_vertex)
                };
                if edge.0 == edge.1 {
                    return Err(ValidationError::DegenerateEdgeInPolygon(polygon_index));
                }

                // Derive connectivity for the edge.

                let state = connectivity_set.entry(edge).or_insert(State::Disconnected);
                match state {
                    State::Disconnected => {
                        *state = State::Boundary {
                            poly: polygon_index,
                            edge: index,
                        };
                    }
                    &mut State::Boundary {
                        poly: polygon_1,
                        edge: edge_1,
                    } => {
                        *state = State::Connected {
                            poly_1: polygon_1,
                            edge_1,
                            poly_2: polygon_index,
                            edge_2: index,
                        };
                        region_sets.join(polygon_1 as u32, polygon_index as u32);
                    }
                    State::Connected { .. } => {
                        return Err(ValidationError::DoublyConnectedEdge(edge.0, edge.1));
                    }
                }

                // Check if the vertex is concave.

                let left_vertex = vertices[left_vertex as usize].xy();
                let center_vertex = vertices[center_vertex as usize].xy();
                let right_vertex = vertices[right_vertex as usize].xy();

                let left_edge = left_vertex - center_vertex;
                let right_edge = right_vertex - center_vertex;

                match right_edge.perp_dot(left_edge).partial_cmp(&0.0) {
                    // The right edge is to the right of the left edge.
                    Some(Ordering::Greater) => {}
                    // The right edge is parallel to the left edge, but they point in
                    // opposite directions.
                    Some(Ordering::Equal) if right_edge.dot(left_edge) < 0.0 => {}
                    // right_edge is to the left of the left_edge (or they are parallel
                    // and point in the same direciton), so the polygon is
                    // concave.
                    _ => return Err(ValidationError::ConcavePolygon(polygon_index)),
                }
            }
        }

        let mut region_to_normalized_region: HashMap<u32, u32> = HashMap::new();

        let mut polygons: Vec<_> = polygons
            .drain(..)
            .enumerate()
            .map(|(index, polygon)| Polygon {
                bounds: BoundingBox::from_point_cloud(
                    polygon.iter().map(|&index| vertices[index as usize]),
                )
                .unwrap_inner(),
                center: polygon.iter().map(|&i| vertices[i as usize]).sum::<Vec3>()
                    / polygon.len() as f32,
                vertices: polygon.into_iter().map(|i| (i, None)).collect(),
                region: {
                    let region = region_sets.root_of(index as u32);
                    // Get around the borrow checker by deciding on the new normalized
                    // region beforehand.
                    let new_normalized_region = region_to_normalized_region.len();
                    // Either lookup the existing normalized region or insert the next
                    // unique index.
                    *region_to_normalized_region
                        .entry(region)
                        .or_insert(new_normalized_region as u32)
                },
                type_index: polygon_type_indices[index],
            })
            .collect();

        let used_type_indices: HashSet<_> = polygons.iter().map(|p| p.type_index).collect();
        let mut boundary_edges = Vec::new();

        for state in connectivity_set.values().copied() {
            match state {
                State::Disconnected => panic!("Value is never stored"),
                State::Boundary { poly, edge } => {
                    boundary_edges.push(MeshEdgeRef { poly, edge });
                }
                State::Connected {
                    poly_1,
                    edge_1,
                    poly_2,
                    edge_2,
                } => {
                    let edge = polygons[poly_1].edge_indices(edge_1);
                    let edge = edge.map(|i| vertices[i as usize]);
                    let edge_center = (edge[0] + edge[1]) / 2.0;
                    let travel_distances_2 = [
                        polygons[poly_1].center.distance(edge_center),
                        polygons[poly_2].center.distance(edge_center),
                    ];
                    let travel_distances_1 = [travel_distances_2[1], travel_distances_2[0]];
                    polygons[poly_1].vertices[edge_1].1 =
                        Some(Connectivity::new(poly_2, travel_distances_2));
                    polygons[poly_2].vertices[edge_2].1 =
                        Some(Connectivity::new(poly_1, travel_distances_1));
                }
            }
        }

        Ok(NavMesh {
            bounds,
            polygons,
            vertices,
            boundary_edges,
            used_type_indices,
            marker: PhantomData,
        })
    }
}

fn is_convex(polygon: &[u32], vertices: &[Vec3]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    polygon
        .windows(3)
        .map(|triangle| TryInto::<[_; 3]>::try_into(triangle).unwrap())
        .all(|triangle| {
            let [a, b, c] = triangle.map(|i| vertices[i as usize].xy());

            let ab_edge = a - b;
            let cb_edge = c - b;

            let product = cb_edge.perp_dot(ab_edge);

            match product.total_cmp(&0.0) {
                // The right edge is to the right of the left edge.
                Ordering::Greater => true,
                // The right edge is parallel to the left edge, but they point in
                // opposite directions.
                Ordering::Equal if cb_edge.dot(ab_edge) <= 0.0 => true,

                //Ordering::Less if product.to_bits() == f32::to_bits(-0.0) => {}

                // right_edge is to the left of the left_edge (or they are parallel
                // and point in the same direciton), so the polygon is
                // concave.
                _ => false,
            }
        })
}

/// A navigation mesh which has been validated and derived data has been computed.
pub struct NavMesh<T: CoordinateSystem> {
    /// The bounds of the mesh data itself.
    ///
    /// This is a tight bounding box around the vertices of the navigation mesh.
    pub(crate) bounds: BoundingBox,
    /// The vertices that make up the polygons.
    pub(crate) vertices: Vec<Vec3>,
    /// The polygons of the mesh.
    pub(crate) polygons: Vec<Polygon>,
    /// The boundary edges in the navigation mesh. Edges are stored as pairs of
    /// vertices in a counter-clockwise direction. That is, moving along an edge
    /// (e.0, e.1) from e.0 to e.1 will move counter-clockwise along the
    /// boundary. The order of edges is undefined.
    pub(crate) boundary_edges: Vec<MeshEdgeRef>,
    /// The type indices used by this navigation mesh. This is a convenience for
    /// just iterating through every polygon and checking its type index. Note
    /// these don't correspond to [`crate::NodeType`]s yet. This occurs once
    /// assigned to an island.
    pub(crate) used_type_indices: HashSet<u32>,
    /// Marker for the `CoordinateSystem`.
    pub(crate) marker: PhantomData<T>,
}

// Manual Debug impl to avoid Debug bound on CoordinateSystem.
impl<T: CoordinateSystem> Clone for NavMesh<T> {
    fn clone(&self) -> Self {
        Self {
            bounds: self.bounds,
            vertices: self.vertices.clone(),
            polygons: self.polygons.clone(),
            boundary_edges: self.boundary_edges.clone(),
            used_type_indices: self.used_type_indices.clone(),
            marker: self.marker,
        }
    }
}

// Manual Debug impl to avoid Debug bound on CoordinateSystem.
impl<T: CoordinateSystem> std::fmt::Debug for NavMesh<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidNavigationMesh")
            .field("mesh_bounds", &self.bounds)
            .field("vertices", &self.vertices)
            .field("polygons", &self.polygons)
            .field("boundary_edges", &self.boundary_edges)
            .field("used_type_indices", &self.used_type_indices)
            .field("marker", &self.marker)
            .finish()
    }
}

/// A valid polygon. This means the polygon is convex and indexes the `vertices`
/// Vec of the corresponding [`NavMesh`].
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Polygon {
    /// The vertices are indexes to the `vertices` Vec of the corresponding [`NavMesh`].
    ///
    /// The connectivity of each edge in the polygon.
    ///
    /// This is the same length as the number of edges (which is equivalent to `self.vertices.len()`).
    /// Entries that are `None` correspond to the boundary of the navigation mesh,
    /// while `Some` entries are connected to another node.
    pub(crate) vertices: Vec<(u32, Option<Connectivity>)>,

    /// The "region" that this polygon belongs to.
    ///
    /// Each region is disjoint from every other.
    /// A "direct" path only exists if the region matches between two nodes.
    /// An "indirect" path exists if regions are joined together through boundary links.
    pub(crate) region: u32,
    /// The "type" of this node.
    ///
    /// This is translated into a [`crate::NodeType`] once it is part of an island.
    pub(crate) type_index: u32,
    /// The bounding box of `vertices`.
    pub(crate) bounds: Aabb3d,
    /// The center of the polygon.
    pub(crate) center: Vec3,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct Connectivity {
    /// The index of the polygon that this edge leads to.
    pub polygon: usize,
    /// The distances of travelling across this connection. The first is the
    /// distance travelled across the starting node, and the second is the
    /// distance travelled across the destination node. These must be multiplied
    /// by the actual node costs.
    pub travel_distances: [f32; 2],
}

impl Connectivity {
    pub fn new(polygon: usize, travel_distances: [f32; 2]) -> Self {
        Self {
            polygon,
            travel_distances,
        }
    }
}

/// A reference to an edge on a navigation mesh.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Default)]
pub(crate) struct MeshEdgeRef {
    /// The index of the polygon that this edge belongs to.
    pub poly: usize,
    /// The index of the edge within the polygon.
    pub edge: usize,
}

impl<T: CoordinateSystem> NavMesh<T> {
    // Gets the points that make up the specified edge.
    pub(crate) fn edge_points(&self, edge_ref: MeshEdgeRef) -> [Vec3; 2] {
        let polygon = &self.polygons[edge_ref.poly];
        let pair = [edge_ref.edge, (edge_ref.edge + 1) % polygon.vertices.len()];
        pair.map(|index| self.vertices[polygon.vertices[index].0 as usize])
    }

    /// Finds the node nearest to (and within `distance_to_node` of) `point`.
    /// Returns the point on the nav mesh nearest to `point` and the index of the
    /// polygon.
    pub(crate) fn sample_point(
        &self,
        point: Vec3,
        sample_distance: T::SampleDistance,
    ) -> Option<(Vec3, usize)> {
        fn project_to_triangle([a, b, c]: [Vec3; 3], point: Vec3) -> Vec3 {
            let [ba_delta, cb_delta, ac_delta] = [b - a, c - b, a - c];
            let [ba_flat, cb_flat, ac_flat] = [ba_delta.xy(), cb_delta.xy(), ac_delta.xy()];

            if ba_flat.perp_dot(point.xy() - a.xy()) < 0.0 {
                let s = ba_flat.dot(point.xy() - a.xy()) / ba_flat.length_squared();
                return ba_delta * s.clamp(0.0, 1.0) + a;
            }
            if cb_flat.perp_dot(point.xy() - b.xy()) < 0.0 {
                let s = cb_flat.dot(point.xy() - b.xy()) / cb_flat.length_squared();
                return cb_delta * s.clamp(0.0, 1.0) + b;
            }
            if ac_flat.perp_dot(point.xy() - c.xy()) < 0.0 {
                let s = ac_flat.dot(point.xy() - c.xy()) / ac_flat.length_squared();
                return ac_delta * s.clamp(0.0, 1.0) + c;
            }

            let normal = -ba_delta.cross(ac_delta).normalize();
            let height = normal.dot(point - a) / normal.z;
            Vec3::new(point.x, point.y, point.z - height)
        }

        let sample_min = Vec3::new(
            -sample_distance.horizontal_distance(),
            -sample_distance.horizontal_distance(),
            -sample_distance.distance_below(),
        );
        let sample_max = Vec3::new(
            sample_distance.horizontal_distance(),
            sample_distance.horizontal_distance(),
            sample_distance.distance_above(),
        );

        let sample_box = Aabb3d {
            min: (point + sample_min).into(),
            max: (point + sample_max).into(),
        };

        let mut best_node = None;

        for (polygon_index, polygon) in self.polygons.iter().enumerate() {
            if !sample_box.intersects(&polygon.bounds) {
                continue;
            }
            for i in 2..polygon.vertices.len() {
                let triangle = [0, i - 1, i]
                    .map(|index| polygon.vertices[index].0 as usize)
                    .map(|index| self.vertices[index]);

                let projected_point = project_to_triangle(triangle, point);

                let distance_to_triangle_horizontal = point.xy().distance(projected_point.xy());
                let distance_to_triangle_vertical = projected_point.z - point.z;
                if distance_to_triangle_horizontal < sample_distance.horizontal_distance()
                    && (-sample_distance.distance_below()..sample_distance.distance_above())
                        .contains(&distance_to_triangle_vertical)
                {
                    let distance_to_triangle = distance_to_triangle_horizontal
                        * sample_distance.vertical_preference_ratio()
                        + distance_to_triangle_vertical.abs();
                    let replace = best_node.is_none_or(|(_, _, previous_best_distance)| {
                        distance_to_triangle < previous_best_distance
                    });
                    if replace {
                        best_node = Some((polygon_index, projected_point, distance_to_triangle));
                    }
                }
            }
        }

        best_node.map(|(polygon_index, projected_point, _)| (projected_point, polygon_index))
    }
}

impl Polygon {
    /// Determines the vertices corresponding to `edge`.
    pub(crate) fn edge_indices(&self, edge: usize) -> [u32; 2] {
        [edge, (edge + 1) % self.vertices.len()].map(|index| self.vertices[index].0)
    }
}
