//use crate::loading::AssetCache;
use crate::raycast::PlaneRaycast;
use crate::state::InGame;
use bevy::pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(InGame), setup_decal).add_systems(
        Update,
        update_transform.after(crate::raycast::plane_raycast),
    );
}

fn setup_decal(
    mut commands: Commands,
    mut decal_standard_materials: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("Decal"),
        Transform::from_scale(Vec3::splat(4.0)),
        ForwardDecal,
        MeshMaterial3d(decal_standard_materials.add(ForwardDecalMaterial {
            base: StandardMaterial {
                base_color_texture: Some(asset_server.load("uv-check.png")),
                ..default()
            },
            extension: ForwardDecalMaterialExt {
                depth_fade_factor: 1.0,
            },
        })),
    ));
}

fn update_transform(
    raycast: Query<&PlaneRaycast>,
    mut marker: Query<&mut Transform, With<ForwardDecal>>,
) {
    let Ok(mut transform) = marker.single_mut() else {
        return;
    };

    if let Some(point) = raycast.single().ok().and_then(|r| r.result) {
        transform.translation = point;
    }
}
