use crate::reflect_editor::{errors, Arg, FieldEditor, ReflectEditor};
use bevy::asset::{Assets, Handle};
use bevy::render::mesh::{Indices, Mesh};

pub struct MeshHandleEditor;

impl FieldEditor for MeshHandleEditor {
    type Target = Handle<Mesh>;

    fn edit_ref(editor: ReflectEditor, arg: Arg, handle: &Self::Target) {
        let meshes = match unsafe { editor.get_resource_mut::<Assets<Mesh>>() } {
            Ok(meshes) => meshes,
            Err(error) => return error.show_typed::<Assets<Mesh>>(arg.ui),
        };
        let Some(mesh) = meshes.get(handle) else {
            errors::dead_asset_handle(arg.ui, handle.into());
            return;
        };

        ui_inner(mesh, arg.ui, arg.id.with("mesh"));
    }

    fn edit_mut(editor: ReflectEditor, arg: Arg, handle: &mut Self::Target) -> bool {
        let handle: &Self::Target = handle;

        let mut meshes = match unsafe { editor.get_resource_mut::<Assets<Mesh>>() } {
            Ok(meshes) => meshes,
            Err(error) => {
                error.show_typed::<Assets<Mesh>>(arg.ui);
                return false;
            }
        };

        let Some(mesh) = meshes.get_mut(handle) else {
            errors::dead_asset_handle(arg.ui, handle.into());
            return false;
        };

        ui_inner(mesh, arg.ui, arg.id.with("mesh"));

        arg.ui.vertical_centered_justified(|ui| {
            ui.add_enabled_ui(mesh.indices().is_some(), |ui| {
                if ui.button("duplicate vertices").clicked() {
                    mesh.duplicate_vertices();
                }
            });

            ui.add_enabled_ui(mesh.indices().is_none(), |ui| {
                if ui.button("compute flat normals").clicked() {
                    mesh.compute_flat_normals();
                }
            });

            if ui.button("generate tangents").clicked() {
                let _ = mesh.generate_tangents();
            }
        });

        false
    }
}

fn ui_inner(mesh: &Mesh, ui: &mut egui::Ui, id: egui::Id) {
    egui::Grid::new(id).num_columns(2).show(ui, |ui| {
        ui.label("topology");
        ui.label(format!("{:?}", mesh.primitive_topology()));
        ui.end_row();

        ui.label("vertices");
        ui.label(mesh.count_vertices().to_string());
        ui.end_row();

        if let Some(indices) = mesh.indices() {
            ui.label("indices");
            let len = match indices {
                Indices::U16(vec) => vec.len(),
                Indices::U32(vec) => vec.len(),
            };
            ui.label(len.to_string());
            ui.end_row();
        }

        ui.label("attributes");

        let builtin_attributes = [
            (Mesh::ATTRIBUTE_POSITION, "position"),
            (Mesh::ATTRIBUTE_NORMAL, "normal"),
            (Mesh::ATTRIBUTE_UV_0, "uv0"),
            (Mesh::ATTRIBUTE_UV_1, "uv1"),
            (Mesh::ATTRIBUTE_TANGENT, "tangent"),
            (Mesh::ATTRIBUTE_COLOR, "color"),
            (Mesh::ATTRIBUTE_JOINT_WEIGHT, "joint_weight"),
            (Mesh::ATTRIBUTE_JOINT_INDEX, "joint_index"),
        ];

        ui.vertical(|ui| {
            for (attribute, name) in builtin_attributes {
                if mesh.attribute(attribute.id).is_some() {
                    ui.label(name);
                }
            }
        });

        ui.end_row();
    });
}
