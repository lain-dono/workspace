pub use crate::{
    face::{FaceKey, FaceSlot, LinkKey, LinkSlot},
    iter::{DiskIter, LinkIter},
    smap::Slots,
    triangulation::{Polygon, is_ccw},
    wire::{Disk, EdgeKey, EdgeSlot, VertKey, VertSlot},
};

macro_rules! define_key {
    ($name:ident [$slot:ident => $kind:ident]) => {
        #[derive(
            Copy,
            Clone,
            Default,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            Debug,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(pub(crate) crate::smap::KeyData);

        #[allow(dead_code)]
        impl $name {
            pub(crate) const fn null() -> Self {
                Self(crate::smap::KeyData::null())
            }

            pub(crate) const fn is_null(self) -> bool {
                self.0.index == u32::MAX
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<crate::smap::KeyData> for $name {
            fn from(value: crate::smap::KeyData) -> Self {
                Self(value)
            }
        }

        impl From<$name> for crate::smap::KeyData {
            fn from($name(key): $name) -> Self {
                key
            }
        }

        impl<T: BMeshData> std::ops::Deref for $slot<T> {
            type Target = T::$kind;
            fn deref(&self) -> &Self::Target {
                &self.data
            }
        }

        impl<T: BMeshData> std::ops::DerefMut for $slot<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.data
            }
        }
    };
}

mod triangulation;

mod disk_sort;
mod face;
mod face_kernel;
mod iter;
mod kernel;
pub mod smap;
mod wire;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum EdgeVertNotFound {
    Prev,
    Next,
}

#[derive(Debug)]
pub struct VertNotFound;

#[derive(Debug)]
pub struct EdgeNotFound;

pub trait BMeshData: Default {
    type Vert;
    type Edge;
    type Link;
    type Face;
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "
        T::Vert: serde::Serialize,
        T::Edge: serde::Serialize,
        T::Link: serde::Serialize,
        T::Face: serde::Serialize",
    deserialize = "
        T::Vert: for<'de_v> serde::Deserialize<'de_v>,
        T::Edge: for<'de_e> serde::Deserialize<'de_e>,
        T::Link: for<'de_l> serde::Deserialize<'de_l>,
        T::Face: for<'de_f> serde::Deserialize<'de_f>",
))]
pub struct BMesh<T: BMeshData> {
    pub verts: Slots<VertKey, VertSlot<T>>,
    pub edges: Slots<EdgeKey, EdgeSlot<T>>,
    pub links: Slots<LinkKey, LinkSlot<T>>,
    pub faces: Slots<FaceKey, FaceSlot<T>>,
}

impl<T: BMeshData> Default for BMesh<T>
where
    T::Vert: Default,
    T::Edge: Default,
    T::Link: Default,
    T::Face: Default,
{
    fn default() -> Self {
        Self {
            verts: Slots::default(),
            edges: Slots::default(),
            links: Slots::default(),
            faces: Slots::default(),
        }
    }
}

impl<T: BMeshData> Clone for BMesh<T>
where
    VertSlot<T>: Clone,
    EdgeSlot<T>: Clone,
    LinkSlot<T>: Clone,
    FaceSlot<T>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            verts: self.verts.clone(),
            edges: self.edges.clone(),
            links: self.links.clone(),
            faces: self.faces.clone(),
        }
    }
}

impl<T: BMeshData> BMesh<T> {
    pub fn clear(&mut self) {
        self.verts.clear();
        self.edges.clear();
        self.links.clear();
        self.faces.clear();
    }
}

impl<T: BMeshData> BMesh<T> {
    pub fn vert_make(&mut self, data: T::Vert) -> VertKey {
        let edge = None;
        self.verts.insert(VertSlot { edge, data })
    }

    pub fn vert_kill(&mut self, vert: VertKey) {
        if self.verts.contains_key(vert) {
            while let Some(edge) = self.verts[vert].edge {
                self.edge_kill(edge).unwrap();
            }
            self.verts.remove(vert).unwrap();
        }
    }
}

impl<T: BMeshData> BMesh<T> {
    pub fn edge_find(&self, v_a: VertKey, v_b: VertKey) -> Option<EdgeKey> {
        assert_ne!(v_a, v_b);

        let [e_a, e_b] = [v_a, v_b].map(|v| self.verts[v].edge);
        let (e_a, e_b) = e_a.zip(e_b)?;

        let (mut iter_a, mut iter_b) = (e_a, e_b);

        loop {
            if self.edges[iter_a].contains_vert(v_b) {
                return Some(iter_a);
            }
            if self.edges[iter_b].contains_vert(v_a) {
                return Some(iter_b);
            }

            let next_a = self.edges[iter_a].disk(v_a)?.next;
            let next_b = self.edges[iter_b].disk(v_b)?.next;

            if next_a != e_a && next_b != e_b {
                (iter_a, iter_b) = (next_a, next_b);
            } else {
                break None;
            }
        }
    }

    pub fn edge_find_or_make(&mut self, prev: VertKey, next: VertKey, data: T::Edge) -> EdgeKey {
        self.edge_find(prev, next)
            .unwrap_or_else(|| self.edge_make(prev, next, data))
    }

    pub fn edge_make(&mut self, prev: VertKey, next: VertKey, data: T::Edge) -> EdgeKey {
        assert_ne!(prev, next);
        assert!(self.verts.contains_key(prev));
        assert!(self.verts.contains_key(next));

        let edge = self.edges.next_key();
        let edge = EdgeSlot {
            data,
            prev: self.new_disk_link(edge, prev),
            next: self.new_disk_link(edge, next),
            link: None,
        };
        self.edges.insert(edge)
    }

    pub fn edge_kill(&mut self, edge: EdgeKey) -> Result<(), EdgeNotFound> {
        while let Some(link) = self.edges[edge].link {
            self.face_kill(self.links[link].face);
        }

        let dl = self.edges.get(edge).map(|data| [data.prev, data.next]);
        for dl in dl.ok_or(EdgeNotFound)? {
            self.edges[dl.prev][dl.vert].next = dl.next;
            self.edges[dl.next][dl.vert].prev = dl.prev;
            if self.verts[dl.vert].edge == Some(edge) {
                self.verts[dl.vert].edge = (edge != dl.next).then_some(dl.next);
            }
        }

        let _ = self.edges.remove(edge).unwrap();

        Ok(())
    }
}

impl<T: BMeshData> BMesh<T> {
    fn _disk_append(&mut self, e: EdgeKey, v: VertKey) {
        match self.verts[v].edge {
            Some(ve) => {
                let [e_dl, v_dl] = self
                    .edges
                    .get_disjoint_mut([e, ve])
                    .map(|[e, ve]| [&mut e[v], &mut ve[v]])
                    .unwrap();

                e_dl.next = ve;
                e_dl.prev = v_dl.prev;

                let prev = v_dl.prev;
                v_dl.prev = e;

                self.edges[prev][v].next = e;
            }
            _ => {
                self.edges[e][v] = Disk::new(v, e, e);
                self.verts[v].edge = Some(e);
            }
        }
    }

    fn _disk_edge_append(&mut self, e: EdgeKey, v: VertKey) {
        match self.verts[v].edge {
            Some(ve) => {
                let [e_dl, v_dl] = self
                    .edges
                    .get_disjoint_mut([e, ve])
                    .map(|[e, ve]| [&mut e[v], &mut ve[v]])
                    .unwrap();

                e_dl.next = ve;
                e_dl.prev = v_dl.prev;

                let prev = v_dl.prev;
                v_dl.prev = e;
                self.edges[prev][v].next = e;
            }
            _ => {
                self.edges[e][v] = Disk::new(v, e, e);
                self.verts[v].edge = Some(e);
            }
        }
    }

    fn disk_edge_remove(&mut self, edge: EdgeKey, vert: VertKey) {
        let dl = self.edges[edge][vert];

        self.edges[dl.prev][vert].next = dl.next;
        self.edges[dl.next][vert].prev = dl.prev;

        if self.verts[vert].edge == Some(edge) {
            self.verts[vert].edge = (edge != dl.next).then_some(dl.next);
        }
    }

    pub fn edges_from_verts(&mut self, verts: &[VertKey]) -> Option<Vec<EdgeKey>> {
        let mut edges = vec![];
        let mut prev = verts.len() - 1;
        for next in 0..verts.len() {
            if let Some(edge) = self.edge_find(verts[prev], verts[next]) {
                edges.push(edge);
            } else {
                return None;
            }
            prev = next;
        }
        Some(edges)
    }
}
