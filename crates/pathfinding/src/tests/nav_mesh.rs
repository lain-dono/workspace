use crate::bvh::BoundingBox;
use crate::landmass::{
    NavMeshBuilder, PointSampleDistance3d, ValidationError,
    coords::XYZ,
    nav_mesh::{Connectivity, MeshEdgeRef, Polygon},
};
use bevy::math::Vec3A;
use bevy::math::bounding::Aabb3d;
use bevy::{
    math::{Vec2, Vec3},
    platform::collections::HashSet,
};

#[test]
fn validation_computes_bounds() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(0.5, 3.0, 0.5),
        Vec3::new(0.75, 4.0, -0.25),
        Vec3::new(0.25, 4.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2])
    .with_polygon(0, [3, 4, 5]);

    let valid_mesh = source_mesh.clone().validate().unwrap();
    assert_eq!(
        valid_mesh.bounds,
        BoundingBox::new(Vec3::new(0.0, 0.0, -0.25), Vec3::new(2.0, 4.0, 1.0))
    );
}

#[test]
fn correctly_computes_bounds_for_small_number_of_points() {
    let valid_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(2.0, 1.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2])
    .validate()
    .unwrap();

    assert_eq!(
        valid_mesh.bounds,
        BoundingBox::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(2.0, 1.0, 1.0))
    );
}

#[test]
fn polygons_derived_and_vertices_copied() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(0.5, 3.0, 0.5),
        Vec3::new(0.75, 4.0, -0.25),
        Vec3::new(0.25, 4.0, 0.0),
    ])
    .with_polygon(1337, [0, 1, 2])
    .with_polygon(123, [3, 4, 5]);

    let expected_polygons = vec![
        Polygon {
            vertices: source_mesh.polygons[0].iter().map(|&i| (i, None)).collect(),
            region: 0,
            type_index: 1337,
            bounds: Aabb3d {
                min: Vec3A::new(0.0, 0.0, 0.0),
                max: Vec3A::new(2.0, 1.0, 1.0),
            },
            center: Vec3::new(3.0, 1.0, 1.0) / 3.0,
        },
        Polygon {
            vertices: source_mesh.polygons[1].iter().map(|&i| (i, None)).collect(),
            region: 1,
            type_index: 123,
            bounds: Aabb3d {
                min: Vec3A::new(0.25, 3.0, -0.25),
                max: Vec3A::new(0.75, 4.0, 0.5),
            },
            center: Vec3::new(1.5, 11.0, 0.25) / 3.0,
        },
    ];

    let valid_mesh = source_mesh.clone().validate().unwrap();
    assert_eq!(valid_mesh.vertices, source_mesh.vertices);
    assert_eq!(valid_mesh.polygons, expected_polygons);
    assert_eq!(valid_mesh.used_type_indices, HashSet::from([1337, 123]));
}

#[test]
fn error_on_wrong_type_indices_length() {
    let mut mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2]);

    mesh.polygon_type_indices.push(0);

    match mesh.clone().validate().unwrap_err() {
        ValidationError::TypeIndicesHaveWrongLength(polygons, type_indices) => {
            assert_eq!(polygons, 1);
            assert_eq!(type_indices, 2);
        }
        err => panic!("Wrong error variant! Expected TypeIndicesHaveWrongLength but got: {err:?}"),
    }
}

#[test]
fn error_on_concave_polygon() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2]);

    match source_mesh.clone().validate().unwrap_err() {
        ValidationError::ConcavePolygon(polygon) => assert_eq!(polygon, 0),
        err => panic!("Wrong error variant! Expected ConcavePolygon but got: {err:?}"),
    }
}

#[test]
fn error_on_small_polygon() {
    let source_mesh =
        NavMeshBuilder::<XYZ>::new([Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0)])
            .with_polygon(0, [0, 1]);

    match source_mesh.clone().validate().unwrap_err() {
        ValidationError::NotEnoughVerticesInPolygon(polygon) => {
            assert_eq!(polygon, 0);
        }
        err => panic!("Wrong error variant! Expected NotEnoughVerticesInPolygon but got: {err:?}"),
    }
}

#[test]
fn error_on_bad_polygon_index() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 3]);

    match source_mesh.clone().validate().unwrap_err() {
        ValidationError::InvalidVertexIndexInPolygon(polygon) => {
            assert_eq!(polygon, 0);
        }
        err => panic!("Wrong error variant! Expected InvalidVertexIndexInPolygon but got: {err:?}"),
    }
}

#[test]
fn error_on_degenerate_edge() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 1, 2]);

    match source_mesh.clone().validate().unwrap_err() {
        ValidationError::DegenerateEdgeInPolygon(polygon) => {
            assert_eq!(polygon, 0);
        }
        err => panic!("Wrong error variant! Expected DegenerateEdgeInPolygon but got: {err:?}"),
    }
}

#[test]
fn error_on_doubly_connected_edge() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(2.0, 0.0, 1.0),
        Vec3::new(2.0, 1.0, 1.0),
    ])
    .with_polygon(0, [0, 1, 2])
    .with_polygon(0, [1, 3, 4, 2])
    .with_polygon(0, [1, 5, 6, 2]);

    match source_mesh.clone().validate().unwrap_err() {
        ValidationError::DoublyConnectedEdge(vertex_1, vertex_2) => {
            assert_eq!((vertex_1, vertex_2), (1, 2));
        }
        err => panic!("Wrong error variant! Expected DoublyConnectedEdge but got: {err:?}"),
    }
}

#[test]
fn derives_connectivity_and_boundary_edges() {
    let source_mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(3.0, 0.0, 1.0),
        Vec3::new(3.0, 1.0, 1.0),
        Vec3::new(1.0, 2.0, 1.0),
        Vec3::new(2.0, 2.0, 1.0),
    ])
    .with_polygon(0, [0, 1, 2])
    .with_polygon(0, [1, 3, 4, 2])
    .with_polygon(0, [3, 5, 6, 4])
    .with_polygon(0, [2, 4, 8, 7]);

    let mut valid_mesh = source_mesh.clone().validate().unwrap();

    // Sort boundary edges to ensure the order is consistent when comparing.
    valid_mesh
        .boundary_edges
        .sort_by_key(|boundary_edge| boundary_edge.poly * 100 + boundary_edge.edge);

    // Each edge has a cost of node 1 to its edge (always 0.5), and each other
    // node to its edge.
    let travel_distances_01 = [
        Vec2::new(2.0 / 3.0, 1.0 / 3.0).distance(Vec2::new(1.0, 0.5)),
        0.5,
    ];
    let travel_distances_12 = [
        0.5,
        Vec3::new(2.5, 0.5, 0.5).distance(Vec3::new(2.0, 0.5, 0.0)),
    ];
    let travel_distances_13 = [
        0.5,
        Vec3::new(1.5, 1.5, 0.5).distance(Vec3::new(1.5, 1.0, 0.0)),
    ];

    let flip = |[a, b]: [f32; 2]| [b, a];

    let expected_connectivity: [&[_]; 4] = [
        &[None, Some(Connectivity::new(1, travel_distances_01)), None],
        &[
            None,
            Some(Connectivity::new(2, travel_distances_12)),
            Some(Connectivity::new(3, travel_distances_13)),
            Some(Connectivity::new(0, flip(travel_distances_01))),
        ],
        &[
            None,
            None,
            None,
            Some(Connectivity::new(1, flip(travel_distances_12))),
        ],
        &[
            Some(Connectivity::new(1, flip(travel_distances_13))),
            None,
            None,
            None,
        ],
    ];
    assert_eq!(
        valid_mesh
            .polygons
            .iter()
            .map(|polygon| polygon.vertices.iter().map(|&(_, c)| c).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        expected_connectivity
    );
    assert_eq!(
        valid_mesh.boundary_edges,
        [
            MeshEdgeRef { poly: 0, edge: 0 },
            MeshEdgeRef { poly: 0, edge: 2 },
            MeshEdgeRef { poly: 1, edge: 0 },
            MeshEdgeRef { poly: 2, edge: 0 },
            MeshEdgeRef { poly: 2, edge: 1 },
            MeshEdgeRef { poly: 2, edge: 2 },
            MeshEdgeRef { poly: 3, edge: 1 },
            MeshEdgeRef { poly: 3, edge: 2 },
            MeshEdgeRef { poly: 3, edge: 3 },
        ]
    );
}

#[test]
fn finds_regions() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 2.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(1.0, 3.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
        //
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(3.0, 1.0, 0.0),
        Vec3::new(2.0, 1.0, 0.0),
        Vec3::new(3.0, 2.0, 0.0),
        Vec3::new(2.0, 2.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2, 3])
    .with_polygon(0, [3, 2, 4, 5])
    .with_polygon(0, [5, 4, 6, 7])
    .with_polygon(0, [8, 9, 10, 11])
    .with_polygon(0, [11, 10, 12, 13])
    .validate()
    .unwrap();

    let regions: Vec<_> = mesh.polygons.iter().map(|polygon| polygon.region).collect();
    assert_eq!(regions, [0, 0, 0, 1, 1],);
}

#[test]
fn sample_point_returns_none_for_far_point() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(4.0, 1.0, 0.0),
        Vec3::new(4.0, 2.0, 0.0),
        Vec3::new(2.0, 3.0, 0.0),
        Vec3::new(1.0, 3.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 5.0, 0.0),
        Vec3::new(2.0, 5.0, 0.0),
        Vec3::new(2.0, 4.0, 0.0),
        Vec3::new(3.0, 5.0, 1.0),
        Vec3::new(3.0, 4.0, 1.0),
        Vec3::new(3.0, 4.0, -2.0),
        Vec3::new(3.0, 3.0, -2.0),
    ])
    .with_polygon(0, [0, 1, 2, 3, 4, 5, 6, 7])
    .with_polygon(0, [5, 4, 10, 9, 8])
    .with_polygon(0, [9, 10, 12, 11])
    .with_polygon(0, [10, 4, 14, 13])
    .validate()
    .unwrap();

    assert_eq!(
        mesh.sample_point(
            Vec3::new(-3.0, 0.0, 0.0),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        None
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(6.0, 0.0, 0.0),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        None
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(0.0, 0.0, -3.0),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        None
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(0.0, 0.0, 2.0),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        None
    );
}

#[test]
fn sample_point_in_nodes() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(4.0, 1.0, 0.0),
        Vec3::new(4.0, 2.0, 0.0),
        Vec3::new(2.0, 3.0, 0.0),
        Vec3::new(1.0, 3.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 5.0, 0.0),
        Vec3::new(2.0, 5.0, 0.0),
        Vec3::new(2.0, 4.0, 0.0),
        Vec3::new(3.0, 5.0, 1.0),
        Vec3::new(3.0, 4.0, 1.0),
        Vec3::new(3.0, 4.0, -2.0),
        Vec3::new(3.0, 3.0, -2.0),
    ])
    .with_polygon(0, [0, 1, 2, 3, 4, 5, 6, 7])
    .with_polygon(0, [5, 4, 10, 9, 8])
    .with_polygon(0, [9, 10, 12, 11])
    .with_polygon(0, [10, 4, 14, 13])
    .validate()
    .unwrap();

    // Flat nodes

    assert_eq!(
        mesh.sample_point(
            Vec3::new(1.5, 1.5, 0.95),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.5, 1.5, 0.0), 0))
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(1.5, 4.0, -0.95),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.5, 4.0, 0.0), 1))
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(1.5, 4.0, -0.95),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.5, 4.0, 0.0), 1))
    );

    // Angled nodes

    assert_eq!(
        mesh.sample_point(
            Vec3::new(2.5, 3.5, -0.55),
            PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_below: 5.0,
                distance_above: 5.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(2.5, 3.5, -1.0), 3))
    );
    assert_eq!(
        mesh.sample_point(
            Vec3::new(2.5, 4.5, 0.1),
            PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_below: 5.0,
                distance_above: 5.0,
                vertical_preference_ratio: 1.0,
            },
        )
        .map(|(point, node)| ((point * 1000.0).round() / 1000.0, node)),
        Some((Vec3::new(2.5, 4.5, 0.5), 2))
    );
}

#[test]
fn sample_point_near_node() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(4.0, 1.0, 0.0),
        Vec3::new(4.0, 2.0, 0.0),
        Vec3::new(2.0, 3.0, 0.0),
        Vec3::new(1.0, 3.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 5.0, 0.0),
        Vec3::new(2.0, 5.0, 0.0),
        Vec3::new(2.0, 4.0, 0.0),
        Vec3::new(3.0, 5.0, 1.0),
        Vec3::new(3.0, 4.0, 1.0),
        Vec3::new(3.0, 4.0, -2.0),
        Vec3::new(3.0, 3.0, -2.0),
    ])
    .with_polygon(0, [0, 1, 2, 3, 4, 5, 6, 7])
    .with_polygon(0, [5, 4, 10, 9, 8])
    .with_polygon(0, [9, 10, 12, 11])
    .with_polygon(0, [10, 4, 14, 13])
    .validate()
    .unwrap();

    // Flat nodes

    assert_eq!(
        mesh.sample_point(
            Vec3::new(-0.5, 1.5, 0.25),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(0.0, 1.5, 0.0), 0))
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(0.5, 5.5, -0.25),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.0, 5.0, 0.0), 1))
    );

    assert_eq!(
        mesh.sample_point(
            Vec3::new(4.5, 1.5, -0.25),
            PointSampleDistance3d {
                horizontal_distance: 1.0,
                distance_below: 1.0,
                distance_above: 1.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(4.0, 1.5, 0.0), 0))
    );

    // Angled nodes

    assert_eq!(
        mesh.sample_point(
            Vec3::new(2.5, 5.5, 0.5),
            PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_below: 5.0,
                distance_above: 5.0,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(2.5, 5.0, 0.5), 2))
    );
}

#[test]
fn valid_polygon_gets_edge_indices() {
    let polygon = Polygon {
        bounds: Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO),
        vertices: vec![(1, None), (3, None), (9, None), (2, None), (7, None)],
        region: 0,
        type_index: 0,
        center: Vec3::ZERO,
    };

    assert_eq!(polygon.edge_indices(0), [1, 3]);
    assert_eq!(polygon.edge_indices(2), [9, 2]);
    assert_eq!(polygon.edge_indices(4), [7, 1]);
}

#[test]
fn sample_ignores_closer_horizontal() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, -1.9),
        Vec3::new(3.0, 0.0, -1.9),
        Vec3::new(3.0, 1.0, -1.9),
        Vec3::new(1.0, 1.0, -1.9),
    ])
    .with_polygon(0, [0, 1, 2, 3])
    .with_polygon(0, [4, 5, 6, 7])
    .validate()
    .unwrap();

    // The first polygon is physically closer, but it is not within the horizontal
    // distance, so it should be filtered out. The second polygon is much further
    // physically, but it is still within the distance_below.
    assert_eq!(
        mesh.sample_point(
            Vec3::new(1.5, 0.5, 0.0),
            PointSampleDistance3d {
                horizontal_distance: 0.25,
                distance_above: 5.0,
                distance_below: 5.0,
                vertical_preference_ratio: 1.0,
            }
        ),
        Some((Vec3::new(1.5, 0.5, -1.9), 1))
    );
}

#[test]
fn sample_favoring_vertical() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, -1.9),
        Vec3::new(3.0, 0.0, -1.9),
        Vec3::new(3.0, 1.0, -1.9),
        Vec3::new(1.0, 1.0, -1.9),
    ])
    .with_polygon(0, [0, 1, 2, 3])
    .with_polygon(0, [4, 5, 6, 7])
    .validate()
    .unwrap();

    // Both polygons are in range, but the one below our query point is preferred,
    // since our vertical preference is 2.0.
    assert_eq!(
        mesh.sample_point(
            Vec3::new(2.0, 0.5, 0.0),
            PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_above: 5.0,
                distance_below: 5.0,
                vertical_preference_ratio: 2.0,
            }
        ),
        Some((Vec3::new(2.0, 0.5, -1.9), 1))
    );
}

#[test]
fn sample_filters_vertical_points_differently() {
    let mesh = NavMeshBuilder::<XYZ>::new([
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2, 3])
    .validate()
    .unwrap();

    let point_sample_distance = PointSampleDistance3d {
        horizontal_distance: 100.0,
        distance_above: 1.0,
        distance_below: 2.0,
        vertical_preference_ratio: 1.0,
    };
    assert_eq!(
        mesh.sample_point(Vec3::new(0.5, 0.5, -0.5), point_sample_distance),
        Some((Vec3::new(0.5, 0.5, 0.0), 0))
    );
    assert_eq!(
        mesh.sample_point(Vec3::new(0.5, 0.5, -1.5), point_sample_distance),
        None
    );
    assert_eq!(
        mesh.sample_point(Vec3::new(0.5, 0.5, 0.5), point_sample_distance),
        Some((Vec3::new(0.5, 0.5, 0.0), 0))
    );
    assert_eq!(
        mesh.sample_point(Vec3::new(0.5, 0.5, 1.5), point_sample_distance),
        Some((Vec3::new(0.5, 0.5, 0.0), 0))
    );
    assert_eq!(
        mesh.sample_point(Vec3::new(0.5, 0.5, 2.5), point_sample_distance),
        None
    );
}
