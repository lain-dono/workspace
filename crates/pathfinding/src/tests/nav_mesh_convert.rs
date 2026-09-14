use crate::coords::{CoordinateSystem, ThreeD, TwoD};
use crate::landmass::NavMeshBuilder;
use crate::nav_mesh::ConvertMeshError;
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::utils::default;

#[test]
fn error_on_wrong_topology() {
    let mesh = Mesh::new(PrimitiveTopology::LineStrip, default());
    match NavMeshBuilder::<ThreeD>::from_bevy_mesh(&mesh, ThreeD::from_mesh_vertex) {
        Ok(_) => panic!("Conversion succeeded."),
        Err(err) => assert_eq!(err, ConvertMeshError::InvalidTopology),
    }
}

#[test]
fn u16_indices() {
    let vertices = vec![
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [2.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [3.0, 2.0, 1.0],
        [3.0, 3.0, 1.0],
    ];

    let indices = vec![0, 1, 2, 2, 3, 0, 3, 2, 4, 3, 4, 5, 4, 2, 6, 4, 6, 7];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::mesh::Indices::U16(indices));

    let nav_mesh = NavMeshBuilder::<TwoD>::from_bevy_mesh(&mesh, TwoD::from_mesh_vertex).unwrap();

    let result_positions = [
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 2.0),
        Vec2::new(2.0, 3.0),
        Vec2::new(1.0, 3.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(3.0, 3.0),
    ];

    let result_indices = vec![
        vec![0, 1, 2],
        vec![2, 3, 0],
        vec![3, 2, 4],
        vec![3, 4, 5],
        vec![4, 2, 6],
        vec![4, 6, 7],
    ];

    assert_eq!(nav_mesh.vertices, result_positions);
    assert_eq!(nav_mesh.polygons, result_indices);
}

#[test]
fn u32_indices() {
    let vertices = vec![
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [2.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
        [2.0, 1.0, 3.0],
        [1.0, 1.0, 3.0],
        [3.0, 1.0, 2.0],
        [3.0, 1.0, 3.0],
    ];

    let indices = vec![0, 1, 2, 2, 3, 0, 3, 2, 4, 3, 4, 5, 4, 2, 6, 4, 6, 7];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let nav_mesh =
        NavMeshBuilder::<ThreeD>::from_bevy_mesh(&mesh, ThreeD::from_mesh_vertex).unwrap();

    let vertices = [
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(2.0, 1.0, 1.0),
        Vec3::new(2.0, 1.0, 2.0),
        Vec3::new(1.0, 1.0, 2.0),
        Vec3::new(2.0, 1.0, 3.0),
        Vec3::new(1.0, 1.0, 3.0),
        Vec3::new(3.0, 1.0, 2.0),
        Vec3::new(3.0, 1.0, 3.0),
    ];

    let indices = vec![
        vec![0, 1, 2],
        vec![2, 3, 0],
        vec![3, 2, 4],
        vec![3, 4, 5],
        vec![4, 2, 6],
        vec![4, 6, 7],
    ];

    assert_eq!(nav_mesh.vertices, vertices);
    assert_eq!(nav_mesh.polygons, indices);
}
