//! A Rust port of the [Earcut](https://github.com/mapbox/earcut) polygon triangulation library.

mod nodes;
pub mod utils3d;

use self::nodes::{NodeIndex, Nodes};
use std::ptr;

pub trait Position {
    fn position(&self) -> [f32; 3];
}

/// Index of a vertex
pub trait Index: Copy + PartialEq + Eq {
    fn into_usize(self) -> usize;
    fn from_usize(v: usize) -> Self;
}
impl Index for u32 {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl Index for u16 {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl Index for usize {
    fn into_usize(self) -> usize {
        self
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}

struct Node<I> {
    /// vertex index in coordinates array
    index: I,

    /// z-order curve value
    z: i32,

    /// vertex coordinates x
    xy: [f32; 2],

    /// prev vertex nodes in a polygon ring
    prev: NodeIndex,
    /// next vertex nodes in a polygon ring
    next: NodeIndex,

    /// previous nodes in z-order
    prev_z: Option<NodeIndex>,
    /// next nodes in z-order
    next_z: Option<NodeIndex>,

    /// indicates whether this is a steiner point
    steiner: bool,
}

struct LinkInfo {
    prev: NodeIndex,
    next: NodeIndex,
    prev_z: Option<NodeIndex>,
    next_z: Option<NodeIndex>,
}

impl<I> Node<I> {
    fn new(index: I, xy: [f32; 2]) -> Self {
        Self {
            index,
            xy,
            prev: unsafe { NodeIndex::new_unchecked(1) },
            next: unsafe { NodeIndex::new_unchecked(1) },
            z: 0,
            prev_z: None,
            next_z: None,
            steiner: false,
        }
    }

    fn link_info(&self) -> LinkInfo {
        LinkInfo {
            prev: self.prev,
            next: self.next,
            prev_z: self.prev_z,
            next_z: self.next_z,
        }
    }
}

/// Instance of the earcut algorithm.
#[derive(Default)]
pub struct Earcut<I> {
    data: Vec<[f32; 2]>,
    nodes: Nodes<I>,
}

impl<I: Index> Earcut<I> {
    /// Creates a new instance of the earcut algorithm.
    ///
    /// You can reuse a single instance for multiple triangulations to reduce memory allocations.
    pub fn new() -> Self
    where
        I: Default,
    {
        Self {
            data: Vec::new(),
            nodes: Nodes::default(),
        }
    }

    fn reset(&mut self, capacity: usize) {
        self.nodes.data.clear();
        if let Some(additional) = capacity.checked_sub(self.nodes.data.capacity()) {
            self.nodes.data.reserve(additional);
        }

        let index = I::from_usize(0);
        self.nodes.push(Node::new(index, [f32::INFINITY; 2])); // dummy node
    }

    /// Performs the earcut triangulation on a polygon.
    ///
    /// The API is similar to the original JavaScript implementation, except you can provide a vector for the output indices.
    pub fn earcut<Iter>(&mut self, data: Iter, output: &mut Vec<[I; 3]>)
    where
        Iter: IntoIterator<Item = [f32; 2]>,
    {
        self.data.clear();
        self.data.extend(data);
        output.clear();
        if self.data.len() >= 3 {
            self.earcut_impl(output);
        }
    }

    pub fn earcut_impl(&mut self, output: &mut Vec<[I; 3]>) {
        if let Some(additional) = (self.data.len() + 1).checked_sub(output.capacity()) {
            output.reserve(additional);
        }

        self.reset(self.data.len() / 2 * 3);

        let outer_len = self.data.len();

        // create nodes
        let Some(outer_node_index) = self.linked_list(0, outer_len, true) else {
            return;
        };

        let outer_node = &self.nodes[outer_node_index];
        if outer_node.next == outer_node.prev {
            return;
        }

        let mut min_x = 0.0;
        let mut min_y = 0.0;
        let mut inv_size = 0.0;

        // if the shape is not too simple, we'll use z-order curve hash later; calculate polygon bbox
        if self.data.len() > 80 {
            let slice = &self.data[1..outer_len];

            let [max_x, max_y] = slice.iter().fold(self.data[0], |[ax, ay], &[bx, by]| {
                [f32::max(ax, bx), f32::max(ay, by)]
            });

            [min_x, min_y] = slice.iter().fold(self.data[0], |[ax, ay], &[bx, by]| {
                [f32::min(ax, bx), f32::min(ay, by)]
            });

            // minX, minY and invSize are later used to transform coords into integers for z-order calculation
            inv_size = (max_x - min_x).max(max_y - min_y);
            if inv_size != 0.0 {
                inv_size = 32767.0 / inv_size;
            }
        }

        self.nodes
            .earcut_linked(outer_node_index, output, min_x, min_y, inv_size, Pass::P0);
    }

    /// create a circular doubly linked list from polygon points in the specified winding order
    fn linked_list(&mut self, start: usize, end: usize, clockwise: bool) -> Option<NodeIndex> {
        let mut last_index: Option<NodeIndex> = None;
        let iter = self.data[start..end].iter().enumerate();

        if clockwise == (signed_area(&self.data, start, end) > 0.0) {
            for (index, &xy) in iter {
                let index = I::from_usize(start + index);
                last_index = Some(self.nodes.insert(index, xy, last_index));
            }
        } else {
            for (index, &xy) in iter.rev() {
                let index = I::from_usize(start + index);
                last_index = Some(self.nodes.insert(index, xy, last_index));
            }
        }

        if let Some(last) = last_index.map(|index| &self.nodes[index]) {
            if last.xy == self.nodes[last.next].xy {
                let (_, next) = self.nodes.remove(last.link_info());
                last_index = Some(next);
            }
        }

        last_index
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    P0 = 0,
    P1 = 1,
    P2 = 2,
}

impl<I: Index> Nodes<I> {
    /// main ear slicing loop which triangulates a polygon (given as a linked list)
    fn earcut_linked(
        &mut self,
        ear_i: NodeIndex,
        triangles: &mut Vec<[I; 3]>,
        min_x: f32,
        min_y: f32,
        inv_size: f32,
        pass: Pass,
    ) {
        let mut ear_index = ear_i;

        // interlink polygon nodes in z-order
        if pass == Pass::P0 && inv_size != 0.0 {
            self.index_curve(ear_index, min_x, min_y, inv_size);
        }

        let mut stop_index = ear_index;

        // iterate through ears, slicing them one by one
        loop {
            let ear = &self[ear_index];
            if ear.prev == ear.next {
                break;
            }

            let ear_next = ear.next;

            let (is_ear, prev, next) = if inv_size != 0.0 {
                self.is_ear_hashed(ear, min_x, min_y, inv_size)
            } else {
                self.is_ear(ear)
            };

            if is_ear {
                let next_index = next.index;
                let next_next = next.next;

                // cut off the triangle
                triangles.push([prev.index, ear.index, next_index]);

                self.remove(ear.link_info());

                // skipping the next vertex leads to less sliver triangles
                (ear_index, stop_index) = (next_next, next_next);

                continue;
            }

            ear_index = ear_next;

            // if we looped through the whole remaining polygon and can't find any more ears
            if ear_index == stop_index {
                match pass {
                    Pass::P0 => {
                        // try filtering points and slicing again
                        ear_index = self.filter_points(ear_index, None);
                        self.earcut_linked(ear_index, triangles, min_x, min_y, inv_size, Pass::P1);
                    }
                    Pass::P1 => {
                        // if this didn't work, try curing all small self-intersections locally
                        let filtered = self.filter_points(ear_index, None);
                        ear_index = self.cure_local_intersections(filtered, triangles);
                        self.earcut_linked(ear_index, triangles, min_x, min_y, inv_size, Pass::P2);
                    }
                    Pass::P2 => {
                        // as a last resort, try splitting the remaining polygon into two
                        self.split_earcut(ear_index, triangles, min_x, min_y, inv_size);
                    }
                }
                return;
            }
        }
    }

    /// check whether a polygon node forms a valid ear with adjacent nodes
    fn is_ear<'a>(&'a self, ear: &'a Node<I>) -> (bool, &'a Node<I>, &'a Node<I>) {
        let b = ear;
        let a = &self[b.prev];
        let c = &self[b.next];

        if tri_area(a.xy, b.xy, c.xy) >= 0.0 {
            // reflex, can't be an ear
            return (false, a, c);
        }

        // now make sure we don't have other points inside the potential ear

        // triangle bbox
        let x0 = a.xy[0].min(b.xy[0].min(c.xy[0]));
        let y0 = a.xy[1].min(b.xy[1].min(c.xy[1]));
        let x1 = a.xy[0].max(b.xy[0].max(c.xy[0]));
        let y1 = a.xy[1].max(b.xy[1].max(c.xy[1]));

        let mut p = &self[c.next];
        let mut p_prev = &self[p.prev];
        while !ptr::eq(p, a) {
            let p_next = &self[p.next];
            if (p.xy[0] >= x0 && p.xy[0] <= x1 && p.xy[1] >= y0 && p.xy[1] <= y1)
                && point_in_triangle([a.xy, b.xy, c.xy], p.xy)
                && tri_area(p_prev.xy, p.xy, p_next.xy) >= 0.0
            {
                return (false, a, c);
            }
            (p_prev, p) = (p, p_next);
        }
        (true, a, c)
    }

    fn is_ear_hashed<'a>(
        &'a self,
        ear: &'a Node<I>,
        min_x: f32,
        min_y: f32,
        inv_size: f32,
    ) -> (bool, &'a Node<I>, &'a Node<I>) {
        let b = ear;
        let a = &self[b.prev];
        let c = &self[b.next];

        if tri_area(a.xy, b.xy, c.xy) >= 0.0 {
            // reflex, can't be an ear
            return (false, a, c);
        }

        // triangle bbox
        let xy_min = [[a.xy[0], b.xy[0], c.xy[0]], [a.xy[1], b.xy[1], c.xy[1]]]
            .map(|[a, b, c]| a.min(b.min(c)));

        let xy_max = [[a.xy[0], b.xy[0], c.xy[0]], [a.xy[1], b.xy[1], c.xy[1]]]
            .map(|[a, b, c]| a.max(b.max(c)));

        let contains = |abc: [[f32; 2]; 3], [x, y]: [f32; 2]| {
            let in_bbox = x >= xy_min[0] && x <= xy_max[0] && y >= xy_min[1] && y <= xy_max[1];
            in_bbox && point_in_triangle(abc, [x, y])
        };

        // z-order range for the current triangle bbox;
        let min_z = z_order(xy_min, min_x, min_y, inv_size);
        let max_z = z_order(xy_max, min_x, min_y, inv_size);

        let mut o_p = ear.prev_z.map(|i| &self[i]);
        let mut o_n = ear.next_z.map(|i| &self[i]);

        // look for points inside the triangle in both directions
        loop {
            let Some(p) = o_p else { break };
            if p.z < min_z {
                break;
            }

            let Some(n) = o_n else { break };
            if n.z > max_z {
                break;
            }

            let p_ab = !(ptr::eq(p, a) || ptr::eq(p, c));

            if (p_ab && contains([a.xy, b.xy, c.xy], p.xy))
                && tri_area(self[p.prev].xy, p.xy, self[p.next].xy) >= 0.0
            {
                return (false, a, c);
            }
            o_p = p.prev_z.map(|i| &self[i]);

            let n_ab = !(ptr::eq(n, a) || ptr::eq(n, c));

            if (n_ab && contains([a.xy, b.xy, c.xy], n.xy))
                && tri_area(self[n.prev].xy, n.xy, self[n.next].xy) >= 0.0
            {
                return (false, a, c);
            }
            o_n = n.next_z.map(|i| &self[i]);
        }

        // look for remaining points in decreasing z-order
        while let Some(p) = o_p {
            if p.z < min_z {
                break;
            }

            let p_ab = !(ptr::eq(p, a) || ptr::eq(p, c));

            if (p_ab && contains([a.xy, b.xy, c.xy], p.xy))
                && tri_area(self[p.prev].xy, p.xy, self[p.next].xy) >= 0.0
            {
                return (false, a, c);
            }
            o_p = p.prev_z.map(|i| &self[i]);
        }

        // look for remaining points in increasing z-order
        while let Some(n) = o_n {
            if n.z > max_z {
                break;
            }

            let n_ab = !(ptr::eq(n, a) || ptr::eq(n, c));

            if (n_ab && contains([a.xy, b.xy, c.xy], n.xy))
                && tri_area(self[n.prev].xy, n.xy, self[n.next].xy) >= 0.0
            {
                return (false, a, c);
            }
            o_n = n.next_z.map(|i| &self[i]);
        }

        (true, a, c)
    }

    /// go through all polygon nodes and cure small local self-intersections
    fn cure_local_intersections(
        &mut self,
        mut start: NodeIndex,
        triangles: &mut Vec<[I; 3]>,
    ) -> NodeIndex {
        let mut iter = start;
        loop {
            let p = &self[iter];
            let p_next_index = p.next;
            let p_next = &self[p_next_index];
            let b_index = p_next.next;
            let a = &self[p.prev];
            let b = &self[b_index];

            if a.xy != b.xy
                && intersects([a.xy, p.xy], [p_next.xy, b.xy])
                && self.locally_inside(a, b)
                && self.locally_inside(b, a)
            {
                triangles.push([a.index, p.index, b.index]);

                let b_next_index = b.next;
                self.remove(p.link_info());
                let pnl = self[p_next_index].link_info();
                self.remove(pnl);

                (iter, start) = (b_next_index, b_index);
            } else {
                iter = p.next;
            }

            if iter == start {
                return self.filter_points(iter, None);
            }
        }
    }

    /// try splitting polygon into two and triangulate them independently
    fn split_earcut(
        &mut self,
        start: NodeIndex,
        triangles: &mut Vec<[I; 3]>,
        min_x: f32,
        min_y: f32,
        inv_size: f32,
    ) {
        // look for a valid diagonal that divides the polygon into two
        let mut ai = start;
        let mut a = &self[ai];
        loop {
            let a_next = &self[a.next];
            let a_prev = &self[a.prev];
            let mut bi = a_next.next;

            while bi != a.prev {
                let b = &self[bi];
                if a.index != b.index && self.is_valid_diagonal(a, b, a_next, a_prev) {
                    // split the polygon in two by the diagonal
                    let mut ci = self.split_polygon(ai, bi);

                    // filter colinear points around the cuts
                    let end_i = Some(self[ai].next);
                    ai = self.filter_points(ai, end_i);

                    let end_i = Some(self[ci].next);
                    ci = self.filter_points(ci, end_i);

                    // run earcut on each half
                    self.earcut_linked(ai, triangles, min_x, min_y, inv_size, Pass::P0);
                    self.earcut_linked(ci, triangles, min_x, min_y, inv_size, Pass::P0);
                    return;
                }
                bi = b.next;
            }

            ai = a.next;
            if ai == start {
                return;
            }
            a = a_next;
        }
    }

    /// interlink polygon nodes in z-order
    fn index_curve(&mut self, start_i: NodeIndex, min_x: f32, min_y: f32, inv_size: f32) {
        let mut index = start_i;
        let mut node = &mut self[index];

        loop {
            if node.z == 0 {
                node.z = z_order(node.xy, min_x, min_y, inv_size);
            }
            node.prev_z = Some(node.prev);
            node.next_z = Some(node.next);
            index = node.next;
            node = &mut self[index];
            if index == start_i {
                break;
            }
        }

        let prev_z = node.prev_z.take().unwrap();
        self[prev_z].next_z = None;
        self.sort_linked(index);
    }

    /// Simon Tatham's linked list merge sort algorithm
    /// http://www.chiark.greenend.org.uk/~sgtatham/algorithms/listsort.html
    fn sort_linked(&mut self, list_i: NodeIndex) {
        let mut in_size: u32 = 1;
        let mut list_i = Some(list_i);

        loop {
            let mut p_i = list_i;
            list_i = None;
            let mut tail_i: Option<NodeIndex> = None;
            let mut num_merges = 0;

            while let Some(p_i_s) = p_i {
                num_merges += 1;
                let mut q_i = self[p_i_s].next_z;
                let mut p_size: u32 = 1;
                for _ in 1..in_size {
                    if let Some(i) = q_i {
                        p_size += 1;
                        q_i = self[i].next_z;
                    } else {
                        break;
                    }
                }
                let mut q_size = in_size;

                loop {
                    let e_i = if p_size > 0 {
                        let Some(p_i_s) = p_i else { break };
                        if q_size > 0 {
                            if let Some(q_i_s) = q_i {
                                if self[p_i_s].z <= self[q_i_s].z {
                                    p_size -= 1;
                                    let e = &mut self[p_i_s];
                                    e.prev_z = tail_i;
                                    p_i = e.next_z;
                                    p_i_s
                                } else {
                                    q_size -= 1;
                                    let e = &mut self[q_i_s];
                                    e.prev_z = tail_i;
                                    q_i = e.next_z;
                                    q_i_s
                                }
                            } else {
                                p_size -= 1;
                                let e = &mut self[p_i_s];
                                e.prev_z = tail_i;
                                p_i = e.next_z;
                                p_i_s
                            }
                        } else {
                            p_size -= 1;
                            let e = &mut self[p_i_s];
                            e.prev_z = tail_i;
                            p_i = e.next_z;
                            p_i_s
                        }
                    } else if q_size > 0 {
                        if let Some(q_i_s) = q_i {
                            q_size -= 1;
                            let e = &mut self[q_i_s];
                            e.prev_z = tail_i;
                            q_i = e.next_z;
                            q_i_s
                        } else {
                            break;
                        }
                    } else {
                        break;
                    };

                    if let Some(tail_i) = tail_i {
                        self[tail_i].next_z = Some(e_i);
                    } else {
                        list_i = Some(e_i);
                    }
                    tail_i = Some(e_i);
                }

                p_i = q_i;
            }

            self[tail_i.unwrap()].next_z = None;
            if num_merges <= 1 {
                break;
            }
            in_size *= 2;
        }
    }

    /// check if a diagonal between two polygon nodes is valid (lies in polygon interior)
    fn is_valid_diagonal(
        &self,
        a: &Node<I>,
        b: &Node<I>,
        a_next: &Node<I>,
        a_prev: &Node<I>,
    ) -> bool {
        let b_next = &self[b.next];
        let b_prev = &self[b.prev];

        // dones't intersect other edges
        if a_next.index == b.index || a_prev.index == b.index || self.intersects_polygon(a, b) {
            return false;
        }

        // locally visible
        if !self.locally_inside(a, b) || !self.locally_inside(b, a) || !self.middle_inside(a, b) {
            return false;
        }

        // does not create opposite-facing sectors
        if tri_area(a_prev.xy, a.xy, b_prev.xy) == 0.0 && tri_area(a.xy, b_prev.xy, b.xy) == 0.0 {
            return false;
        }

        // special zero-length case
        a.xy == b.xy
            && tri_area(a_prev.xy, a.xy, a_next.xy) > 0.0
            && tri_area(b_prev.xy, b.xy, b_next.xy) > 0.0
    }

    /// check if a polygon diagonal intersects any polygon segments
    fn intersects_polygon(&self, a: &Node<I>, b: &Node<I>) -> bool {
        let mut p = a;
        loop {
            let p_next = &self[p.next];
            let curr_index = p.index != a.index && p.index != b.index;
            let next_index = p_next.index != a.index && p_next.index != b.index;

            if curr_index && next_index && intersects([p.xy, p_next.xy], [a.xy, b.xy]) {
                return true;
            }

            p = p_next;
            if ptr::eq(p, a) {
                return false;
            }
        }
    }

    /// check if the middle point of a polygon diagonal is inside the polygon
    fn middle_inside(&self, a: &Node<I>, b: &Node<I>) -> bool {
        let mut p = a;
        let mut inside = false;
        let (px, py) = ((a.xy[0] + b.xy[0]) / 2.0, (a.xy[1] + b.xy[1]) / 2.0);
        loop {
            let p_next = &self[p.next];
            inside ^= (p.xy[1] > py) != (p_next.xy[1] > py)
                && p_next.xy[1] != p.xy[1]
                && (px
                    < (p_next.xy[0] - p.xy[0]) * (py - p.xy[1]) / (p_next.xy[1] - p.xy[1])
                        + p.xy[0]);
            p = p_next;
            if ptr::eq(p, a) {
                return inside;
            }
        }
    }

    /// check if a polygon diagonal is locally inside the polygon
    fn locally_inside(&self, a: &Node<I>, b: &Node<I>) -> bool {
        let a_prev = &self[a.prev];
        let a_next = &self[a.next];
        if tri_area(a_prev.xy, a.xy, a_next.xy) < 0.0 {
            tri_area(a.xy, b.xy, a_next.xy) >= 0.0 && tri_area(a.xy, a_prev.xy, b.xy) >= 0.0
        } else {
            tri_area(a.xy, b.xy, a_prev.xy) < 0.0 || tri_area(a.xy, a_next.xy, b.xy) < 0.0
        }
    }

    /// eliminate colinear or duplicate points
    fn filter_points(&mut self, start_i: NodeIndex, end_i: Option<NodeIndex>) -> NodeIndex {
        let mut end_i = end_i.unwrap_or(start_i);

        let mut node_index = start_i;
        let mut node = &self[node_index];
        loop {
            let next = &self[node.next];
            if !node.steiner
                && (node.xy == next.xy || tri_area(self[node.prev].xy, node.xy, next.xy) == 0.0)
            {
                let (prev_i, next_i) = self.remove(node.link_info());
                (node_index, end_i) = (prev_i, prev_i);
                if node_index == next_i {
                    return end_i;
                }
                node = &self[node_index];
            } else {
                node_index = node.next;
                if node_index == end_i {
                    return end_i;
                }
                node = next;
            };
        }
    }

    /// link two polygon vertices with a bridge; if the vertices belong to the same ring, it splits polygon into two;
    /// if one belongs to the outer ring and another to a hole, it merges it into a single ring
    fn split_polygon(&mut self, a_i: NodeIndex, b_i: NodeIndex) -> NodeIndex {
        debug_assert!(!self.data.is_empty());

        let a2_i = unsafe { NodeIndex::new_unchecked(self.data.len() as u32) };
        let b2_i = unsafe { NodeIndex::new_unchecked(self.data.len() as u32 + 1) };

        let a = &mut self[a_i];
        let mut a2 = Node::new(a.index, a.xy);
        let an_i = a.next;
        a.next = b_i;
        a2.prev = b2_i;
        a2.next = an_i;

        let b = &mut self[b_i];
        let mut b2 = Node::new(b.index, b.xy);
        let bp_i = b.prev;
        b.prev = a_i;
        b2.next = a2_i;
        b2.prev = bp_i;

        self[an_i].prev = a2_i;
        self[bp_i].next = b2_i;

        self.data.extend([a2, b2]);

        b2_i
    }
}

/// check if two segments intersect
fn intersects([p1, q1]: [[f32; 2]; 2], [p2, q2]: [[f32; 2]; 2]) -> bool {
    let o1 = sign(tri_area(p1, q1, p2));
    let o2 = sign(tri_area(p1, q1, q2));
    let o3 = sign(tri_area(p2, q2, p1));
    let o4 = sign(tri_area(p2, q2, q1));
    (o1 != o2 && o3 != o4) // general case
        || (o3 == 0 && on_segment(p2, p1, q2)) // p2, q2 and p1 are collinear and p1 lies on p2q2
        || (o4 == 0 && on_segment(p2, q1, q2)) // p2, q2 and q1 are collinear and q1 lies on p2q2
        || (o2 == 0 && on_segment(p1, q2, q1)) // p1, q1 and q2 are collinear and q2 lies on p1q1
        || (o1 == 0 && on_segment(p1, p2, q1)) // p1, q1 and p2 are collinear and p2 lies on p1q1
}

/// Returns a percentage difference between the polygon area and its triangulation area;
/// used to verify correctness of triangulation
pub fn deviation<N: Index>(data: &[[f32; 2]], triangles: &[[N; 3]]) -> f32 {
    let outer_len = data.len();
    let polygon_area = if data.len() < 3 {
        0.0
    } else {
        signed_area(data, 0, outer_len).abs()
    };

    let mut triangles_area = 0.0;
    for abc in triangles {
        let [a, b, c] = abc.map(|index| data[index.into_usize()]);
        triangles_area += ((a[0] - c[0]) * (b[1] - a[1]) - (a[0] - b[0]) * (c[1] - a[1])).abs();
    }
    if polygon_area == 0.0 && triangles_area == 0.0 {
        0.0
    } else {
        ((polygon_area - triangles_area) / polygon_area).abs()
    }
}

/// check if a point lies within a convex triangle
fn signed_area(data: &[[f32; 2]], start: usize, end: usize) -> f32 {
    let [mut bx, mut by] = data[end - 1];
    let mut sum = 0.0;
    for &[ax, ay] in &data[start..end] {
        sum += (bx - ax) * (ay + by);
        (bx, by) = (ax, ay);
    }
    sum
}

/// z-order of a point given coords and inverse of the longer side of data bbox
fn z_order(xy: [f32; 2], min_x: f32, min_y: f32, inv_size: f32) -> i32 {
    // coords are transformed into non-negative 15-bit integer range
    let x = ((xy[0] - min_x) * inv_size) as u32;
    let y = ((xy[1] - min_y) * inv_size) as u32;
    let mut xy = ((x as i64) << 32) | y as i64;
    xy = (xy | (xy << 8)) & 0x00FF00FF00FF00FF;
    xy = (xy | (xy << 4)) & 0x0F0F0F0F0F0F0F0F;
    xy = (xy | (xy << 2)) & 0x3333333333333333;
    xy = (xy | (xy << 1)) & 0x5555555555555555;
    ((xy >> 32) | (xy << 1)) as i32
}

fn point_in_triangle([[ax, ay], [bx, by], [cx, cy]]: [[f32; 2]; 3], [px, py]: [f32; 2]) -> bool {
    let aa = (cx - px) * (ay - py) >= (ax - px) * (cy - py);
    let bb = (ax - px) * (by - py) >= (bx - px) * (ay - py);
    let cc = (bx - px) * (cy - py) >= (cx - px) * (by - py);
    aa && bb && cc
}

/// signed area of a triangle
#[inline]
fn area<I: Index>(p: &Node<I>, q: &Node<I>, r: &Node<I>) -> f32 {
    tri_area(p.xy, q.xy, r.xy)
}

#[inline]
fn tri_area([px, py]: [f32; 2], [qx, qy]: [f32; 2], [rx, ry]: [f32; 2]) -> f32 {
    (qy - py) * (rx - qx) - (qx - px) * (ry - qy)
}

/// for collinear points p, q, r, check if point q lies on segment pr
#[inline]
fn on_segment([px, py]: [f32; 2], [qx, qy]: [f32; 2], [rx, ry]: [f32; 2]) -> bool {
    let [min_x, min_y] = [px.min(rx), py.min(ry)];
    let [max_x, max_y] = [px.max(rx), py.max(ry)];
    qx <= max_x && qy <= max_y && qx >= min_x && qy >= min_y
}

fn sign(v: f32) -> i32 {
    (v > 0.0) as i32 - (v < 0.0) as i32
}
