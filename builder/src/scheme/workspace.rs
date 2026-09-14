use super::floor::{Building, Floor, FloorEdge, FloorVert, Hole};
use bevy::prelude::*;
use bevy_egui::egui;
use bmesh::{BMesh, EdgeKey, VertKey};
use egui::Ui;

#[derive(Resource)]
pub struct Workspace {
    pub current_floor: usize,
    pub edge_data: FloorEdge,
    pub selected_verts: Vec<VertKey>,
    edge_ui_hover: Option<EdgeKey>,
    pub snap_size: Vec2,
    pub hover_distance: f32,
    save_file: String,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            edge_data: FloorEdge {
                hole: None,
                prev_width: 0.10,
                next_width: 0.10,
            },
            selected_verts: vec![],
            edge_ui_hover: None,
            current_floor: 0,
            snap_size: Vec2::splat(0.1),
            hover_distance: 16.0,
            save_file: String::from("data.json"),
        }
    }
}

pub fn example_building() -> Building {
    let building = Building {
        floors: vec![example_mesh()],
    };
    let path = "floor.json";
    building.save(path).unwrap();
    Building::load(path).unwrap()
}

fn example_mesh() -> Floor {
    let edge = FloorEdge {
        hole: None,
        prev_width: 0.10,
        next_width: 0.10,
    };

    let mut mesh: BMesh<Floor> = BMesh::default();

    let a = Vec2::new(0.4, 0.2) * 2.0;
    let b = Vec2::new(0.6, 0.2) * 2.0;
    let c = Vec2::new(0.5, 0.8) * 2.0;
    let mid = Vec2::new(0.5, 0.5) * 2.0;

    let snap = |position: Vec2| (position / 0.01).round() * 0.01;
    let [a, b, c, mid] = [a, b, c, mid].map(snap);
    let [a, b, c, mid] = [a, b, c, mid].map(|p| mesh.vert_make(FloorVert::new(p)));

    let _ = mesh.edge_find_or_make(a, mid, edge);
    let _ = mesh.edge_find_or_make(b, mid, edge);
    let c = mesh.edge_find_or_make(mid, c, edge);

    mesh.edges[c].hole = Some(Hole {
        offset: Vec2::new(0.0, 0.2),
        size: Vec2::new(0.2, 0.1),
    });

    Floor {
        mesh,
        min: 2.0,
        max: 2.0 + 2.5,
        ref_scale: 0.025,
        ref_offset_x: 0.0,
        ref_offset_y: 0.0,
    }
}

fn collapsing(
    text: &str,
    ui: &mut egui::Ui,
    add_body: impl FnOnce(&mut Ui),
) -> egui::CollapsingResponse<()> {
    egui::CollapsingHeader::new(text)
        .default_open(true)
        .show_unindented(ui, Box::new(add_body))
}

impl Workspace {
    pub fn left_ui(&mut self, ui: &mut Ui, building: &mut Building) {
        let top_spacing_amount = ui.spacing().window_margin.top;
        ui.add_space(top_spacing_amount.into());

        fn drag(
            ui: &mut Ui,
            value: &mut f32,
            prefix: &str,
            range: std::ops::RangeInclusive<f32>,
            speed: f32,
        ) -> egui::Response {
            let widget = egui::DragValue::new(value).prefix(prefix);
            ui.add(widget.range(range).speed(speed))
        }

        ui.columns_const(|[x, y]| {
            drag(x, &mut self.snap_size.x, "grid x: ", 0.001..=1.0, 0.005);
            drag(y, &mut self.snap_size.y, "grid y: ", 0.001..=1.0, 0.005);
        });

        ui.columns_const(|[ui]| drag(ui, &mut self.hover_distance, "hover: ", 1.0..=100.0, 1.0));

        ui.text_edit_singleline(&mut self.save_file);

        ui.columns_const(|[a, b]| {
            let path = &self.save_file;
            if a.button("save").clicked() {
                match building.save(path) {
                    Ok(_) => bevy::log::info!("save ok {path}"),
                    Err(err) => bevy::log::error!("save err {path} {err}"),
                }
            }
            if b.button("load").clicked() {
                match Building::load(path) {
                    Ok(file) => {
                        bevy::log::info!("load ok {path}");
                        *building = file;
                    }
                    Err(err) => bevy::log::error!("load err {path} {err}"),
                }
            }
        });

        ui.separator();

        for index in 0..building.floors.len() {
            ui.horizontal(|ui| {
                let checked = self.current_floor == index;
                let text = format!("floor #{index}");
                if ui.selectable_label(checked, text).clicked() {
                    self.current_floor = index;
                    self.selected_verts.clear();
                }
                if ui.button("🗙").clicked() {
                    building.floors.remove(index);
                }
            });
        }

        ui.columns_const(|[ui]| {
            if ui.button("add floor").clicked() {
                let min = building.floors.last().map_or(0.0, |f| f.max);
                building.floors.push(Floor {
                    min,
                    max: min + 1.0,
                    ..default()
                });
            }
        });

        if self.current_floor >= building.floors.len() {
            self.current_floor = 0;
            self.selected_verts.clear();
        }

        if let Some(floor) = building.floors.get_mut(self.current_floor) {
            ui.columns_const(|[min_ui, max_ui]| {
                drag(min_ui, &mut floor.min, "min y: ", -20.0..=20.0, 0.05);
                drag(max_ui, &mut floor.max, "max y: ", -20.0..=20.0, 0.05);
            });

            ui.columns_const(|[scale, x, y]| {
                drag(scale, &mut floor.ref_scale, "scale: ", 0.001..=1.0, 0.001);
                drag(x, &mut floor.ref_offset_x, "ref x: ", -3000.0..=3000.0, 1.0);
                drag(y, &mut floor.ref_offset_y, "ref y: ", -3000.0..=3000.0, 1.0);
            });
        }

        ui.separator();

        let Some(Floor { mesh, .. }) = &mut building.floors.get_mut(self.current_floor) else {
            return;
        };
        if let Some(&vert) = self.selected_verts.last() {
            if let Some(slot) = mesh.verts.get_mut(vert) {
                ui.heading(format!("Vert: {vert}"));

                ui.columns_const(|[x, y]| {
                    drag(x, &mut slot.position.x, "x: ", -1000.0..=1000.0, 0.01);
                    drag(y, &mut slot.position.y, "y: ", -1000.0..=1000.0, 0.01);
                });

                ui.columns_const(|[w]| drag(w, &mut slot.cap_width, "cap: ", 0.0..=100.0, 1.0));
            }

            self.edge_ui_hover = None;

            let disk = mesh.disk_iter(vert).into_iter().flatten();
            for edge in disk.collect::<Vec<_>>().into_iter() {
                let slot = &mut mesh.edges[edge];
                let [prev, next] = [slot.prev.vert, slot.next.vert];

                if ui.heading(format!("{edge} ({prev}->{next})")).hovered() {
                    self.edge_ui_hover = Some(edge);
                }

                if let Some(hole) = slot.hole.as_mut() {
                    ui.columns_const(|[x, y]| {
                        drag(x, &mut hole.offset.x, "ox: ", -25.0..=25.0, 0.1);
                        drag(y, &mut hole.offset.y, "oy: ", -25.0..=25.0, 0.1);
                    });

                    ui.columns_const(|[x, y]| {
                        drag(x, &mut hole.size.x, "sx: ", 0.0..=10.0, 0.01);
                        drag(y, &mut hole.size.y, "sy: ", 0.0..=10.0, 0.01);
                    });
                } else if ui.button("add hole").clicked() {
                    slot.hole = Some(Hole {
                        offset: Vec2::new(0.0, 0.2),
                        size: Vec2::new(0.2, 0.1),
                    });
                }

                ui.columns_const(|[w]| {
                    if w.button("Reverse").clicked() {
                        slot.reverse();
                        (slot.prev_width, slot.next_width) = (slot.next_width, slot.prev_width);
                    }
                });

                ui.columns_const(|[p, n]| {
                    drag(p, &mut slot.prev_width, "prev: ", 0.0..=10.0, 0.01);
                    drag(n, &mut slot.next_width, "next: ", 0.0..=10.0, 0.01);
                });
            }
        }

        ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    }

    pub fn right_ui(&mut self, ui: &mut Ui, building: &mut Building) {
        let px = ui.ctx().pixels_per_point().recip();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let top_spacing_amount = ui.spacing().window_margin.top;
            ui.add_space(top_spacing_amount.into());

            let Some(Floor { mesh, .. }) = &building.floors.get(self.current_floor) else {
                return;
            };

            collapsing("vertices", ui, |ui| {
                for (key, slot) in &mesh.verts {
                    let text = if let Some(edge) = slot.edge {
                        format!("{key}  edge: {edge}")
                    } else {
                        format!("{key}")
                    };
                    let position = self.selected_verts.iter().position(|&k| k == key);
                    ui.columns_const(|[ui]| {
                        if ui.selectable_label(position.is_some(), text).clicked() {
                            if let Some(index) = position {
                                self.selected_verts.remove(index);
                            } else {
                                self.selected_verts.push(key);
                            }
                        }
                    });
                }
            });

            collapsing("edges", ui, |ui| {
                let style = ui.style();
                let color = style.visuals.text_color();
                let font_id = egui::FontSelection::Default.resolve(style);
                let u = egui::Stroke::new(px, color);
                let nou = egui::Stroke::NONE;

                for (key, slot) in &mesh.edges {
                    let [prev, next] = [slot.prev.vert, slot.next.vert];

                    let prev_u = self.selected_verts.contains(&prev);
                    let next_u = self.selected_verts.contains(&next);

                    let text = [
                        (key.to_string(), nou),
                        (String::from("  ["), nou),
                        (prev.to_string(), if prev_u { u } else { nou }),
                        (String::from(" -> "), nou),
                        (next.to_string(), if next_u { u } else { nou }),
                        (String::from("]"), nou),
                    ];

                    let mut job = egui::text::LayoutJob::default();
                    for (text, underline) in text {
                        let format = egui::TextFormat {
                            font_id: font_id.clone(),
                            color,
                            valign: egui::Align::Center,
                            underline,
                            ..Default::default()
                        };
                        job.append(&text, 0.0, format);
                    }

                    ui.label(job);
                }
            });

            collapsing("links", ui, |ui| {
                let style = ui.style();
                let color = style.visuals.text_color();
                let font_id = egui::FontSelection::Default.resolve(style);
                let u = egui::Stroke::new(px, color);
                let nou = egui::Stroke::NONE;

                for (key, slot) in &mesh.links {
                    let vert = slot.vert;
                    let edge = slot.edge;
                    let face = slot.face;
                    let text = format!("{key} v{vert} e{edge} f{face}");
                    let is_start = mesh.faces[face].link == key;

                    let mut job = egui::text::LayoutJob::default();
                    let underline = if is_start { u } else { nou };

                    let format = egui::TextFormat {
                        font_id: font_id.clone(),
                        color,
                        valign: egui::Align::Center,
                        underline,
                        ..Default::default()
                    };
                    job.append(&text, 0.0, format);

                    ui.label(job);
                }
            });

            collapsing("faces", ui, |ui| {
                for (key, slot) in &mesh.faces {
                    let link = slot.link;
                    ui.label(format!("{key} -> {link}"));
                }
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });
    }
}
