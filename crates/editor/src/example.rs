use bevy::prelude::*;

pub const BOX_SIZE: f32 = 2.0;
pub const BOX_THICKNESS: f32 = 0.15;
pub const BOX_OFFSET: f32 = (BOX_SIZE + BOX_THICKNESS) / 2.0;

pub fn setup_entity_tree(mut commands: Commands) {
    commands.spawn_empty().with_children(|builder| {
        builder.spawn_empty().with_children(|builder| {
            builder.spawn_empty().with_children(|builder| {
                builder.spawn_empty();
            });
        });
    });
}

pub fn setup_pleasure_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });

    // top light
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(0.4))),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, BOX_SIZE + 0.5 * BOX_THICKNESS, 0.0),
            )),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::WHITE * 100.0,
                ..default()
            }),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::WHITE,
                    intensity: 25.0,
                    ..default()
                },
                transform: Transform::from_translation((BOX_THICKNESS + 0.05) * Vec3::Y),
                ..default()
            });
        });

    let mesh_left_right = shape::Box::new(BOX_SIZE, BOX_THICKNESS, BOX_SIZE);
    let mesh_left_right = meshes.add(Mesh::from(mesh_left_right));

    let mesh_top_bottom = shape::Box::new(BOX_SIZE + 2.0 * BOX_THICKNESS, BOX_THICKNESS, BOX_SIZE);
    let mesh_top_bottom = meshes.add(Mesh::from(mesh_top_bottom));

    let mesh_back = meshes.add(Mesh::from(shape::Box::new(
        BOX_SIZE + 2.0 * BOX_THICKNESS,
        BOX_THICKNESS,
        BOX_SIZE + 2.0 * BOX_THICKNESS,
    )));

    // left - red
    let mut transform = Transform::from_xyz(-BOX_OFFSET, BOX_OFFSET, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: mesh_left_right.clone(),
        transform,
        material: materials.add(StandardMaterial::from(Color::rgb(0.63, 0.065, 0.05))),
        ..default()
    });

    // right - green
    let mut transform = Transform::from_xyz(BOX_OFFSET, BOX_OFFSET, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: mesh_left_right.clone(),
        transform,
        material: materials.add(StandardMaterial::from(Color::rgb(0.14, 0.45, 0.091))),
        ..default()
    });

    // bottom - white
    commands.spawn(PbrBundle {
        mesh: mesh_top_bottom.clone(),
        material: materials.add(StandardMaterial::from(Color::rgb(0.725, 0.71, 0.68))),
        ..default()
    });

    // top - white
    commands.spawn(PbrBundle {
        mesh: mesh_top_bottom.clone(),
        transform: Transform::from_xyz(0.0, 2.0 * BOX_OFFSET, 0.0),
        material: materials.add(StandardMaterial::from(Color::rgb(0.725, 0.71, 0.68))),
        ..default()
    });

    // back - white
    let mut transform = Transform::from_xyz(0.0, BOX_OFFSET, -BOX_OFFSET);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: mesh_back,
        transform,
        material: materials.add(StandardMaterial::from(Color::rgb(0.725, 0.71, 0.68))),
        ..default()
    });
}

/*
fn example_scene() -> World {
    use crate::scene::component::{ProxyHandle, ProxyMeta, ProxyPointLight, ProxyTransform};
    use crate::ui::icon;

    fn new<'a>(
        builder: &'a mut WorldChildBuilder,
        prefix: &str,
        counter: usize,
    ) -> EntityWorldMut<'a> {
        let icon = match counter % 14 {
            0 => icon::MESH_CONE,
            1 => icon::MESH_PLANE,
            2 => icon::MESH_CYLINDER,
            3 => icon::MESH_ICOSPHERE,
            4 => icon::MESH_CAPSULE,
            5 => icon::MESH_UVSPHERE,
            6 => icon::MESH_CIRCLE,
            7 => icon::MESH_MONKEY,
            8 => icon::MESH_TORUS,
            9 => icon::MESH_CUBE,

            10 => icon::OUTLINER_OB_CAMERA,
            11 => icon::OUTLINER_OB_EMPTY,
            12 => icon::OUTLINER_OB_LIGHT,
            13 => icon::OUTLINER_OB_SPEAKER,

            _ => unreachable!(),
        };

        builder.spawn((
            ProxyMeta::new(icon, format!("{} #{}", prefix, counter)),
            ProxyHandle::<Mesh>::new("models/BoxTextured.glb#Mesh0/Primitive0"),
            ProxyHandle::<StandardMaterial>::new("models/BoxTextured.glb#Material0"),
        ))
    }

    let mut world = World::new();

    world.spawn((
        ProxyMeta::new(icon::LIGHT_POINT, "Point Light"),
        ProxyTransform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            ..default()
        },
        ProxyPointLight::default(),
    ));

    let mut counter = 0;

    for _ in 0..2 {
        let icon = icon::MESH_CUBE;
        world
            .spawn((
                ProxyMeta::new(icon, format!("Root #{}", counter)),
                ProxyTransform {
                    translation: Vec3::new(
                        (counter % 2) as f32,
                        (counter % 3) as f32,
                        (counter % 4) as f32,
                    ),
                    ..default()
                },
                ProxyHandle::<Mesh>::new("models/BoxTextured.glb#Mesh0/Primitive0"),
                ProxyHandle::<StandardMaterial>::new("models/BoxTextured.glb#Material0"),
            ))
            .with_children(|builder| {
                for _ in 0..2 {
                    counter += 1;
                    new(builder, "Child", counter)
                        .insert(ProxyTransform {
                            translation: Vec3::new(
                                (counter % 2) as f32,
                                (counter % 3) as f32,
                                (counter % 4) as f32,
                            ),
                            ..default()
                        })
                        .with_children(|builder| {
                            for _ in 0..2 {
                                counter += 1;
                                new(builder, "Sub Child", counter).insert(ProxyTransform {
                                    translation: Vec3::new(
                                        (counter % 2) as f32,
                                        (counter % 3) as f32,
                                        (counter % 4) as f32,
                                    ),
                                    ..default()
                                });
                            }
                        });
                }
            });
        counter += 1;
    }

    world
}

*/
