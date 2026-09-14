use super::{BMesh, GraphData, KeyData, LoopKey};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EdgeKey(KeyData);

impl From<KeyData> for EdgeKey {
    fn from(value: KeyData) -> Self {
        Self(value)
    }
}

impl From<EdgeKey> for KeyData {
    fn from(EdgeKey(key): EdgeKey) -> Self {
        key
    }
}

#[derive(Clone)]
pub struct DiskEdgeLink {
    pub vert: VertKey,
    pub next: Option<EdgeKey>,
    pub prev: Option<EdgeKey>,
}

pub struct EdgeSlot<T: GraphData> {
    pub loop_key: Option<LoopKey>,
    pub link: [DiskEdgeLink; 2],
    pub data: T::Edge,
}

impl<T: GraphData> EdgeSlot<T> {
    #[inline]
    pub fn is_wire(&self) -> bool {
        self.loop_key.is_none()
    }

    #[inline]
    fn link(&self, v: VertKey) -> Option<&DiskEdgeLink> {
        if v == self.link[0].vert {
            Some(&self.link[0])
        } else if v == self.link[1].vert {
            Some(&self.link[1])
        } else {
            None
        }
    }

    #[inline]
    fn link_mut(&mut self, v: VertKey) -> Option<&mut DiskEdgeLink> {
        if v == self.link[0].vert {
            Some(&mut self.link[0])
        } else if v == self.link[1].vert {
            Some(&mut self.link[1])
        } else {
            None
        }
    }

    // #[inline]
    // fn other(&self, v: VertKey) -> Option<&DiskEdgeLink> {
    //     Some(&self.link[(self.index(v)? + 1) % 2])
    // }

    // #[inline]
    // fn other_mut(&mut self, v: VertKey) -> Option<&mut DiskEdgeLink> {
    //     Some(&mut self.link[(self.index(v)? + 1) % 2])
    // }
}

impl<T: GraphData> super::SlotMap<EdgeKey, EdgeSlot<T>> {
    #[inline]
    pub fn link(&self, e: EdgeKey, v: VertKey) -> &DiskEdgeLink {
        self.get_link(e, v).unwrap()
    }

    #[inline]
    pub fn link_mut(&mut self, e: EdgeKey, v: VertKey) -> &mut DiskEdgeLink {
        self.get_link_mut(e, v).unwrap()
    }

    #[inline]
    pub fn get_link(&self, e: EdgeKey, v: VertKey) -> Option<&DiskEdgeLink> {
        self[e].link(v)
    }

    #[inline]
    pub fn get_link_mut(&mut self, e: EdgeKey, v: VertKey) -> Option<&mut DiskEdgeLink> {
        self[e].link_mut(v)
    }

    #[inline]
    pub fn next(&self, e: EdgeKey, v: VertKey) -> Option<EdgeKey> {
        self[e].link(v).and_then(|dl| dl.next)
    }

    #[inline]
    pub fn prev(&self, e: EdgeKey, v: VertKey) -> Option<EdgeKey> {
        self[e].link(v).and_then(|dl| dl.prev)
    }

    #[inline]
    pub fn link_disjoint_mut<const N: usize>(
        &mut self,
        keys: [EdgeKey; N],
        v: VertKey,
    ) -> [&mut DiskEdgeLink; N] {
        self.get_link_disjoint_mut(keys, v).map(Option::unwrap)
    }

    #[inline]
    pub fn get_link_disjoint_mut<const N: usize>(
        &mut self,
        keys: [EdgeKey; N],
        v: VertKey,
    ) -> [Option<&mut DiskEdgeLink>; N] {
        self.get_disjoint_mut(keys).unwrap().map(|e| e.link_mut(v))
    }
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VertKey(KeyData);

impl From<KeyData> for VertKey {
    fn from(value: KeyData) -> Self {
        Self(value)
    }
}

impl From<VertKey> for KeyData {
    fn from(VertKey(key): VertKey) -> Self {
        key
    }
}

impl<T: GraphData> Index<VertKey> for BMesh<T> {
    type Output = VertSlot<T>;

    #[inline]
    fn index(&self, key: VertKey) -> &Self::Output {
        &self.verts[key]
    }
}

impl<T: GraphData> IndexMut<VertKey> for BMesh<T> {
    #[inline]
    fn index_mut(&mut self, key: VertKey) -> &mut Self::Output {
        &mut self.verts[key]
    }
}

pub struct VertSlot<T: GraphData> {
    pub edge: Option<EdgeKey>,
    pub data: T::Vert,
}

impl<T: GraphData> VertSlot<T> {
    pub fn edge(&self) -> Option<EdgeKey> {
        self.edge
    }
}

impl<T: GraphData> Deref for VertSlot<T> {
    type Target = T::Vert;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: GraphData> DerefMut for VertSlot<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
