use crate::{BMesh, BMeshData, FaceKey};

impl<T: BMeshData> BMesh<T> {
    fn face_triangulate(&mut self, f: FaceKey) {
        /*
                                 BMFace *f,
                                 BMFace **r_faces_new,
                                 int *r_faces_new_tot,
                                 BMEdge **r_edges_new,
                                 int *r_edges_new_tot,
                                 LinkNode **r_faces_double,
                                 const int quad_method,
                                 const int ngon_method,
                                 const bool use_tag,
                                 /* use for ngons only! */
                                 MemArena *pf_arena,

                                 /* use for MOD_TRIANGULATE_NGON_BEAUTY only! */
                                 Heap *pf_heap)
        {
          const int cd_loop_mdisp_offset = CustomData_get_offset(&bm->ldata, CD_MDISPS);
          const bool use_beauty = (ngon_method == MOD_TRIANGULATE_NGON_BEAUTY);
          BMLoop *l_first, *l_new;
          BMFace *f_new;
          int nf_i = 0;
          int ne_i = 0;

          BLI_assert(BM_face_is_normal_valid(f));

          /* ensure both are valid or nullptr */
          BLI_assert((r_faces_new == nullptr) == (r_faces_new_tot == nullptr));

          BLI_assert(f->len > 3);

          {
            BMLoop **loops = BLI_array_alloca(loops, f->len);
            uint(*tris)[3] = BLI_array_alloca(tris, f->len);
            const int totfilltri = f->len - 2;
            const int last_tri = f->len - 3;
            int i;
            /* for mdisps */
            float f_center[3];

            if (f->len == 4) {
              /* even though we're not using BLI_polyfill, fill in 'tris' and 'loops'
               * so we can share code to handle face creation afterwards. */
              BMLoop *l_v1, *l_v2;

              l_first = BM_FACE_FIRST_LOOP(f);

              switch (quad_method) {
                case MOD_TRIANGULATE_QUAD_FIXED: {
                  l_v1 = l_first;
                  l_v2 = l_first->next->next;
                  break;
                }
                case MOD_TRIANGULATE_QUAD_ALTERNATE: {
                  l_v1 = l_first->next;
                  l_v2 = l_first->prev;
                  break;
                }
                case MOD_TRIANGULATE_QUAD_SHORTEDGE:
                case MOD_TRIANGULATE_QUAD_LONGEDGE:
                case MOD_TRIANGULATE_QUAD_BEAUTY:
                default: {
                  BMLoop *l_v3, *l_v4;
                  bool split_24;

                  l_v1 = l_first->next;
                  l_v2 = l_first->next->next;
                  l_v3 = l_first->prev;
                  l_v4 = l_first;

                  if (quad_method == MOD_TRIANGULATE_QUAD_SHORTEDGE) {
                    float d1, d2;
                    d1 = len_squared_v3v3(l_v4->v->co, l_v2->v->co);
                    d2 = len_squared_v3v3(l_v1->v->co, l_v3->v->co);
                    split_24 = ((d2 - d1) > 0.0f);
                  }
                  else if (quad_method == MOD_TRIANGULATE_QUAD_LONGEDGE) {
                    float d1, d2;
                    d1 = len_squared_v3v3(l_v4->v->co, l_v2->v->co);
                    d2 = len_squared_v3v3(l_v1->v->co, l_v3->v->co);
                    split_24 = ((d2 - d1) < 0.0f);
                  }
                  else {
                    /* first check if the quad is concave on either diagonal */
                    const int flip_flag = is_quad_flip_v3(
                        l_v1->v->co, l_v2->v->co, l_v3->v->co, l_v4->v->co);
                    if (UNLIKELY(flip_flag & (1 << 0))) {
                      split_24 = true;
                    }
                    else if (UNLIKELY(flip_flag & (1 << 1))) {
                      split_24 = false;
                    }
                    else {
                      split_24 = (BM_verts_calc_rotate_beauty(l_v1->v, l_v2->v, l_v3->v, l_v4->v, 0, 0) > 0.0f);
                    }
                  }

                  /* named confusingly, l_v1 is in fact the second vertex */
                  if (split_24) {
                    l_v1 = l_v4;
                    // l_v2 = l_v2;
                  }
                  else {
                    // l_v1 = l_v1;
                    l_v2 = l_v3;
                  }
                  break;
                }
              }

              loops[0] = l_v1;
              loops[1] = l_v1->next;
              loops[2] = l_v2;
              loops[3] = l_v2->next;

              ARRAY_SET_ITEMS(tris[0], 0, 1, 2);
              ARRAY_SET_ITEMS(tris[1], 0, 2, 3);
            }
            else {
              BMLoop *l_iter;
              float axis_mat[3][3];
              float(*projverts)[2] = BLI_array_alloca(projverts, f->len);

              axis_dominant_v3_to_m3_negate(axis_mat, f->no);

              for (i = 0, l_iter = BM_FACE_FIRST_LOOP(f); i < f->len; i++, l_iter = l_iter->next) {
                loops[i] = l_iter;
                mul_v2_m3v3(projverts[i], axis_mat, l_iter->v->co);
              }

              BLI_polyfill_calc_arena(projverts, f->len, 1, tris, pf_arena);

              if (use_beauty) {
                BLI_polyfill_beautify(projverts, f->len, tris, pf_arena, pf_heap);
              }

              BLI_memarena_clear(pf_arena);
            }

            if (cd_loop_mdisp_offset != -1) {
              BM_face_calc_center_median(f, f_center);
            }

            /* loop over calculated triangles and create new geometry */
            for (i = 0; i < totfilltri; i++) {
              BMLoop *ltri[3] = {loops[tris[i][0]], loops[tris[i][1]], loops[tris[i][2]]};

              BMVert *v_tri[3] = {ltri[0]->v, ltri[1]->v, ltri[2]->v};

              f_new = BM_face_create_verts(bm, v_tri, 3, f, BM_CREATE_NOP, true);
              l_new = BM_FACE_FIRST_LOOP(f_new);

              BLI_assert(v_tri[0] == l_new->v);

              /* check for duplicate */
              if (l_new->radial_next != l_new) {
                BMLoop *l_iter = l_new->radial_next;
                do {
                  if (UNLIKELY((l_iter->f->len == 3) && (l_new->prev->v == l_iter->prev->v))) {
                    /* Check the last tri because we swap last f_new with f at the end... */
                    BLI_linklist_prepend(r_faces_double, (i != last_tri) ? f_new : f);
                    break;
                  }
                } while ((l_iter = l_iter->radial_next) != l_new);
              }

              /* copy CD data */
              BM_elem_attrs_copy(bm, ltri[0], l_new);
              BM_elem_attrs_copy(bm, ltri[1], l_new->next);
              BM_elem_attrs_copy(bm, ltri[2], l_new->prev);

              /* add all but the last face which is swapped and removed (below) */
              if (i != last_tri) {
                if (use_tag) {
                  BM_elem_flag_enable(f_new, BM_ELEM_TAG);
                }
                if (r_faces_new) {
                  r_faces_new[nf_i++] = f_new;
                }
              }

              if (use_tag || r_edges_new) {
                /* new faces loops */
                BMLoop *l_iter;

                l_iter = l_first = l_new;
                do {
                  BMEdge *e = l_iter->e;
                  /* Confusing! if its not a boundary now, we know it will be later since this will be an
                   * edge of one of the new faces which we're in the middle of creating. */
                  bool is_new_edge = (l_iter == l_iter->radial_next);

                  if (is_new_edge) {
                    if (use_tag) {
                      BM_elem_flag_enable(e, BM_ELEM_TAG);
                    }
                    if (r_edges_new) {
                      r_edges_new[ne_i++] = e;
                    }
                  }
                  /* NOTE: never disable tag's. */
                } while ((l_iter = l_iter->next) != l_first);
              }

              if (cd_loop_mdisp_offset != -1) {
                float f_new_center[3];
                BM_face_calc_center_median(f_new, f_new_center);
                BM_face_interp_multires_ex(bm, f_new, f, f_new_center, f_center, cd_loop_mdisp_offset);
              }
            }

            {
              /* we can't delete the real face, because some of the callers expect it to remain valid.
               * so swap data and delete the last created tri */
              bmesh_face_swap_data(f, f_new);
              BM_face_kill(bm, f_new);
            }
          }
          bm->elem_index_dirty |= BM_FACE;

          if (r_faces_new_tot) {
            *r_faces_new_tot = nf_i;
          }


          if (r_edges_new_tot) {
            *r_edges_new_tot = ne_i;
          }
          */
    }
}
