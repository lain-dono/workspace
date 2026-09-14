use super::floor::{Floor, Hole};
use super::math::{ExtVec2, clamp_projection};
use super::mesh_builder::MeshBuilder;
use bevy::prelude::*;
use bmesh::{FaceKey, LinkSlot, VertKey};

#[derive(Debug, Clone, Copy)]
pub struct SegmentVert {
    pub vert: VertKey,
    pub prev: Vec2,
    pub curr: Vec2,
    pub next: Vec2,
    pub valence: usize,
    pub width: f32,
}

pub struct Segment {
    pub direction: Vec2,
    pub hole: Option<Hole>,
    pub prev: SegmentVert,
    pub next: SegmentVert,
}

impl Segment {
    #[inline(always)]
    fn normal([a, b]: [Vec2; 2]) -> Vec3 {
        (b - a).normalize().perp().extend_y(0.0)
    }

    #[inline(always)]
    fn quad([a, b]: [Vec2; 2], min: f32, max: f32) -> [Vec3; 4] {
        let [a_min, a_max] = [a.extend_y(min), a.extend_y(max)];
        let [b_min, b_max] = [b.extend_y(min), b.extend_y(max)];
        [a_max, a_min, b_min, b_max]
    }

    pub fn build(&self, builder: &mut MeshBuilder, min: f32, max: f32) {
        let color = [1.0; 4];

        let [prev, next] = [self.prev, self.next];

        if prev.valence == 1 {
            let pts = [prev.prev, prev.next];
            builder.quad(Self::quad(pts, min, max), Self::normal(pts), color);
        }
        if next.valence == 1 {
            let pts = [next.prev, next.next];
            builder.quad(Self::quad(pts, min, max), Self::normal(pts), color);
        }

        {
            let v_min = [prev.prev, next.next, next.prev, prev.next];
            let v_max = [prev.next, next.prev, next.next, prev.prev];
            builder.quad(v_min.map(|p| p.extend_y(min)), Vec3::NEG_Y, color);
            builder.quad(v_max.map(|p| p.extend_y(max)), Vec3::Y, color);
        }

        if prev.valence > 2 {
            let p = [prev.prev, prev.curr, prev.next];
            let min = p.map(|p| p.extend_y(min));
            let max = p.map(|p| p.extend_y(max));
            builder.triangle(min, [2, 1, 0], Vec3::NEG_Y, color);
            builder.triangle(max, [0, 1, 2], Vec3::Y, color);
        }
        if next.valence > 2 {
            let p = [next.prev, next.curr, next.next];
            let min = p.map(|p| p.extend_y(min));
            let max = p.map(|p| p.extend_y(max));
            builder.triangle(min, [2, 1, 0], Vec3::NEG_Y, color);
            builder.triangle(max, [0, 1, 2], Vec3::Y, color);
        }

        if let Some(Hole { offset, size }) = self.hole {
            let direction = self.direction;
            let norm = direction.perp();

            let [l_pos, l_neg] = [[prev.next, next.prev], [next.next, prev.prev]];

            let [o_pos, o_neg] = [l_pos, l_neg].map(|p| Self::quad(p, min, max));
            let [n_pos, n_neg] = [l_pos, l_neg].map(Self::normal);

            let [i_pos, i_neg] = {
                let [p, n] = [prev.width, -next.width];
                let pos = [prev.curr, next.curr];
                let neg = [next.curr, prev.curr];

                let cy = (min + max) * 0.5 + offset.y;
                let [min, max] = [cy - size.y, cy + size.y].map(|v| v.clamp(min, max));

                [(pos, l_pos, p), (neg, l_neg, n)].map(|(p, limit, w)| {
                    let [a, b] = p.map(|p| p + norm * w);
                    let center = a.midpoint(b) + direction * offset.x;
                    let size = direction * size.x * w.signum();
                    let pts = [center - size, center + size].map(|p| clamp_projection(limit, p));
                    Self::quad(pts, min, max)
                })
            };

            let positive = ([o_pos, i_pos], n_pos);
            let negative = ([o_neg, i_neg], n_neg);

            let indices = [
                4, 7, 3, 4, 3, 0, 6, 5, 1, 6, 1, 2, 1, 5, 4, 1, 4, 0, 6, 2, 3, 6, 3, 7,
            ];
            for (vertices, normal) in [positive, negative] {
                let vertices = vertices.as_flattened().iter().copied();
                builder.extend(vertices, indices, normal, color);
            }

            let normal = direction.extend_y(0.0);
            builder.quad([i_pos[1], i_pos[2], i_neg[1], i_neg[2]], Vec3::Y, color);
            builder.quad([i_pos[3], i_pos[0], i_neg[3], i_neg[0]], Vec3::NEG_Y, color);
            builder.quad([i_pos[1], i_neg[2], i_neg[3], i_pos[0]], normal, color);
            builder.quad([i_neg[1], i_pos[2], i_pos[3], i_neg[0]], -normal, color);
        } else {
            let pts = [prev.next, next.prev];
            builder.quad(Self::quad(pts, min, max), Self::normal(pts), color);

            let pts = [next.next, prev.prev];
            builder.quad(Self::quad(pts, min, max), Self::normal(pts), color);
        }
    }
}

impl Floor {
    pub fn build(&self, builder: &mut MeshBuilder<'_>) {
        let fix = 0.0001;
        let [min, max] = [self.min + fix, self.max - fix];

        for segment in self.segments() {
            segment.build(builder, min, max);
        }

        let [min_up, max_up] = [true, false];
        let color = [1.0, 0.0, 0.0, 1.0];
        let mut mesher = FaceTriangulation::default();

        for (face, _) in &self.mesh.faces {
            let (face_indices, face_vertices) = mesher.triangulate_face(self, face);

            let indices = face_indices.iter().copied();
            let vertices = face_vertices.iter().map(|p| p.extend_y(min));
            if min_up {
                builder.extend(vertices, indices.rev(), Vec3::Y, color);
            } else {
                builder.extend(vertices, indices, Vec3::NEG_Y, color);
            }

            let indices = face_indices.iter().copied();
            let vertices = face_vertices.iter().map(|p| p.extend_y(max));
            if max_up {
                builder.extend(vertices, indices.rev(), Vec3::Y, color);
            } else {
                builder.extend(vertices, indices, Vec3::NEG_Y, color);
            }
        }
    }
}

#[derive(Default)]
pub struct FaceTriangulation {
    vertices: Vec<Vec2>,
    indices: Vec<u32>,
    mesher: bmesh::Polygon<u32>,
}

impl FaceTriangulation {
    pub fn triangulate_face(&mut self, floor: &Floor, face: FaceKey) -> (&[u32], &[Vec2]) {
        self.vertices.clear();
        self.indices.clear();
        self.mesher.vertices.clear();

        let face = &floor.mesh.faces[face];
        let mut link = face.link;
        loop {
            let LinkSlot { vert, edge, .. } = floor.mesh.links[link];
            let Segment { prev, next, .. } = floor.segment(edge);

            self.vertices.push(if prev.vert == vert {
                prev.prev
            } else {
                next.prev
            });

            link = floor.mesh.links[link].face_next;
            if link == face.link {
                break;
            }
        }

        let iter = self.vertices.iter();
        let iter = iter.enumerate().map(|(i, &p)| (i as u32, p.into()));

        self.mesher.vertices.extend(iter);
        self.mesher.triangulate(&mut self.indices);

        (&self.indices, &self.vertices)
    }
}
