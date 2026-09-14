use super::ArchipelagoWorld;
use crate::agent::AgentState;
use crate::bvh::Transform;
use crate::landmass::{
    Agent, AgentOptions, Character, FromAgentRadius, Island, NavMeshBuilder, PointSampleDistance3d,
    coords::{XY, XYZ},
};
use crate::update::PathingResult;
use bevy::ecs::observer::On;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::ResMut;
use bevy::math::{Vec2, Vec3};
use bevy::platform::collections::HashMap;
use bevy::utils::default;
use std::sync::Arc;

#[test]
fn add_and_remove_agents() {
    let mut world = ArchipelagoWorld::<XYZ>::new();

    let agent_1 = world.add_agent(Agent::new(Vec3::ZERO, 1.0, 0.0, 0.0));
    let agent_2 = world.add_agent(Agent::new(Vec3::ZERO, 2.0, 0.0, 0.0));
    let agent_3 = world.add_agent(Agent::new(Vec3::ZERO, 3.0, 0.0, 0.0));

    assert_eq!(world.agent_ids(), vec![agent_1, agent_2, agent_3]);
    assert_eq!(
        [
            world.agent(agent_1).radius,
            world.agent(agent_2).radius,
            world.agent(agent_3).radius,
        ],
        [1.0, 2.0, 3.0],
    );

    world.remove_agent(agent_2);

    assert_eq!(world.agent_ids(), vec![agent_1, agent_3]);
    assert_eq!(
        [world.agent(agent_1).radius, world.agent(agent_3).radius],
        [1.0, 3.0],
    );

    world.remove_agent(agent_3);

    assert_eq!(world.agent_ids(), vec![agent_1]);
    assert_eq!([world.agent(agent_1).radius], [1.0]);

    world.remove_agent(agent_1);

    assert_eq!(world.agent_ids(), []);
}

#[test]
fn add_and_remove_characters() {
    let mut world = ArchipelagoWorld::<XYZ>::new();

    let character_1 = world.add_character(Character {
        radius: 1.0,
        ..default()
    });

    let character_2 = world.add_character(Character {
        radius: 2.0,
        ..default()
    });

    let character_3 = world.add_character(Character {
        radius: 3.0,
        ..default()
    });

    assert_eq!(
        world.character_ids(),
        vec![character_1, character_2, character_3],
    );
    assert_eq!(
        [
            world.character(character_1).radius,
            world.character(character_2).radius,
            world.character(character_3).radius,
        ],
        [1.0, 2.0, 3.0],
    );

    world.remove_character(character_2);

    assert_eq!(world.character_ids(), vec![character_1, character_3],);
    assert_eq!(
        [
            world.character(character_1).radius,
            world.character(character_3).radius,
        ],
        [1.0, 3.0],
    );

    world.remove_character(character_3);

    assert_eq!(world.character_ids(), vec![character_1],);
    assert_eq!([world.character(character_1).radius], [1.0]);

    world.remove_character(character_1);

    assert_eq!(world.character_ids(), []);
}

#[test]
fn computes_and_follows_path() {
    #[derive(Resource, Default, Clone)]
    struct PathingResults(Vec<PathingResult>);

    let mut options = AgentOptions {
        point_sample_distance: PointSampleDistance3d {
            horizontal_distance: 0.1,
            distance_above: 0.1,
            distance_below: 0.1,
            vertical_preference_ratio: 1.0,
        },
        ..AgentOptions::from_agent_radius(0.5)
    };

    let mut world = ArchipelagoWorld::<XYZ>::new();

    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 1.0, 1.0),
            Vec3::new(3.0, 1.0, 1.0),
            Vec3::new(4.0, 1.0, 1.0),
            Vec3::new(4.0, 2.0, 1.0),
            Vec3::new(4.0, 3.0, 1.0),
            Vec3::new(4.0, 4.0, 1.0),
            Vec3::new(3.0, 4.0, 1.0),
            Vec3::new(3.0, 3.0, 1.0),
            Vec3::new(3.0, 2.0, 1.0),
            Vec3::new(2.0, 2.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
        ],
        polygons: vec![
            vec![0, 1, 10, 11],
            vec![1, 2, 9, 10],
            vec![2, 3, 4, 9],
            vec![4, 5, 8, 9],
            vec![5, 6, 7, 8],
        ],
        polygon_type_indices: vec![0, 0, 0, 0, 0],
    }
    .validate()
    .expect("is valid");

    world.add_island(Island::new(
        Transform::new(Vec3::ZERO, 0.0),
        Arc::new(nav_mesh),
    ));

    options.neighbourhood = 0.0;
    options.obstacle_avoidance_time_horizon = 0.01;

    let agent_1 = world.add_agent(Agent::new(Vec3::new(1.5, 1.5, 1.09), 0.5, 2.0, 2.0));
    let agent_2 = world.add_agent(Agent::new(Vec3::new(3.5, 3.5, 0.95), 0.5, 2.0, 2.0));
    let agent_off_mesh = world.add_agent(Agent::new(Vec3::new(1.5, 2.5, 1.0), 0.5, 2.0, 2.0));
    let agent_too_high_above_mesh =
        world.add_agent(Agent::new(Vec3::new(1.5, 1.5, 1.11), 0.5, 2.0, 2.0));

    world.agent_mut(agent_1).current_target = Some(Vec3::new(3.5, 3.5, 0.95));
    world.agent_mut(agent_off_mesh).current_target = Some(Vec3::new(3.5, 3.5, 0.95));

    world.agent_mut(agent_too_high_above_mesh).current_target = Some(Vec3::new(3.5, 3.5, 0.95));
    world.agent_mut(agent_2).current_target = Some(Vec3::new(1.5, 1.5, 1.09));

    // Nothing has happened yet.
    assert_eq!(world.agent(agent_1).state(), AgentState::Idle);
    assert_eq!(world.agent(agent_2).state(), AgentState::Idle);
    assert_eq!(world.agent(agent_off_mesh).state(), AgentState::Idle);
    assert_eq!(
        world.agent(agent_too_high_above_mesh).state(),
        AgentState::Idle
    );

    assert_eq!(world.agent(agent_1).desired_velocity(), Vec3::ZERO);
    assert_eq!(world.agent(agent_2).desired_velocity(), Vec3::ZERO);
    assert_eq!(world.agent(agent_off_mesh).desired_velocity(), Vec3::ZERO);
    assert_eq!(
        world.agent(agent_too_high_above_mesh).desired_velocity(),
        Vec3::ZERO
    );

    world.world.insert_resource(PathingResults::default());
    world.world.flush();
    world.world.add_observer(
        |trigger: On<PathingResult>, mut results: ResMut<PathingResults>| {
            dbg!();
            results.0.push(*trigger.event());
        },
    );
    world.world.flush();
    world.world.clear_trackers();

    world.update(&options, 0.01);

    // These agents found a path and started following it.
    assert_eq!(world.agent(agent_1).state(), AgentState::Moving);
    assert_eq!(world.agent(agent_2).state(), AgentState::Moving);
    assert!(
        world
            .agent(agent_1)
            .desired_velocity()
            .abs_diff_eq(Vec3::new(1.5, 0.5, 0.0).normalize() * 2.0, 1e-2)
    );
    assert!(
        world
            .agent(agent_2)
            .desired_velocity()
            .abs_diff_eq(Vec3::new(-0.5, -1.5, 0.0).normalize() * 2.0, 1e-2)
    );
    // These agents are not on the nav mesh, so they don't do anything.
    assert_eq!(
        world.agent(agent_off_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );
    assert_eq!(
        world.agent(agent_too_high_above_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );
    assert_eq!(world.agent(agent_off_mesh).desired_velocity(), Vec3::ZERO);
    assert_eq!(
        world.agent(agent_too_high_above_mesh).desired_velocity(),
        Vec3::ZERO
    );

    {
        let PathingResults(results) = world.world.resource::<PathingResults>().clone();

        assert_eq!(results.len(), 2);
        let path_result_1 = results[0];
        let path_result_2 = results[1];
        assert!(path_result_1.success);
        assert!(path_result_2.success);
        assert!(path_result_1.explored_nodes > 0);
        assert!(path_result_2.explored_nodes > 0);
    }

    // Move agent_1 forward.
    world.agent_mut(agent_1).position = Vec3::new(2.5, 1.5, 1.0);

    world.update(&options, 0.01);

    assert!(
        world
            .agent(agent_1)
            .desired_velocity()
            .abs_diff_eq(Vec3::new(0.5, 0.5, 0.0).normalize() * 2.0, 1e-7)
    );
    // These agents don't change.
    assert!(
        world
            .agent(agent_2)
            .desired_velocity()
            .abs_diff_eq(Vec3::new(-0.5, -1.5, 0.0).normalize() * 2.0, 1e-2)
    );
    assert_eq!(world.agent(agent_off_mesh).desired_velocity(), Vec3::ZERO);
    assert_eq!(
        world.agent(agent_too_high_above_mesh).desired_velocity(),
        Vec3::ZERO
    );
    assert_eq!(world.agent(agent_1).state(), AgentState::Moving);
    assert_eq!(world.agent(agent_2).state(), AgentState::Moving);
    assert_eq!(
        world.agent(agent_off_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );
    assert_eq!(
        world.agent(agent_too_high_above_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );

    // Move agent_1 close enough to destination and agent_2 forward.
    world.agent_mut(agent_1).position = Vec3::new(3.4, 3.4, 1.0);
    world.agent_mut(agent_2).position = Vec3::new(3.5, 2.5, 1.0);
    world.update(&options, 0.01);

    assert_eq!(world.agent(agent_1).state(), AgentState::ReachedTarget);
    assert_eq!(world.agent(agent_2).state(), AgentState::Moving);
    assert_eq!(world.agent(agent_1).desired_velocity(), Vec3::ZERO);
    assert!(
        world
            .agent(agent_2)
            .desired_velocity()
            .abs_diff_eq(Vec3::new(-0.5, -0.5, 0.0).normalize() * 2.0, 1e-2)
    );
    // These agents don't change.
    assert_eq!(world.agent(agent_off_mesh).desired_velocity(), Vec3::ZERO);
    assert_eq!(
        world.agent(agent_too_high_above_mesh).desired_velocity(),
        Vec3::ZERO
    );
    assert_eq!(
        world.agent(agent_off_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );
    assert_eq!(
        world.agent(agent_too_high_above_mesh).state(),
        AgentState::AgentNotOnNavMesh
    );
}

#[test]
fn agent_speeds_up_to_avoid_character() {
    let mut options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

    options.avoidance_time_horizon = 100.0;
    options.neighbourhood = 10.0;

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
        .expect("Validation succeeded."),
    );

    world.add_island(Island::new(Transform::default(), nav_mesh));

    let agent_id = world.add_agent(
        Agent::new(Vec2::new(5.0, 0.0), 0.5, 1.0, 2.0)
            .with_velocity(Vec2::new(-1.0, 0.0))
            .with_target(Vec2::new(-5.0, 0.0)),
    );

    world.update(&options, 0.01);

    // The agent will move at its desired speed normally.
    assert_eq!(
        world.agent(agent_id).desired_velocity(),
        Vec2::new(-1.0, 0.0)
    );

    world.add_character(Character {
        position: Vec2::new(0.0, 5.0),
        velocity: Vec2::new(0.0, -1.0),
        radius: 0.5,
        sampled_position: None,
    });

    world.update(&options, 0.01);

    let agent_desired_velocity = world.agent(agent_id).desired_velocity();
    // The agent speeds up to avoid the character.
    assert!(
        agent_desired_velocity.length() > 1.1,
        "actual={agent_desired_velocity} actual_length={} expected=greater than 1.0",
        agent_desired_velocity.length(),
    );
}

#[test]
fn add_and_remove_islands() {
    let mut world = ArchipelagoWorld::<XY>::new();
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![],
            polygons: vec![],
            polygon_type_indices: vec![],
        }
        .validate()
        .unwrap(),
    );

    let island_id_1 = world.add_island(Island::new(Transform::default(), nav_mesh.clone()));
    let island_id_2 = world.add_island(Island::new(Transform::default(), nav_mesh.clone()));
    let island_id_3 = world.add_island(Island::new(Transform::default(), nav_mesh.clone()));

    assert_eq!(world.island_ids(), [island_id_1, island_id_2, island_id_3]);

    world.remove_island(island_id_2);

    assert_eq!(world.island_ids(), [island_id_1, island_id_3]);
}

#[test]
fn samples_point() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

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
        .expect("nav mesh is valid"),
    );

    let offset = Vec2::new(10.0, 10.0);
    world.add_island(Island::new(Transform::new(offset, 0.0), nav_mesh));

    world.update(&options, 1.0);

    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(-0.5, 0.5), 0.6)
            .map(|p| p.point()),
        Ok(offset + Vec2::new(0.0, 0.5))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(0.5, 0.5), 0.6)
            .map(|p| p.point()),
        Ok(offset + Vec2::new(0.5, 0.5))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(1.2, 1.2), 0.6)
            .map(|p| p.point()),
        Ok(offset + Vec2::new(1.0, 1.0))
    );
}

#[test]
fn finds_path() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

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
        .expect("nav mesh is valid"),
    );

    let offset = Vec2::new(10.0, 10.0);
    world.add_island(Island::new(Transform::new(offset, 0.0), nav_mesh.clone()));
    world.add_island(Island::new(
        Transform::new(offset + Vec2::new(1.0, 0.0), 0.0),
        nav_mesh.clone(),
    ));
    world.add_island(Island::new(
        Transform::new(offset + Vec2::new(2.0, 0.5), 0.0),
        nav_mesh,
    ));

    world.update(&options, 1.0);

    let start_point = world
        .query_sample_point(offset + Vec2::new(0.5, 0.5), 1e-5)
        .expect("point is on nav mesh.");
    let end_point = world
        .query_sample_point(offset + Vec2::new(2.5, 1.25), 1e-5)
        .expect("point is on nav mesh.");
    assert_eq!(
        world.query_find_path(&start_point, &end_point, &HashMap::new()),
        Ok(vec![
            offset + Vec2::new(0.5, 0.5),
            offset + Vec2::new(2.0, 1.0),
            offset + Vec2::new(2.5, 1.25)
        ])
    );
}

#[test]
fn agent_overrides_node_costs() {
    let nav_mesh = Arc::new(
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
                //
                Vec2::new(2.0, 11.0),
                Vec2::new(3.0, 11.0),
                //
                Vec2::new(2.0, 12.0),
                Vec2::new(3.0, 12.0),
                //
                Vec2::new(1.0, 12.0),
                Vec2::new(1.0, 11.0),
                //
                Vec2::new(0.0, 12.0),
                Vec2::new(0.0, 11.0),
            ],
            polygons: vec![
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                //
                vec![5, 7, 9, 8],
                vec![8, 9, 11, 10],
                //
                vec![8, 10, 12, 13],
                vec![13, 12, 14, 15],
                //
                vec![3, 2, 13, 15],
            ],
            polygon_type_indices: vec![0, 0, 0, 0, 0, 0, 0, 1],
        }
        .validate()
        .expect("nav mesh is valid"),
    );

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

    let node_type = world.add_node_type(1.0);

    world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, node_type)])),
    );

    let agent_id = world.add_agent({
        let mut agent =
            Agent::new(Vec2::new(0.5, 0.5), 0.5, 1.0, 1.0).with_target(Vec2::new(0.5, 11.5));
        assert!(agent.override_node_type_cost(node_type, 10.0));
        agent
    });

    world.update(&options, 1.0);

    // The agent **could** go directly up, but due to its overridden node cost, it
    // is better to take the detour to the right.
    assert_eq!(
        world.agent(agent_id).desired_velocity(),
        Vec2::new(1.5, 0.5).normalize(),
    );
}
