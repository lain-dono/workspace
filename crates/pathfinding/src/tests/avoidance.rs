use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::landmass::{
    Agent, AgentOptions, Character, FromAgentRadius, Island, NavMeshBuilder,
    avoidance::nav_mesh_borders_to_dodgy_obstacles,
    coords::{XY, XYZ},
    nav_data::NodeRef,
};
use bevy::{
    ecs::entity::Entity,
    math::{Vec2, Vec3},
};
use std::sync::Arc;

fn obstacle_matches(left: &dodgy_2d::Obstacle, right: &dodgy_2d::Obstacle) -> bool {
    match (left, right) {
        (
            dodgy_2d::Obstacle::Closed {
                vertices: left_vertices,
            },
            dodgy_2d::Obstacle::Closed {
                vertices: right_vertices,
            },
        ) => {
            for left_offset in 0..left_vertices.len() {
                if left_vertices[left_offset..]
                    .iter()
                    .copied()
                    .chain(left_vertices[..left_offset].iter().copied())
                    .collect::<Vec<_>>()
                    == *right_vertices
                {
                    return true;
                }
            }
            false
        }
        (
            dodgy_2d::Obstacle::Open {
                vertices: left_vertices,
            },
            dodgy_2d::Obstacle::Open {
                vertices: right_vertices,
            },
        ) => left_vertices == right_vertices,
        _ => false,
    }
}

macro_rules! assert_obstacles_match {
  ($left: expr, $right: expr) => {{
    let left = $left;
    let mut right = $right;

    'outer: for (left_index, left_obstacle) in left.iter().enumerate() {
      for (right_index, right_obstacle) in right.iter().enumerate() {
        if obstacle_matches(left_obstacle, right_obstacle) {
          right.remove(right_index);
          continue 'outer;
        }
      }
      panic!("Failed to match left obstacle: index={} obstacle={:?}\n\nleft_obstacles={:?}\n\nremaining_obstacles={:?}\n",
        left_index, left_obstacle, left, right);
    }

    if !right.is_empty() {
      panic!("Failed to match right obstacles:\n\nleft_obstacles={:?}\n\nremaining_obstacles={:?}\n", left, right);
    }
  }};
}

#[test]
fn computes_obstacle_for_box() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(1.0, 2.0, 0.0),
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .expect("Validation succeeds");

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let island_offset = Vec3::new(130.0, -50.0, 20.0);
    let island_offset_dodgy = dodgy_2d::Vec2::new(island_offset.x, island_offset.y);

    let island = world.add_island(Island::new(
        Transform::new(island_offset, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_node = (
        Vec3::new(1.5, 1.5, 0.0) + island_offset,
        NodeRef::new(island, 0),
    );

    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0),
        vec![dodgy_2d::Obstacle::Closed {
            vertices: vec![
                dodgy_2d::Vec2::new(1.0, 1.0) + island_offset_dodgy,
                dodgy_2d::Vec2::new(1.0, 2.0) + island_offset_dodgy,
                dodgy_2d::Vec2::new(2.0, 2.0) + island_offset_dodgy,
                dodgy_2d::Vec2::new(2.0, 1.0) + island_offset_dodgy
            ]
        }]
    );
}

#[test]
fn dead_end_makes_open_obstacle() {
    let nav_mesh = NavMeshBuilder::<XYZ> {
        vertices: vec![
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(1.0, 2.0, 0.0),
            Vec3::new(3.0, 1.0, 0.0),
            Vec3::new(3.0, 2.0, 0.0),
            Vec3::new(4.0, 1.0, 0.0),
            Vec3::new(4.0, 2.0, 0.0),
            Vec3::new(4.0, 3.0, 0.0),
            Vec3::new(3.0, 3.0, 0.0),
            Vec3::new(4.0, 4.0, 0.0),
            Vec3::new(3.0, 4.0, 0.0),
        ],
        polygons: vec![
            vec![0, 1, 2, 3],
            vec![2, 1, 4, 5],
            vec![5, 4, 6, 7],
            vec![5, 7, 8, 9],
            vec![9, 8, 10, 11],
        ],
        polygon_type_indices: vec![0, 0, 0, 0, 0],
    }
    .validate()
    .expect("Validation succeeds");

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_node = (Vec3::new(1.5, 1.5, 0.0), NodeRef::new(island, 0));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0),
        vec![dodgy_2d::Obstacle::Open {
            vertices: vec![
                dodgy_2d::Vec2::new(4.0, 3.0),
                dodgy_2d::Vec2::new(4.0, 2.0),
                dodgy_2d::Vec2::new(4.0, 1.0),
                dodgy_2d::Vec2::new(3.0, 1.0),
                dodgy_2d::Vec2::new(2.0, 1.0),
                dodgy_2d::Vec2::new(1.0, 1.0),
                dodgy_2d::Vec2::new(1.0, 2.0),
                dodgy_2d::Vec2::new(2.0, 2.0),
                dodgy_2d::Vec2::new(3.0, 2.0),
            ]
        }]
    );

    let agent_node = (Vec3::new(3.5, 3.5, 0.0), NodeRef::new(island, 4));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0),
        vec![dodgy_2d::Obstacle::Open {
            vertices: vec![
                dodgy_2d::Vec2::new(3.0, 2.0),
                dodgy_2d::Vec2::new(3.0, 3.0),
                dodgy_2d::Vec2::new(3.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 3.0),
                dodgy_2d::Vec2::new(4.0, 2.0),
                dodgy_2d::Vec2::new(4.0, 1.0),
                dodgy_2d::Vec2::new(3.0, 1.0),
                dodgy_2d::Vec2::new(2.0, 1.0),
            ]
        }]
    );

    // Decrease the distance limit to limit the size of the open obstacle.
    let agent_node = (Vec3::new(3.5, 3.5, 0.0), NodeRef::new(island, 4));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 1.0),
        vec![dodgy_2d::Obstacle::Open {
            vertices: vec![
                dodgy_2d::Vec2::new(3.0, 2.0),
                dodgy_2d::Vec2::new(3.0, 3.0),
                dodgy_2d::Vec2::new(3.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 3.0),
                dodgy_2d::Vec2::new(4.0, 2.0),
            ]
        }]
    );

    let agent_node = (Vec3::new(3.5, 1.5, 0.0), NodeRef::new(island, 2));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0),
        vec![dodgy_2d::Obstacle::Closed {
            vertices: vec![
                dodgy_2d::Vec2::new(1.0, 1.0),
                dodgy_2d::Vec2::new(1.0, 2.0),
                dodgy_2d::Vec2::new(2.0, 2.0),
                dodgy_2d::Vec2::new(3.0, 2.0),
                dodgy_2d::Vec2::new(3.0, 3.0),
                dodgy_2d::Vec2::new(3.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 4.0),
                dodgy_2d::Vec2::new(4.0, 3.0),
                dodgy_2d::Vec2::new(4.0, 2.0),
                dodgy_2d::Vec2::new(4.0, 1.0),
                dodgy_2d::Vec2::new(3.0, 1.0),
                dodgy_2d::Vec2::new(2.0, 1.0),
            ]
        }]
    );
}

#[test]
fn split_borders() {
    let nav_mesh = NavMeshBuilder::<XYZ> {
        vertices: vec![
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(3.0, 1.0, 0.0),
            Vec3::new(4.0, 1.0, 0.0),
            Vec3::new(5.0, 1.0, 0.0),
            Vec3::new(5.0, 2.0, 0.0),
            Vec3::new(5.0, 3.0, 0.0),
            Vec3::new(5.0, 4.0, 0.0),
            Vec3::new(6.0, 4.0, 0.0),
            Vec3::new(6.0, 3.0, 0.0),
            Vec3::new(6.0, 2.0, 0.0),
            Vec3::new(6.0, 1.0, 0.0),
            Vec3::new(6.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
            Vec3::new(1.0, 4.0, 0.0),
            Vec3::new(1.0, 3.0, 0.0),
            Vec3::new(1.0, 2.0, 0.0),
        ],
        polygons: vec![
            vec![0, 17, 16, 1],
            vec![1, 16, 15, 2],
            vec![2, 15, 14, 3],
            vec![3, 14, 13, 4],
            vec![4, 13, 12, 11],
            vec![4, 11, 10, 5],
            vec![5, 10, 9, 6],
            vec![6, 9, 8, 7],
            vec![0, 19, 18, 17],
            vec![25, 20, 19, 0],
            vec![24, 21, 20, 25],
            vec![23, 22, 21, 24],
        ],
        polygon_type_indices: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    }
    .validate()
    .expect("Validation succeeds");

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_node = (Vec3::new(3.0, 0.9, 0.0), NodeRef::new(island, 0));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0),
        vec![
            dodgy_2d::Obstacle::Open {
                vertices: vec![
                    dodgy_2d::Vec2::new(1.0, 1.0),
                    dodgy_2d::Vec2::new(2.0, 1.0),
                    dodgy_2d::Vec2::new(3.0, 1.0),
                    dodgy_2d::Vec2::new(4.0, 1.0),
                    dodgy_2d::Vec2::new(5.0, 1.0),
                ]
            },
            dodgy_2d::Obstacle::Open {
                vertices: vec![
                    dodgy_2d::Vec2::new(6.0, 2.0),
                    dodgy_2d::Vec2::new(6.0, 1.0),
                    dodgy_2d::Vec2::new(6.0, 0.0),
                    dodgy_2d::Vec2::new(5.0, 0.0),
                    dodgy_2d::Vec2::new(4.0, 0.0),
                    dodgy_2d::Vec2::new(3.0, 0.0),
                    dodgy_2d::Vec2::new(2.0, 0.0),
                    dodgy_2d::Vec2::new(1.0, 0.0),
                    dodgy_2d::Vec2::new(0.0, 0.0),
                    dodgy_2d::Vec2::new(0.0, 1.0),
                    dodgy_2d::Vec2::new(0.0, 2.0),
                ]
            }
        ]
    );
}

#[test]
fn creates_obstacles_across_boundary_link() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(2.0, 1.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(1.0, 2.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("Validation succeeds"),
    );

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let _island_1 = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::clone(&nav_mesh),
    ));
    let island_2 = world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 0.0, 0.0), 0.0),
        nav_mesh,
    ));

    world.update(&AgentOptions::from_agent_radius(1.0), 0.0);

    let agent_node = (Vec3::new(2.5, 1.5, 1.0), NodeRef::new(island_2, 0));
    assert_obstacles_match!(
        nav_mesh_borders_to_dodgy_obstacles(agent_node, world.nav(), world.islands(), 10.0,),
        vec![
            dodgy_2d::Obstacle::Open {
                vertices: vec![
                    dodgy_2d::Vec2::new(2.0, 1.0),
                    dodgy_2d::Vec2::new(1.0, 1.0),
                    dodgy_2d::Vec2::new(1.0, 2.0),
                    dodgy_2d::Vec2::new(2.0, 2.0),
                ]
            },
            dodgy_2d::Obstacle::Open {
                vertices: vec![
                    dodgy_2d::Vec2::new(2.0, 2.0),
                    dodgy_2d::Vec2::new(3.0, 2.0),
                    dodgy_2d::Vec2::new(3.0, 1.0),
                    dodgy_2d::Vec2::new(2.0, 1.0),
                ]
            }
        ]
    );
}

#[test]
fn applies_no_avoidance_for_far_agents() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(13.0, -1.0, 0.0),
            Vec3::new(13.0, 3.0, 0.0),
            Vec3::new(-1.0, 3.0, 0.0),
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .expect("Validation succeeded.");

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_1 = world.add_agent({
        let position = Vec3::new(1.0, 1.0, 0.0);
        let mut agent = Agent::<XYZ>::new(position, 0.01, 1.0, 1.0);
        agent.desired_velocity = Vec3::new(1.0, 0.0, 0.0);

        agent.sampled_position = Some((position, NodeRef::new(island, 0)));
        agent
    });
    let agent_2 = world.add_agent({
        let position = Vec3::new(11.0, 1.0, 0.0);
        let mut agent = Agent::<XYZ>::new(position, 0.01, 1.0, 1.0);
        agent.desired_velocity = Vec3::new(-1.0, 0.0, 0.0);
        agent.sampled_position = Some((agent.position, NodeRef::new(island, 0)));
        agent
    });
    let agent_3 = world.add_agent({
        let mut agent = Agent::<XYZ>::new(Vec3::new(5.0, 4.0, 0.0), 0.01, 1.0, 1.0);
        agent.desired_velocity = Vec3::new(0.0, 1.0, 0.0);
        // `agent_3` is not on a node.
        agent
    });

    let options = AgentOptions {
        neighbourhood: 5.0,
        ..AgentOptions::from_agent_radius(0.5)
    };
    world.update_avoidance(&options, 0.01);

    assert_eq!(
        world.agent(agent_1).desired_velocity(),
        Vec3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
        world.agent(agent_2).desired_velocity(),
        Vec3::new(-1.0, 0.0, 0.0)
    );
    assert_eq!(
        world.agent(agent_3).desired_velocity(),
        Vec3::new(0.0, 1.0, 0.0)
    );
}

#[test]
fn applies_avoidance_for_two_agents() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(13.0, -1.0, 0.0),
            Vec3::new(13.0, 3.0, 0.0),
            Vec3::new(-1.0, 3.0, 0.0),
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .expect("Validation succeeded.");

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_1 = world.add_agent({
        let position = Vec3::new(1.0, 1.0, 0.0);
        let mut agent =
            Agent::<XYZ>::new(position, 1.0, 1.0, 1.0).with_velocity(Vec3::new(1.0, 0.0, 0.0));
        agent.desired_velocity = Vec3::new(1.0, 0.0, 0.0);
        agent.sampled_position = Some((position, NodeRef::new(island, 0)));
        agent
    });
    let agent_2 = world.add_agent({
        let position = Vec3::new(11.0, 1.01, 0.0);
        let mut agent =
            Agent::<XYZ>::new(position, 1.0, 1.0, 1.0).with_velocity(Vec3::new(-1.0, 0.0, 0.0));
        agent.desired_velocity = Vec3::new(-1.0, 0.0, 0.0);
        agent.sampled_position = Some((position, NodeRef::new(island, 0)));
        agent
    });

    let options = AgentOptions {
        neighbourhood: 15.0,
        avoidance_time_horizon: 15.0,
        ..AgentOptions::from_agent_radius(0.5)
    };
    world.update_avoidance(&options, 0.01);

    // The agents each have a radius of 1, and they are separated by a distance
    // of 10 (they start at (1,0) and (11,0)). So in order to pass each other, one
    // agent must go to (6,1) and the other agent must go to (6,-1). That's a rise
    // over run of 1/5 or 0.2, which is our expected Z velocity. We derive the X
    // velocity by just making the length of the vector 1 (the agent's max speed).
    let agent_1_desired_velocity = world.agent(agent_1).desired_velocity();
    assert!(
        agent_1_desired_velocity.abs_diff_eq(Vec3::new(0.98, -0.2, 0.0), 0.05),
        "left={agent_1_desired_velocity}, right=Vec3(0.98, -0.2, 0.0)"
    );
    let agent_2_desired_velocity = world.agent(agent_2).desired_velocity();
    assert!(
        agent_2_desired_velocity.abs_diff_eq(Vec3::new(-0.98, 0.2, 0.0), 0.05),
        "left={agent_2_desired_velocity}, right=Vec3(-0.98, 0.2, 0.0)"
    );
}

#[test]
fn agent_avoids_character() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(13.0, -1.0, 0.0),
            Vec3::new(13.0, 3.0, 0.0),
            Vec3::new(-1.0, 3.0, 0.0),
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .expect("Validation succeeded.");

    let mut world = ArchipelagoWorld::<XYZ>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent = world.add_agent({
        let position = Vec3::new(1.0, 1.0, 0.0);
        let mut agent =
            Agent::<XYZ>::new(position, 1.0, 1.0, 1.0).with_velocity(Vec3::new(1.0, 0.0, 0.0));
        agent.desired_velocity = Vec3::new(1.0, 0.0, 0.0);
        agent.sampled_position = Some((position, NodeRef::new(island, 0)));
        agent
    });

    world.add_character(Character {
        position: Vec3::new(11.0, 1.01, 0.0),
        velocity: Vec3::new(-1.0, 0.0, 0.0),
        radius: 1.0,
        sampled_position: Some((
            Vec3::new(11.0, 1.01, 0.0),
            NodeRef::new(Entity::PLACEHOLDER, 0),
        )),
    });

    let options = AgentOptions {
        neighbourhood: 15.0,
        avoidance_time_horizon: 15.0,
        ..AgentOptions::from_agent_radius(0.5)
    };
    world.update_avoidance(&options, 0.01);

    // The agent+character each have a radius of 1, and they are separated by a
    // distance of 10 (they start at (1,0) and (11,0)). Only the agent is
    // managed by landmass, so it must go to (6,2), since the character will go to
    // (6,0). That's a rise over run of 2/5 or 0.4, which is our expected Z
    // velocity. We derive the X velocity by just making the length of the
    // vector 1 (the agent's max speed).
    let agent_desired_velocity = world.agent(agent).desired_velocity();
    assert!(
        agent_desired_velocity.abs_diff_eq(Vec3::new((1.0f32 - 0.4 * 0.4).sqrt(), -0.4, 0.0), 0.05),
        "left={agent_desired_velocity}, right=Vec3(0.9165..., -0.4, 0.0)"
    );
}

#[test]
fn agent_speeds_up_to_avoid_character() {
    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec2::new(-10.0, -10.0),
            Vec2::new(10.0, -10.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(-10.0, 10.0),
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .expect("Validation succeeded.");

    let mut world = ArchipelagoWorld::<XY>::new();
    let island = world.add_island(Island::new(
        Transform::new(Vec2::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent = world.add_agent({
        let position = Vec2::new(5.0, 0.0);
        let mut agent =
            Agent::<XY>::new(position, 0.5, 1.0, 2.0).with_velocity(Vec2::new(-1.0, 0.0));
        agent.desired_velocity = Vec2::new(1.0, 0.0);
        agent.sampled_position = Some((position.extend(0.0), NodeRef::new(island, 0)));
        agent
    });

    let options = AgentOptions {
        neighbourhood: 15.0,
        avoidance_time_horizon: 15.0,
        ..AgentOptions::from_agent_radius(0.5)
    };
    world.update_avoidance(&options, 0.01);

    // The agent sticks to its desired velocity.
    assert_eq!(world.agent(agent).desired_velocity(), Vec2::new(1.0, 0.0));

    world.add_character(Character::<XY> {
        // Just slightly closer to the agent so it prefers to "speed up".
        position: Vec2::new(0.0, 5.0),
        velocity: Vec2::new(0.0, -1.0),
        radius: 0.5,
        sampled_position: Some((
            Vec3::new(0.0, 5.0, 0.0),
            NodeRef::new(Entity::PLACEHOLDER, 0),
        )),
    });

    let options = AgentOptions {
        neighbourhood: 15.0,
        avoidance_time_horizon: 15.0,
        ..AgentOptions::from_agent_radius(0.5)
    };
    world.update_avoidance(&options, 0.01);

    let agent_desired_velocity = world.agent(agent).desired_velocity();
    // Check the agent has sped up to avoid the character.
    assert!(
        agent_desired_velocity.length() > 1.1,
        "actual={agent_desired_velocity} actual_length={} expected=greater than 1.0",
        agent_desired_velocity.length(),
    );
}

#[test]
fn reached_target_agent_has_different_avoidance() {
    let mut world = ArchipelagoWorld::<XY>::new();

    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec2::new(-10.0, -10.0),
                Vec2::new(10.0, -10.0),
                Vec2::new(10.0, 10.0),
                Vec2::new(-10.0, 10.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .unwrap(),
    );

    world.add_island(Island::new(Transform::default(), nav_mesh));

    let agent_1 =
        world.add_agent(Agent::new(Vec2::ZERO, 0.5, 1.0, 1.0).with_target(Vec2::new(0.0, 0.0)));

    let agent_2 = world.add_agent(
        Agent::new(Vec2::new(0.0, -3.0), 0.5, 1.0, 1.0)
            .with_velocity(Vec2::new(0.0, 1.0))
            .with_target(Vec2::new(0.0, 3.0)),
    );

    let mut options = AgentOptions::from_agent_radius(0.5);

    options.avoidance_time_horizon = 100.0;
    options.obstacle_avoidance_time_horizon = 0.1;
    // Use a responsibility of one third, so that agent_2 has 3/4 responsibility
    // and agent_1 has 1/4 responsibility.
    options.reached_destination_avoidance_responsibility = 1.0 / 3.0;

    // 35 was chosen by just running until the second agent crosses y=0 (roughly).
    // This is probably easy to break, but I couldn't think of another way to get
    // the right value here...

    for _ in 0..35 {
        world.update(&options, 0.1);

        // Update the velocities to match the desired velocities.
        {
            let mut agent = world.agent_mut(agent_1);
            agent.current_velocity = agent.desired_velocity();
            let new = agent.position + agent.current_velocity * 0.1;
            agent.position = new;
            dbg!(agent.position);
        }

        {
            let mut agent = world.agent_mut(agent_2);
            agent.current_velocity = agent.desired_velocity();
            let new = agent.position + agent.current_velocity * 0.1;
            agent.position = new;
            dbg!(agent.position);
        }
    }

    let agent_1 = world.agent(agent_1);
    let agent_2 = world.agent(agent_2);

    // Since agent_1 takes 1/4 responsibility, it moves away by 0.25.
    assert!(
        (agent_1.position.x - 0.25) < 0.01,
        "left={}, right={}",
        agent_1.position.x,
        0.25
    );
    // Since agent_2 takes 3/4 responsibility, it moves away by 0.75.
    assert!(
        (agent_2.position.x - 0.75) < 0.01,
        "left={}, right={}",
        agent_2.position.x,
        0.75
    );
}

#[test]
fn switching_nav_mesh_to_fewer_vertices_does_not_result_in_panic() {
    // This isn't really a test of the avoidance, but rather of repeatedly
    // modifying the nav data. However, the avoidance is what triggers the panic,
    // so best to test it here.
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

    let redundant_mesh = NavMeshBuilder {
        vertices: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, 0.6),
            Vec2::new(0.0, 0.2),
        ],
        polygons: vec![vec![0, 1, 2, 3, 4, 5]],
        polygon_type_indices: vec![0],
    }
    .validate()
    .unwrap();
    let redundant_mesh = Arc::new(redundant_mesh);

    let simplified_mesh = NavMeshBuilder {
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
    .unwrap();
    let simplified_mesh = Arc::new(simplified_mesh);

    let island_1 = world.add_island(Island::new(
        Transform::default(),
        Arc::clone(&redundant_mesh),
    ));
    world.add_island(Island::new(
        // This island is shifted over but is slightly misaligned to generate new vertices.
        Transform::new(Vec2::new(1.0, 0.25), 0.0),
        redundant_mesh,
    ));

    let agent = world
        .add_agent(Agent::new(Vec2::new(0.5, 0.5), 0.5, 1.0, 1.0).with_target(Vec2::new(1.5, 0.5)));

    world.update(&options, 0.01);

    // This doesn't really matter for the test, but ensures that pathing +
    // avoidance is running.
    assert_eq!(world.agent(agent).desired_velocity(), Vec2::new(1.0, 0.0));

    world
        .island_mut(island_1)
        .unwrap()
        .set_nav_mesh(simplified_mesh);

    world.update(&options, 0.01);
}
