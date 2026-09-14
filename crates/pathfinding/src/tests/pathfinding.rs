use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::landmass::{
    AgentOptions, FromAgentRadius, Island,
    coords::{XY, XYZ},
    nav_data::NodeRef,
    nav_mesh::NavMeshBuilder,
    path::{BoundaryLinkSegment, IslandSegment, Path},
};
use bevy::{
    math::{Vec2, Vec3},
    platform::collections::HashMap,
};
use std::{f32::consts::PI, sync::Arc};

#[test]
fn finds_path_in_archipelago() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
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
        ],
        polygons: vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![5, 4, 10, 9, 8],
            vec![9, 10, 12, 11],
            vec![10, 4, 14, 13],
        ],
        polygon_type_indices: vec![0, 0, 0, 0],
    }
    .validate()
    .unwrap();

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island = world.add_island(Island::new(Transform::default(), Arc::new(nav_mesh)));

    let path_result = world.find_path(
        NodeRef::new(island, 0),
        NodeRef::new(island, 2),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![0, 1, 2],
                portal_edge_index: vec![4, 2],
            }],
            boundary_link_segments: vec![],
        })
    );

    let path_result = world.find_path(
        NodeRef::new(island, 2),
        NodeRef::new(island, 0),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![2, 1, 0],
                portal_edge_index: vec![0, 0],
            }],
            boundary_link_segments: vec![],
        })
    );

    let path_result = world.find_path(
        NodeRef::new(island, 3),
        NodeRef::new(island, 0),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![3, 1, 0],
                portal_edge_index: vec![0, 0],
            }],
            boundary_link_segments: vec![],
        })
    );
}

#[test]
fn finds_paths_on_two_islands() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
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
        ],
        polygons: vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![5, 4, 10, 9, 8],
            vec![9, 10, 12, 11],
            vec![10, 4, 14, 13],
        ],
        polygon_type_indices: vec![0, 0, 0, 0],
    }
    .validate()
    .unwrap();

    let nav_mesh = Arc::new(nav_mesh);

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1 = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));

    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(6.0, 0.0, 0.0), PI * -0.5),
        Arc::clone(&nav_mesh),
    ));

    let path_result = world.find_path(
        NodeRef::new(island_1, 0),
        NodeRef::new(island_1, 2),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island: island_1,
                corridor: vec![0, 1, 2],
                portal_edge_index: vec![4, 2],
            }],
            boundary_link_segments: vec![],
        })
    );

    let path_result = world.find_path(
        NodeRef::new(island_2, 0),
        NodeRef::new(island_2, 2),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island: island_2,
                corridor: vec![0, 1, 2],
                portal_edge_index: vec![4, 2],
            }],
            boundary_link_segments: vec![],
        })
    );
}

#[test]
fn no_path_between_disconnected_islands() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
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
        ],
        polygons: vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![5, 4, 10, 9, 8],
            vec![9, 10, 12, 11],
            vec![10, 4, 14, 13],
        ],
        polygon_type_indices: vec![0, 0, 0, 0],
    }
    .validate()
    .unwrap();

    let nav_mesh = Arc::new(nav_mesh);

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1 = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(6.0, 0.0, 0.0), PI * -0.5),
        Arc::clone(&nav_mesh),
    ));

    assert!(
        world
            .find_path(
                NodeRef::new(island_1, 0),
                NodeRef::new(island_2, 0),
                &HashMap::new(),
            )
            .path
            .is_none()
    );

    assert!(
        world
            .find_path(
                NodeRef::new(island_2, 0),
                NodeRef::new(island_1, 0),
                &HashMap::new(),
            )
            .path
            .is_none()
    );
}

#[test]
fn find_path_across_connected_islands() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .unwrap(),
    );

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1 = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 0.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));
    // island_3 is unused.
    let _island_3 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, -1.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));
    let island_4 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 1.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));
    let island_5 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 2.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));

    world.update(&options, 1.0);

    let boundary_links = world
        .nav()
        .node_to_boundary_link_ids
        .iter()
        .flat_map(|(node_ref, link_ids)| {
            link_ids.iter().map(|&id| {
                let link = world.nav().boundary_links.get(id).unwrap();
                ((node_ref.island, link.destination.island), id)
            })
        })
        .collect::<HashMap<_, _>>();

    let path_result = world.find_path(
        NodeRef::new(island_1, 0),
        NodeRef::new(island_5, 0),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![
                IslandSegment {
                    island: island_1,
                    corridor: vec![0],
                    portal_edge_index: vec![],
                },
                IslandSegment {
                    island: island_2,
                    corridor: vec![0],
                    portal_edge_index: vec![],
                },
                IslandSegment {
                    island: island_4,
                    corridor: vec![0],
                    portal_edge_index: vec![],
                },
                IslandSegment {
                    island: island_5,
                    corridor: vec![0],
                    portal_edge_index: vec![],
                },
            ],
            boundary_link_segments: vec![
                BoundaryLinkSegment {
                    starting_node: NodeRef::new(island_1, 0),
                    boundary_link: boundary_links[&(island_1, island_2)],
                },
                BoundaryLinkSegment {
                    starting_node: NodeRef::new(island_2, 0),
                    boundary_link: boundary_links[&(island_2, island_4)],
                },
                BoundaryLinkSegment {
                    starting_node: NodeRef::new(island_4, 0),
                    boundary_link: boundary_links[&(island_4, island_5)],
                },
            ],
        })
    );
}

#[test]
fn finds_path_across_different_islands() {
    let nav_mesh_1 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .unwrap(),
    );
    let nav_mesh_2 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
                Vec3::new(1.5, -0.5, 0.0),
                Vec3::new(1.5, 0.5, 0.0),
            ],
            polygons: vec![vec![0, 1, 2, 3], vec![2, 1, 4, 5]],
            polygon_type_indices: vec![0, 0],
        }
        .validate()
        .unwrap(),
    );

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1 = world.add_island(Island::new(Transform::default(), nav_mesh_1));
    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 0.0, 0.0), 0.0),
        nav_mesh_2,
    ));

    world.update(&options, 1.0);

    let boundary_links = world
        .nav()
        .node_to_boundary_link_ids
        .iter()
        .flat_map(|(node_ref, link_ids)| {
            link_ids.iter().map(|&id| {
                let link = world.nav().boundary_links.get(id).unwrap();
                ((node_ref.island, link.destination.island), id)
            })
        })
        .collect::<HashMap<_, _>>();

    let path_result = world.find_path(
        NodeRef::new(island_1, 0),
        NodeRef::new(island_2, 1),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![
                IslandSegment {
                    island: island_1,
                    corridor: vec![0],
                    portal_edge_index: vec![],
                },
                IslandSegment {
                    island: island_2,
                    corridor: vec![0, 1],
                    portal_edge_index: vec![1],
                },
            ],
            boundary_link_segments: vec![BoundaryLinkSegment {
                starting_node: NodeRef::new(island_1, 0),
                boundary_link: boundary_links[&(island_1, island_2)],
            }],
        })
    );
}

#[test]
fn aborts_early_for_unconnected_regions() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(-0.5, -1.5, 0.0),
                Vec3::new(0.5, -1.5, 0.0),
                Vec3::new(0.5, 1.5, 0.0),
                Vec3::new(-0.5, 1.5, 0.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .unwrap(),
    );

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1 = world.add_island(Island::new(Transform::default(), nav_mesh.clone()));
    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(2.0, 0.0, 0.0), 0.0),
        nav_mesh.clone(),
    ));
    let island_3 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.5, 2.0, 0.0), PI * 0.5),
        nav_mesh.clone(),
    ));

    world.update(&options, 1.0);

    // Verify that with island_3, the islands are connected.
    assert!(
        world
            .find_path(
                NodeRef::new(island_1, 0),
                NodeRef::new(island_2, 0),
                &HashMap::new(),
            )
            .path
            .is_some()
    );

    // Remove island_3 which will disconnect the other two islands.
    world.remove_island(island_3);

    world.update(&options, 1.0);

    let path_result = world.find_path(
        NodeRef::new(island_1, 0),
        NodeRef::new(island_2, 0),
        &HashMap::new(),
    );

    assert!(path_result.path.is_none());
    assert_eq!(
        path_result.stats.explored_nodes, 0,
        "No nodes should have been explored since the regions are completely disconnected."
    );
}

#[test]
fn detour_for_high_cost_path() {
    let nav_mesh = Arc::new(
        NavMeshBuilder::<XY> {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                // Extrude right.
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 1.0),
                // Extrude right.
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 1.0),
                // Extrude up.
                Vec2::new(2.0, 2.0),
                Vec2::new(3.0, 2.0),
                // Extrude up.
                Vec2::new(2.0, 3.0),
                Vec2::new(3.0, 3.0),
                // Extrude left.
                Vec2::new(1.0, 2.0),
                Vec2::new(1.0, 3.0),
                // Extrude left.
                Vec2::new(0.0, 2.0),
                Vec2::new(0.0, 3.0),
            ],
            polygons: vec![
                // The bottom row.
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                // The right two cells.
                vec![5, 7, 9, 8],
                vec![8, 9, 11, 10],
                // The top two cells.
                vec![8, 10, 13, 12],
                vec![12, 13, 15, 14],
                // The "slow" bridge.
                vec![3, 2, 12, 14],
            ],
            polygon_type_indices: vec![0, 0, 0, 0, 0, 0, 0, 1],
        }
        .validate()
        .unwrap(),
    );

    let mut world = ArchipelagoWorld::<XY>::new();

    let slow_node_type = world.add_node_type(10.0);

    let island = world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, slow_node_type)])),
    );

    let path_result = world.find_path(
        NodeRef::new(island, 0),
        NodeRef::new(island, 6),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![0, 1, 2, 3, 4, 5, 6],
                portal_edge_index: vec![1, 2, 3, 2, 3, 2],
            }],
            boundary_link_segments: vec![],
        })
    );
}

#[test]
fn detour_for_high_cost_path_across_boundary_links() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

    let nav_mesh_1 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                //
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 1.0),
                //
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3], vec![2, 1, 4, 5], vec![5, 4, 6, 7]],
            polygon_type_indices: vec![0, 0, 0],
        }
        .validate()
        .unwrap(),
    );

    let nav_mesh_2 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 2.0),
                Vec2::new(1.0, 2.0),
                Vec2::new(1.0, 3.0),
                Vec2::new(0.0, 3.0),
                //
                Vec2::new(2.0, 2.0),
                Vec2::new(2.0, 3.0),
                //
                Vec2::new(3.0, 2.0),
                Vec2::new(3.0, 3.0),
                //
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                //
                Vec2::new(2.0, 1.0),
                Vec2::new(3.0, 1.0),
            ],
            polygons: vec![
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                vec![1, 0, 8, 9],
                vec![6, 4, 10, 11],
            ],
            polygon_type_indices: vec![0, 0, 0, 1, 0],
        }
        .validate()
        .unwrap(),
    );

    let slow_node_type = world.add_node_type(5.1);

    let island_1 = world.add_island(Island::new(Transform::default(), nav_mesh_1));
    let island_2 = world.add_island(
        Island::new(Transform::default(), nav_mesh_2)
            .with_node_types(HashMap::from([(1, slow_node_type)])),
    );

    world.update(&options, 1.0);

    let path_result = world.find_path(
        NodeRef::new(island_1, 0),
        NodeRef::new(island_2, 0),
        &HashMap::new(),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![
                IslandSegment {
                    island: island_1,
                    corridor: vec![0, 1, 2],
                    portal_edge_index: vec![1, 2],
                },
                IslandSegment {
                    island: island_2,
                    corridor: vec![4, 2, 1, 0],
                    portal_edge_index: vec![0, 0, 0],
                }
            ],
            boundary_link_segments: vec![BoundaryLinkSegment {
                boundary_link: *world.nav().node_to_boundary_link_ids[&NodeRef::new(island_1, 2)]
                    .iter()
                    .next()
                    .unwrap(),
                starting_node: NodeRef::new(island_1, 2),
            }]
        })
    );
}

#[test]
fn fast_path_not_ignored_by_heuristic() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                // Extrude right.
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 1.0),
                // Extrude right.
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 1.0),
                // Extrude up.
                Vec2::new(2.0, 11.0),
                Vec2::new(3.0, 11.0),
                // Extrude up.
                Vec2::new(2.0, 12.0),
                Vec2::new(3.0, 12.0),
                // Extrude left.
                Vec2::new(1.0, 11.0),
                Vec2::new(1.0, 12.0),
                // Extrude left.
                Vec2::new(0.0, 11.0),
                Vec2::new(0.0, 12.0),
            ],
            polygons: vec![
                // The bottom row.
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                // The right two cells.
                vec![5, 7, 9, 8],
                vec![8, 9, 11, 10],
                // The top two cells.
                vec![8, 10, 13, 12],
                vec![12, 13, 15, 14],
                // The "fast" bridge.
                vec![3, 2, 12, 14],
            ],
            polygon_type_indices: vec![1, 0, 0, 0, 0, 1, 1, 1],
        }
        .validate()
        .unwrap(),
    );

    let mut world = ArchipelagoWorld::<XY>::new();

    // This node type is faster than default.
    let fast_type = world.add_node_type(0.5);

    let island = world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, fast_type)])),
    );

    let path_result = world.find_path(
        NodeRef::new(island, 2),
        NodeRef::new(island, 4),
        &HashMap::new(),
    );

    // The most direct route is [2, 3, 4], but there is a faster detour:
    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![2, 1, 0, 7, 6, 5, 4],
                portal_edge_index: vec![0, 0, 2, 2, 0, 0],
            }],
            boundary_link_segments: vec![],
        })
    );
}

#[test]
fn infinite_or_nan_cost_cannot_find_path() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 1.0),
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3], vec![2, 1, 4, 5], vec![5, 4, 6, 7]],
            polygon_type_indices: vec![0, 1, 0],
        }
        .validate()
        .unwrap(),
    );

    let mut world = ArchipelagoWorld::<XY>::new();
    let node_type = world.add_node_type(f32::INFINITY);

    let island = world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, node_type)])),
    );

    let path_result = world.find_path(
        NodeRef::new(island, 0),
        NodeRef::new(island, 2),
        &HashMap::new(),
    );

    assert_eq!(path_result.path, None);
    assert_eq!(path_result.stats.explored_nodes, 1);

    world.set_node_type_cost(node_type, f32::NAN);

    let path_result = world.find_path(
        NodeRef::new(island, 0),
        NodeRef::new(island, 2),
        &HashMap::new(),
    );

    assert_eq!(path_result.path, None);
    assert_eq!(path_result.stats.explored_nodes, 1);
}

#[test]
fn detour_for_overridden_high_cost_path() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                // Extrude right.
                Vec2::new(2.0, 0.0),
                Vec2::new(2.0, 1.0),
                // Extrude right.
                Vec2::new(3.0, 0.0),
                Vec2::new(3.0, 1.0),
                // Extrude up.
                Vec2::new(2.0, 2.0),
                Vec2::new(3.0, 2.0),
                // Extrude up.
                Vec2::new(2.0, 3.0),
                Vec2::new(3.0, 3.0),
                // Extrude left.
                Vec2::new(1.0, 2.0),
                Vec2::new(1.0, 3.0),
                // Extrude left.
                Vec2::new(0.0, 2.0),
                Vec2::new(0.0, 3.0),
            ],
            polygons: vec![
                // The bottom row.
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                // The right two cells.
                vec![5, 7, 9, 8],
                vec![8, 9, 11, 10],
                // The top two cells.
                vec![8, 10, 13, 12],
                vec![12, 13, 15, 14],
                // The "slow" bridge.
                vec![3, 2, 12, 14],
            ],
            polygon_type_indices: vec![0, 0, 0, 0, 0, 0, 0, 1],
        }
        .validate()
        .unwrap(),
    );

    let mut world = ArchipelagoWorld::<XY>::new();

    let slow_node_type = world.add_node_type(1.0);

    let island = world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, slow_node_type)])),
    );

    let path_result = world.find_path(
        NodeRef::new(island, 0),
        NodeRef::new(island, 6),
        &HashMap::from([(slow_node_type, 10.0)]),
    );

    assert_eq!(
        path_result.path,
        Some(Path {
            island_segments: vec![IslandSegment {
                island,
                corridor: vec![0, 1, 2, 3, 4, 5, 6],
                portal_edge_index: vec![1, 2, 3, 2, 3, 2],
            }],
            boundary_link_segments: vec![],
        })
    );
}
