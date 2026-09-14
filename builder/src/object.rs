use bevy::{
    gltf::{GltfMaterial, GltfMesh},
    prelude::*,
};

use crate::scheme::{editor::FloorEditor, floor::Building};

pub fn plugin(app: &mut App) {
    app.init_resource::<DragState>();
    app.add_systems(Startup, load_furniture);
    app.add_systems(Update, spawn_furniture);
}

#[derive(Resource)]
pub struct FurnitureScene(Handle<Gltf>);

#[derive(Component)]
pub struct ObjectRoot;

#[derive(Component)]
pub struct ObjectPart;

fn load_furniture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let gltf = asset_server.load("models/furniture.glb");
    commands.insert_resource(FurnitureScene(gltf));
}

fn spawn_furniture(
    mut commands: Commands,
    scene: Res<FurnitureScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_materials: Res<Assets<GltfMaterial>>,
    mut loaded: Local<bool>,

    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *loaded {
        return;
    }
    let Some(gltf) = gltf_assets.get(&scene.0) else {
        return;
    };

    *loaded = true;

    let names = [
        //"stand1",
        //"stand2",
        //"standHalf",
        //"stand",
        "frige",
        "shelf",
        "stove",
        "toilet",
        "bed90",
        "bed180",
        "stand40",
        "stand60a",
        "stand60b",
        "dishwasher",
    ];

    dbg!(&gltf.named_meshes);

    for (index, name) in names.into_iter().enumerate() {
        println!("{name}");

        let mesh = &gltf.named_meshes[name];
        let mesh = gltf_meshes.get(mesh).unwrap();

        let mut base = commands.spawn((
            ObjectRoot,
            Transform::from_xyz(1.0 + index as f32, 2.0, 3.0),
            Visibility::Inherited,
        ));

        base.with_children(|base| {
            for primitive in &mesh.primitives {
                let Some(material) = primitive.material.as_ref() else {
                    continue;
                };
                let Some(material) = gltf_materials.get(material) else {
                    continue;
                };

                let material = materials.add(StandardMaterial {
                    base_color: material.base_color,
                    ..default()
                });
                //let material = standard_material_from_gltf_material(material);
                let mut child = base.spawn((
                    ObjectPart,
                    Mesh3d(primitive.mesh.clone()),
                    MeshMaterial3d(material),
                ));

                child.observe(drag_start);
                child.observe(drag);
                child.observe(drag_end);
            }
        });
    }
}

#[derive(Resource, Default)]
struct DragState(Option<DragData>);

struct DragData {
    root: Entity,
    base: Transform,
}

fn drag_start(
    trigger: On<Pointer<DragStart>>,
    mut state: ResMut<DragState>,

    child_of: Query<&ChildOf>,
    root: Query<(), With<ObjectRoot>>,
    transforms: Query<&Transform>,

    mut commands: Commands,
) {
    let root = child_of
        .iter_ancestors(trigger.event_target())
        .find(|&entity| root.contains(entity));

    let Some(root) = root else {
        return;
    };
    let Ok(&base) = transforms.get(root) else {
        return;
    };

    bevy::log::info!("start {root}");
    state.0 = Some(DragData { root, base });

    let mut entity = commands.entity(trigger.event_target());
    entity.insert(Pickable::IGNORE);
}
fn drag_end(trigger: On<Pointer<DragEnd>>, mut state: ResMut<DragState>, mut commands: Commands) {
    state.0 = None;

    let mut entity = commands.entity(trigger.event_target());
    entity.insert(Pickable::default());
}

fn drag(
    trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    root: Query<(), With<ObjectRoot>>,
    mut transforms: Query<&mut Transform>,
    floor: Single<&FloorEditor>,
) {
    let root = child_of
        .iter_ancestors(trigger.event_target())
        .find(|&entity| root.contains(entity));

    let Some(root) = root else {
        return;
    };
    let Some(hit) = floor.last_hit_position else {
        return;
    };
    let Ok(mut root_transform) = transforms.get_mut(root) else {
        return;
    };

    root_transform.translation = hit;
}
