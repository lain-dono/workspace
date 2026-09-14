use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::landmass::{
    CoordinateSystem, Island,
    coords::{XY, XYZ},
    nav_data::{Archipelago, BoundaryLinkId, NodeRef},
    nav_mesh::NavMeshBuilder,
    path::{BoundaryLinkSegment, IslandSegment, Path, PathIndex, Waypoint},
};
use bevy::{
    ecs::{entity::Entity, system::Query},
    math::{Vec2, Vec3},
    platform::collections::HashSet,
};
use slotmap::HopSlotMap;
use std::{f32::consts::PI, sync::Arc};

fn collect_straight_path<T: CoordinateSystem>(
    path: &Path,
    islands: Query<(Entity, &Island<T>)>,
    nav_data: &Archipelago<T>,
    start: Waypoint,
    end: Waypoint,
    iteration_limit: u32,
) -> Vec<Waypoint> {
    let mut straight_path = Vec::with_capacity(iteration_limit as usize);

    let mut current = start;
    let mut iterations = 0;
    while current.index != end.index && iterations < iteration_limit {
        iterations += 1;
        current = path.find_next_point_in_straight_path(islands, nav_data, current, end);
        straight_path.push(current);
    }

    straight_path
}

#[test]
fn finds_next_point_for_organic_map() {
    let nav_mesh = NavMeshBuilder::<XYZ> {
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

    let mut world = ArchipelagoWorld::new();
    let nav = Archipelago::<XYZ>::new(0.01);

    let transform = Transform::new(Vec3::new(5.0, 9.0, 7.0), PI * -0.35);
    let island = world.add_island(Island::new(transform, Arc::new(nav_mesh)));

    let path = Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![0, 1, 2],
            portal_edge_index: vec![4, 2],
        }],
        boundary_link_segments: vec![],
    };

    let start = Waypoint::new(
        PathIndex::new(0, 0),
        transform.apply(Vec3::new(3.0, 1.5, 0.0)),
    );
    let end = Waypoint::new(
        PathIndex::new(0, 2),
        transform.apply(Vec3::new(2.5, 4.5, 0.5)),
    );

    assert_eq!(
        collect_straight_path(&path, world.islands(), &nav, start, end, 3),
        [
            Waypoint::new(
                PathIndex::new(0, 0),
                transform.apply(Vec3::new(2.0, 3.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(2.0, 4.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.5, 4.5, 0.5))
            ),
        ]
    );
}

#[test]
fn finds_next_point_in_zig_zag() {
    let nav_mesh = NavMeshBuilder::<XY> {
        vertices: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(1.0, 3.0),
            Vec2::new(0.0, 3.0),
            Vec2::new(1.0, 4.0),
            Vec2::new(0.0, 4.0),
            Vec2::new(1.0, 5.0), // Turn right
            Vec2::new(2.0, 4.0),
            Vec2::new(2.0, 5.0),
            Vec2::new(3.0, 4.0),
            Vec2::new(3.0, 5.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(4.0, 5.0),
            Vec2::new(5.0, 5.0), // Turn left
            Vec2::new(5.0, 6.0),
            Vec2::new(4.0, 6.0),
            Vec2::new(5.0, 7.0),
            Vec2::new(4.0, 7.0),
            Vec2::new(4.0, 8.0), // Turn left
            Vec2::new(-3.0, 8.0),
            Vec2::new(-3.0, 7.0),
            Vec2::new(-4.0, 8.0), // Turn right
            Vec2::new(-3.0, 15.0),
            Vec2::new(-4.0, 15.0),
        ],
        polygons: vec![
            vec![0, 1, 2, 3],
            vec![3, 2, 4, 5],
            vec![5, 4, 6, 7],
            vec![7, 6, 8, 9],
            vec![9, 8, 10],
            vec![10, 8, 11, 12],
            vec![12, 11, 13, 14],
            vec![14, 13, 15, 16],
            vec![16, 15, 17],
            vec![16, 17, 18, 19],
            vec![19, 18, 20, 21],
            vec![21, 20, 22],
            vec![21, 22, 23, 24],
            vec![24, 23, 25],
            vec![25, 23, 26, 27],
        ],
        polygon_type_indices: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    }
    .validate()
    .unwrap();

    let mut world = ArchipelagoWorld::<XY>::new();
    let nav = Archipelago::<XY>::new(0.01);

    let transform = Transform::new(Vec2::new(-1.0, -3.0), PI * -1.8);
    let island = world.add_island(Island::<XY>::new(transform, Arc::new(nav_mesh)));

    let path = Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            portal_edge_index: vec![2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1],
        }],
        boundary_link_segments: vec![],
    };

    assert_eq!(
        collect_straight_path(
            &path,
            world.islands(),
            &nav,
            Waypoint::new(
                PathIndex::new(0, 0),
                transform.apply(Vec3::new(0.5, 0.5, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 14),
                transform.apply(Vec3::new(-3.5, 14.0, 0.0))
            ),
            5,
        ),
        [
            Waypoint::new(
                PathIndex::new(0, 4),
                transform.apply(Vec3::new(1.0, 4.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 8),
                transform.apply(Vec3::new(4.0, 5.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 11),
                transform.apply(Vec3::new(4.0, 7.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 13),
                transform.apply(Vec3::new(-3.0, 8.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 14),
                transform.apply(Vec3::new(-3.5, 14.0, 0.0))
            ),
        ]
    );
}

#[test]
fn starts_at_end_index_goes_to_end_point() {
    let nav_mesh = NavMeshBuilder::<XYZ> {
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 2.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ],
        polygons: vec![vec![0, 1, 2, 3], vec![3, 2, 4, 5]],
        polygon_type_indices: vec![0, 0],
    }
    .validate()
    .unwrap();

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let nav = Archipelago::<XYZ>::new(0.01);
    let island = world.add_island(Island::new(Transform::default(), Arc::new(nav_mesh)));

    let path = Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![0, 1],
            portal_edge_index: vec![2],
        }],
        boundary_link_segments: vec![],
    };

    let start = Waypoint::new(PathIndex::new(0, 1), Vec3::new(0.25, 1.1, 0.0));
    let end = Waypoint::new(PathIndex::new(0, 1), Vec3::new(0.75, 1.9, 0.0));

    let next = path.find_next_point_in_straight_path(world.islands(), &nav, start, end);
    assert_eq!(next, end);
}

#[test]
fn path_not_valid_for_invalidated_islands_or_boundary_links() {
    // Create unused slotmaps just to get `IslandId`s and `BoundaryLinkId`s.
    let island_1 = Entity::from_raw_u32(1).unwrap();
    let island_2 = Entity::from_raw_u32(2).unwrap();
    let island_3 = Entity::from_raw_u32(3).unwrap();
    let mut slotmap = HopSlotMap::<BoundaryLinkId, _>::with_key();
    let boundary_link_1 = slotmap.insert(0);
    let boundary_link_2 = slotmap.insert(0);

    let path = Path {
        island_segments: vec![
            IslandSegment {
                island: island_1,
                corridor: vec![0],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_2,
                corridor: vec![0, 1],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_3,
                corridor: vec![0],
                portal_edge_index: vec![],
            },
        ],
        boundary_link_segments: vec![
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_1, 0),
                boundary_link: boundary_link_1,
            },
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_2, 1),
                boundary_link: boundary_link_2,
            },
        ],
    };

    assert!(path.is_valid(&HashSet::new(), &HashSet::new()));

    // Each island is invalidated.
    assert!(!path.is_valid(&HashSet::new(), &HashSet::from([island_1])));
    assert!(!path.is_valid(&HashSet::new(), &HashSet::from([island_2])));
    assert!(!path.is_valid(&HashSet::new(), &HashSet::from([island_3])));

    // Each boundary link is invalidated.
    assert!(!path.is_valid(&HashSet::from([boundary_link_1]), &HashSet::new()));
    assert!(!path.is_valid(&HashSet::from([boundary_link_2]), &HashSet::new()));
}

#[test]
fn indices_in_path_are_found() {
    // Create unused slotmaps just to get `IslandId`s and `BoundaryLinkId`s.
    let island_1 = Entity::from_raw_u32(1).unwrap();
    let island_2 = Entity::from_raw_u32(2).unwrap();
    let island_3 = Entity::from_raw_u32(3).unwrap();
    let island_4 = Entity::from_raw_u32(4).unwrap();
    let mut slotmap = HopSlotMap::<BoundaryLinkId, _>::with_key();
    let boundary_link_1 = slotmap.insert(0);
    let boundary_link_2 = slotmap.insert(0);

    let path = Path {
        island_segments: vec![
            IslandSegment {
                island: island_1,
                corridor: vec![3],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_2,
                corridor: vec![2, 1],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_3,
                corridor: vec![0],
                portal_edge_index: vec![],
            },
        ],
        boundary_link_segments: vec![
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_1, 0),
                boundary_link: boundary_link_1,
            },
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_2, 1),
                boundary_link: boundary_link_2,
            },
        ],
    };

    assert_eq!(
        path.find_index_of_node(NodeRef::new(island_3, 0)),
        Some(PathIndex::new(2, 0))
    );
    assert_eq!(
        path.find_index_of_node(NodeRef::new(island_1, 3)),
        Some(PathIndex::new(0, 0))
    );
    assert_eq!(
        path.find_index_of_node(NodeRef::new(island_2, 1)),
        Some(PathIndex::new(1, 1))
    );

    // Missing NodeRefs.
    assert_eq!(path.find_index_of_node(NodeRef::new(island_3, 3)), None);
    assert_eq!(path.find_index_of_node(NodeRef::new(island_1, 1)), None);
    assert_eq!(path.find_index_of_node(NodeRef::new(island_4, 4)), None);
}

#[test]
fn indices_in_path_are_found_rev() {
    // Create unused slotmaps just to get `IslandId`s and `BoundaryLinkId`s.
    let island_id_1 = Entity::from_raw_u32(1).unwrap();
    let island_id_2 = Entity::from_raw_u32(2).unwrap();
    let island_id_3 = Entity::from_raw_u32(3).unwrap();
    let island_id_4 = Entity::from_raw_u32(4).unwrap();
    let mut slotmap = HopSlotMap::<BoundaryLinkId, _>::with_key();
    let boundary_link_id_1 = slotmap.insert(0);
    let boundary_link_id_2 = slotmap.insert(0);

    let path = Path {
        island_segments: vec![
            IslandSegment {
                island: island_id_1,
                corridor: vec![3],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_id_2,
                corridor: vec![2, 1],
                portal_edge_index: vec![],
            },
            IslandSegment {
                island: island_id_3,
                corridor: vec![0],
                portal_edge_index: vec![],
            },
        ],
        boundary_link_segments: vec![
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_id_1, 0),
                boundary_link: boundary_link_id_1,
            },
            BoundaryLinkSegment {
                starting_node: NodeRef::new(island_id_2, 1),
                boundary_link: boundary_link_id_2,
            },
        ],
    };

    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_3, 0)),
        Some(PathIndex::new(2, 0))
    );
    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_1, 3)),
        Some(PathIndex::new(0, 0))
    );
    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_2, 1)),
        Some(PathIndex::new(1, 1))
    );

    // Missing NodeRefs.
    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_3, 3)),
        None
    );
    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_1, 1)),
        None
    );
    assert_eq!(
        path.find_index_of_node_rev(NodeRef::new(island_id_4, 4)),
        None
    );
}
