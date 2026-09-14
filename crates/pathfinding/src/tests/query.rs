use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::landmass::{
    AgentOptions, FindPathError, FromAgentRadius, Island, NavMeshBuilder, SamplePointError,
    coords::XY,
};
use bevy::{math::Vec2, platform::collections::HashMap};
use std::sync::Arc;

#[test]
#[ignore]
fn error_on_dirty_nav_mesh() {
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

    world.add_island(Island::new(Transform::default(), nav_mesh));
    assert_eq!(
        world
            .query_sample_point(Vec2::new(0.5, 0.5), 1.0)
            .map(|p| p.point()),
        Err(SamplePointError::NavDataDirty)
    );
}

#[test]
fn error_on_out_of_range() {
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

    world.add_island(Island::new(Transform::default(), nav_mesh));

    world.update(&options, 1.0);

    assert_eq!(
        world
            .query_sample_point(Vec2::new(-0.5, 0.5), 0.1)
            .map(|p| p.point()),
        Err(SamplePointError::OutOfRange)
    );
}

#[test]
fn samples_point_on_nav_mesh_or_near_nav_mesh() {
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
    let island_id = world.add_island(Island::new(Transform::new(offset, 0.0), nav_mesh));

    world.update(&options, 1.0);

    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(-0.5, 0.5), 0.6)
            .map(|p| (p.island(), p.point())),
        Ok((island_id, offset + Vec2::new(0.0, 0.5)))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(0.5, 0.5), 0.6)
            .map(|p| (p.island(), p.point())),
        Ok((island_id, offset + Vec2::new(0.5, 0.5)))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(1.2, 1.2), 0.6)
            .map(|p| (p.island(), p.point())),
        Ok((island_id, offset + Vec2::new(1.0, 1.0)))
    );
}

#[test]
fn samples_node_types() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

    let node_type_1 = world.add_node_type(1.0);
    let node_type_2 = world.add_node_type(2.0);
    let node_type_3 = world.add_node_type(3.0);

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
                Vec2::new(4.0, 0.0),
                Vec2::new(4.0, 1.0),
            ],
            polygons: vec![
                vec![0, 1, 2, 3],
                vec![2, 1, 4, 5],
                vec![5, 4, 6, 7],
                vec![7, 6, 8, 9],
            ],
            polygon_type_indices: vec![0, 1, 2, 3],
        }
        .validate()
        .expect("nav mesh is valid"),
    );

    let offset = Vec2::new(10.0, 10.0);
    world.add_island(
        Island::new(Transform::new(offset, 0.0), nav_mesh).with_node_types(HashMap::from([
            (1, node_type_1),
            (2, node_type_2),
            (3, node_type_3),
        ])),
    );

    world.update(&options, 1.0);

    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(0.5, 0.5), 0.1)
            .map(|p| p.node_type()),
        Ok(None)
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(1.5, 0.5), 0.1)
            .map(|p| p.node_type()),
        Ok(Some(node_type_1))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(2.5, 0.5), 0.1)
            .map(|p| p.node_type()),
        Ok(Some(node_type_2))
    );
    assert_eq!(
        world
            .query_sample_point(offset + Vec2::new(3.5, 0.5), 0.1)
            .map(|p| p.node_type()),
        Ok(Some(node_type_3))
    );
}

#[test]
fn no_path() {
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
        Transform::new(offset + Vec2::new(2.0, 0.0), 0.0),
        nav_mesh,
    ));

    world.update(&options, 1.0);

    let start_point = world
        .query_sample_point(offset + Vec2::new(0.5, 0.5), 1e-5)
        .expect("point is on nav mesh.");
    let end_point = world
        .query_sample_point(offset + Vec2::new(2.5, 0.5), 1e-5)
        .expect("point is on nav mesh.");
    assert_eq!(
        world.query_find_path(&start_point, &end_point, &HashMap::new()),
        Err(FindPathError::NoPathFound)
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
fn finds_path_with_override_node_types() {
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XY>::new();

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

    let node_type = world.add_node_type(1.0);

    world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(1, node_type)])),
    );

    world.update(&options, 1.0);

    let start_point = world.query_sample_point(Vec2::new(0.5, 0.5), 0.1).unwrap();
    let end_point = world.query_sample_point(Vec2::new(0.5, 11.5), 0.1).unwrap();

    let path = world
        .query_find_path(
            &start_point,
            &end_point,
            &HashMap::from([(node_type, 10.0)]),
        )
        .expect("Path found");

    assert_eq!(
        path,
        [
            Vec2::new(0.5, 0.5),
            Vec2::new(2.0, 1.0),
            Vec2::new(2.0, 11.0),
            Vec2::new(0.5, 11.5)
        ]
    );
}

#[test]
fn find_path_returns_error_on_invalid_node_cost() {
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

    let node_type = world.add_node_type(1.0);

    world.add_island(
        Island::new(Transform::default(), nav_mesh)
            .with_node_types(HashMap::from([(0, node_type)])),
    );

    world.update(&options, 1.0);

    let start_point = world
        .query_sample_point(Vec2::new(0.25, 0.25), 0.1)
        .unwrap();
    let end_point = world
        .query_sample_point(Vec2::new(0.25, 0.25), 0.1)
        .unwrap();

    assert_eq!(
        world.query_find_path(&start_point, &end_point, &HashMap::from([(node_type, 0.0)]),),
        Err(FindPathError::NonPositiveNodeTypeCost(node_type, 0.0))
    );
    assert_eq!(
        world.query_find_path(
            &start_point,
            &end_point,
            &HashMap::from([(node_type, -0.5)]),
        ),
        Err(FindPathError::NonPositiveNodeTypeCost(node_type, -0.5))
    );
}
