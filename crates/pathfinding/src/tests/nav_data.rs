use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::island::Island;
use crate::landmass::{
    AgentOptions, FromAgentRadius, IslandId, NewNodeTypeError, NodeType, PointSampleDistance3d,
    SetNodeTypeCostError,
    coords::{XY, XYZ},
    nav_data::{
        Archipelago, BoundaryLink, BoundaryLinkId, ModifiedNode, NodeRef, island_edges_bbh,
        link_edges_between_islands,
    },
    nav_mesh::NavMeshBuilder,
};
use bevy::{
    ecs::entity::Entity,
    math::{Vec2, Vec3},
    platform::collections::{HashMap, HashSet},
};
use slotmap::SlotMap;
use std::{cmp::Ordering, f32::consts::PI, sync::Arc};

#[test]
fn samples_points() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 1.0, 1.0),
            Vec3::new(3.0, 1.0, 1.0),
            Vec3::new(4.0, 1.0, 1.0),
            Vec3::new(4.0, 2.0, 1.0),
            Vec3::new(3.0, 2.0, 1.0),
            Vec3::new(2.0, 2.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
        ],
        polygons: vec![vec![0, 1, 6, 7], vec![1, 2, 5, 6], vec![2, 3, 4, 5]],
        polygon_type_indices: vec![0, 0, 0],
    }
    .validate()
    .expect("is valid");
    let nav_mesh = Arc::new(nav_mesh);

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_id_1 = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_id_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(5.0, 0.0, 0.1), PI * 0.5),
        Arc::clone(&nav_mesh),
    ));

    // Just above island 1 node.
    assert_eq!(
        world.sample_point(
            Vec3::new(1.5, 1.5, 1.09),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.5, 1.5, 1.0), NodeRef::new(island_id_1, 0))),
    );
    // Just outside island 1 node.
    assert_eq!(
        world.sample_point(
            Vec3::new(0.95, 0.95, 0.95),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(1.0, 1.0, 1.0), NodeRef::new(island_id_1, 0))),
    );
    // At overlap, but closer to island 1.
    assert_eq!(
        world.sample_point(
            Vec3::new(3.5, 1.5, 1.04),
            PointSampleDistance3d {
                horizontal_distance: 0.1,
                distance_below: 0.1,
                distance_above: 0.1,
                vertical_preference_ratio: 1.0,
            },
        ),
        Some((Vec3::new(3.5, 1.5, 1.0), NodeRef::new(island_id_1, 2))),
    );
    // At overlap, but closer to island 2.
    assert_eq!(
        world
            .sample_point(
                Vec3::new(3.5, 1.5, 1.06),
                PointSampleDistance3d {
                    horizontal_distance: 0.1,
                    distance_below: 0.1,
                    distance_above: 0.1,
                    vertical_preference_ratio: 1.0,
                },
            )
            .map(|(p, n)| ((p * 1e6).round() / 1e6, n)),
        Some((Vec3::new(3.5, 1.5, 1.1), NodeRef::new(island_id_2, 0))),
    );
}

fn node_ref_to_num(node_ref: &NodeRef, island_order: &[IslandId]) -> u32 {
    let island_index = island_order
        .iter()
        .position(|id| node_ref.island == *id)
        .unwrap();
    island_index as u32 * 100 + node_ref.polygon as u32
}

fn clone_sort_round_links(
    boundary_links: &SlotMap<BoundaryLinkId, BoundaryLink>,
    node_to_boundary_link_ids: &HashMap<NodeRef, HashSet<BoundaryLinkId>>,
    island_order: &[IslandId],
    round_amount: f32,
) -> Vec<(NodeRef, Vec<BoundaryLink>)> {
    let mut links = node_to_boundary_link_ids
        .iter()
        .map(|(key, value)| {
            (*key, {
                let mut v = value
                    .iter()
                    .map(|link_id| boundary_links.get(*link_id).unwrap())
                    .map(|link| BoundaryLink {
                        destination: link.destination,
                        destination_node_type: link.destination_node_type,
                        portal: link
                            .portal
                            .map(|lp| (lp / round_amount).round() * round_amount),
                        travel_distances: (
                            (link.travel_distances.0 / round_amount).round() * round_amount,
                            (link.travel_distances.1 / round_amount).round() * round_amount,
                        ),
                    })
                    .collect::<Vec<_>>();
                v.sort_by_key(|link| node_ref_to_num(&link.destination, island_order));
                v
            })
        })
        .collect::<Vec<_>>();
    links.sort_by_key(|(a, _)| node_ref_to_num(a, island_order));
    links
}

fn chain_length_rounded(chain: [Vec2; 3], round_amount: f32) -> (f32, f32) {
    let distances = (chain[0].distance(chain[1]), chain[1].distance(chain[2]));
    (
        (distances.0 / round_amount).round() * round_amount,
        (distances.1 / round_amount).round() * round_amount,
    )
}

fn flip((a, b): (f32, f32)) -> (f32, f32) {
    (b, a)
}

#[test]
fn link_edges_between_islands_links_touching_islands() {
    fn transform_and_round_portal(transform: &Transform<XYZ>, a: Vec3, b: Vec3) -> [Vec3; 2] {
        [
            (transform.apply(a) * 1e6).round() * 1e-6,
            (transform.apply(b) * 1e6).round() * 1e-6,
        ]
    }

    let nav_mesh_1 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(-1.0, 1.0, 1.0),
                Vec3::new(-1.0, 0.0, 1.0),
                Vec3::new(-1.0, -1.0, 1.0),
                Vec3::new(1.0, -1.0, 1.0),
                //
                Vec3::new(2.0, 0.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(-2.0, 2.0, 1.0),
                Vec3::new(-2.0, 0.0, 1.0),
                Vec3::new(-2.0, -2.0, 1.0),
                Vec3::new(2.0, -2.0, 1.0),
            ],
            polygons: vec![
                vec![0, 6, 7, 1],
                vec![1, 7, 8, 2],
                vec![2, 8, 9, 3],
                vec![10, 4, 3, 9],
                vec![10, 11, 5, 4],
                vec![6, 0, 5, 11],
            ],
            polygon_type_indices: vec![0, 1, 1, 1, 0, 0],
        }
        .validate()
        .expect("is valid."),
    );

    let nav_mesh_2 = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(-1.0, -0.5, 1.0),
                Vec3::new(-0.5, -0.5, 1.0),
                Vec3::new(0.5, -0.5, 1.0),
                Vec3::new(1.0, -0.5, 1.0),
                Vec3::new(-1.0, 0.5, 1.0),
                Vec3::new(-0.5, 0.5, 1.0),
                Vec3::new(0.5, 0.5, 1.0),
                Vec3::new(1.0, 0.5, 1.0),
                Vec3::new(-0.5, 1.0, 1.0),
                Vec3::new(0.5, 1.0, 1.0),
                Vec3::new(-0.5, -1.0, 1.0),
                Vec3::new(0.5, -1.0, 1.0),
            ],
            polygons: vec![
                vec![5, 4, 0, 1],
                vec![1, 2, 6, 5],
                vec![3, 7, 6, 2],
                vec![5, 6, 9, 8],
                vec![10, 11, 2, 1],
            ],
            polygon_type_indices: vec![0, 0, 0, 0, 1],
        }
        .validate()
        .expect("is valid."),
    );

    // Create unused slotmaps just to get `IslandId`s and `NodeType`s.
    let island_1_id = Entity::from_raw_u32(1).unwrap();
    let island_2_id = Entity::from_raw_u32(2).unwrap();
    let mut slotmap = SlotMap::<NodeType, _>::with_key();
    let node_type_1 = slotmap.insert(0);
    let node_type_2 = slotmap.insert(0);

    let transform = Transform::new(Vec3::new(1.0, 2.0, 3.0), PI * -0.25);

    let island_1 = Island::new(transform, Arc::clone(&nav_mesh_1))
        .with_node_types(HashMap::from([(1, node_type_1)]));
    let island_2 = Island::new(transform, Arc::clone(&nav_mesh_2))
        .with_node_types(HashMap::from([(1, node_type_2)]));

    let island_1_edge_bbh = island_edges_bbh(&island_1);
    let island_2_edge_bbh = island_edges_bbh(&island_2);

    let mut boundary_links = SlotMap::with_key();
    let mut node_to_boundary_link_ids = HashMap::new();
    let mut modified_node_refs_to_update = HashSet::new();

    link_edges_between_islands(
        (island_1_id, &island_1),
        (island_2_id, &island_2),
        &island_1_edge_bbh,
        1e-5,
        &mut boundary_links,
        &mut node_to_boundary_link_ids,
        &mut modified_node_refs_to_update,
    );

    // The cost of a link between island_2's horizontal nodes, and island_1's
    // left/right nodes. All these are symmetric, so we can just pick a candidate.
    let diagonal_distances = chain_length_rounded(
        [
            Vec2::new(1.5, 0.75),
            Vec2::new(1.0, 0.25),
            Vec2::new(0.75, 0.0),
        ],
        1e-6,
    );
    // The same logic applies for the vertical connections.
    let vertical_distances = chain_length_rounded(
        [
            Vec2::new(0.0, 1.5),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, 0.75),
        ],
        1e-6,
    );

    let expected_links = [
        (
            NodeRef::new(island_1_id, 0),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 2),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(1.0, 0.5, 1.0),
                    Vec3::new(1.0, 0.0, 1.0),
                ),
                travel_distances: diagonal_distances,
            }],
        ),
        (
            NodeRef::new(island_1_id, 1),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 3),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(-0.5, 1.0, 1.0),
                    Vec3::new(0.5, 1.0, 1.0),
                ),
                travel_distances: vertical_distances,
            }],
        ),
        (
            NodeRef::new(island_1_id, 2),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 0),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(-1.0, 0.0, 1.0),
                    Vec3::new(-1.0, 0.5, 1.0),
                ),
                travel_distances: diagonal_distances,
            }],
        ),
        (
            NodeRef::new(island_1_id, 3),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 0),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(-1.0, -0.5, 1.0),
                    Vec3::new(-1.0, 0.0, 1.0),
                ),
                travel_distances: diagonal_distances,
            }],
        ),
        (
            NodeRef::new(island_1_id, 4),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 4),
                destination_node_type: Some(node_type_2),
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(0.5, -1.0, 1.0),
                    Vec3::new(-0.5, -1.0, 1.0),
                ),
                travel_distances: vertical_distances,
            }],
        ),
        (
            NodeRef::new(island_1_id, 5),
            vec![BoundaryLink {
                destination: NodeRef::new(island_2_id, 2),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(1.0, 0.0, 1.0),
                    Vec3::new(1.0, -0.5, 1.0),
                ),
                travel_distances: diagonal_distances,
            }],
        ),
        // Reverse links
        (
            NodeRef::new(island_2_id, 0),
            vec![
                BoundaryLink {
                    destination: NodeRef::new(island_1_id, 2),
                    destination_node_type: Some(node_type_1),
                    portal: transform_and_round_portal(
                        &transform,
                        Vec3::new(-1.0, 0.5, 1.0),
                        Vec3::new(-1.0, 0.0, 1.0),
                    ),
                    travel_distances: flip(diagonal_distances),
                },
                BoundaryLink {
                    destination: NodeRef::new(island_1_id, 3),
                    destination_node_type: Some(node_type_1),
                    portal: transform_and_round_portal(
                        &transform,
                        Vec3::new(-1.0, 0.0, 1.0),
                        Vec3::new(-1.0, -0.5, 1.0),
                    ),
                    travel_distances: flip(diagonal_distances),
                },
            ],
        ),
        (
            NodeRef::new(island_2_id, 2),
            vec![
                BoundaryLink {
                    destination: NodeRef::new(island_1_id, 0),
                    destination_node_type: None,
                    portal: transform_and_round_portal(
                        &transform,
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.5, 1.0),
                    ),
                    travel_distances: flip(diagonal_distances),
                },
                BoundaryLink {
                    destination: NodeRef::new(island_1_id, 5),
                    destination_node_type: None,
                    portal: transform_and_round_portal(
                        &transform,
                        Vec3::new(1.0, -0.5, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                    ),
                    travel_distances: flip(diagonal_distances),
                },
            ],
        ),
        (
            NodeRef::new(island_2_id, 3),
            vec![BoundaryLink {
                destination: NodeRef::new(island_1_id, 1),
                destination_node_type: Some(node_type_1),
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(0.5, 1.0, 1.0),
                    Vec3::new(-0.5, 1.0, 1.0),
                ),
                travel_distances: flip(vertical_distances),
            }],
        ),
        (
            NodeRef::new(island_2_id, 4),
            vec![BoundaryLink {
                destination: NodeRef::new(island_1_id, 4),
                destination_node_type: None,
                portal: transform_and_round_portal(
                    &transform,
                    Vec3::new(-0.5, -1.0, 1.0),
                    Vec3::new(0.5, -1.0, 1.0),
                ),
                travel_distances: flip(vertical_distances),
            }],
        ),
    ];
    assert_eq!(
        clone_sort_round_links(
            &boundary_links,
            &node_to_boundary_link_ids,
            &[island_1_id, island_2_id],
            1e-6
        ),
        &expected_links
    );

    let mut modified_node_refs_to_update_sorted = modified_node_refs_to_update
        .iter()
        .copied()
        .collect::<Vec<_>>();
    modified_node_refs_to_update_sorted.sort();
    assert_eq!(
        modified_node_refs_to_update_sorted,
        [
            NodeRef::new(island_1_id, 0),
            NodeRef::new(island_1_id, 1),
            NodeRef::new(island_1_id, 2),
            NodeRef::new(island_1_id, 3),
            NodeRef::new(island_1_id, 4),
            NodeRef::new(island_1_id, 5),
            //
            NodeRef::new(island_2_id, 0),
            NodeRef::new(island_2_id, 2),
            NodeRef::new(island_2_id, 3),
            NodeRef::new(island_2_id, 4),
        ]
    );

    boundary_links = SlotMap::with_key();
    node_to_boundary_link_ids = HashMap::new();
    modified_node_refs_to_update = HashSet::new();

    link_edges_between_islands(
        (island_2_id, &island_2),
        (island_1_id, &island_1),
        &island_2_edge_bbh,
        1e-5,
        &mut boundary_links,
        &mut node_to_boundary_link_ids,
        &mut modified_node_refs_to_update,
    );
    assert_eq!(
        clone_sort_round_links(
            &boundary_links,
            &node_to_boundary_link_ids,
            &[island_1_id, island_2_id],
            1e-6
        ),
        &expected_links
    );
}

#[test]
fn update_links_islands_and_unlinks_on_delete() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(2.0, 0.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(0.0, 2.0, 1.0),
                Vec3::new(0.0, 1.0, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 5], vec![4, 5, 2, 3]],
            polygon_type_indices: vec![0, 0],
        }
        .validate()
        .expect("is valid."),
    );

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_1_id = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_2_id = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, PI * 0.5),
        Arc::clone(&nav_mesh),
    ));
    let island_3_id = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, PI),
        Arc::clone(&nav_mesh),
    ));
    let island_4_id = world.add_island(Island::new(
        Transform::new(Vec3::new(3.0, 0.0, 0.0), PI),
        Arc::clone(&nav_mesh),
    ));
    let island_5_id = world.add_island(Island::new(
        Transform::new(Vec3::new(2.0, 3.0, 0.0), PI * -0.5),
        Arc::clone(&nav_mesh),
    ));

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);

    let lined_up_cost = chain_length_rounded(
        [
            Vec2::new(1.5, 0.75),
            Vec2::new(1.5, 0.0),
            Vec2::new(1.5, -0.75),
        ],
        1e-6,
    );
    let offset_cost = chain_length_rounded(
        [
            Vec2::new(1.5, 0.75),
            Vec2::new(2.0, 1.5),
            Vec2::new(2.75, 1.5),
        ],
        1e-6,
    );

    assert_eq!(
        clone_sort_round_links(
            &world.nav().boundary_links,
            &world.nav().node_to_boundary_link_ids,
            &[
                island_1_id,
                island_2_id,
                island_3_id,
                island_4_id,
                island_5_id
            ],
            1e-6
        ),
        [
            (
                NodeRef::new(island_1_id, 0),
                vec![
                    BoundaryLink {
                        destination: NodeRef::new(island_4_id, 0),
                        destination_node_type: None,
                        portal: [Vec3::new(1.0, 0.0, 1.0), Vec3::new(2.0, 0.0, 1.0)],
                        travel_distances: lined_up_cost,
                    },
                    BoundaryLink {
                        destination: NodeRef::new(island_5_id, 0),
                        destination_node_type: None,
                        portal: [Vec3::new(2.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 1.0)],
                        travel_distances: offset_cost,
                    },
                ],
            ),
            (
                NodeRef::new(island_1_id, 1),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_2_id, 0,),
                    destination_node_type: None,
                    portal: [Vec3::new(0.0, 2.0, 1.0), Vec3::new(0.0, 1.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_2_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_1_id, 1,),
                    destination_node_type: None,
                    portal: [Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 2.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_2_id, 1),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_3_id, 0,),
                    destination_node_type: None,
                    portal: [Vec3::new(-2.0, 0.0, 1.0), Vec3::new(-1.0, 0.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_3_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_2_id, 1,),
                    destination_node_type: None,
                    portal: [Vec3::new(-1.0, 0.0, 1.0), Vec3::new(-2.0, 0.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_4_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_1_id, 0,),
                    destination_node_type: None,
                    portal: [Vec3::new(2.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_5_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_1_id, 0,),
                    destination_node_type: None,
                    portal: [Vec3::new(2.0, 2.0, 1.0), Vec3::new(2.0, 1.0, 1.0)],
                    travel_distances: flip(offset_cost),
                }],
            ),
        ],
    );

    // Delete island_2 and island_4.
    world.remove_island(island_2_id);
    world.remove_island(island_4_id);
    // Move island_5 to replace island_2.
    world
        .island_mut(island_5_id)
        .unwrap()
        .set_transform(Transform::new(Vec3::ZERO, PI * 0.5));

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);

    assert_eq!(
        clone_sort_round_links(
            &world.nav().boundary_links,
            &world.nav().node_to_boundary_link_ids,
            &[
                island_1_id,
                island_2_id,
                island_3_id,
                island_4_id,
                island_5_id
            ],
            1e-6
        ),
        [
            (
                NodeRef::new(island_1_id, 1),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_5_id, 0),
                    destination_node_type: None,
                    portal: [Vec3::new(0.0, 2.0, 1.0), Vec3::new(0.0, 1.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_3_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_5_id, 1,),
                    destination_node_type: None,
                    portal: [Vec3::new(-1.0, 0.0, 1.0), Vec3::new(-2.0, 0.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_5_id, 0),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_1_id, 1,),
                    destination_node_type: None,
                    portal: [Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 2.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
            (
                NodeRef::new(island_5_id, 1),
                vec![BoundaryLink {
                    destination: NodeRef::new(island_3_id, 0,),
                    destination_node_type: None,
                    portal: [Vec3::new(-2.0, 0.0, 1.0), Vec3::new(-1.0, 0.0, 1.0)],
                    travel_distances: lined_up_cost,
                }],
            ),
        ],
    );
}

fn clone_sort_round_modified_nodes(
    modified_nodes: &HashMap<NodeRef, ModifiedNode>,
    island_id_to_vertices: &HashMap<IslandId, u32>,
    island_order: &[IslandId],
    round_amount: f32,
) -> Vec<(NodeRef, ModifiedNode)> {
    fn order_vec2(a: Vec2, b: Vec2) -> Ordering {
        match a.x.partial_cmp(&b.x).unwrap() {
            Ordering::Equal => {}
            other => return other,
        }
        a.y.partial_cmp(&b.y).unwrap()
    }

    let mut nodes = modified_nodes
        .iter()
        .map(|(key, value)| {
            (*key, {
                let nav_mesh_vertices = *island_id_to_vertices.get(&key.island).unwrap();

                let new_vertices_rounded = value
                    .new_vertices
                    .iter()
                    .map(|&point| (point / round_amount).round() * round_amount)
                    .collect::<Vec<_>>();

                let mut new_vertices_sort_indices =
                    (0..new_vertices_rounded.len() as u32).collect::<Vec<_>>();
                new_vertices_sort_indices.sort_by(|&a, &b| {
                    order_vec2(
                        new_vertices_rounded[a as usize],
                        new_vertices_rounded[b as usize],
                    )
                });

                let new_vertices_sorted = new_vertices_sort_indices
                    .iter()
                    .copied()
                    .map(|index| new_vertices_rounded[index as usize])
                    .collect::<Vec<_>>();

                let mut new_boundary_sorted = value
                    .new_boundary
                    .iter()
                    .map(|&(mut left, mut right)| {
                        if left >= nav_mesh_vertices {
                            left = new_vertices_sort_indices[(left - nav_mesh_vertices) as usize]
                                + nav_mesh_vertices;
                        }
                        if right >= nav_mesh_vertices {
                            right = new_vertices_sort_indices[(right - nav_mesh_vertices) as usize]
                                + nav_mesh_vertices;
                        }
                        (left, right)
                    })
                    .collect::<Vec<_>>();

                new_boundary_sorted.sort_unstable();

                ModifiedNode {
                    new_boundary: new_boundary_sorted,
                    new_vertices: new_vertices_sorted,
                }
            })
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|(node_ref, _)| node_ref_to_num(node_ref, island_order));
    nodes
}

#[test]
fn modifies_node_boundaries_for_linked_islands() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(2.0, 1.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(1.0, 2.0, 1.0),
                Vec3::new(2.0, 3.0, 1.0),
                Vec3::new(1.0, 3.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3], vec![3, 2, 4, 5]],
            polygon_type_indices: vec![0, 0],
        }
        .validate()
        .expect("is valid."),
    );

    let mut world = ArchipelagoWorld::<XYZ>::with_config(1e-6);

    let island_1_id = world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_2_id = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, -1.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));
    let island_3_id = world.add_island(Island::new(
        Transform::new(Vec3::new(2.0, 3.5, 0.0), PI * -0.5),
        Arc::clone(&nav_mesh),
    ));

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);

    let expected_modified_nodes = [
        (
            NodeRef::new(island_1_id, 0),
            ModifiedNode {
                new_boundary: vec![(0, 1), (3, 0)],
                new_vertices: vec![],
            },
        ),
        (
            NodeRef::new(island_2_id, 1),
            ModifiedNode {
                new_boundary: vec![(2, 6), (4, 5)],
                new_vertices: vec![Vec2::new(3.0, 1.5)],
            },
        ),
        (
            NodeRef::new(island_3_id, 0),
            ModifiedNode {
                new_boundary: vec![(0, 6), (1, 2), (3, 0)],
                new_vertices: vec![Vec2::new(3.0, 2.0)],
            },
        ),
    ];

    assert_eq!(
        clone_sort_round_modified_nodes(
            &world.nav().modified_nodes,
            &HashMap::from([(island_1_id, 6), (island_2_id, 6), (island_3_id, 6)]),
            &[island_1_id, island_2_id, island_3_id],
            1e-4
        ),
        &expected_modified_nodes
    );
}

#[test]
fn stale_modified_nodes_are_removed() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(2.0, 1.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(1.0, 2.0, 1.0),
                Vec3::new(2.0, 3.0, 1.0),
                Vec3::new(1.0, 3.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3], vec![3, 2, 4, 5]],
            polygon_type_indices: vec![0, 0],
        }
        .validate()
        .expect("is valid."),
    );

    let mut world = ArchipelagoWorld::<XYZ>::with_config(1e-6);

    world.add_island(Island::new(Transform::default(), Arc::clone(&nav_mesh)));
    let island_2_id = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, -1.0, 0.0), 0.0),
        Arc::clone(&nav_mesh),
    ));

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);
    assert_eq!(world.nav().modified_nodes.len(), 2);

    world.remove_island(island_2_id);

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);
    assert_eq!(world.nav().modified_nodes.len(), 0);
}

#[test]
fn empty_navigation_mesh_is_safe() {
    let full_nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("A square nav mesh is valid."),
    );

    let empty_nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![],
            polygons: vec![],
            polygon_type_indices: vec![],
        }
        .validate()
        .expect("An empty nav mesh is valid."),
    );

    let mut world = ArchipelagoWorld::<XYZ>::with_config(1e-6);
    world.add_island(Island::new(Transform::default(), full_nav_mesh));
    world.add_island(Island::new(Transform::default(), empty_nav_mesh));

    // Nothing should panic here.
    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);
}

#[test]
fn error_on_create_zero_or_negative_node_type() {
    let mut nav = Archipelago::<XY>::new(0.01);
    assert_eq!(
        nav.add_node_type(0.0),
        Err(NewNodeTypeError::NonPositiveCost(0.0))
    );
    assert_eq!(
        nav.add_node_type(-1.0),
        Err(NewNodeTypeError::NonPositiveCost(-1.0))
    );
}

#[test]
fn false_on_setting_zero_or_negative_node_type() {
    let mut nav = Archipelago::<XY>::new(0.01);

    let node_type = nav.add_node_type(1.0).unwrap();

    assert_eq!(
        nav.set_node_type_cost(node_type, 0.0),
        Err(SetNodeTypeCostError::NonPositiveCost(0.0))
    );
    assert_eq!(
        nav.set_node_type_cost(node_type, -1.0),
        Err(SetNodeTypeCostError::NonPositiveCost(-1.0))
    );
}

#[test]
fn cannot_remove_used_node_type() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("mesh is valid"),
    );

    let mut world = ArchipelagoWorld::<XY>::new();

    let node_type_1 = world.add_node_type(2.0);
    let node_type_2 = world.add_node_type(3.0);

    let island_id_1 = world.add_island(
        Island::new(Transform::default(), nav_mesh.clone())
            .with_node_types(HashMap::from([(0, node_type_1)])),
    );

    let island_id_2 = world.add_island(
        Island::new(Transform::default(), nav_mesh.clone())
            .with_node_types(HashMap::from([(0, node_type_1)])),
    );

    // Another island that has no effect since it doesn't mention `node_type_1`.
    world.add_island(
        Island::new(Transform::default(), nav_mesh.clone())
            .with_node_types(HashMap::from([(0, node_type_2)])),
    );

    assert_eq!(
        world.node_types().collect::<Vec<_>>(),
        [(node_type_1, 2.0), (node_type_2, 3.0)]
    );

    // Two islands still reference `node_type_1`.
    assert!(!world.remove_node_type(node_type_1));
    assert_eq!(
        world.node_types().collect::<Vec<_>>(),
        [(node_type_1, 2.0), (node_type_2, 3.0)]
    );

    world.remove_island(island_id_1);

    // One island still references `node_type_1`.
    assert!(!world.remove_node_type(node_type_1));
    assert_eq!(
        world.node_types().collect::<Vec<_>>(),
        [(node_type_1, 2.0), (node_type_2, 3.0)]
    );

    world.remove_island(island_id_2);
    // Now we can delete it!
    assert!(world.remove_node_type(node_type_1));

    // We can't delete it twice.
    assert!(!world.remove_node_type(node_type_1));

    assert_eq!(world.node_types().collect::<Vec<_>>(), [(node_type_2, 3.0)]);
}

#[test]
// #[should_panic = "Island IslandId(1v1) uses node type NodeType(1v1) which is not in this navigation data."]
#[should_panic]
fn panics_on_invalid_node_type() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();
    let deleted_node_type = world.add_node_type(2.0);
    assert!(world.remove_node_type(deleted_node_type));

    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("mesh is valid"),
    );

    let index_to_type = HashMap::from([(0, deleted_node_type)]);
    world.add_island(Island::new(Transform::default(), nav_mesh).with_node_types(index_to_type));

    // Panics due to referencing invalid node type.
    world.update(&options, 1.0);
}
