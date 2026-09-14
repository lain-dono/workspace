use crate::{BMesh, BMeshData, Disk};

#[derive(Default)]
pub struct TestBMeshData;

impl BMeshData for TestBMeshData {
    type Vert = ();
    type Edge = ();
    type Link = ();
    type Face = ();
}

#[test]
fn make_kill_edge() {
    let mut mesh: BMesh<TestBMeshData> = BMesh::default();

    let start = mesh.vert_make(());
    let end = mesh.vert_make(());

    assert_eq!(mesh.edge_find(start, end), None);

    let edge = mesh.edge_find_or_make(start, end, ());

    assert_eq!(mesh.verts[start].edge, Some(edge));
    assert_eq!(mesh.verts[end].edge, Some(edge));

    assert_eq!(mesh.edges[edge].prev, Disk::new(start, edge, edge));
    assert_eq!(mesh.edges[edge].next, Disk::new(end, edge, edge));

    assert_eq!(mesh.edge_find(start, end), Some(edge));
    assert_eq!(mesh.edge_find(end, start), Some(edge));

    let mut iter = mesh.disk_iter(start).unwrap();
    assert_eq!((iter.next(), iter.next()), (Some(edge), None));
    let mut iter = mesh.disk_iter(start).unwrap().rev();
    assert_eq!((iter.next(), iter.next()), (Some(edge), None));

    let mut iter = mesh.disk_iter(end).unwrap();
    assert_eq!((iter.next(), iter.next()), (Some(edge), None));
    let mut iter = mesh.disk_iter(end).unwrap().rev();
    assert_eq!((iter.next(), iter.next()), (Some(edge), None));

    mesh.edge_kill(edge).unwrap();

    assert_eq!(mesh.verts[start].edge, None);
    assert_eq!(mesh.verts[end].edge, None);
    assert!(!mesh.edges.contains_key(edge));

    assert!(mesh.disk_iter(start).is_none());
    assert!(mesh.disk_iter(end).is_none());
}

#[test]
fn two_edges() {
    let mut mesh: BMesh<TestBMeshData> = BMesh::default();

    let v_a = mesh.vert_make(());
    let v_b = mesh.vert_make(());
    let v_c = mesh.vert_make(());

    let e_a = mesh.edge_make(v_a, v_b, ());
    let e_b = mesh.edge_make(v_b, v_c, ());

    assert_eq!(mesh.verts[v_a].edge, Some(e_a));
    assert_eq!(mesh.verts[v_b].edge, Some(e_a));
    assert_eq!(mesh.verts[v_c].edge, Some(e_b));
}

#[test]
fn edge_split() {
    // start
    {
        let mut mesh: BMesh<TestBMeshData> = BMesh::default();

        let v_s = mesh.vert_make(());
        let v_e = mesh.vert_make(());

        let e_old = mesh.edge_make(v_s, v_e, ());
        let (v_new, e_new) = mesh.edge_split(e_old, v_s, (), ());

        assert_eq!(mesh.verts[v_s].edge, Some(e_new));
        assert_eq!(mesh.verts[v_new].edge, Some(e_old));
        assert_eq!(mesh.verts[v_e].edge, Some(e_old));

        assert_eq!(mesh.edges[e_new].prev, Disk::new(v_s, e_new, e_new));
        assert_eq!(mesh.edges[e_new].next, Disk::new(v_new, e_old, e_old));

        assert_eq!(mesh.edges[e_old].prev, Disk::new(v_new, e_new, e_new));
        assert_eq!(mesh.edges[e_old].next, Disk::new(v_e, e_old, e_old));

        mesh.vert_disconnect(v_new);

        assert_eq!(mesh.verts[v_s].edge, Some(e_old));
        assert_eq!(mesh.verts[v_e].edge, Some(e_old));

        assert_eq!(mesh.edges[e_old].prev, Disk::new(v_s, e_old, e_old));
        assert_eq!(mesh.edges[e_old].next, Disk::new(v_e, e_old, e_old));
        assert!(!mesh.edges.contains_key(e_new));
        assert!(mesh.verts.contains_key(v_new));
    }

    // end
    {
        let mut mesh: BMesh<TestBMeshData> = BMesh::default();

        let v_s = mesh.vert_make(());
        let v_e = mesh.vert_make(());

        let e_old = mesh.edge_make(v_s, v_e, ());
        let (v_new, e_new) = mesh.edge_split(e_old, v_e, (), ());

        assert_eq!(mesh.verts[v_s].edge, Some(e_old));
        assert_eq!(mesh.verts[v_new].edge, Some(e_old));
        assert_eq!(mesh.verts[v_e].edge, Some(e_new));

        assert_eq!(mesh.edges[e_old].prev, Disk::new(v_s, e_old, e_old));
        assert_eq!(mesh.edges[e_old].next, Disk::new(v_new, e_new, e_new));

        assert_eq!(mesh.edges[e_new].prev, Disk::new(v_new, e_old, e_old));
        assert_eq!(mesh.edges[e_new].next, Disk::new(v_e, e_new, e_new));

        mesh.vert_disconnect(v_new);

        assert_eq!(mesh.verts[v_s].edge, Some(e_old));
        assert_eq!(mesh.verts[v_e].edge, Some(e_old));

        assert_eq!(mesh.edges[e_old].prev, Disk::new(v_s, e_old, e_old));
        assert_eq!(mesh.edges[e_old].next, Disk::new(v_e, e_old, e_old));

        assert!(!mesh.edges.contains_key(e_new));
        assert!(mesh.verts.contains_key(v_new));
    }
}

#[test]
fn make_triangle() {
    let mut mesh: BMesh<TestBMeshData> = BMesh::default();

    let a = mesh.vert_make(());
    let b = mesh.vert_make(());
    let c = mesh.vert_make(());

    let ab = mesh.edge_make(a, b, ());
    let bc = mesh.edge_make(b, c, ());
    let ca = mesh.edge_make(c, a, ());

    let items = [(a, ab, ()), (b, bc, ()), (c, ca, ())];
    let face = mesh.face_create(items, ());

    let a_link = mesh.faces[face].link;
    let b_link = mesh.links[a_link].face_next;
    let c_link = mesh.links[a_link].face_prev;

    assert_ne!(c_link, b_link);
    assert_ne!(c_link, a_link);

    let tests = [
        (a_link, a, ab, c_link, b_link),
        (b_link, b, bc, a_link, c_link),
        (c_link, c, ca, b_link, a_link),
    ];

    for (link, vert, edge, prev, next) in tests {
        assert_eq!(mesh.links[link].vert, vert);
        assert_eq!(mesh.links[link].edge, edge);
        assert_eq!(mesh.links[link].edge_prev, link);
        assert_eq!(mesh.links[link].edge_next, link);
        assert_eq!(mesh.links[link].face_prev, prev);
        assert_eq!(mesh.links[link].face_next, next);
    }

    mesh.face_kill(face);

    assert!(mesh.links.is_empty());
}
