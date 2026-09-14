use crate::{BMesh, BMeshData, Disk, EdgeKey, EdgeSlot, LinkKey, LinkSlot, VertKey};

impl<T: BMeshData> BMesh<T> {
    pub fn edge_split(
        &mut self,
        e_old: EdgeKey,
        v_move: VertKey,
        v_data: T::Vert,
        e_data: T::Edge,
    ) -> (VertKey, EdgeKey)
    where
        T::Link: Default,
    {
        let EdgeSlot { prev, next, .. } = self.edges[e_old];

        //assert!(link.is_none());
        assert!(prev.vert == v_move || next.vert == v_move);

        let v_new = self.vert_make(v_data);
        let e_new = self.edges.next_key();

        self.verts[v_new].edge = Some(e_old);

        let (prev, next) = if prev.vert == v_move {
            self.edges[prev.prev][prev.vert].next = e_new;
            self.edges[prev.next][prev.vert].prev = e_new;

            if self.verts[prev.vert].edge == Some(e_old) {
                self.verts[prev.vert].edge = Some(e_new);
            }

            let start = Disk::new(v_new, e_new, e_new);
            let start = std::mem::replace(&mut self.edges[e_old].prev, start);

            (start, Disk::new(v_new, e_old, e_old))
        } else {
            self.edges[next.prev][next.vert].next = e_new;
            self.edges[next.next][next.vert].prev = e_new;

            if self.verts[next.vert].edge == Some(e_old) {
                self.verts[next.vert].edge = Some(e_new);
            }

            let end = Disk::new(v_new, e_new, e_new);
            let end = std::mem::replace(&mut self.edges[e_old].next, end);

            (Disk::new(v_new, e_old, e_old), end)
        };

        self.edges.insert(EdgeSlot {
            prev,
            next,
            data: e_data,
            link: None,
        });

        let mut is_first = true;
        let mut l_next = self.edges[e_old].link.take();

        while let Some(l_old) = l_next {
            let LinkSlot {
                face,
                face_next: next,
                edge_next: radial_next,
                ..
            } = self.links[l_old];

            let prev = l_old;

            l_next = (l_old != radial_next).then_some(radial_next);

            self.radial_unlink(l_old);

            let l_new = self.links.insert(LinkSlot {
                data: T::Link::default(),
                face,
                edge: EdgeKey::null(),
                vert: v_new,
                edge_prev: LinkKey::null(),
                edge_next: LinkKey::null(),
                face_prev: prev,
                face_next: next,
            });

            self.links[prev].face_next = l_new;
            self.links[next].face_prev = l_new;

            let v_next = self.links[next].vert;

            let is_old = self.edges[e_old].contains_pair(v_new, v_next);
            let is_new = self.edges[e_new].contains_pair(v_new, v_next);

            if is_old {
                (self.links[l_new].edge, self.links[l_old].edge) = (e_old, e_new);
            } else if is_new {
                (self.links[l_new].edge, self.links[l_old].edge) = (e_new, e_old);
            }

            if is_old || is_new {
                if is_first {
                    is_first = false;
                    self.links[l_old].edge_next = LinkKey::null();
                    self.links[l_old].edge_prev = LinkKey::null();
                }

                self.radial_append(self.links[l_new].edge, l_new);
                self.radial_append(self.links[l_old].edge, l_old);
            }
        }

        (v_new, e_new)
    }

    pub fn vert_valence(&self, vert: VertKey) -> usize {
        self.disk_iter(vert).into_iter().flatten().count()
    }

    pub fn vert_two_valence(&self, vert: VertKey) -> Option<[EdgeKey; 2]> {
        let base = self.verts[vert].edge?;
        let disk = self.edges[base].disk(vert)?;
        if disk.prev == disk.next && disk.prev != base {
            Some([base, disk.prev])
        } else {
            None
        }
    }

    pub fn vert_disconnect(&mut self, vert: VertKey) {
        let [e_base, e_kill] = self.vert_two_valence(vert).unwrap();

        let EdgeSlot { prev, next, .. } = self.edges[e_kill];
        let target = if prev.vert == vert {
            self.edges[next.prev][next.vert].next = e_base;
            self.edges[next.next][next.vert].prev = e_base;
            self.edges[e_kill].next
        } else if next.vert == vert {
            self.edges[prev.prev][prev.vert].next = e_base;
            self.edges[prev.next][prev.vert].prev = e_base;
            self.edges[e_kill].prev
        } else {
            unreachable!()
        };

        if self.verts[target.vert].edge == Some(e_kill) {
            self.verts[target.vert].edge = Some(e_base)
        }

        if let Some(start) = self.edges[e_kill].link {
            let mut iter = start;

            loop {
                let LinkSlot {
                    face_next: next,
                    face_prev: prev,
                    face,
                    edge_next: radial_next,
                    ..
                } = self.links[iter];

                if self.links[next].vert == vert {
                    self.links[next].vert = target.vert;
                }

                self.links[next].face_prev = prev;
                self.links[prev].face_next = next;

                if self.faces[face].link == iter {
                    self.faces[face].link = next;
                }

                self.links.remove(iter);
                iter = radial_next;

                if iter == start {
                    break;
                }
            }
        }

        self.verts[vert].edge = None;
        self.edges.remove(e_kill).unwrap();
        self.edges[e_base][vert] = target;
    }
}
