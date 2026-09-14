pub mod decal;

use self::decal::{Decal, DecalBundle, DecalMaterial};
use crate::loading::AssetCache;
use crate::raycast::PlaneRaycast;
use crate::state::InGame;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;

pub fn plugin(app: &mut App) {
    app.add_plugins(self::decal::DecalPlugin)
        .add_systems(OnEnter(InGame), crate::render::init_scene)
        .add_systems(
            Update,
            update_transform.after(crate::raycast::plane_raycast),
        );
}

pub fn init_scene(
    mut cache: ResMut<AssetCache>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,

    mut dmat: ResMut<Assets<DecalMaterial>>,
) {
    let cube = meshes.add(Cuboid::from_length(1.0));

    commands.spawn((
        StateScoped(InGame),
        Mesh3d(cube.clone()),
        MeshMaterial3d(cache.get_material(
            &mut materials,
            Color::Srgba(bevy::color::palettes::basic::GREEN),
        )),
        Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_y(0.2),
            ..default()
        },
    ));

    commands.spawn((
        StateScoped(InGame),
        Mesh3d(cube.clone()),
        MeshMaterial3d(cache.get_material(
            &mut materials,
            Color::Srgba(bevy::color::palettes::basic::YELLOW),
        )),
        Transform {
            translation: Vec3::new(-2.0, 0.5, 1.0),
            rotation: Quat::from_rotation_y(0.7),
            ..default()
        },
    ));

    let texture = asset_server.load("uv-check.png");

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::all(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 14]);

    commands.spawn((
        StateScoped(InGame),
        DecalBundle {
            mesh: Mesh3d(meshes.add(mesh)),
            material: MeshMaterial3d(dmat.add(DecalMaterial { texture })),
            transform: Transform {
                translation: Vec3::new(-1.0, 0.5, 2.0),
                rotation: Quat::from_rotation_y(0.5),
                scale: Vec3::new(5.0, 5.0, 5.0),
            },
            ..default()
        },
    ));
}

fn update_transform(raycast: Query<&PlaneRaycast>, mut marker: Query<&mut Transform, With<Decal>>) {
    let Ok(mut transform) = marker.single_mut() else {
        return;
    };

    if let Some(point) = raycast.single().ok().and_then(|r| r.result) {
        transform.translation = point;
    }
}
