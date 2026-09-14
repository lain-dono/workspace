use super::ArchipelagoWorld;
use crate::agent::ReachedCondition;
use crate::bvh::Transform;
use crate::landmass::path::Waypoint;
use crate::landmass::{
    Agent, Island, NavMeshBuilder, NodeType,
    agent::RepathResult,
    coords::{XY, XYZ},
    nav_data::NodeRef,
    path::{IslandSegment, Path, PathIndex},
};
use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use bevy::platform::collections::HashSet;
use slotmap::HopSlotMap;
use std::{f32::consts::PI, sync::Arc};

#[test]
fn overrides_node_type_costs() {
    // Create a fake slotmap just to get the NodeTypes out of it.
    let mut slotmap = HopSlotMap::<NodeType, _>::with_key();
    let node_type_1 = slotmap.insert(0);
    let node_type_2 = slotmap.insert(0);

    let mut agent = Agent::<XY>::new(Vec2::ZERO, 1.0, 1.0, 1.0);
    assert!(agent.override_node_type_cost(node_type_1, 3.0));
    assert!(agent.override_node_type_cost(node_type_2, 0.5));

    assert_eq!(
        {
            let mut vec = agent.node_type_cost_overrides().collect::<Vec<_>>();
            vec.sort_by_key(|&(a, _)| a);
            vec
        },
        [(node_type_1, 3.0), (node_type_2, 0.5)]
    );

    agent.override_node_type_cost(node_type_1, 5.0);

    assert_eq!(
        {
            let mut vec = agent.node_type_cost_overrides().collect::<Vec<_>>();
            vec.sort_by_key(|&(a, _)| a);
            vec
        },
        [(node_type_1, 5.0), (node_type_2, 0.5)]
    );

    agent.remove_overridden_node_type_cost(node_type_1);

    assert_eq!(
        {
            let mut vec = agent.node_type_cost_overrides().collect::<Vec<_>>();
            vec.sort_by_key(|&(a, _)| a);
            vec
        },
        [(node_type_2, 0.5)]
    );
}

#[test]
fn negative_or_zero_node_type_cost_returns_false() {
    // Create a fake slotmap just to get the NodeTypes out of it.
    let mut slotmap = HopSlotMap::<NodeType, _>::with_key();
    let node_type = slotmap.insert(0);

    let mut agent = Agent::<XY>::new(Vec2::ZERO, 1.0, 1.0, 1.0);
    assert!(!agent.override_node_type_cost(node_type, 0.0));
    assert!(!agent.override_node_type_cost(node_type, -0.5));
}

#[test]
fn has_reached_target_at_end_node() {
    let nav_mesh = NavMeshBuilder::new([])
        .validate()
        .expect("nav mesh is valid");

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let transform = Transform::new(Vec3::new(2.0, 3.0, 4.0), PI * 0.85);
    let island = world.add_island(Island::new(transform, Arc::new(nav_mesh)));
    let mut agent = Agent::new(transform.apply(Vec3::new(1.0, 0.0, 1.0)), 0.0, 0.0, 0.0);

    let path = Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![0],
            portal_edge_index: vec![],
        }],
        boundary_link_segments: vec![],
    };

    let first_path_index = PathIndex::new(0, 0);
    for condition in [
        ReachedCondition::Distance(Some(2.0)),
        ReachedCondition::StraightPathDistance(Some(2.0)),
        ReachedCondition::VisibleAtDistance(Some(2.0)),
    ] {
        agent.reached_condition = condition;

        assert!(agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.5, 0.0, 1.0))),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.5, 0.0, 1.0))),
        ));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(3.5, 0.0, 1.0))),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(3.5, 0.0, 1.0))),
        ));
        assert!(agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.0, 0.0, 2.0))),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.0, 0.0, 2.0))),
        ));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.5, 0.0, 2.5))),
            Waypoint::new(first_path_index, transform.apply(Vec3::new(2.5, 0.0, 2.5))),
        ));
    }
}

#[test]
fn long_detour_reaches_target_in_different_ways() {
    let nav_mesh = NavMeshBuilder::new([
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(1.0, 11.0, 0.0),
        Vec3::new(0.0, 12.0, 0.0),
        Vec3::new(2.0, 11.0, 0.0),
        Vec3::new(3.0, 12.0, 0.0),
        Vec3::new(2.0, 1.0, 0.0),
    ])
    .with_polygon(0, [0, 1, 2])
    .with_polygon(0, [2, 1, 3, 4])
    .with_polygon(0, [4, 3, 5])
    .validate()
    .expect("nav mesh is valid");

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let transform = Transform::new(Vec3::new(2.0, 4.0, 3.0), PI * -0.85);
    let island = world.add_island(Island::<XYZ>::new(transform, Arc::new(nav_mesh)));

    let mut agent = Agent::new(Vec3::ZERO, 0.0, 0.0, 0.0);

    let path = Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![0, 1, 2],
            portal_edge_index: vec![1, 2],
        }],
        boundary_link_segments: vec![],
    };

    {
        agent.position = transform.apply(Vec3::new(1.0, 1.0, 0.0));
        agent.reached_condition = ReachedCondition::Distance(Some(1.1));

        // Agent started within 1.1 units of the destination, so they are close enough.
        assert!(agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // Agent is just outside of 1.1 units, so they still have not reached the end.
        agent.position = transform.apply(Vec3::new(1.0, 2.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));
    }

    {
        agent.reached_condition = ReachedCondition::VisibleAtDistance(Some(15.0));

        // The agent cannot see the target and its path is still too long.
        agent.position = transform.apply(Vec3::new(1.0, 1.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // The agent only has 12 units left to travel to the target, and yet the
        // agent still hasn't reached the target, since the target is not visible.
        agent.position = transform.apply(Vec3::new(1.0, 10.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // The agent has now "rounded the corner", and so can see the target (and
        // is within the correct distance).
        agent.position = transform.apply(Vec3::new(2.0, 11.0, 0.0));
        assert!(agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // The agent can see the target but is still too far away.
        agent.position = transform.apply(Vec3::new(2.0, 20.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));
    }

    {
        agent.reached_condition = ReachedCondition::StraightPathDistance(Some(15.0));

        // The agent's path is too long (21 units).
        agent.position = transform.apply(Vec3::new(1.0, 1.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // The agent only has 12 units left to travel to the target, so they have
        // reached the target.
        agent.position = transform.apply(Vec3::new(1.0, 10.0, 0.0));
        assert!(agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 1),
                transform.apply(Vec3::new(1.0, 11.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));

        // The agent can see the target but is still too far away.
        agent.position = transform.apply(Vec3::new(2.0, 20.0, 0.0));
        assert!(!agent.has_reached_target(
            world.islands(),
            &path,
            world.nav(),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
            Waypoint::new(
                PathIndex::new(0, 2),
                transform.apply(Vec3::new(2.0, 1.0, 0.0))
            ),
        ));
    }
}

#[test]
fn nothing_or_clear_path_for_no_target() {
    let mut agent = Agent::<XYZ>::new(Vec3::ZERO, 0.0, 0.0, 0.0);

    assert_eq!(
        agent.need_repath(None, None, &HashSet::new(), &HashSet::new()),
        RepathResult::DoNothing
    );

    agent.current_path = Some(Path {
        island_segments: vec![],
        boundary_link_segments: vec![],
    });

    assert_eq!(
        agent.need_repath(None, None, &HashSet::new(), &HashSet::new()),
        RepathResult::ClearPathNoTarget,
    );
}

#[test]
fn clears_path_for_missing_nodes() {
    let agent = Agent::<XYZ>::new(Vec3::ZERO, 0.0, 0.0, 0.0).with_target(Vec3::ZERO);

    // Create an unused slotmap just to get `IslandId`s.
    let island = Entity::PLACEHOLDER;

    assert_eq!(
        agent.need_repath(
            None,
            Some(NodeRef::new(island, 0)),
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::ClearPathBadAgent,
    );

    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 0)),
            None,
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::ClearPathBadTarget,
    );
}

#[test]
fn repaths_for_invalid_path_or_nodes_off_path() {
    let mut agent = Agent::<XYZ>::new(Vec3::ZERO, 0.0, 0.0, 0.0).with_target(Vec3::ZERO);

    // Create an unused slotmap just to get `IslandId`s.
    let island = Entity::PLACEHOLDER;
    let missing_island = Entity::from_raw_u32(u32::MAX - 1).unwrap();

    // No path.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 1)),
            Some(NodeRef::new(island, 3)),
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::NeedsRepath,
    );

    agent.current_path = Some(Path {
        island_segments: vec![IslandSegment {
            island,
            corridor: vec![2, 3, 4, 1, 0],
            portal_edge_index: vec![],
        }],
        boundary_link_segments: vec![],
    });

    // Invalidated island.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 3)),
            Some(NodeRef::new(island, 1)),
            &HashSet::new(),
            &HashSet::from([island]),
        ),
        RepathResult::NeedsRepath
    );

    // Missing agent node in path.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 5)),
            Some(NodeRef::new(island, 1)),
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::NeedsRepath,
    );

    // Missing target node.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 3)),
            Some(NodeRef::new(island, 6)),
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::NeedsRepath,
    );

    // Agent and target are in the wrong order.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 1)),
            Some(NodeRef::new(island, 3)),
            &HashSet::new(),
            &HashSet::new(),
        ),
        RepathResult::NeedsRepath,
    );

    // Following is now fine.
    assert_eq!(
        agent.need_repath(
            Some(NodeRef::new(island, 3)),
            Some(NodeRef::new(island, 1)),
            &HashSet::new(),
            // This island is not involved in the path, so the path is still valid.
            &HashSet::from([missing_island]),
        ),
        RepathResult::FollowPath(PathIndex::new(0, 1), PathIndex::new(0, 3)),
    );
}
