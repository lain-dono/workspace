use crate::{BMesh, BMeshData, EdgeKey, FaceKey, FaceSlot, LinkKey, LinkSlot};

impl<T: BMeshData> BMesh<T> {
    pub fn face_split(
        &mut self,
        f_old: FaceKey,
        l_prev: LinkKey,
        l_next: LinkKey,
        e_data: T::Edge,
        f_data: T::Face,
    ) -> (FaceKey, LinkKey)
    where
        T::Link: Default,
    {
        let v_prev = self.links[l_prev].vert;
        let v_next = self.links[l_next].vert;

        assert_eq!(self.links[l_prev].face, f_old);
        assert_eq!(self.links[l_next].face, f_old);

        // allocate new edge between v1 and v2
        let edge = self.edge_make(v_prev, v_next, e_data);

        let f_new = self.faces.insert(FaceSlot {
            data: f_data,
            link: LinkKey::null(),
        });

        let l_old = LinkSlot::new(Default::default(), f_old, edge, v_next);
        let l_old = self.links.insert(l_old);

        let l_new = LinkSlot::new(Default::default(), f_new, edge, v_prev);
        let l_new = self.links.insert(l_new);

        let l_old_prev = self.links[l_next].face_prev;
        let l_new_prev = self.links[l_prev].face_prev;

        self.links[l_old].face_prev = l_old_prev;
        self.links[l_new].face_prev = l_new_prev;

        self.links[l_old_prev].face_next = l_old;
        self.links[l_new_prev].face_next = l_new;

        self.links[l_old].face_next = l_prev;
        self.links[l_new].face_next = l_next;

        self.links[l_prev].face_prev = l_old;
        self.links[l_next].face_prev = l_new;

        // find which of the faces the original first loop is in
        let start = l_old;
        let mut iter = start;
        let mut found_old = false;
        loop {
            found_old |= iter == self.faces[f_old].link;
            iter = self.links[iter].face_next;
            if iter == start || found_old {
                break;
            }
        }

        if found_old {
            // Original first loop was in f_old, find a suitable first loop for f_new
            // which is as similar as possible to f_old.
            // the order matters for tools such as dupli-faces.
            let link = self.faces[f_old].link;
            self.faces[f_new].link = if self.links[link].face_prev == l_old {
                self.links[l_new].face_prev
            } else if self.links[link].face_next == l_old {
                self.links[l_new].face_next
            } else {
                l_new
            }
        } else {
            // original first loop was in f_new, further do same as above
            self.faces[f_new].link = self.faces[f_old].link;

            let link = self.faces[f_old].link;
            self.faces[f_old].link = if self.links[link].face_prev == l_new {
                self.links[l_old].face_prev
            } else if self.links[link].face_next == l_new {
                self.links[l_old].face_next
            } else {
                l_old
            }
        }

        // go through all of f_new's loops and make sure they point to it properly
        let start = self.faces[f_new].link;
        let mut iter = start;
        loop {
            self.links[iter].face = f_new;
            iter = self.links[iter].face_next;
            if iter == start {
                break;
            }
        }

        // link up the new loops into the new edges radial
        self.radial_append(edge, l_old);
        self.radial_append(edge, l_new);

        (f_new, l_new)
    }

    pub fn face_join(&mut self, f1: FaceKey, f2: FaceKey, e: EdgeKey) -> Option<FaceKey> {
        assert_ne!(f1, f2);

        if !self.edge_is_manifold(e) {
            return None;
        }

        let l_f1 = self.face_edge_share_loop(f1, e)?;
        let l_f2 = self.face_edge_share_loop(f2, e)?;

        if self.links[l_f1].vert == self.links[l_f2].vert {
            return None;
        }

        let [l_f1_prev, l_f1_next] = [self.links[l_f1].face_prev, self.links[l_f1].face_next];
        let [l_f2_prev, l_f2_next] = [self.links[l_f2].face_prev, self.links[l_f2].face_next];

        let [l_f1_prev, l_f1_next] = [l_f1_prev, l_f1_next].map(|link| self.links[link].edge);
        let [l_f2_prev, l_f2_next] = [l_f2_prev, l_f2_next].map(|link| self.links[link].edge);

        let f1_edges = self.edge_in_face(f1, l_f2_prev) || self.edge_in_face(f1, l_f2_next);
        let f2_edges = self.edge_in_face(f2, l_f1_prev) || self.edge_in_face(f2, l_f1_next);

        if f2_edges || f1_edges {
            return None;
        }

        // validate only one shared edge
        if self.face_share_edge_count(f1, f2) > 1 {
            return None;
        }

        // validate no internal join
        {
            /*
            let mut is_dupe = false;

              /* TODO: skip clearing once this is ensured. */
              for (i = 0, l_iter = BM_FACE_FIRST_LOOP(f2); i < f2len; i++, l_iter = l_iter->next) {
                BM_elem_flag_disable(l_iter->v, BM_ELEM_INTERNAL_TAG);
              }

              for (i = 0, l_iter = BM_FACE_FIRST_LOOP(f1); i < f1len; i++, l_iter = l_iter->next) {
                BM_elem_flag_set(l_iter->v, BM_ELEM_INTERNAL_TAG, l_iter != l_f1);
              }
              for (i = 0, l_iter = BM_FACE_FIRST_LOOP(f2); i < f2len; i++, l_iter = l_iter->next) {
                if (l_iter != l_f2) {
                  /* as soon as a duplicate is found, bail out */
                  if (BM_elem_flag_test(l_iter->v, BM_ELEM_INTERNAL_TAG)) {
                    is_dupe = true;
                    break;
                  }
                }
              }
              /* Cleanup tags. */
              for (i = 0, l_iter = BM_FACE_FIRST_LOOP(f1); i < f1len; i++, l_iter = l_iter->next) {
                BM_elem_flag_disable(l_iter->v, BM_ELEM_INTERNAL_TAG);
              }
              if (is_dupe) {
                return nullptr;
              }
            */
        }

        // join the two loop
        let [prev, next] = [self.links[l_f1].face_prev, self.links[l_f2].face_next];
        self.links[next].face_prev = prev;
        self.links[prev].face_next = next;

        let [prev, next] = [self.links[l_f2].face_prev, self.links[l_f1].face_next];
        self.links[next].face_prev = prev;
        self.links[prev].face_next = next;

        // If `l_f1` was base-loop, make `l_f1->next` the base.
        if self.faces[f1].link == l_f1 {
            self.faces[f1].link = self.links[l_f1].face_next;
        }

        // make sure each loop points to the proper face
        let start = self.faces[f1].link;
        let mut iter = start;
        loop {
            self.links[iter].face = f1;

            iter = self.links[iter].face_next;
            if iter == start {
                break;
            }
        }

        // remove edge from the disk cycle of its two vertices
        let edge = self.links[l_f1].edge;
        self.disk_edge_remove(edge, self.edges[edge].prev.vert);
        self.disk_edge_remove(edge, self.edges[edge].next.vert);

        self.edges.remove(self.links[l_f1].edge);
        self.links.remove(l_f1);
        self.links.remove(l_f2);
        self.faces.remove(f2);

        Some(f1)
    }

    fn edge_is_manifold(&self, edge: EdgeKey) -> bool {
        self.edges[edge].link.is_some_and(|link| {
            let prev = self.links[link].edge_prev;
            let next = self.links[link].edge_next;
            let prev = prev != link && self.links[prev].edge_prev == link;
            let next = next != link && self.links[next].edge_next == link;
            prev && next
        })
    }

    fn face_edge_share_loop(&self, face: FaceKey, edge: EdgeKey) -> Option<LinkKey> {
        self.edges[edge].link.and_then(|start| {
            let mut iter = start;
            loop {
                if self.links[iter].face == face {
                    break Some(iter);
                }
                iter = self.links[iter].edge_next;
                if iter == start {
                    break None;
                }
            }
        })
    }

    fn edge_in_face(&self, face: FaceKey, edge: EdgeKey) -> bool {
        self.face_edge_share_loop(face, edge).is_some()
    }

    fn face_share_edge_count(&self, f_a: FaceKey, f_b: FaceKey) -> usize {
        let mut count = 0;

        let start = self.faces[f_a].link;
        let mut iter = start;
        loop {
            if self.edge_in_face(f_b, self.links[iter].edge) {
                count += 1;
            }
            iter = self.links[iter].face_next;
            if iter == start {
                break;
            }
        }

        count
    }
}
