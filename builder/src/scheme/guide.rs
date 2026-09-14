use super::{editor::FloorEditor, floor::Building, math::ExtVec2, workspace::Workspace};
use bevy::{color::palettes::tailwind, picking::pointer::PointerInteraction, prelude::*};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GuideGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct WallGizmos;

pub fn plugin(app: &mut App) {
    app.init_gizmo_group::<GuideGizmos>()
        .init_gizmo_group::<WallGizmos>()
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, (draw_guides, draw_mesh_intersections));
}

fn setup(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<GuideGizmos>();
    config.depth_bias = -1.0;
    config.line.width = 1.0;

    let (config, _) = config_store.config_mut::<WallGizmos>();
    config.depth_bias = -1.0;
    config.line.width = 1.0;
}

fn draw_guides(
    mut guides: Gizmos<GuideGizmos>,
    mut walls: Gizmos<WallGizmos>,
    workspace: Res<Workspace>,
    mut building: ResMut<Building>,
    plane: Single<&FloorEditor>,
) {
    let Some(floor) = building.floors.get_mut(plane.floor) else {
        return;
    };
    floor.sort_segments();

    let hovered_vert = plane.hovered_vert;

    let elevation = floor.min;

    use super::editor::HOVER_DISTANCE as RADIUS;
    use tailwind::{LIME_400 as HOVERED, RED_600 as NORMAL, RED_600 as LINE, RED_900 as WALL};

    for segment in floor.segments() {
        let [prev, next] = [segment.prev.vert, segment.next.vert];
        let [prev, next] = [prev, next].map(|v| floor.mesh.verts[v].position.extend_y(elevation));
        guides.line(prev, next, LINE);
    }

    for (vert, slot) in &floor.mesh.verts {
        let hovered = hovered_vert.is_some_and(|v| v == vert);
        let selected = workspace.selected_verts.contains(&vert);
        let active = hovered || selected;
        let position = slot.position.extend_y(elevation);
        guides.sphere(position, RADIUS, if active { HOVERED } else { NORMAL });
    }

    for [prev, next] in floor.walls().map(|p| p.map(|v| v.extend_y(elevation))) {
        walls.line(prev, next, WALL);
    }
}

fn draw_mesh_intersections(
    pointers: Query<&PointerInteraction>,
    what: Query<&FloorEditor>,
    mut gizmos: Gizmos<WallGizmos>,
) {
    for pointer in &pointers {
        for &(entity, ref hit) in pointer.as_slice() {
            let ground = what.get(entity);
            let is_ground = ground.is_ok();

            if let Some((prev, normal)) = hit.position.zip(hit.normal) {
                let next = prev + normal.normalize() * 0.5;
                if is_ground {
                    // if ground.is_ok_and(|ground| ground.touched) {
                    //     gizmos.sphere(prev, 0.025, tailwind::RED_500);
                    // }
                    break;
                } else {
                    gizmos.sphere(prev, 0.025, tailwind::RED_500);
                    gizmos.arrow(prev, next, tailwind::GRAY_800);
                }
            }
        }
    }
}
