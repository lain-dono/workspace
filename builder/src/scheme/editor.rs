use super::{
    floor::{Building, FloorVert},
    workspace::Workspace,
};
use bevy::{mesh::PlaneMeshBuilder, prelude::*};
use bmesh::VertKey;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_system);
    app.add_systems(Update, (update_ground, make_face_system));
}

#[derive(Component)]
pub struct FloorEditor {
    pub floor: usize,
    pub last_hit_position: Option<Vec3>,
    pub hovered_vert: Option<VertKey>,
    pub dragged_vert: Option<VertKey>,
}

impl Default for FloorEditor {
    fn default() -> Self {
        Self {
            floor: usize::MAX,
            last_hit_position: None,
            hovered_vert: None,
            dragged_vert: None,
        }
    }
}

fn setup_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = PlaneMeshBuilder::from_size(Vec2::new(1000.0, 1000.0)).subdivisions(1);

    commands
        .spawn((FloorEditor::default(), Mesh3d(meshes.add(mesh))))
        .observe(click)
        .observe(movement)
        .observe(out)
        .observe(drag_start)
        .observe(drag_move)
        .observe(drag_end);
}

fn update_ground(
    mut ground: Query<(&mut Transform, &mut FloorEditor)>,
    workspace: Res<Workspace>,
    building: Res<Building>,
) {
    let ground = ground.single_mut().ok();

    if let Some((mut transform, mut ground)) = ground {
        let current_floor = workspace.current_floor;
        if ground.floor != current_floor {
            ground.floor = current_floor;

            ground.hovered_vert = None;
            ground.dragged_vert = None;

            if let Some(floor) = building.floors.get(ground.floor) {
                transform.translation.y = floor.min;
            }
        }
    }
}

pub const HOVER_DISTANCE: f32 = 0.05;
pub const HOVER_DISTANCE_PX: f32 = 4.0;

fn make_face_system(
    mut workspace: ResMut<Workspace>,
    mut building: ResMut<Building>,
    keyboard: Res<ButtonInput<KeyCode>>,
    plane: Single<&FloorEditor>,
) {
    if keyboard.just_released(KeyCode::KeyF)
        && let Some(floor) = &mut building.floors.get_mut(plane.floor)
    {
        floor.new_face(&mut workspace.selected_verts);
    }
}

pub fn out(trigger: On<Pointer<Out>>, mut ground: Query<&mut FloorEditor>) {
    if let Ok(mut floor) = ground.get_mut(trigger.event_target()) {
        floor.hovered_vert = None;
    }
}

pub fn movement(
    trigger: On<Pointer<Move>>,
    building: Res<Building>,
    mut ground: Query<&mut FloorEditor>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Ok(mut plane) = ground.get_mut(trigger.event_target()) {
        let floor = building.floors.get(plane.floor);
        if let Some((floor, pointer)) = floor.zip(trigger.hit.position) {
            plane.hovered_vert =
                floor.hovered(camera, camera_transform, pointer, HOVER_DISTANCE_PX);
        }

        plane.last_hit_position = trigger.hit.position;
    }
}

pub fn click(
    trigger: On<Pointer<Click>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workspace: ResMut<Workspace>,
    mut building: ResMut<Building>,
    plane: Query<&FloorEditor>,
) {
    let &mut Workspace {
        snap_size,
        ref mut edge_data,
        ref mut selected_verts,
        ..
    } = workspace.as_mut();

    let Ok(plane) = plane.get(trigger.event_target()) else {
        return;
    };
    let Some(floor) = building.floors.get_mut(plane.floor) else {
        return;
    };
    if plane.dragged_vert.is_some() {
        return;
    }

    let pointer = trigger.hit.position.map(|v| v.xz());
    let hovered_vert = plane.hovered_vert;

    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let primary_pointer = pointer.filter(|_| matches!(trigger.button, PointerButton::Primary));
    let secondary_clicked = pointer.filter(|_| matches!(trigger.button, PointerButton::Secondary));

    if let Some(pointer) = primary_pointer {
        if let Some(&start) = selected_verts.last() {
            if shift_pressed {
                if let Some(vert) = hovered_vert {
                    if let Some(index) = selected_verts.iter().position(|&v| v == vert) {
                        selected_verts.remove(index);
                    } else {
                        selected_verts.push(vert);
                    }
                }
            } else {
                let end = if let Some(end) = hovered_vert {
                    end
                } else {
                    let vert = FloorVert::new_snap(snap_size, pointer);
                    floor.mesh.vert_make(vert)
                };

                if start != end {
                    let _ = floor.mesh.edge_find_or_make(start, end, *edge_data);
                }
                selected_verts.clear();
                selected_verts.push(end);
            }
        } else {
            let vert = FloorVert::new_snap(snap_size, pointer);
            let vert = hovered_vert.unwrap_or_else(|| floor.mesh.vert_make(vert));

            selected_verts.clear();
            selected_verts.push(vert);
        }
    } else if let Some(_pointer) = secondary_clicked {
        selected_verts.clear();
        if let Some(vert) = hovered_vert {
            floor.mesh.vert_kill(vert);
        }
    }
}

pub fn drag_start(
    trigger: On<Pointer<DragStart>>,
    building: Res<Building>,
    mut plane: Query<&mut FloorEditor>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if !matches!(trigger.button, PointerButton::Primary) {
        return;
    }

    let (camera, camera_transform) = camera.into_inner();

    if let Ok(mut plane) = plane.get_mut(trigger.event_target()) {
        let floor = building.floors.get(plane.floor);
        if let Some((floor, pointer)) = floor.zip(trigger.hit.position) {
            plane.dragged_vert =
                floor.hovered(camera, camera_transform, pointer, HOVER_DISTANCE_PX);
        }
    }
}

pub fn drag_end(
    trigger: On<Pointer<DragEnd>>,
    mut plane: Query<&mut FloorEditor>,
    mut workspace: ResMut<Workspace>,
) {
    if let Ok(mut plane) = plane.get_mut(trigger.event_target())
        && let Some(vert) = plane.dragged_vert.take()
    {
        workspace.selected_verts.clear();
        workspace.selected_verts.push(vert);
    }
}

pub fn drag_move(
    trigger: On<Pointer<Drag>>,
    mut building: ResMut<Building>,
    workspace: Res<Workspace>,
    plane: Query<&FloorEditor>,
) {
    if !matches!(trigger.button, PointerButton::Primary) {
        return;
    }

    if let Ok(plane) = plane.get(trigger.event_target()) {
        let snap = workspace.snap_size;
        let floor = building.floors.get_mut(plane.floor);

        if let Some((floor, pointer)) = floor.zip(plane.last_hit_position) {
            let floor_position = pointer.xz();
            let vert = plane.dragged_vert;
            let vert = vert.and_then(|key| floor.mesh.verts.get_mut(key));

            if let Some(vert) = vert {
                vert.position = (floor_position / snap).round() * snap;
            }
        }
    }
}
