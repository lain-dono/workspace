use crate::ui::{icon, EditorTab, Style};
use bevy::asset::io::AssetReader;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use egui::Widget as _;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Default, Component)]
pub struct FileBrowser {
    open: HashSet<PathBuf>,
}

impl EditorTab for FileBrowser {
    type Param = (SRes<Style>, SRes<AssetServer>);

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, assets): &mut SystemParamItem<'_, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        /*
        let io = assets.asset_reader();
        ui.scope(|ui| {
            //ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            style.set_theme_visuals(ui);
            style.for_scrollbar(ui);

            let scroll = egui::ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                style.scrollarea(ui);
                read_dir(ui, io, ".".as_ref(), &mut self.open);
            })
        });
        */
    }
}

/*
fn read_dir(ui: &mut egui::Ui, io: &dyn AssetReader, path: &Path, open: &mut HashSet<PathBuf>) {
    ui.indent(&path, |ui| {
        if let Ok(dir) = io.read_directory(path) {
            for path in dir {
                let path_string = path.file_name().unwrap().to_string_lossy();
                let response = if io.is_dir(&path) {
                    let is_open = open.contains(&path);
                    let icon = if is_open {
                        icon::TRIA_DOWN
                    } else {
                        icon::TRIA_RIGHT
                    };
                    //let icon = icon::FILE_FOLDER;
                    let text = format!("{} {}", icon, path_string);

                    let response = egui::Label::new(text).sense(egui::Sense::click()).ui(ui);

                    if response.clicked() {
                        if open.contains(&path) {
                            open.remove(&path);
                        } else {
                            open.insert(path.clone());
                        }
                    }

                    if open.contains(&path) {
                        read_dir(ui, io, &path, open);
                    }

                    response
                } else {
                    let ext = path.extension().map(OsStr::to_string_lossy);
                    let icon = if let Some(ext) = ext.as_deref() {
                        match ext {
                            "scene" => icon::SCENE_DATA,
                            "material" => icon::MATERIAL_DATA,
                            "image" => icon::TEXTURE_DATA,
                            "mesh" => icon::MESH_DATA,

                            "shader" => icon::NODE_MATERIAL,

                            "gltf" | "glb" => icon::FILE_3D,
                            "jpg" | "jpeg" | "png" => icon::FILE_IMAGE,
                            "mp3" | "ogg" | "flac" | "wav" => icon::FILE_SOUND,

                            _ => icon::FILE,
                        }
                    } else {
                        icon::FILE
                    };

                    ui.label(format!("{} {}", icon, path_string))
                };

                //response.on_hover_text(path.to_string_lossy());
            }
        }
    });
}
*/
