use super::{
    math::ExtVec2,
    segment::{Segment, SegmentVert},
};
use bevy::prelude::*;
use bmesh::{BMesh, BMeshData, EdgeKey, EdgeSlot, VertKey};

#[derive(Resource, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Building {
    pub floors: Vec<Floor>,
}

impl Building {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let reader = std::io::BufReader::new(std::fs::File::open(path)?);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        Ok(serde_json::to_writer(writer, self)?)
    }
}

type DualEdge = (HalfEdge, HalfEdge);
type HalfEdge = (EdgeKey, Vec2, f32);

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FloorVert {
    pub position: Vec2,
    pub cap_width: f32,
}

impl FloorVert {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            cap_width: 0.1,
        }
    }

    pub fn new_snap(size: Vec2, position: Vec2) -> Self {
        Self::new((position / size).round() * size)
    }
}

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FloorEdge {
    pub hole: Option<Hole>,
    pub prev_width: f32,
    pub next_width: f32,
}

impl BMeshData for Floor {
    type Vert = FloorVert;
    type Edge = FloorEdge;
    type Link = ();
    type Face = ();
}

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Hole {
    pub offset: Vec2,
    pub size: Vec2,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Floor {
    pub mesh: BMesh<Self>,
    pub min: f32,
    pub max: f32,

    pub ref_scale: f32,
    pub ref_offset_x: f32,
    pub ref_offset_y: f32,
}

impl Default for Floor {
    fn default() -> Self {
        Self {
            mesh: BMesh::default(),
            min: 0.0,
            max: 1.0,

            ref_scale: 1.0,
            ref_offset_x: 0.0,
            ref_offset_y: 0.0,
        }
    }
}

impl Floor {
    pub fn new_face(&mut self, selected: &mut [VertKey]) {
        let reverse = !bmesh::is_ccw(selected, |&v| self.mesh.verts[v].position.into());
        if reverse {
            selected.reverse();
        }

        if let Some(edges) = self.mesh.edges_from_verts(selected) {
            let iter = selected.iter().zip(edges.iter());
            self.mesh
                .face_create(iter.map(|(&vert, &edge)| (vert, edge, ())), ());
        }

        if reverse {
            selected.reverse();
        }
    }

    pub fn hovered(
        &self,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        pointer: Vec3,
        max_distance: f32,
    ) -> Option<VertKey> {
        if let Ok(pointer) = camera.world_to_viewport(camera_transform, pointer) {
            let iter = self.mesh.verts.iter();
            iter.filter_map(|(key, slot)| {
                let p = slot.position.extend_y(self.min);
                let p = camera.world_to_viewport(camera_transform, p);
                p.ok().map(|p| (key, p.distance(pointer)))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .filter(|&(_, distance)| distance < max_distance)
            .map(|(key, _)| key)
        } else {
            None
        }
    }

    pub fn sort_segments(&mut self) {
        let mut order = vec![];
        self.mesh.sort_disk_all(&mut order, compare_segments);
    }

    pub fn walls(&self) -> impl Iterator<Item = [Vec2; 2]> {
        let edges = self.segments().flat_map(|Segment { prev, next, .. }| {
            [
                Some([prev.next, next.prev]),
                Some([next.next, prev.prev]),
                (prev.valence == 1).then_some([prev.prev, prev.next]),
                (next.valence == 1).then_some([next.prev, next.next]),
            ]
        });
        edges.flatten()
    }

    pub fn find_pair(&self, edge: EdgeKey, vert: VertKey, sign: f32) -> Option<DualEdge> {
        let [prev, next] = self.mesh.edges[edge]
            .disk(vert)
            .and_then(|d| (d.prev != edge && d.next != edge).then_some([d.prev, d.next]))?
            .map(|edge| {
                let slot = &self.mesh.edges[edge];
                let [s, e] = slot.pair(vert).unwrap().map(|disk| disk.vert);
                let width = if slot.prev.vert == vert {
                    [slot.prev_width, slot.next_width]
                } else {
                    [slot.next_width, slot.prev_width]
                };
                let dir = (self.mesh.verts[e].position - self.mesh.verts[s].position).normalize();
                (edge, dir, width)
            });

        Some((
            (prev.0, sign * prev.1, prev.2[0]),
            (next.0, sign * next.1, next.2[1]),
        ))
    }

    pub fn segments(&self) -> SegmentsIter<'_> {
        SegmentsIter {
            floor: self,
            edges: self.mesh.edges.iter(),
        }
    }

    pub fn segment(&self, edge: EdgeKey) -> Segment {
        let slot = &self.mesh.edges[edge];
        let [prev_vert, next_vert] = [slot.prev.vert, slot.next.vert];
        let [prev_data, next_data] = [prev_vert, next_vert].map(|v| *self.mesh.verts[v]);
        let direction = (next_data.position - prev_data.position).normalize();

        let prev_pair = self.find_pair(edge, prev_vert, 1.0);
        let next_pair = self.find_pair(edge, next_vert, -1.0);

        let (prev_prev, prev_next) = prev_pair.unzip();
        let (next_prev, next_next) = next_pair.unzip();

        let prev_cap = prev_pair.map_or(direction * prev_data.cap_width, |_| Vec2::ZERO);
        let next_cap = next_pair.map_or(direction * next_data.cap_width, |_| Vec2::ZERO);

        Segment {
            direction,
            hole: slot.hole,

            prev: SegmentVert {
                vert: prev_vert,
                valence: self.mesh.vert_valence(prev_vert),
                prev: prev_data.position - mity(direction, slot.next_width, prev_prev) - prev_cap,
                curr: prev_data.position,
                next: prev_data.position + mity(direction, slot.prev_width, prev_next) - prev_cap,
                width: slot.prev_width,
            },

            next: SegmentVert {
                vert: next_vert,
                valence: self.mesh.vert_valence(next_vert),
                prev: next_data.position + mity(direction, slot.prev_width, next_prev) + next_cap,
                curr: next_data.position,
                next: next_data.position - mity(direction, slot.next_width, next_next) + next_cap,
                width: slot.next_width,
            },
        }
    }
}

pub struct SegmentsIter<'a> {
    floor: &'a Floor,
    edges: bmesh::smap::Iter<'a, EdgeKey, EdgeSlot<Floor>>,
}

impl std::iter::FusedIterator for SegmentsIter<'_> {}
impl std::iter::ExactSizeIterator for SegmentsIter<'_> {}

impl Iterator for SegmentsIter<'_> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let (edge, _) = self.edges.next()?;
        Some(self.floor.segment(edge))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.edges.size_hint()
    }
}

fn mity(first: Vec2, u: f32, second: Option<HalfEdge>) -> Vec2 {
    if let Some((_, second, v)) = second {
        let det_sin = first.x * second.y - first.y * second.x;
        if approx::abs_diff_eq!(det_sin, 0.0) {
            let a = first * u;
            let b = second * v;
            ((a - b) / 2.0).perp()
        } else {
            (first * v + second * u) / det_sin
        }
    } else {
        first.perp() * u
    }
}

// #[inline(always)]
// fn lerp(a: f32, b: f32, t: f32) -> f32 {
//     (1.0 - t) * a + t * b
// }

fn compare_segments(
    mesh: &BMesh<Floor>,
    vert: VertKey,
    a: &EdgeSlot<Floor>,
    b: &EdgeSlot<Floor>,
) -> std::cmp::Ordering {
    let a = {
        let pair = a.pair(vert).unwrap();
        let [prev, next] = pair.map(|disk| mesh.verts[disk.vert].position);
        (next - prev).normalize().to_angle()
    };

    let b = {
        let pair = b.pair(vert).unwrap();
        let [prev, next] = pair.map(|disk| mesh.verts[disk.vert].position);
        (next - prev).normalize().to_angle()
    };

    a.total_cmp(&b)
}
