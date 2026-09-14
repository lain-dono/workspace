use self::table::{Column, TableBuilder};
use eframe::egui;
use slotmap::Key;

pub mod layout;
pub mod sizing;
pub mod storage;
pub mod strip;
pub mod table;

use self::storage::{Database, Meta, Record, Table, TableMeta, Ty, Value};

pub struct DatabaseEditor {
    pub db: Database,
    pub active: Option<Table>,
    pub table: TableMeta,

    pub row_height: f32,
    pub edit_fields: bool,
}

impl Default for DatabaseEditor {
    fn default() -> Self {
        let mut db = Database::default();
        let mut table = TableMeta::new("Table 1");

        table.add_field(Meta::singleline("Name"));
        table.add_field(Meta::singleline("Description"));
        table.add_field(Meta::singleline("Amout"));
        table.add_field(Meta::variants(
            "Tags",
            true,
            vec![String::from("A"), String::from("B"), String::from("C")],
        ));

        table.append([
            Value::text("Adam"),
            Value::text("human flesh"),
            Value::text("123"),
            Value::variants(["A", "B"]),
        ]);

        table.append([
            Value::text("Eva"),
            Value::text("human flesh"),
            Value::text("456"),
            Value::variants(["B", "C"]),
        ]);

        let active = db.tables.insert(table);

        Self {
            db,
            table: TableMeta::default(),
            active: Some(active),
            row_height: 20.0,

            edit_fields: false,
        }
    }
}

impl DatabaseEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (key, table) in &self.db.tables {
                ui.selectable_value(&mut self.active, Some(key), &table.name);
            }

            let response = ui.button("+");
            egui::Popup::menu(&response)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.text_edit_singleline(&mut self.table.name);
                    if ui.button("add table").clicked() {
                        let id = self.db.tables.insert(std::mem::take(&mut self.table));
                        self.active = Some(id);
                        ui.close();
                    }
                });
        });

        if let Some(table) = self.active {
            let body_text_size = egui::TextStyle::Body.resolve(ui.style()).size;
            use self::sizing::Size;
            use self::strip::StripBuilder;
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0)) // for the table
                .size(Size::exact(body_text_size)) // for the source code link
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            // self.table_ui(ui, reset);
                            self.show_table(ui, table);
                        });
                    });
                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            if ui.button("+ add row").clicked()
                                && let Some(table) = self.db.tables.get_mut(table)
                            {
                                table.append_default();
                            }
                        });
                    });
                });
        }
    }

    fn show_table(&mut self, ui: &mut egui::Ui, table: Table) {
        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut self.row_height, 20.0..=128.0)
                    .text("row height")
                    .step_by(4.0),
            );

            if ui.button("edit fields").clicked() {
                self.edit_fields = true;
            }
            self.table_modal(ui, table);
        });

        let data = &mut self.db.tables[table];

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let row_height = self.row_height;

        let available_height = ui.available_height();

        let mut builder = TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .cell_layout(
                egui::Layout::left_to_right(egui::Align::Min)
                    .with_main_justify(true)
                    .with_cross_align(egui::Align::Center),
            )
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            // .striped(true)
            .column(Column::auto());
        // .column(Column::remainder());

        for field in data.fields.values() {
            builder = builder.column(field.column);
        }

        let table = builder.header(20.0, |mut header| {
            header.col(|ui| {
                //ui.label("---");
            });
            for field in data.fields.values() {
                header.col(|ui| {
                    ui.strong(&field.desc.name);
                });
            }
        });

        let body = table.body(|mut body| {
            for (record, _) in &data.records {
                body.row(row_height, |mut row| {
                    // row.set_overline(true);

                    let is_hovered = row.hovered;

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_max_width(40.0);

                            let text = if is_hovered {
                                allocate_icon(ui, egui::vec2(10.0, 10.0), |painter, rect| {
                                    drag_handle(&painter, rect, egui::Color32::from_gray(128))
                                });

                                String::from("⛶")
                            } else {
                                format!("{:>4}", record.data().as_ffi() & u32::MAX as u64)
                            };

                            ui.add(egui::Label::new(text).selectable(false));

                            let size = ui.available_size_before_wrap();
                            ui.allocate_at_least(size, egui::Sense::empty());
                        });
                    });

                    for meta in data.fields.values_mut() {
                        row.col(|ui| {
                            meta.ui(ui, record);
                        });
                    }

                    let response = row.response();
                    egui::Popup::context_menu(&response).show(|ui| {
                        if ui.button("edit").clicked() {
                            //
                        }
                        if ui.button("remove").clicked() {
                            //
                        }
                    });
                });
            }
        });
    }
}

impl Meta {
    fn ui(&mut self, ui: &mut egui::Ui, record: Record) {
        let Some(value) = self.data.get_mut(record) else {
            ui.add(egui::Label::new("<empty>").selectable(false));
            return;
        };

        match (&self.desc.ty, value) {
            (&Ty::Blob, Value::Blob(data)) => {
                let text = format!("{} bytes", data.len());
                ui.add(egui::Label::new(text).selectable(false));
            }
            (&Ty::Text { singleline }, Value::Text(text)) => {
                let edit = if singleline {
                    egui::TextEdit::singleline(text)
                } else {
                    egui::TextEdit::multiline(text)
                };
                ui.add(edit.frame(true));
            }
            (&Ty::Link { table, many }, Value::Link(items)) => {
                ui.horizontal_wrapped(|ui| {
                    let text = format!("{table:?}");
                    ui.add(egui::Label::new(text).selectable(false));
                    for &item in items.iter() {
                        let text = format!("{item:?}");
                        ui.add(egui::Label::new(text).selectable(false));
                    }
                });
            }
            (&Ty::Enum { ref variants, many }, Value::Enum(items)) => {
                ui.horizontal_wrapped(|ui| {
                    for text in items.iter() {
                        egui::Frame::canvas(ui.style())
                            .corner_radius(4)
                            .inner_margin(egui::Margin::symmetric(8, 0))
                            .show(ui, |ui| {
                                ui.add(egui::Label::new(text).selectable(false));
                            });
                    }
                });
            }
            (this, value) => unreachable!("{:?} {:?}", this, value),
        }
    }
}

fn drag_handle(painter: &egui::Painter, rect: egui::Rect, fill: egui::Color32) {
    let (x, y) = (2.0, 4.0);
    let pts = [
        egui::vec2(-x, -y),
        egui::vec2(-x, 0.0),
        egui::vec2(-x, y),
        egui::vec2(x, -y),
        egui::vec2(x, 0.0),
        egui::vec2(x, y),
    ];

    let center = rect.center();

    for p in pts {
        // painter.circle_filled(center + p, radius, fill);
        let c = center + p;
        let r = egui::Rect::from_center_size(c, egui::vec2(1.5, 1.5));
        painter.rect_filled(r, 0, fill);
    }
}

fn allocate_icon(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    icon: impl FnOnce(egui::Painter, egui::Rect),
) {
    let (response, painter) = ui.allocate_painter(size, egui::Sense::empty());
    icon(painter, response.rect)
}

impl DatabaseEditor {
    fn table_modal(&mut self, ui: &mut egui::Ui, table: Table) {
        if !self.edit_fields {
            return;
        }

        let id = egui::Id::new("Table Modal");

        let modal = egui::Modal::new(id).show(ui.ctx(), |ui| {
            use sizing::Size;

            ui.set_width(300.0);
            ui.heading("Save? Are you sure?");

            ui.add_space(32.0);

            let table = &mut self.db.tables[table];

            for (field, meta) in &mut table.fields {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(100.0);
                        ui.strong(&meta.desc.name);
                    });

                    ui.vertical(|ui| {
                        let ty = &mut meta.desc.ty;
                        let selected = match ty {
                            Ty::Blob => "Blob",
                            Ty::Text { singleline: true } => "Singleline text",
                            Ty::Text { singleline: false } => "Multiline text",
                            Ty::Link { .. } => "Link to another table",
                            Ty::Enum { .. } => "Enum",
                        };

                        let combo = egui::ComboBox::from_id_salt(field)
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                ui.set_width(150.0);
                                ui.selectable_label(matches!(ty, Ty::Blob), "Blob");
                                ui.selectable_label(
                                    matches!(ty, Ty::Text { singleline: true }),
                                    "Singleline text",
                                );
                                ui.selectable_label(
                                    matches!(ty, Ty::Text { singleline: false }),
                                    "Multiline text",
                                );
                                ui.selectable_label(
                                    matches!(ty, Ty::Link { .. }),
                                    "Link to another table",
                                );
                                ui.selectable_label(matches!(ty, Ty::Enum { .. }), "Enum");
                            });

                        /*
                        match &meta.desc.ty {
                            Ty::Blob => ui.label("blob"),
                            Ty::Text { singleline } => ui.label("text"),
                            Ty::Link { table, many } => ui.label("link"),
                            Ty::Enum { variants, many } => ui.label("enum"),
                        };

                        // ui.label("");
                        */
                    });
                });
            }

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    // if ui.button("Yes Please").clicked() {
                    //     *save_progress = Some(0.0);
                    // }

                    if ui.button("No Thanks").clicked() {
                        ui.close();
                    }
                },
            );
        });

        if modal.should_close() {
            self.edit_fields = false;
        }
    }
}
