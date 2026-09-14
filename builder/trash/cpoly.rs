use emath::Vec2;
use std::collections::HashMap;

fn dot(lhs: Vec2, rhs: Vec2) -> f32 {
    lhs.x * rhs.y + lhs.y * rhs.x
}

fn cross_perp_dot(lhs: Vec2, rhs: Vec2) -> f32 {
    lhs.x * rhs.y - lhs.y * rhs.x
}

fn signed_area(a: Vec2, b: Vec2) -> f32 {
    (b.x - a.x) * (b.y + a.y)
}

//     static Vec2 norm(Vec2 const & v)
//     {
//         if(length(v) < 1e-30)
//             return {0.0f, 0.0f};
//     }

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    position: Vec2,
}

fn handedness(v1: Vertex, v2: Vertex, v3: Vertex) -> f32 {
    let e21 = v2.position - v1.position;
    let e32 = v3.position - v2.position;
    cross_perp_dot(e21, e32)
}

struct SliceVertex {
    vertex: Vertex,
    index: usize,
    distance_to_slice: f32,
}

#[derive(Clone, Copy)]
struct LineSegment(Vec2, Vec2);

impl LineSegment {
    fn direction(&self) -> Vec2 {
        self.1 - self.0
    }

    fn add(self, rhs: Self) -> Self {
        let start = (self.0 + rhs.0) / 2.0;
        let end = (self.1 + rhs.1) / 2.0;
        Self(start, end)
    }

    fn intersects(a: Self, b: Self) -> Option<Vec2> {
        let tolerance = 1e-2;

        let [sa, sb] = [a.0, b.0];
        let [da, db] = [a.direction(), b.direction()];

        if cross_perp_dot(da, db).abs() < 1e-30 {
            return None;
        }

        let t1 = cross_perp_dot(sb - sa, db) / cross_perp_dot(da, db);
        if t1 < 0.0 - tolerance || t1 > 1.0 + tolerance {
            return None;
        }

        let intersect = sa + da * t1;
        let t2 = dot(intersect - sb, b.1 - sb);
        let ee = b.1 - sb;

        if t2 < 0.0 - tolerance || t2 / dot(ee, ee) >= 1.0 - tolerance {
            None
        } else {
            Some(intersect)
        }
    }
}

#[derive(Clone)]
struct ConcavePolygon {
    vertices: Vec<Vertex>,
    polygons: Vec<Self>,
}

fn umod(x: usize, m: usize) -> usize {
    (x + m) % m
}

fn flip_polygon(verts: &mut [Vertex]) {
    verts.reverse();
    /*
    let mut limit = verts.len() / 2;
    if verts.len() % 2 != 0 {
        limit += 1;
    }
    for i in 1..limit {
        verts.swap(i, verts.len() - 1);
    }
    */
}

fn check_if_right_handed(slice: &[Vertex]) -> bool {
    if slice.len() < 3 {
        false
    } else {
        0.0 > (0..slice.len())
            .map(|i| signed_area(slice[i].position, slice[umod(i + 1, slice.len())].position))
            .sum::<f32>()
    }
}

fn is_vertex_in_cone(a: LineSegment, b: LineSegment, origin: Vec2, vert: Vertex) -> bool {
    let diff = vert.position - origin;
    let a = cross_perp_dot(diff, a.direction()) < 0.0;
    let b = cross_perp_dot(diff, b.direction()) > 0.0;
    a && b
}

fn find_vertices_in_cone(
    a: LineSegment,
    b: LineSegment,
    origin: Vec2,
    input: &[Vertex],
) -> impl Iterator<Item = usize> + '_ {
    let iter = input.iter().enumerate();
    iter.filter_map(move |(i, &vert)| is_vertex_in_cone(a, b, origin, vert).then_some(i))
}

fn check_visibility(start: Vec2, vert: Vertex, polygon: &[Vertex]) -> bool {
    let segment = LineSegment(start, vert.position);
    let intersecting = dbg!(vertices_along_line_segment(segment, polygon).count());
    intersecting <= 3
}

fn vertices_along_line_segment(
    a: LineSegment,
    vertices: &[Vertex],
) -> impl Iterator<Item = (usize, Vertex)> + '_ {
    (0..vertices.len()).filter_map(move |i| {
        let start = vertices[i].position;
        let end = vertices[umod(i + 1, vertices.len())].position;
        let b = LineSegment(start, end);
        LineSegment::intersects(a, b).map(|position| (i, Vertex { position }))
    })
}

fn best_vertex_to_connect(indices: &[usize], polygon: &[Vertex], origin: Vec2) -> Option<usize> {
    match indices.len() {
        0 => None,
        1 => check_visibility(origin, polygon[indices[0]], polygon).then_some(indices[0]),
        _ => {
            for &index in indices {
                let prev = polygon[umod(index - 1, polygon.len())];
                let curr = polygon[index];
                let next = polygon[umod(index + 1, polygon.len())];

                let a = LineSegment(prev.position, curr.position);
                let b = LineSegment(next.position, curr.position);

                if handedness(prev, curr, next) < 0.0
                    && is_vertex_in_cone(a, b, curr.position, Vertex { position: origin })
                    && check_visibility(origin, curr, polygon)
                {
                    return Some(index);
                }
            }

            for &index in indices {
                let prev = polygon[umod(index - 1, polygon.len())];
                let curr = polygon[index];
                let next = polygon[umod(index + 1, polygon.len())];

                if handedness(prev, curr, next) < 0.0 && check_visibility(origin, curr, polygon) {
                    return Some(index);
                }
            }

            let mut min = 1e+15;
            let mut closest = indices[0];
            for &index in indices {
                let diff = polygon[index].position - origin;
                let distance = dot(diff, diff);
                if distance < min {
                    min = distance;
                    closest = index;
                }
            }

            Some(closest)
        }
    }
}

impl ConcavePolygon {
    fn convex_decomp(&mut self) {
        if self.polygons.len() > 0 {
            return;
        }

        let Some(index) = find_first_reflex_vertex(&self.vertices) else {
            return;
        };

        let prev = self.vertices[umod(index - 1, self.vertices.len())].position;
        let curr = self.vertices[index].position;
        let next = self.vertices[umod(index + 1, self.vertices.len())].position;

        let ls1 = LineSegment(prev, curr);
        let ls2 = LineSegment(next, curr);

        let in_cone = find_vertices_in_cone(ls1, ls2, curr, &self.vertices).collect::<Vec<_>>();

        let mut best_vert = None;

        if !in_cone.is_empty() {
            best_vert = best_vertex_to_connect(&in_cone, &self.vertices, curr);
        }

        if let Some(best_vert) = best_vert {
            self.slice_polygon_segment(LineSegment(curr, self.vertices[best_vert].position));
        }

        if in_cone.is_empty() || best_vert.is_none() {
            let second = (ls1.direction() + ls2.direction()) * 1e+10;
            self.slice_polygon_segment(LineSegment(curr, second));
        }

        for sub in &mut self.polygons {
            if sub.vertices.len() > 3 {
                sub.convex_decomp();
            }
        }
    }

    fn slice_polygon_segment(&mut self, segment: LineSegment) {
        todo!()
        /*
        if !subPolygons.is_empty() {
            subPolygons[0].slice_polygon_segment(segment);
            subPolygons[1].slice_polygon_segment(segment);
            return;
        }

        const float TOLERANCE = 1e-5;

        VertexIntMap slicedVertices = verticesAlongLineSegment(segment, vertices);
        slicedVertices = cullByDistance(slicedVertices, segment.startPos, 2);

        if(slicedVertices.size() < 2)
            return;

        VertexArray leftVerts;
        VertexArray rightVerts;

        for(int i=0; i<(int)vertices.size(); ++i)
        {
            Vec2 relativePosition = vertices[i].position - segment.startPos;

            auto it = slicedVertices.begin();

            float perpDistance = std::abs(Vec2::cross(relativePosition, segment.direction()));

            if( perpDistance > TOLERANCE ||
              ( perpDistance <= TOLERANCE && (slicedVertices.find(i)==slicedVertices.end()) )
            )
            {
                //std::cout << relCrossProd << ", i: " << i << "\n";
                if((i > it->first) && (i <= (++it)->first))
                {
                    leftVerts.push_back(vertices[i]);
                    //std::cout << i << " leftVertAdded\n";
                }
                else
                {
                    rightVerts.push_back(vertices[i]);
                    //std::cout << i << " rightVertAdded\n";
                }

            }

            if(slicedVertices.find(i) != slicedVertices.end())
            {
                rightVerts.push_back(slicedVertices[i]);
                leftVerts.push_back(slicedVertices[i]);
            }
        }

        subPolygons.push_back(ConcavePolygon(leftVerts));
        subPolygons.push_back(ConcavePolygon(rightVerts));
        */
    }

    /*
    fn slice_polygon(&mut self, vertex1: usize, vertex2: usize) {
        if vertex1 == vertex2 || vertex2 == vertex1 + 1 || vertex2 == vertex1 - 1 {
            return;
        }

        let (vertex1, vertex2) = if vertex1 > vertex2 {
            (vertex2, vertex1)
        } else {
            (vertex1, vertex2)
        };

        /*
        VertexArray returnVerts;
        VertexArray newVerts;
        for(int i=0; i<(int)vertices.size(); ++i)
        {
            if(i==vertex1 || i==vertex2)
            {
                returnVerts.push_back(vertices[i]);
                newVerts.push_back(vertices[i]);
            }
            else if(i > vertex1 && i <vertex2)
                returnVerts.push_back(vertices[i]);
            else
                newVerts.push_back(vertices[i]);
        }

        subPolygons.push_back(ConcavePolygon(returnVerts));
        subPolygons.push_back(ConcavePolygon(newVerts));
        */
    }
    */
}

fn find_first_reflex_vertex(vertices: &[Vertex]) -> Option<usize> {
    (0..vertices.len()).find(|&i| {
        let prev = vertices[umod(i - 1, vertices.len())];
        let next = vertices[umod(i + 1, vertices.len())];
        0.0 > handedness(prev, vertices[i], next)
    })
}

fn cull_by_distance(
    input: &HashMap<usize, Vertex>,
    origin: Vec2,
    max_verts_to_keep: usize,
) -> HashMap<usize, Vertex> {
    assert!(max_verts_to_keep > 0);

    if max_verts_to_keep >= input.len() {
        return input.clone();
    }

    let mut slice = input
        .iter()
        .map(|(&index, &vertex)| SliceVertex {
            index,
            vertex,
            distance_to_slice: dot(vertex.position - origin, vertex.position - origin),
        })
        .collect::<Vec<_>>();

    for i in 1..slice.len() {
        let mut j = i;
        while j > 0 && slice[j].distance_to_slice < slice[j - 1].distance_to_slice {
            slice.swap(j, j - 1);
            j -= 1
        }
    }

    while slice.len() > max_verts_to_keep {
        slice.pop().unwrap();
    }

    for i in 1..slice.len() {
        let mut j = i;
        while j > 0 && slice[j].index < slice[j - 1].index {
            slice.swap(j, j - 1);
            j -= 1;
        }
    }

    slice.into_iter().map(|v| (v.index, v.vertex)).collect()
}

impl ConcavePolygon {
    pub fn lowest_level_polys(&self, dst: &mut Vec<Self>) {
        if self.polygons.len() >= 2 {
            self.polygons[0].lowest_level_polys(dst);
            self.polygons[1].lowest_level_polys(dst);
        } else {
            dst.push(self.clone());
        }
    }
}

/*

public:
    ConcavePolygon(VertexArray const & _vertices) : vertices{_vertices}
    {
        if(vertices.size() > 2)
            if(checkIfRightHanded() == false)
                flipPolygon();
    }
    ConcavePolygon() {}

    bool checkIfRightHanded()
    {
        return checkIfRightHanded(vertices);
    }


    Vec2 getPoint(unsigned int index) const
    {
        if(index >= 0 && index < vertices.size())
            return vertices[index].position;

        return {0.0f, 0.0f};
    }

}


*/
