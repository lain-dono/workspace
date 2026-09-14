use crate::id_internals::is_valid;
use crate::kernel::{Connectivity, EdgeId, FaceId, HalfEdge, NO_EDGE, VertId};
use std::mem::transmute;

/// Iterates over the half edges around a face.
pub struct EdgeIdLoop<'a> {
    kernel: &'a Connectivity,
    curr: EdgeId,
    last: EdgeId,
    done: bool,
}

impl<'a> Iterator for EdgeIdLoop<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        let current = self.curr;
        if !self.done {
            self.done |= self.curr == self.last;
            if self.curr != NO_EDGE {
                self.curr = self.kernel.edge(self.curr).next();
                Some(current)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> EdgeIdLoop<'a> {
    pub fn new(kernel: &'a Connectivity, first: EdgeId, last: EdgeId) -> Self {
        Self {
            kernel,
            curr: first,
            last,
            done: false,
        }
    }
}

impl Connectivity {
    pub fn iter_edge_loop(&self, edge_loop: EdgeId) -> EdgeIdLoop<'_> {
        EdgeIdLoop {
            kernel: self,
            curr: edge_loop,
            last: self.edge(edge_loop).prev(),
            done: false,
        }
    }
}

/// Iterates over the half edges around a face.
pub struct MutEdgeLoop<'a> {
    kernel: &'a mut Connectivity,
    current_edge: EdgeId,
    last_edge: EdgeId,
    done: bool,
}

impl<'a> Iterator for MutEdgeLoop<'a> {
    type Item = &'a mut HalfEdge;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.current_edge;
        if self.done {
            return None;
        }
        if self.current_edge == self.last_edge {
            self.done = true;
        }
        if self.current_edge == NO_EDGE {
            return None;
        }
        self.current_edge = self.kernel[self.current_edge].next;
        // TODO could remove transmute
        Some(unsafe { transmute::<&mut HalfEdge, &mut HalfEdge>(&mut self.kernel[res]) })
    }
}

impl<'a> MutEdgeLoop<'a> {
    pub fn new(kernel: &'a mut Connectivity, first: EdgeId, last: EdgeId) -> Self {
        Self {
            kernel,
            current_edge: first,
            last_edge: last,
            done: false,
        }
    }
}

/// Iterates over the half edges around a face in reverse order.
pub struct ReverseEdgeIdLoop<'a> {
    kernel: &'a Connectivity,
    curr: EdgeId,
    last: EdgeId,
    done: bool,
}

impl<'a> Iterator for ReverseEdgeIdLoop<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        let res = self.curr;
        if self.done {
            return None;
        }
        if self.curr == self.last {
            self.done = true;
        }
        self.curr = self.kernel[self.curr].prev;
        Some(res)
    }
}

impl<'a> ReverseEdgeIdLoop<'a> {
    pub fn new(kernel: &'a Connectivity, first: EdgeId, last: EdgeId) -> Self {
        Self {
            kernel,
            curr: first,
            last,
            done: false,
        }
    }
}

/// Iterates over the half edges that point to a vertex.
pub struct VertEdgeIterator<'a> {
    kernel: &'a Connectivity,
    curr: EdgeId,
    start: EdgeId,
}

impl<'a> Iterator for VertEdgeIterator<'a> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        if !is_valid(self.curr) {
            return None;
        }
        let temp = self.curr;
        self.curr = self.kernel[self.kernel[self.curr].next].twin;
        if self.curr == self.start {
            self.curr = NO_EDGE;
        }
        Some(temp)
    }
}

//pub struct VertexIdIterator {
//    current: Index,
//    stop: Index,
//}
//
//impl<'a> Iterator for VertexIdIterator {
//    type Item = VertexId;
//
//    fn next(&mut self) -> Option<VertexId> {
//        if self.current == self.stop { return None; }
//        let res = self.current;
//        self.current += 1;
//        return Some(vertex_id(res));
//    }
//}
//
//pub struct EdgeIdIterator {
//    current: Index,
//    stop: Index,
//}
//
//impl<'a> Iterator for EdgeIdIterator {
//    type Item = EdgeId;
//
//    fn next(&mut self) -> Option<EdgeId> {
//        if self.current == self.stop { return None; }
//        let res = self.current;
//        self.current += 1;
//        return Some(edge_id(res));
//    }
//}
//
//pub struct FaceIdIterator {
//    current: Index,
//    stop: Index,
//}
//
//impl<'a> Iterator for FaceIdIterator {
//    type Item = FaceId;
//
//    fn next(&mut self) -> Option<FaceId> {
//        if self.current == self.stop { return None; }
//        let res = self.current;
//        self.current += 1;
//        return Some(face_id(res));
//    }
//}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    pub fn reverse(self) -> Direction {
        match self {
            Self::Forward => Direction::Backward,
            Self::Backward => Direction::Forward,
        }
    }
}

#[derive(Copy, Clone)]
pub struct EdgeCirculator<'a> {
    kernel: &'a Connectivity,
    edge: EdgeId,
}

impl<'a> EdgeCirculator<'a> {
    pub fn new(kernel: &'a Connectivity, edge: EdgeId) -> Self {
        Self { kernel, edge }
    }

    pub fn edge(&'a self) -> &'a HalfEdge {
        &self.kernel[self.edge]
    }

    pub fn next(self) -> Self {
        Self {
            kernel: self.kernel,
            edge: self.edge().next,
        }
    }

    pub fn prev(self) -> Self {
        Self {
            kernel: self.kernel,
            edge: self.edge().prev,
        }
    }

    pub fn advance(self, direction: Direction) -> Self {
        match direction {
            Direction::Forward => self.next(),
            Direction::Backward => self.prev(),
        }
    }

    pub fn edge_id(&self) -> EdgeId {
        self.edge
    }

    pub fn vertex_id(&self) -> VertId {
        self.edge().vert
    }

    pub fn face_id(&self) -> FaceId {
        self.edge().face
    }
}

impl<'a> PartialEq<EdgeCirculator<'a>> for EdgeCirculator<'a> {
    fn eq(&self, other: &EdgeCirculator) -> bool {
        self.edge.eq(&other.edge)
    }
}

#[derive(Copy, Clone)]
pub struct DirectedEdgeCirculator<'a> {
    circulator: EdgeCirculator<'a>,
    direction: Direction,
}

impl<'a> DirectedEdgeCirculator<'a> {
    pub fn new(kernel: &'a Connectivity, edge: EdgeId, direction: Direction) -> Self {
        Self {
            circulator: EdgeCirculator::new(kernel, edge),
            direction,
        }
    }

    pub fn edge(&'a self) -> &'a HalfEdge {
        self.circulator.edge()
    }

    pub fn next(self) -> Self {
        Self {
            circulator: self.circulator.advance(self.direction),
            direction: self.direction,
        }
    }

    pub fn prev(self) -> Self {
        Self {
            circulator: self.circulator.advance(self.direction.reverse()),
            direction: self.direction,
        }
    }

    pub fn advance(self, direction: Direction) -> Self {
        match self.direction == direction {
            true => self.next(),
            false => self.prev(),
        }
    }

    pub fn edge_id(&self) -> EdgeId {
        self.circulator.edge
    }

    pub fn vertex_id(&self) -> VertId {
        self.circulator.vertex_id()
    }

    pub fn face_id(&self) -> FaceId {
        self.circulator.face_id()
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

impl<'a> PartialEq<DirectedEdgeCirculator<'a>> for DirectedEdgeCirculator<'a> {
    fn eq(&self, other: &DirectedEdgeCirculator) -> bool {
        self.circulator.edge.eq(&other.circulator.edge)
    }
}
