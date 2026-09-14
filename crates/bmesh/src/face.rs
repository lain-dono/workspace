use crate::{BMeshData, EdgeKey, VertKey};

define_key!(LinkKey [LinkSlot => Link]);
define_key!(FaceKey [FaceSlot => Face]);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FaceSlot<T: BMeshData> {
    pub data: T::Face,
    pub link: LinkKey,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LinkSlot<T: BMeshData> {
    pub data: T::Link,

    pub vert: VertKey,

    pub edge: EdgeKey,
    pub edge_prev: LinkKey,
    pub edge_next: LinkKey,

    pub face: FaceKey,
    pub face_prev: LinkKey,
    pub face_next: LinkKey,
}

impl<T: BMeshData> LinkSlot<T> {
    pub(crate) fn new(data: T::Link, face: FaceKey, edge: EdgeKey, vert: VertKey) -> Self {
        Self {
            data,
            vert,

            edge,
            edge_next: LinkKey::null(),
            edge_prev: LinkKey::null(),

            face,
            face_next: LinkKey::null(),
            face_prev: LinkKey::null(),
        }
    }
}

impl<T: BMeshData> super::BMesh<T> {
    pub fn face_make(
        &mut self,
        vert: VertKey,
        edge: EdgeKey,
        link_data: T::Link,
        face_data: T::Face,
    ) -> FaceKey {
        let face = self.faces.next_key();
        let link = self.links.next_key();

        let edge_link = self.edges[edge].link.replace(link);

        let (edge_prev, edge_next) = if let Some(prev) = edge_link {
            let next = self.links[prev].edge_next;
            self.links[next].edge_prev = link;
            self.links[prev].edge_next = link;
            (prev, next)
        } else {
            (link, link)
        };

        let link = self.links.insert(LinkSlot {
            vert,
            edge,
            face,
            data: link_data,

            edge_next,
            edge_prev,

            face_next: link,
            face_prev: link,
        });

        let data = face_data;
        self.faces.insert(FaceSlot { data, link })
    }

    pub fn face_kill(&mut self, face: FaceKey) {
        let start = self.faces[face].link;
        let mut link = start;

        loop {
            let next = self.links[link].face_next;
            self.radial_remove(self.links[link].edge, link);
            self.links.remove(link);
            link = next;
            if link == start {
                break;
            }
        }

        self.faces.remove(face);
    }

    pub fn face_create<I>(&mut self, iter: I, data: T::Face) -> FaceKey
    where
        I: IntoIterator<Item = (VertKey, EdgeKey, T::Link)>,
    {
        let face = self.faces.next_key();

        let mut iter = iter.into_iter();

        let init = {
            let (vert, edge, data) = iter.next().unwrap();
            let link = self.links.insert(LinkSlot::new(data, face, edge, vert));
            self.radial_append(edge, link);
            link
        };

        let mut prev = init;
        for (vert, edge, data) in iter {
            let next = self.links.insert(LinkSlot::new(data, face, edge, vert));
            self.radial_append(edge, next);

            self.links[next].face_prev = prev;
            self.links[prev].face_next = next;

            prev = next;
        }

        self.links[init].face_prev = prev;
        self.links[prev].face_next = init;
        // self.faces.insert(FaceSlot { data, link: prev })
        self.faces.insert(FaceSlot { data, link: init })
    }

    pub(crate) fn radial_append(&mut self, edge: EdgeKey, link: LinkKey) {
        assert!(
            self.links[link].edge.is_null() || self.links[link].edge == edge,
            "loop is already in a radial cycle for a different edge"
        );

        let (prev, next) = self.edges[edge].link.map_or((link, link), |prev| {
            let next = self.links[prev].edge_next;
            self.links[next].edge_prev = link;
            self.links[prev].edge_next = link;
            (prev, next)
        });

        self.edges[edge].link = Some(link);
        self.links[link].edge = edge;
        self.links[link].edge_prev = prev;
        self.links[link].edge_next = next;
    }

    pub(crate) fn radial_remove(&mut self, edge: EdgeKey, link: LinkKey) {
        assert_eq!(self.links[link].edge, edge);

        let (prev, next);
        (prev, self.links[link].edge_prev) = (self.links[link].edge_prev, LinkKey::null());
        (next, self.links[link].edge_next) = (self.links[link].edge_next, LinkKey::null());

        if next != link {
            self.links[prev].edge_next = next;
            self.links[next].edge_prev = prev;

            if self.edges[edge].link == Some(link) {
                self.edges[edge].link = Some(next);
            }
        } else {
            assert_eq!(self.edges[edge].link, Some(link));
            self.edges[edge].link = None;
        }

        self.links[link].edge = EdgeKey::null();
    }

    pub(crate) fn radial_unlink(&mut self, link: LinkKey) {
        let (prev, next);
        (prev, self.links[link].edge_prev) = (self.links[link].edge_prev, LinkKey::null());
        (next, self.links[link].edge_next) = (self.links[link].edge_next, LinkKey::null());

        if next != link {
            self.links[prev].edge_next = next;
            self.links[next].edge_prev = prev;
        }

        self.links[link].edge = EdgeKey::null();
    }
}
