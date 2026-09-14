use crate::id_internals::{Index, is_valid};
use crate::iter::{EdgeIdLoop, MutEdgeLoop, ReverseEdgeIdLoop};
use sid::{Id, IdRange, IdVec};
use std::marker::PhantomData;
use std::ops;

#[derive(Debug)]
pub struct Vert_;

#[derive(Debug)]
pub struct Edge_;

#[derive(Debug)]
pub struct Face_;

pub type VertId = Id<Vert_, Index>;
pub type EdgeId = Id<Edge_, Index>;
pub type FaceId = Id<Face_, Index>;

pub const NO_EDGE: EdgeId = EdgeId {
    handle: u16::MAX,
    _marker: PhantomData,
};
pub const NO_FACE: FaceId = FaceId {
    handle: u16::MAX,
    _marker: PhantomData,
};
pub const NO_VERTEX: VertId = VertId {
    handle: u16::MAX,
    _marker: PhantomData,
};

/// A range of Id pointing to contiguous vertices.
pub type VertIdRange = IdRange<Vert_, Index>;

/// A range of Id pointing to contiguous half edges.
pub type EdgeIdRange = IdRange<Edge_, Index>;

/// A range of Id pointing to contiguous faces.
pub type FaceIdRange = IdRange<Face_, Index>;

/// The structure holding the data specific to each half edge.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HalfEdge {
    pub next: EdgeId, // next half edge around the face
    pub prev: EdgeId, // previous half edge around the face
    pub twin: EdgeId, // oppositely oriented adjacent half edge
    pub vert: VertId, // vertex this edge originates from
    pub face: FaceId, // adjacent face
}

impl HalfEdge {
    pub fn next(&self) -> EdgeId {
        self.next
    }

    pub fn prev(&self) -> EdgeId {
        self.prev
    }

    pub fn opposite(&self) -> EdgeId {
        self.twin
    }

    pub fn vertex(&self) -> VertId {
        self.vert
    }

    pub fn face(&self) -> FaceId {
        self.face
    }

    pub fn set_next(&mut self, id: EdgeId) {
        self.next = id;
    }

    pub fn set_prev(&mut self, id: EdgeId) {
        self.prev = id;
    }

    pub fn set_opposite(&mut self, id: EdgeId) {
        self.twin = id;
    }

    pub fn set_vertex(&mut self, id: VertId) {
        self.vert = id;
    }

    pub fn set_face(&mut self, id: FaceId) {
        self.face = id;
    }
}

/// The structure holding the data specific to each face.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub inner: Vec<EdgeId>,
    pub first: EdgeId,
}

/// The data structure that contains a mesh's connectivity information
///
/// It does not contain other attributes such as positions. Use IdVector for that.
pub struct Connectivity {
    edges: IdVec<EdgeId, HalfEdge>,
    faces: IdVec<FaceId, Face>,
}

impl Default for Connectivity {
    /// Create an empty kernel.
    fn default() -> Self {
        Self {
            edges: IdVec::new(),
            faces: IdVec::new(),
        }
    }
}

impl Connectivity {
    /// Create an empty kernel and preallocate memory for vertices, edges and faces.
    pub fn with_capacities(e: u16, f: u16) -> Self {
        Self {
            edges: IdVec::with_capacity(e),
            faces: IdVec::with_capacity(f),
        }
    }

    /// Create a ConnectivityKernel initialized with a loop
    pub fn from_loop(vertices: VertIdRange) -> Self {
        let (lower, upper) = vertices.size_hint();
        let capacity = if let Some(size) = upper { size } else { lower } as u16;
        let mut kernel = Self::with_capacities(capacity * 2, 2);

        let back_face = kernel.add_face();
        let main_face = kernel.add_face();

        kernel.add_loop(vertices, Some(main_face), Some(back_face));

        kernel.debug_assert_face_invariants(main_face);
        kernel.debug_assert_face_invariants(back_face);

        kernel
    }

    pub fn edge(&self, id: EdgeId) -> &HalfEdge {
        &self.edges[id]
    }

    pub fn mut_edge(&mut self, id: EdgeId) -> &mut HalfEdge {
        &mut self[id]
    }

    pub fn face(&self, id: FaceId) -> &Face {
        &self.faces[id]
    }

    pub fn first_edge(&self) -> Option<EdgeId> {
        self.edges.first_id()
    }

    pub fn first_face(&self) -> Option<FaceId> {
        self.faces.first_id()
    }

    pub fn contains_edge(&self, id: EdgeId) -> bool {
        self.edges.has_id(id)
    }

    pub fn contains_face(&self, id: FaceId) -> bool {
        self.faces.has_id(id)
    }

    pub fn walk_edge_ids_around_face<'a>(&'a self, id: FaceId) -> EdgeIdLoop<'a> {
        let edge = self[id].first;
        let prev = if is_valid(edge) {
            self[edge].prev
        } else {
            NO_EDGE
        };
        EdgeIdLoop::new(self, edge, prev)
    }

    /// Iterate over halfedge ids around a loop
    pub fn walk_edge_ids<'a>(&'a self, first: EdgeId) -> EdgeIdLoop<'a> {
        EdgeIdLoop::new(self, first, self[first].prev)
    }

    /// Iterate over halfedges around a loop
    pub fn walk_edges_mut<'a>(&'a mut self, first: EdgeId) -> MutEdgeLoop<'a> {
        let stop = self[first].prev;
        MutEdgeLoop::new(self, first, stop)
    }

    /// Shorthand for walk_edge_ids for a given face's loop
    pub fn walk_edge_ids_around_face_reverse<'a>(&'a self, id: FaceId) -> ReverseEdgeIdLoop<'a> {
        let edge = self[id].first;
        ReverseEdgeIdLoop::new(self, edge, self[edge].next)
    }

    /// Return the next edge id when circulating around a vertex.
    /// TODO: needs tests
    pub fn next_edge_id_around_vertex(&self, id: EdgeId) -> Option<EdgeId> {
        let opposite = self[id].twin;
        if !is_valid(opposite) {
            return None;
        }
        let next = self[opposite].next;
        if !is_valid(next) {
            return None;
        }
        Some(next)
    }

    /// Run a few debug-only assertions to check the state of a given edge.
    pub fn debug_assert_edge_invariants(&self, id: EdgeId) {
        //return;
        debug_assert!(is_valid(self[id].face));
        debug_assert_eq!(self[self[id].next].prev, id);
        debug_assert_eq!(self[self[id].prev].next, id);
        debug_assert_eq!(self[id].face, self[self[id].next].face);
        debug_assert_eq!(self[id].face, self[self[id].prev].face);
        if is_valid(self[id].twin) {
            debug_assert_eq!(self[self[id].twin].twin, id);
            debug_assert_eq!(self[id].vert, self[self[self[id].twin].next].vert);
        }
    }

    /// Run a few debug-only assertions to check the state of a given face,
    /// and the edges in its loop.
    pub fn debug_assert_face_invariants(&self, face: FaceId) {
        if !is_valid(face) {
            return;
        }
        for e in self.walk_edge_ids_around_face(face) {
            self.debug_assert_edge_invariants(e);
        }
    }

    /// Insert new_vertex on this edge.
    pub fn split_edge(&mut self, prev: EdgeId, vert: VertId) {
        // from:
        //     a ---[id]-----------------------------------------> b
        //     a <----------------------------------[opposite]---- b
        // to:
        //     a ---[id]------------> new_vertex --[new_edge]----> b
        //     a <--[new_opposite]--- new_vertex <-[opposite]----- b

        debug_assert!(is_valid(prev));
        debug_assert!(is_valid(vert));

        let twin = self[prev].twin;

        // patch up existing edges
        self[prev].next = self.add_edge(HalfEdge {
            vert,
            twin,
            face: self[prev].face,
            next: self[prev].next,
            prev,
        });

        // new_opposite
        if is_valid(twin) {
            let opposite = self[twin];
            self[twin].next = self.add_edge(HalfEdge {
                vert,
                twin: prev,
                face: opposite.face,
                next: opposite.next,
                prev: twin,
            });
        }
    }

    /// Connect edges e1 and e2 such that e1->[new edge]->e2.
    ///
    /// This operation may add a new face. If so, the face's id is returned.
    /// If a face id is provided as parameter, and a face must be added, the
    /// provided face will be used instead of creating a new one.
    pub fn connect_edges(&mut self, e1: EdgeId, e2: EdgeId) -> Option<FaceId> {
        //
        //   -e1--> v1 --e1_next->
        //          |^
        //          n|
        //          ||   new_face
        //          |o
        //          v|
        //   <--e2- v2 <--e2_prev-
        // ______________________
        //
        // n: new_edge (returned)
        // o: new_opposite_edge

        debug_assert!(is_valid(e1));
        debug_assert!(is_valid(e2));

        let mut add_face = true;
        let original_face = self[e1].face;

        debug_assert!(is_valid(original_face));

        // Check whether we are connecting to a hole in the face, in which case
        // we should not add a face.
        for i in 0..self[original_face].inner.len() {
            for e in self.walk_edge_ids(self[original_face].inner[i]) {
                if e == e1 || e == e2 {
                    // connecting to one of the inner loops
                    add_face = false;
                    // remove the hole from this face
                    break;
                }
            }
            if !add_face {
                self[original_face].inner.remove(i);
                break;
            }
        }

        let e1_next = self[e1].next;
        let e2_prev = self[e2].prev;
        let v1 = self[e1_next].vert;
        let v2 = self[e2].vert;

        debug_assert!(is_valid(e1_next));
        debug_assert!(is_valid(e2_prev));

        let new_edge = self.add_edge(HalfEdge {
            next: e2,
            prev: e1,
            twin: NO_EDGE,
            face: original_face,
            vert: v1,
        });
        let new_opposite_edge = self.add_edge(HalfEdge {
            next: e1_next,
            prev: e2_prev,
            twin: new_edge,
            face: original_face, // may become opposite_face
            vert: v2,
        });
        self[new_edge].twin = new_opposite_edge;

        //println!(" connect_edges({}, {}) -> {}", e1.handle, e2.handle, new_edge.handle);

        self[e1].next = new_edge;
        self[e2].prev = new_edge;
        self[e1_next].prev = new_opposite_edge;
        self[e2_prev].next = new_opposite_edge;

        self[original_face].first = new_edge;
        self.debug_assert_face_invariants(original_face);

        if add_face {
            let opposite_face = self.add_face_with_edge(e1_next);
            let mut it = new_opposite_edge;
            loop {
                let edge = &mut self[it];
                edge.face = opposite_face;
                it = edge.next;
                //println!(" -> new face loop {}", it.handle);
                if it == new_opposite_edge {
                    break;
                }
            }
            self.debug_assert_face_invariants(opposite_face);
            return Some(opposite_face);
        }

        None
    }

    /// Connect edges e1 and e2 .
    ///
    /// This operation may add a new face. If so, the face's id is returned.
    /// If a face id is provided as parameter, and a face must be added, the
    /// provided face will be used instead of creating a new one.
    pub fn connect_edges2(&mut self, e1: EdgeId, e2: EdgeId) -> Option<FaceId> {
        //
        // -e1prev-> v1 --e1----->
        //           |^
        //           n|
        //           ||   new_face
        //           |o
        //           v|
        //    <--e2- v2 <--e2_prev-
        // ______________________
        //
        // n: new_edge (returned)
        // o: new_opposite_edge

        debug_assert!(is_valid(e1));
        debug_assert!(is_valid(e2));

        let mut add_face = true;
        let original_face = self[e1].face;

        debug_assert!(is_valid(original_face));

        // Check whether we are connecting to a hole in the face, in which case
        // we should not add a face.
        for i in 0..self[original_face].inner.len() {
            for e in self.walk_edge_ids(self[original_face].inner[i]) {
                if e == e1 || e == e2 {
                    // connecting to one of the inner loops
                    add_face = false;
                    // remove the hole from this face
                    break;
                }
            }
            if !add_face {
                self[original_face].inner.remove(i);
                break;
            }
        }

        let e1_prev = self[e1].prev;
        let e2_prev = self[e2].prev;
        let v1 = self[e1].vert;
        let v2 = self[e2].vert;

        debug_assert!(is_valid(e1_prev));
        debug_assert!(is_valid(e2_prev));

        let new_edge = self.add_edge(HalfEdge {
            next: e2,
            prev: e1_prev,
            twin: NO_EDGE,
            face: original_face,
            vert: v1,
        });
        let new_opposite_edge = self.add_edge(HalfEdge {
            next: e1,
            prev: e2_prev,
            twin: new_edge,
            face: original_face, // may become opposite_face
            vert: v2,
        });
        self[new_edge].twin = new_opposite_edge;

        //println!(" connect_edges({}, {}) -> {}", e1.handle, e2.handle, new_edge.handle);

        self[e1].prev = new_opposite_edge;
        self[e2].prev = new_edge;
        self[e1_prev].next = new_edge;
        self[e2_prev].next = new_opposite_edge;

        self[original_face].first = new_edge;
        self.debug_assert_face_invariants(original_face);

        if add_face {
            let opposite_face = self.add_face_with_edge(new_opposite_edge);
            let mut it = new_opposite_edge;
            loop {
                let edge = &mut self[it];
                edge.face = opposite_face;
                it = edge.next;
                //println!(" -> new face loop {}", it.handle);
                if it == new_opposite_edge {
                    break;
                }
            }
            self.debug_assert_face_invariants(opposite_face);
            return Some(opposite_face);
        }

        None
    }

    /// Insert a half edge in the kernel
    pub fn add_empty_edge(&mut self) -> EdgeId {
        self.add_edge(HalfEdge {
            next: NO_EDGE,
            prev: NO_EDGE,
            twin: NO_EDGE,
            face: NO_FACE,
            vert: NO_VERTEX,
        })
    }

    /// Insert a half-edge in the kernel.
    fn add_edge(&mut self, data: HalfEdge) -> EdgeId {
        self.edges.push(data)
    }

    /// Insert a Face in the kernel.
    pub fn add_face(&mut self) -> FaceId {
        self.add_face_with_edge(NO_EDGE)
    }

    /// Insert a Face in the kernel.
    pub fn add_face_with_edge(&mut self, first: EdgeId) -> FaceId {
        self.faces.push(Face {
            first,
            inner: vec![],
        })
    }

    /// Extrude the vertex that the edge passed as parameter starts from, adding one or two
    /// half edges to the kernel.
    ///
    /// The original edge *must* have a next vertex
    pub fn extrude_vertex(&mut self, edge: EdgeId, to: VertId) -> EdgeId {
        //              to
        //              ^|
        //    (new_edge)||new_opposite
        //              |v
        //   (------->) v1 ---edge--->

        debug_assert!(is_valid(edge));
        debug_assert!(is_valid(to));

        let edge_data = self[edge];
        let v1 = edge_data.vert;

        let new_edge = if is_valid(edge_data.prev) {
            self.add_edge(HalfEdge {
                next: NO_EDGE, // will be new_oppsite
                prev: edge_data.prev,
                twin: NO_EDGE, // will be new_oppsite
                face: edge_data.face,
                vert: v1,
            })
        } else {
            NO_EDGE
        };

        let new_opposite = self.add_edge(HalfEdge {
            next: edge,
            prev: new_edge,
            twin: new_edge,
            face: edge_data.face,
            vert: to,
        });

        if is_valid(new_edge) {
            self[edge_data.prev].next = new_edge;
            let new_edge_data = &mut self[new_edge];
            new_edge_data.twin = new_opposite;
            new_edge_data.next = new_opposite;
        }

        self[edge].prev = new_opposite;

        new_opposite
    }

    /// Connect two vertices.
    ///
    /// Only use this on isolated vertices.
    pub fn add_segment(&mut self, v1: VertId, v2: VertId, face: FaceId) -> EdgeId {
        let e12 = self.add_edge(HalfEdge {
            next: NO_EDGE,
            prev: NO_EDGE,
            twin: NO_EDGE,
            vert: v1,
            face,
        });
        let e21 = self.add_edge(HalfEdge {
            next: e12,
            prev: e12,
            twin: e12,
            vert: v2,
            face,
        });

        {
            let edge12 = &mut self[e12];
            edge12.next = e21;
            edge12.prev = e21;
            edge12.twin = e21;
        }

        e12
    }

    // Add a loop of edges, using existing vertices.
    pub fn add_loop(
        &mut self,
        vertices: VertIdRange,
        inner_face: Option<FaceId>,
        outer_face: Option<FaceId>,
    ) -> EdgeId {
        //println!(" ------ adding loop {} - {}", vertices.start, vertices.end);
        let add_inner_loop = inner_face.is_some();
        let add_outer_loop = outer_face.is_some();
        debug_assert!(add_inner_loop || add_outer_loop);

        let num_vertices = vertices.len();

        debug_assert!(num_vertices > 1);
        let num_vert_i32 = num_vertices as i32;

        let base_edge = self.edges.len() as Index;
        let base_vertex = vertices.start;

        //println!("base edge {} base vertex {}", base_edge, base_vertex);

        let first_inner_edge = EdgeId::new(base_edge);
        let first_outer_edge =
            EdgeId::new(base_edge + if add_inner_loop { num_vertices } else { 0 });

        if add_inner_loop {
            let face = inner_face.unwrap();
            debug_assert!(is_valid(face));
            for (i, vertex) in vertices.enumerate() {
                debug_assert!(is_valid(vertex));
                let next_edge =
                    EdgeId::new(base_edge + modulo(i as i32 + 1, num_vert_i32) as Index);
                let prev_edge =
                    EdgeId::new(base_edge + modulo(i as i32 - 1, num_vert_i32) as Index);
                let opposite = if add_outer_loop {
                    EdgeId::new(base_edge + 2 * num_vertices - 1 - i as u16)
                } else {
                    NO_EDGE
                };
                let id = self.edges.push(HalfEdge {
                    vert: vertex,
                    next: next_edge,
                    prev: prev_edge,
                    twin: opposite,
                    face,
                });
                //println!(" ------ adding edge id {:?} vertex {:?} next {:?} prev {:?} opp {:?}",
                //    id.handle, vertex.handle, next_edge.handle, prev_edge.handle, opposite.handle);
                debug_assert_eq!(id, EdgeId::new(base_edge + i as u16));
            }
            self[face].first = first_inner_edge;
        }

        if add_outer_loop {
            let face = outer_face.unwrap();
            debug_assert!(is_valid(face));
            let base_edge_2 = if add_inner_loop {
                base_edge + num_vertices
            } else {
                base_edge
            };
            for i in 0..num_vert_i32 {
                let next_edge = EdgeId::new(base_edge_2 + modulo(i + 1, num_vert_i32) as Index);
                let prev_edge = EdgeId::new(base_edge_2 + modulo(i - 1, num_vert_i32) as Index);
                let opposite = if add_inner_loop {
                    EdgeId::new(base_edge + (num_vertices - 1) - (i as Index))
                } else {
                    NO_EDGE
                };
                let vertex = VertId::new(
                    base_vertex + (num_vertices - 1) - modulo(i - 1, num_vert_i32) as Index,
                );
                let id = self.edges.push(HalfEdge {
                    vert: vertex,
                    next: next_edge,
                    prev: prev_edge,
                    twin: opposite,
                    face,
                });
                //println!(" ------ adding opp edge id {:?} vertex {:?} next {:?} prev {:?} opp {:?}",
                //    id.handle, vertex.handle, next_edge.handle, prev_edge.handle, opposite.handle);
                debug_assert_eq!(id, EdgeId::new(base_edge_2 + i as Index));
            }
            // If outer_face already has edges, we assume that the loop is a hole in f2
            let face_data = &mut self[face];
            if is_valid(face_data.first) {
                //println!(" - add hole ");
                face_data.inner.push(first_outer_edge);
            } else {
                face_data.first = first_outer_edge;
            }
        }

        if add_inner_loop {
            first_inner_edge
        } else {
            first_outer_edge
        }
    }

    /// Add a loop of edges adn a face, creating a hole in an existing face.
    pub fn add_hole(&mut self, outer_face: FaceId, vertices: VertIdRange) -> FaceId {
        let hole_face = self.add_face();
        let _ = self.add_loop(vertices, Some(hole_face), Some(outer_face));
        hole_face
    }

    pub fn set_hole(&mut self, outer_face: FaceId, hole_interior_loop: EdgeId) {
        let opp = self[hole_interior_loop].twin;
        self[outer_face].inner.push(opp);
    }
}

impl ops::Index<EdgeId> for Connectivity {
    type Output = HalfEdge;
    fn index(&self, id: EdgeId) -> &HalfEdge {
        &self.edges[id]
    }
}

impl ops::IndexMut<EdgeId> for Connectivity {
    fn index_mut(&mut self, id: EdgeId) -> &mut HalfEdge {
        &mut self.edges[id]
    }
}

impl ops::Index<FaceId> for Connectivity {
    type Output = Face;
    fn index(&self, id: FaceId) -> &Face {
        &self.faces[id]
    }
}

impl ops::IndexMut<FaceId> for Connectivity {
    fn index_mut(&mut self, id: FaceId) -> &mut Face {
        &mut self.faces[id]
    }
}

fn vertex_range(first: u16, count: u16) -> VertIdRange {
    VertIdRange::new(first..(first + count))
}

/// Well behaved i32 modulo (doesn't mess up with negative values).
fn modulo(a: i32, m: i32) -> i32 {
    (a % m + m) % m
}

#[test]
fn test_add_segment() {
    let mut kernel = Connectivity::default();
    for _ in 0..5 {
        let f1 = kernel.add_face();
        let vertices = vertex_range(0, 2);
        let e = kernel.add_segment(vertices.nth(0), vertices.nth(1), f1);
        let o = kernel[e].twin;

        kernel.debug_assert_edge_invariants(e);
        kernel.debug_assert_edge_invariants(o);
        assert_eq!(kernel[e].face, f1);
        assert_eq!(kernel[e].next, o);
        assert_eq!(kernel[e].twin, o);
        assert_eq!(kernel[e].prev, o);
        assert_eq!(kernel[o].next, e);
        assert_eq!(kernel[o].twin, e);
        assert_eq!(kernel[o].prev, e);
        assert_eq!(kernel[o].face, f1);
    }
}

#[test]
fn test_extrude_vertex() {
    let mut kernel = Connectivity::default();
    for _ in 0..5 {
        //    v1 ----e1----> v2 ----o2---> v3
        //       <---o1-----    <---e2----
        let f1 = kernel.add_face();
        let vertices = vertex_range(0, 3);
        let v1 = vertices.nth(0);
        let v2 = vertices.nth(1);
        let v3 = vertices.nth(2);
        let e1 = kernel.add_segment(v1, v2, f1);
        let o1 = kernel[e1].twin;

        let e2 = kernel.extrude_vertex(o1, v3);
        let o2 = kernel[e2].twin;

        assert_eq!(kernel[e1].next, o2);
        assert_eq!(kernel[e1].prev, o1);
        assert_eq!(kernel[e2].next, o1);
        assert_eq!(kernel[e2].prev, o2);
        assert_eq!(kernel[o2].next, e2);
        assert_eq!(kernel[o2].prev, e1);

        assert_eq!(kernel[e2].vert, v3);
        assert_eq!(kernel[o2].vert, v2);

        assert_eq!(kernel[e2].face, f1);
        assert_eq!(kernel[o2].face, f1);

        kernel.debug_assert_edge_invariants(e1);
        kernel.debug_assert_edge_invariants(e2);
        kernel.debug_assert_edge_invariants(o1);
        kernel.debug_assert_edge_invariants(o2);
    }
}

//#[test]
//fn test_make_loop() {
//    let n_vertices = 4;
//    let mut kernel = ConnectivityKernel::new();
//    let f1 = kernel.add_face();
//    let f2 = kernel.add_face();
//    let vertices = vertex_range(0, n_vertices);
//    let v1 = vertices.nth(0);
//    let v2 = vertices.nth(1);
//    let first_edge = kernel.add_segment(v1, v2, f1);
//    let mut edge = first_edge;
//    for i in 2..n_vertices {
//        edge = kernel.extrude_vertex(edge, vertex_id(i));
//    }
//    // close the loop
//    let f2 = kernel.connect_edges(edge, first_edge).unwrap();
//
//    kernel[f1].first_edge = first_edge;
//    kernel[f2].first_edge = kernel[first_edge].opposite;
//
//    println!(" -- built loop, testing f1");
//    kernel.debug_assert_face_invariants(f1);
//    println!(" -- testing f1");
//    kernel.debug_assert_face_invariants(f2);
//}

#[test]
fn test_add_loop_with_vertices() {
    let mut kernel = Connectivity::default();
    for n_vertices in 3..10 {
        let vertex_ids = vertex_range(0, n_vertices);

        let f1 = kernel.add_face();
        let f2 = kernel.add_face();

        kernel.add_loop(vertex_ids, Some(f1), Some(f2));

        kernel.debug_assert_face_invariants(f1);
        kernel.debug_assert_face_invariants(f2);

        assert_eq!(
            kernel.walk_edge_ids_around_face(f1).count(),
            n_vertices as usize
        );

        let mut edge = NO_EDGE;
        for e in kernel.walk_edge_ids_around_face(f1) {
            if kernel[e].vert == vertex_ids.nth(0) {
                edge = e;
                break;
            }
        }
        assert!(edge != NO_EDGE);

        // Check that the winding order is correct.
        for i in 0..n_vertices {
            assert_eq!(kernel[edge].vert, vertex_ids.nth(i));
            assert_eq!(kernel[edge].face, f1);
            edge = kernel[edge].next;
        }
    }
}

#[test]
fn test_from_loop() {
    for n in 3..10 {
        //println!(" --- test {} ", n);
        let kernel = Connectivity::from_loop(vertex_range(0, n));
        let face = kernel.first_face().unwrap();

        assert_eq!(kernel.walk_edge_ids_around_face(face).count() as Index, n);

        let mut i = 0;
        for e in kernel.walk_edge_ids_around_face(face) {
            assert!(kernel.contains_edge(e));
            i += 1;
        }
        assert_eq!(i, n);

        for i in 0..(kernel.edges.len() as u16) {
            let e = EdgeId::new(i);
            assert_eq!(kernel[kernel[e].twin].twin, e);
            assert_eq!(kernel[kernel[e].next].prev, e);
            assert_eq!(kernel[kernel[e].prev].next, e);
        }

        for e in kernel.walk_edge_ids_around_face_reverse(face) {
            assert!(kernel.contains_edge(e));
            assert_eq!(kernel[e].face, face);
        }

        let face2 = kernel[kernel[kernel[face].first].twin].face;
        let mut i = 0;
        for e in kernel.walk_edge_ids_around_face_reverse(face2) {
            assert!(kernel.contains_edge(e));
            assert_eq!(kernel[e].face, face2);
            i += 1;
        }

        assert!(face2 != face);
        assert_eq!(i, n);
    }
}

#[test]
fn test_hole() {
    let mut kernel = Connectivity::from_loop(vertex_range(0, 4));

    let f1 = kernel.first_face().unwrap();
    kernel.add_hole(f1, vertex_range(4, 3));

    assert_eq!(kernel[f1].inner.len(), 1);
    let inner1 = kernel[f1].inner[0];
    for e in kernel.walk_edge_ids(inner1) {
        kernel.debug_assert_edge_invariants(e);
    }
    let inner_opp = kernel[inner1].twin;
    for e in kernel.walk_edge_ids(inner_opp) {
        kernel.debug_assert_edge_invariants(e);
    }

    for e in kernel.walk_edge_ids(kernel[f1].first) {
        kernel.debug_assert_edge_invariants(e);
    }
}

#[test]
fn test_connect_1() {
    let mut kernel = Connectivity::from_loop(vertex_range(0, 4));
    let f1 = kernel.first_face().unwrap();
    let e1 = kernel[f1].first;
    let e2 = kernel[e1].next;
    let e3 = kernel[e2].next;
    let e4 = kernel[e3].next;
    assert_eq!(kernel[e4].next, e1);
    assert_eq!(kernel.walk_edge_ids_around_face(f1).count(), 4);

    // x---e1---->x
    // ^          |
    // |          |
    // |          e2
    // e4   f1    |
    // |          |
    // |          v
    // x<-----e3--x

    kernel.connect_edges(e2, e1);

    // x---e1---->x
    // ^ \ ^   f1 |
    // | e5 \     |
    // |   \ \    e2
    // e4   \ \   |
    // |     \ e6 |
    // | f2   v \ v
    // x<-----e3--x

    let f2 = kernel[e4].face;
    assert!(f1 != f2);
    assert!(kernel[f1].first != kernel[f2].first);

    assert_eq!(kernel[kernel[f1].first].face, f1);
    assert_eq!(kernel[kernel[f2].first].face, f2);

    let e5 = kernel[e4].next;
    let e6 = kernel[e2].next;

    assert_eq!(kernel[e6].next, e1);
    assert_eq!(kernel[e1].prev, e6);
    assert_eq!(kernel[e5].next, e3);
    assert_eq!(kernel[e3].prev, e5);
    assert_eq!(kernel[e6].prev, e2);
    assert_eq!(kernel[e2].next, e6);
    assert_eq!(kernel[e5].prev, e4);
    assert_eq!(kernel[e4].next, e5);

    assert_eq!(kernel[e1].face, f1);
    assert_eq!(kernel[e2].face, f1);
    assert_eq!(kernel[e6].face, f1);
    assert_eq!(kernel[e3].face, f2);
    assert_eq!(kernel[e4].face, f2);
    assert_eq!(kernel[e5].face, f2);

    assert_eq!(kernel.walk_edge_ids_around_face(f1).count(), 3);
    assert_eq!(kernel.walk_edge_ids_around_face(f2).count(), 3);
}

#[test]
fn test_connect_2() {
    use crate::iter::{DirectedEdgeCirculator, Direction};

    let mut kernel = Connectivity::from_loop(vertex_range(0, 10));
    let f1 = kernel.first_face().unwrap();

    let e1 = kernel[f1].first;
    let e2 = kernel[e1].next;
    let e3 = kernel[e2].next;
    let e4 = kernel[e3].next;

    let f2 = kernel.connect_edges(e4, e2).unwrap();

    for e in kernel.walk_edge_ids_around_face(f2) {
        assert_eq!(kernel[e].face, f2);
    }

    for e in kernel.walk_edge_ids_around_face(f1) {
        assert_eq!(kernel[e].face, f1);
    }

    for dir in [Direction::Forward, Direction::Backward].iter() {
        for face in [f1, f2].iter() {
            let mut it = DirectedEdgeCirculator::new(&kernel, kernel[*face].first, *dir);
            let stop = it.prev();
            loop {
                assert_eq!(it.face_id(), *face);
                if it == stop {
                    break;
                }
                it = it.next();
            }
        }
    }
}
