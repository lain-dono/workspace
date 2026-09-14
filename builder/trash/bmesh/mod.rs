mod fill;
mod smap;
mod wire;

pub use self::fill::{FaceKey, FaceSlot, LoopKey, LoopSlot, RadialLoop};
pub use self::wire::{DiskEdgeLink, EdgeKey, EdgeSlot, VertKey, VertSlot};
use smap::{KeyData, SlotMap};

pub trait GraphData {
    type Vert;
    type Edge;
    type Loop;
    type Face;
}

pub struct BMesh<T: GraphData> {
    verts: SlotMap<VertKey, VertSlot<T>>,
    edges: SlotMap<EdgeKey, EdgeSlot<T>>,
    loops: SlotMap<LoopKey, LoopSlot<T>>,
    faces: SlotMap<FaceKey, FaceSlot<T>>,
}

impl<T: GraphData> BMesh<T> {
    pub fn vert_create(&mut self, data: T::Vert) -> VertKey {
        self.verts.insert(VertSlot { edge: None, data })
    }

    pub fn vert_kill(&mut self, v: VertKey) -> Option<VertSlot<T>> {
        while let Some(e) = self.verts[v].edge {
            self.edge_kill(e);
        }
        self.verts.remove(v)
    }

    pub fn edge_create(&mut self, a: VertKey, b: VertKey, data: T::Edge) -> EdgeKey {
        assert!(self.verts.contains_key(a));
        assert!(self.verts.contains_key(b));

        let a_dl = DiskEdgeLink {
            vert: a,
            next: None,
            prev: None,
        };
        let b_dl = DiskEdgeLink {
            vert: b,
            next: None,
            prev: None,
        };

        let e = self.edges.insert(EdgeSlot {
            loop_key: None,
            link: [a_dl, b_dl],
            data,
        });

        self.disk_edge_append(e, a);
        self.disk_edge_append(e, b);

        e
    }

    pub fn edge_kill(&mut self, e: EdgeKey) {
        todo!();

        //   while (e->l) {
        //     BM_face_kill(bm, e->l->f);
        //   }

        let [a, b] = self.edges[e].link.clone().map(|dl| dl.vert);

        self.disk_edge_remove(e, a);
        self.disk_edge_remove(e, b);

        self.edges.remove(e);
    }

    fn loop_create(
        &mut self,
        vert: VertKey,
        edge: EdgeKey,
        face: FaceKey,
        data: T::Loop,
    ) -> LoopKey {
        self.loops.insert(LoopSlot {
            vert,
            edge: Some(edge),
            face,

            radial: None,

            next: KeyData::null().into(),
            prev: KeyData::null().into(),

            data,
        })
    }

    pub fn face_create(
        &mut self,
        mut iter: impl Iterator<Item = (VertKey, EdgeKey)>,
        f_data: T::Face,
        l_data: T::Loop,
    ) -> Option<FaceKey>
    where
        T::Loop: Clone,
    {
        let (start_v, start_e) = iter.next()?;

        let f = self.faces.insert(FaceSlot {
            loop_first: None,
            len: 0,
            data: f_data,
        });

        let base = {
            let l = self.loop_create(start_v, start_e, f, l_data.clone()); // start_e.l
            self.radial_loop_append(start_e, l);
            self.faces[f].loop_first = Some(l);
            l
        };

        let mut last = base;

        let mut len = 1;

        for (v, e) in iter {
            len += 1;

            let next = self.loop_create(v, e, f, l_data.clone()); // self.edges[e].l
            self.radial_loop_append(e, next);

            self.loops[next].prev = last;
            self.loops[last].next = next;

            last = next;
        }

        self.loops[base].prev = last;
        self.loops[last].next = base;

        self.faces[f].len = len;

        Some(f)
    }

    pub fn face_kill(&mut self, f: FaceKey) -> Option<FaceSlot<T>> {
        if let Some(first) = self.faces[f].loop_first {
            let mut iter = first;

            loop {
                let next = self.loops[iter].next;

                let e = self.loops[iter].edge.unwrap();
                self.radial_loop_remove(e, iter);
                self.loops.remove(iter);

                iter = next;

                if iter == first {
                    break;
                }
            }
        }

        self.faces.remove(f)
    }

    pub fn kernel_split_edge_make_vert(
        &mut self, /*BMVert *tv, BMEdge *e, BMEdge **r_e*/
    ) -> VertKey {
        /*
        BMLoop *l_next;
        BMEdge *e_new;
        BMVert *v_new, *v_old;

        BLI_assert(BM_vert_in_edge(e, tv) != false);

        let v_old = self.edges[e].other_vert(e, tv);

        // order of 'e_new' verts should match 'e'
        // (so extruded faces don't flip)
        let v_new = self.vert_create(bm, tv, BM_CREATE_NOP);
        let e_new = self.edge_create(bm, tv, v_new, e, BM_CREATE_NOP);

        self.disk_edge_remove(e_new, tv);
        self.disk_edge_remove(e_new, v_new);

        self.disk_vert_replace(e, v_new, tv);

        // add e_new to v_new's disk cycle
        self.disk_edge_append(e_new, v_new);

        // add e_new to tv's disk cycle
        self.disk_edge_append(e_new, tv);

        // Split the radial cycle if present
        let l_next = e.l.take();

        if (l_next) {
          BMLoop *l_new, *l;
          let mut is_first = true;

          // Take the next loop. Remove it from radial. Split it. Append to appropriate radials
          while let Some(l) = l_next {
            self.faces[self.loops[l].face].len += 1;

            l_next = if l_next != self.loops[l_next].radial_next { l_next->radial_next } else { None };
            self.radial_loop_unlink(l);

            let l_new = self.loop_create(bm, nullptr, nullptr, l->f, l, eBMCreateFlag(0));
            l_new.prev = l;
            l_new.next = l->next;
            l_new.prev.next = l_new;
            l_new.next.prev = l_new;
            l_new.v = v_new;

            // assign the correct edge to the correct loop
            if BM_verts_in_edge(l_new->v, l_new->next->v, e) {
              l_new->e = e;
              l->e = e_new;

              // append l into e_new's rad cycle
              if is_first {
                is_first = false;
                l->radial_next = l->radial_prev = nullptr;
              }

              self.radial_loop_append(l_new->e, l_new);
              self.radial_loop_append(l->e, l);
            } else if (BM_verts_in_edge(l_new->v, l_new->next->v, e_new)) {
              l_new->e = e_new;
              l->e = e;

              // append l into e_new's rad cycle
              if (is_first) {
                is_first = false;
                self.loops[l].radial = None
              }

              self.radial_loop_append(l_new->e, l_new);
              self.radial_loop_append(l->e, l);
            }
          }
        }

        BM_CHECK_ELEMENT(e_new);
        BM_CHECK_ELEMENT(v_new);
        BM_CHECK_ELEMENT(v_old);
        BM_CHECK_ELEMENT(e);
        BM_CHECK_ELEMENT(tv);

        if (r_e) {
          *r_e = e_new;
        }
        return v_new;
          */

        todo!()
    }
}

impl<G: GraphData> BMesh<G> {
    // fn disk_edge_exists(&self, v1: VertKey, v2: VertKey) -> Option<EdgeKey> {
    //   if let Some(e_first) = self.verts[v1].edge {
    //     let mut e_iter = e_first;

    //     do {
    //       if (BM_verts_in_edge(v1, v2, e_iter)) {
    //         return e_iter;
    //       }
    //     } while ((e_iter = bmesh_disk_edge_next(e_iter, v1)) != e_first);
    //   }

    //   None
    // }

    fn disk_edge_append(&mut self, e: EdgeKey, v: VertKey) {
        if let Some(ve) = self.verts[v].edge {
            let [e_dl, v_dl] = self.edges.link_disjoint_mut([e, ve], v);

            e_dl.next = Some(ve);
            e_dl.prev = v_dl.prev;

            if let Some(prev) = v_dl.prev.replace(e) {
                self.edges.link_mut(prev, v).next = Some(e);
            }
        } else {
            let e_dl = self.edges.link_mut(e, v);
            e_dl.next = Some(e);
            e_dl.prev = Some(e);
            self.verts[v].edge = Some(e);
        }
    }

    fn disk_edge_remove(&mut self, e: EdgeKey, v: VertKey) {
        let dl = self.edges.link_mut(e, v);

        let (next, prev) = (dl.next.take(), dl.prev.take());

        if let Some(prev) = prev {
            self.edges.link_mut(prev, v).next = next;
        }
        if let Some(next) = next {
            self.edges.link_mut(next, v).prev = prev;
        }

        if self.verts[v].edge == Some(e) {
            self.verts[v].edge = if Some(e) != next { next } else { None };
        }
    }

    fn disk_vert_replace(&mut self, e: EdgeKey, dst: VertKey, src: VertKey) {
        assert!(self.edges[e].link[0].vert == src || self.edges[e].link[1].vert == src);
        // Remove `e` from `v_src` disk cycle.
        // Swap out `v_src` for `v_dst` in `e`.
        // Add `e` to `v_dst` disk cycle.

        self.disk_edge_remove(e, src);
        *self.edges.link_mut(e, src) = DiskEdgeLink {
            vert: dst,
            next: None,
            prev: None,
        };
        self.disk_edge_append(e, dst);
        assert_ne!(self.edges[e].link[0].vert, self.edges[e].link[1].vert);
    }

    fn edge_vert_swap(&mut self, e: EdgeKey, v_dst: VertKey, v_src: VertKey) {
        // swap out loops
        if let Some(first) = self.edges[e].loop_key {
            let mut iter = first;
            loop {
                let next = self.loops[iter].next;
                if self.loops[iter].vert == v_src {
                    self.loops[iter].vert = v_dst;
                } else if self.loops[next].vert == v_src {
                    self.loops[next].vert = v_dst;
                } else {
                    assert_ne!(self.loops[self.loops[iter].prev].vert, v_src);
                }

                iter = self.loops[iter].radial.as_ref().unwrap().next;

                if iter == first {
                    break;
                }
            }
        }

        // swap out edges
        self.disk_vert_replace(e, v_dst, v_src);
    }

    fn radial_loop_append(&mut self, e: EdgeKey, l: LoopKey) {
        assert!(self.loops[l].edge.is_none() || self.loops[l].edge == Some(e));

        if let Some(prev) = self.edges[e].loop_key.replace(l) {
            let next = self.loops[prev].radial.as_ref().map(|l| l.next).unwrap();

            self.loops[l].radial = Some(RadialLoop { prev, next });
            self.loops[next].radial = Some(RadialLoop { prev: l, next: l });
        } else {
            self.loops[l].radial = Some(RadialLoop { prev: l, next: l });
        }

        self.loops[l].edge = Some(e);
    }

    fn radial_loop_remove(&mut self, e: EdgeKey, l: LoopKey) {
        // if e is non-nullptr, l must be in the radial cycle of e
        assert_eq!(self.loops[l].edge, Some(e));

        // l is no longer in a radial cycle; empty the links
        // to the cycle and the link back to an edge
        self.loops[l].edge = None;
        let RadialLoop { next, prev } = self.loops[l].radial.take().unwrap();

        if next == l {
            assert_eq!(self.edges[e].loop_key.take(), Some(l));
        } else {
            if self.edges[e].loop_key == Some(l) {
                self.edges[e].loop_key = Some(next);
            }

            self.loops[next].radial.as_mut().unwrap().prev = prev;
            self.loops[prev].radial.as_mut().unwrap().next = next;
        }
    }

    fn radial_loop_unlink(&mut self, l: LoopKey) {
        let radial = self.loops[l].radial.take().unwrap();

        if radial.next != l {
            self.loops[radial.next].radial = Some(radial.clone());
        }

        // l is no longer in a radial cycle; empty the links
        // to the cycle and the link back to an edge
        self.loops[l].edge = None;
    }
}
