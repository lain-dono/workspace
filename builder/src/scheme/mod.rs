use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub mod editor;
pub mod floor;
pub mod guide;
pub mod interaction;
pub mod math;
pub mod mesh_builder;
pub mod reference;
pub mod segment;
pub mod workspace;

pub fn plugin(app: &mut App) {
    app.insert_resource(self::workspace::Workspace::default())
        .insert_resource(self::workspace::example_building())
        .add_plugins((
            self::guide::plugin,
            self::editor::plugin,
            self::reference::plugin,
        ))
        .add_systems(Startup, (setup_mesh, setup))
        .add_systems(EguiPrimaryContextPass, (ui_workspace, update_grid));
}

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

fn setup(mut commands: Commands) {
    // let pos = Vec3::new(-2.0, 2.5, 5.0) * 10.0;
    let pos = Vec3::new(0.0, 2.5, 5.0) * 5.0;
    let tx = Transform::from_translation(pos).looking_at(CAMERA_TARGET, Vec3::Y);
    commands.insert_resource(OriginalCameraTransform(tx));
}

fn update_grid(
    workspace: Res<self::workspace::Workspace>,
    building: Res<self::floor::Building>,
    mut camera: Query<&mut crate::post::GridSettings>,
) {
    if let Ok(mut grid) = camera.single_mut() {
        grid.size = workspace.snap_size;
        if let Some(floor) = building.floors.get(workspace.current_floor) {
            grid.offset_y = floor.min;
        }
    }
}

#[derive(Component)]
pub struct FloorMesh;

pub fn setup_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use self::interaction::ExtOnEvent;

    let normal = materials.add(Color::from(tailwind::GRAY_50));
    let hovered = materials.add(Color::from(tailwind::GRAY_100));
    let pressed = materials.add(Color::from(tailwind::GRAY_300));

    type SMat = StandardMaterial;
    type Mat = MeshMaterial3d<StandardMaterial>;

    let action = |new: Handle<SMat>, mut material: Mut<'_, Mat>| material.0 = new.clone();

    commands
        .spawn((
            FloorMesh,
            Mesh3d(meshes.add(empty_mesh())),
            MeshMaterial3d(normal.clone()),
            Pickable {
                should_block_lower: false,
                is_hoverable: true,
            },
        ))
        .on_event::<Out, Mut<'_, Mat>, _>(normal.clone(), action)
        .on_event::<Over, Mut<'_, Mat>, _>(hovered.clone(), action)
        .on_event::<Press, Mut<'_, Mat>, _>(pressed.clone(), action)
        .on_event::<Release, Mut<'_, Mat>, _>(hovered.clone(), action);
}

fn empty_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
        .with_inserted_indices(Indices::U32(vec![]))
}

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

fn ui_workspace(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut workspace: ResMut<self::workspace::Workspace>,
    mut building: ResMut<self::floor::Building>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Mesh3d), With<FloorMesh>>,
) {
    use egui::panel::Panel;

    let mesh = query.single().ok();
    let entity = mesh.map(|(entity, _)| entity);
    let mesh_dst = mesh.and_then(|(_, Mesh3d(id))| meshes.get_mut(id));
    let mesh_dst = mesh_dst.map(self::mesh_builder::MeshBuilder::new);

    let ctx = contexts.ctx_mut().unwrap();
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    let _occupied_l = h_side(&mut viewport_ui, Panel::left("#left"), |ui| {
        workspace.left_ui(ui, &mut building)
    });
    let _occupied_r = h_side(&mut viewport_ui, Panel::right("#rp"), |ui| {
        workspace.right_ui(ui, &mut building)
    });

    if let Some(mut builder) = mesh_dst {
        builder.clear();
        for floor in &mut building.floors {
            floor.sort_segments();
            floor.build(&mut builder);
        }
    }

    /*
    if let Ok((Projection::Perspective(projection), mut dst)) = camera.get_single_mut() {
        let distance_to_target = (CAMERA_TARGET - src.translation).length();
        let frustum_h = 2.0 * distance_to_target * (projection.fov * 0.5).tan();
        let frustum = Vec2::new(frustum_h * projection.aspect_ratio, frustum_h);

        let inv_size = windows.single().size().recip();

        let dx = occupied_r - occupied_l;
        // let dy = occupied.top - occupied.bottom;
        let rot = Vec2::new(dx, 0.0) * inv_size * frustum * 0.5;

        dst.translation = src.translation + dst.rotation.mul_vec3(rot.extend(0.0));
    }
    */

    if let Some(mut entity) = entity.map(|entity| commands.entity(entity)) {
        entity.remove::<bevy::camera::primitives::Aabb>();
    }
}

fn h_side<R, F>(ui: &mut egui::Ui, panel: egui::panel::Panel, f: F) -> f32
where
    F: FnOnce(&mut egui::Ui) -> R,
{
    let panel = panel.resizable(true).min_size(200.0);
    panel.show_inside(ui, f).response.rect.width()
}

/*
fn v_side<R, I, F>(ctx: &mut egui::Context, side: egui::panel::TopBottomSide, id: I, f: F) -> f32
where
    I: Into<egui::Id>,
    F: FnOnce(&mut egui::Ui) -> R,
{
    let panel = egui::TopBottomPanel::new(side, id);
    panel.show(ctx, f).response.rect.height()
}

let occupied_t = v_side(ctx,  egui::panel::TopBottomSide::Top, "#tp", |ui| {
    ui.label("Top resizeable panel");
    ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
});

let occupied_b = v_side(ctx,  egui::panel::TopBottomSide::Bottom, "#bp", |ui| {
    ui.label("Bottom resizeable panel");
    ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
});

fn convert_generic<T, A: Into<T>, B: From<T>>(from: A) -> B {
    from.into().into()
}

fn convert<A: Into<[f32; 2]>, B: From<[f32; 2]>>(from: A) -> B {
    convert_generic(from)
}

fn apply_tx(tx: emath::TSTransform, pos: Vec2) -> Vec2 {
    convert(tx.mul_pos(convert(pos)))
}

*/
