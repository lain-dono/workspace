use crate::{BMesh, BMeshData, DiskIter, LinkKey, Slots};

define_key!(EdgeKey [EdgeSlot => Edge]);
define_key!(VertKey [VertSlot => Vert]);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VertSlot<T: BMeshData> {
    pub data: T::Vert,
    pub edge: Option<EdgeKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Disk {
    pub vert: VertKey,
    pub prev: EdgeKey,
    pub next: EdgeKey,
}

impl Disk {
    #[inline]
    pub const fn new(vert: VertKey, prev: EdgeKey, next: EdgeKey) -> Self {
        Self { vert, prev, next }
    }

    #[inline]
    pub fn next<T: BMeshData>(&self, edges: &Slots<EdgeKey, EdgeSlot<T>>) -> Option<Self> {
        edges[self.next].disk(self.vert)
    }

    #[inline]
    pub fn prev<T: BMeshData>(&self, edges: &Slots<EdgeKey, EdgeSlot<T>>) -> Option<Self> {
        edges[self.prev].disk(self.vert)
    }
}

impl<T: BMeshData> BMesh<T> {
    pub fn disk_iter(&self, vert: VertKey) -> Option<DiskIter<'_, T>> {
        let edge = self.verts.get(vert).and_then(|v| v.edge)?;
        Some(DiskIter::new(&self.edges, vert, edge))
    }

    #[inline]
    pub(crate) fn new_disk_link(&mut self, edge: EdgeKey, vert: VertKey) -> Disk {
        let (prev, next) = match self.verts[vert].edge {
            Some(next) => {
                let dl = &mut self.edges[next][vert];
                let prev = dl.prev;
                dl.prev = edge;
                self.edges[prev][vert].next = edge;
                (prev, next)
            }
            None => {
                self.verts[vert].edge = Some(edge);
                (edge, edge)
            }
        };
        Disk::new(vert, prev, next)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EdgeSlot<T: BMeshData> {
    pub data: T::Edge,
    pub link: Option<LinkKey>,
    pub prev: Disk, // start
    pub next: Disk, // end
}

impl<T: BMeshData> std::ops::Index<VertKey> for EdgeSlot<T> {
    type Output = Disk;

    #[inline]
    fn index(&self, vert: VertKey) -> &Self::Output {
        if self.prev.vert == vert {
            &self.prev
        } else if self.next.vert == vert {
            &self.next
        } else {
            unreachable!()
        }
    }
}

impl<T: BMeshData> std::ops::IndexMut<VertKey> for EdgeSlot<T> {
    #[inline]
    fn index_mut(&mut self, vert: VertKey) -> &mut Self::Output {
        if self.prev.vert == vert {
            &mut self.prev
        } else if self.next.vert == vert {
            &mut self.next
        } else {
            unreachable!()
        }
    }
}

impl<T: BMeshData> EdgeSlot<T> {
    #[inline]
    pub fn reverse(&mut self) {
        (self.next, self.prev) = (self.prev, self.next);
    }

    fn _swap(&mut self, dst: VertKey, src: VertKey) {
        self[src] = Disk {
            vert: dst,
            next: EdgeKey::null(),
            prev: EdgeKey::null(),
        }
    }

    #[inline]
    pub(crate) fn contains_vert(&self, vert: VertKey) -> bool {
        self.prev.vert == vert || self.next.vert == vert
    }

    #[inline]
    pub(crate) fn contains_pair(&self, a: VertKey, b: VertKey) -> bool {
        let pair = [self.prev.vert, self.next.vert];
        pair == [a, b] || pair == [b, a]
    }

    #[inline]
    pub fn pair(&self, v: VertKey) -> Option<[Disk; 2]> {
        if v == self.prev.vert {
            Some([self.prev, self.next])
        } else if v == self.next.vert {
            Some([self.next, self.prev])
        } else {
            None
        }
    }

    #[inline]
    pub fn disk(&self, vert: VertKey) -> Option<Disk> {
        if self.prev.vert == vert {
            Some(self.prev)
        } else if self.next.vert == vert {
            Some(self.next)
        } else {
            None
        }
    }
}

impl<T: BMeshData> Slots<EdgeKey, EdgeSlot<T>> {
    pub fn iter_data(&self) -> impl Iterator<Item = (VertKey, VertKey, EdgeKey, &T::Edge)> {
        self.iter()
            .map(|(key, v)| (v.prev.vert, v.next.vert, key, &v.data))
    }
}
