use crate::utils::SpawnEntity;
use bevy::{
    color::palettes::css, input::common_conditions::input_just_pressed, mesh::Mesh2d,
    platform::collections::HashMap, prelude::*,
};
use pathfinding::{
    agent::{AgentNodeTypeCostOverrides, KeepAvoidanceData},
    debug::{EnableLandmassDebug, Landmass2dDebugPlugin},
    landmass::{Archipelago, NavMeshBuilder, NodeType},
    nav_mesh::NavMeshHandle,
    prelude::*,
};
use std::sync::Arc;

pub fn plugin(app: &mut App) {
    let press_f12 = input_just_pressed(KeyCode::F12);

    app.add_plugins(LandmassPlugin2d::default())
        .add_plugins(Landmass2dDebugPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_debug.run_if(press_f12))
        .add_systems(Update, (convert_mesh, handle_clicks, update_and_move_agent));
}

// A utility to wait for a mesh to be loaded and convert the mesh to a nav mesh.
#[derive(Component)]
struct ConvertMesh {
    mesh: Handle<Mesh>,
    nav_mesh: Handle<NavMesh2d>,
    slow_area: Rect,
    slow_node_type: NodeType,
}

fn convert_mesh(
    converters: Query<(Entity, &ConvertMesh)>,
    meshes: Res<Assets<Mesh>>,
    mut nav_meshes: ResMut<Assets<NavMesh2d>>,
    mut commands: Commands,
) {
    for (entity, converter) in converters.iter() {
        let Some(mesh) = meshes.get(&converter.mesh) else {
            continue;
        };

        let map = |[x, y, _]: [f32; 3]| Vec2::new(x, y);
        let mut nav_mesh = NavMeshBuilder::<TwoD>::from_bevy_mesh(mesh, map).unwrap();
        mark_slow_polygons(&mut nav_mesh, converter.slow_area);

        let valid_nav_mesh = nav_mesh.validate().unwrap();
        let nav_mesh = NavMesh2d {
            nav_mesh: Arc::new(valid_nav_mesh),
            type_index_to_node_type: HashMap::from([(1u32, converter.slow_node_type)]),
        };
        nav_meshes.insert(&converter.nav_mesh, nav_mesh).unwrap();
        commands.entity(entity).remove::<ConvertMesh>();
    }
}

fn mark_slow_polygons(nav_mesh: &mut NavigationMesh2d, slow_area: Rect) {
    for (index, polygon) in nav_mesh.polygons.iter().enumerate() {
        let sum = polygon
            .iter()
            .map(|&i| nav_mesh.vertices[i as usize])
            .sum::<Vec2>();
        let center = sum / polygon.len() as f32;
        if slow_area.contains(center) {
            nav_mesh.polygon_type_indices[index] = 1;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    nav_meshes: Res<Assets<NavMesh2d>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Transform::from_xyz(5.0, 0.0, 0.0),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 16.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let message = "LMB - Spawn agent\nShift+LMB - Spawn agent (fast on mud)\nRMB - Change target point\nF12 - Toggle debug view";
    commands.spawn((
        Text(message.into()),
        TextLayout::new_with_justify(Justify::Right),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
    ));

    let slow_area = Rect::from_corners(Vec2::new(-3.99582, -2.89418), Vec2::new(3.30418, 4.00582));
    commands.spawn((
        Transform::from_translation(slow_area.center().extend(1.0)),
        Mesh2d(meshes.add(Rectangle {
            half_size: slow_area.size() * 0.5,
        })),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(css::BROWN.with_alpha(0.5))))),
    ));

    let mut archipelago = Archipelago::<TwoD>::new(0.01);
    let slow_node_type = archipelago.add_node_type(1000.0).unwrap();
    commands.insert_resource(archipelago);
    commands.insert_resource(AgentOptions::<TwoD>::from_agent_radius(0.5));

    // Spawn the islands.
    let mesh = asset_server.load("nav_mesh.glb#Mesh0/Primitive0");
    let nav_mesh = nav_meshes.reserve_handle();
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(css::ANTIQUE_WHITE)))),
        NavMeshHandle(nav_mesh.clone()),
        ConvertMesh {
            mesh,
            nav_mesh,
            slow_area,
            slow_node_type,
        },
    ));

    let mesh = asset_server.load("nav_mesh.glb#Mesh1/Primitive0");
    let nav_mesh = nav_meshes.reserve_handle();
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(css::ANTIQUE_WHITE)))),
        Transform::from_translation(Vec3::new(12.0, 0.0, 0.0)),
        NavMeshHandle(nav_mesh.clone()),
        ConvertMesh {
            mesh,
            nav_mesh,
            slow_area: Rect::EMPTY,
            slow_node_type,
        },
    ));

    // Spawn the target.
    let target_entity = commands.spawn_entity((
        Transform::from_translation(Vec3::new(0.0, 6.0, 0.11)),
        Mesh2d(meshes.add(Circle { radius: 0.25 })),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(css::PURPLE)))),
        PathTarget,
    ));

    commands.insert_resource(AgentSpawner {
        mesh: meshes.add(Circle { radius: 0.5 }),
        material: materials.add(ColorMaterial::from(Color::from(css::SEA_GREEN))),
        target_entity,
        fast_material: materials.add(ColorMaterial::from(Color::from(css::BLUE_VIOLET))),
        slow_node_type,
    });
}

#[derive(Resource)]
struct AgentSpawner {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    target_entity: Entity,
    fast_material: Handle<ColorMaterial>,
    slow_node_type: NodeType,
}

impl AgentSpawner {
    fn spawn(&self, position: Vec2, commands: &mut Commands, fast_agent: bool) {
        let mut entity = commands.spawn((
            Transform::from_translation(position.extend(0.1)),
            Mesh2d(self.mesh.clone()),
            MeshMaterial2d(self.material.clone()),
            Agent2d::new(Vec2::ZERO, 0.5, 2.0, 3.0),
            AgentTarget2d::Entity(self.target_entity),
            KeepAvoidanceData,
        ));

        if fast_agent {
            let mut node_cost_overrides = AgentNodeTypeCostOverrides::default();
            assert!(node_cost_overrides.set_node_type_cost(self.slow_node_type, 1.0));
            entity.insert((
                MeshMaterial2d(self.fast_material.clone()),
                node_cost_overrides,
            ));
        }
    }
}

/// Use the desired velocity as the agent's velocity.
/// Apply the agent's velocity to its position.
fn update_and_move_agent(
    time: Res<Time>,
    agent_query: Query<(&mut Transform, &mut Agent2d, &GlobalTransform)>,
) {
    for (mut transform, mut agent, global_transform) in agent_query {
        let global_transform = global_transform.affine().inverse();
        let local_velocity =
            global_transform.transform_vector3(agent.desired_velocity().extend(0.0));
        transform.translation += local_velocity * time.delta_secs();
        agent.current_velocity = agent.desired_velocity();
    }
}

/// Marker component for the target entity.
#[derive(Component)]
struct PathTarget;

/// Handles clicks by spawning agents with LMB and moving the target with RMB.
fn handle_clicks(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    agent_spawner: Res<AgentSpawner>,
    target: Single<Entity, With<PathTarget>>,
    mut commands: Commands,
) {
    let left = buttons.just_pressed(MouseButton::Left);
    let right = buttons.just_pressed(MouseButton::Right);
    let shift = keys.pressed(KeyCode::ShiftLeft);
    if !(left || right) {
        return;
    }

    let target = target.into_inner();
    let cursor_position = window.into_inner().cursor_position();
    let (camera, camera_transform) = camera.into_inner();

    let Some(world_position) = cursor_position
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            ray.intersect_plane(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Z })
                .map(|distance| ray.get_point(distance))
        })
    else {
        return;
    };

    if left {
        agent_spawner.spawn(world_position.xy(), &mut commands, shift);
    }

    if right {
        let transform = Transform::from_translation(world_position.xy().extend(0.11));
        commands.entity(target).insert(transform);
    }
}

/// System for toggling the `EnableLandmassDebug` resource.
fn toggle_debug(mut debug: ResMut<EnableLandmassDebug>) {
    **debug = !**debug;
}
