#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

pub mod database;
mod log;
mod map;
mod mark;
mod stat;
mod time;

mod w;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            cc.egui_ctx.style_mut(|_style| {
                // style.visuals.button_frame = false;
                //
            });

            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Tabs {
    Game,
    Mark,
    Table,
}

struct MyApp {
    name: String,
    age: u32,

    tabs: Tabs,

    log: Vec<String>,
    stats: Vec<stat::Stat>,
    actions: Vec<log::ActionButton>,

    map: map::Map,

    editor: mark::EasyMarkEditor,

    db: database::DatabaseEditor,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,

            tabs: Tabs::Table,

            log: {
                std::fs::read_to_string("log.txt")
                    .unwrap()
                    .lines()
                    .filter_map(|s| {
                        let s = s.trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    })
                    .collect()
            },
            stats: stat::stats(),

            actions: vec![
                log::action(1, "Enter the inn", time::clock(5, 6, 7)),
                log::action(2, "Use the willage well", time::clock(5, 6, 7)),
                log::action(3, "Go to the workshop", time::clock(5, 6, 7)),
                log::action(4, "Go to the plasa", time::clock(5, 6, 7)),
                log::action(5, "Take the road to the checkpoint", time::clock(5, 6, 7)),
            ],

            map: map::Map::default(),
            editor: mark::EasyMarkEditor::default(),

            db: database::DatabaseEditor::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top")
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.tabs, Tabs::Game, "Game");
                    ui.selectable_value(&mut self.tabs, Tabs::Mark, "Mark");
                    ui.selectable_value(&mut self.tabs, Tabs::Table, "Table");
                });
            });

        match self.tabs {
            Tabs::Game => {
                egui::SidePanel::left("log")
                    .min_width(300.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for p in &self.log {
                                ui.label(p);
                            }
                            for action in self.actions.clone() {
                                if ui.add(action.clone()).clicked() {
                                    self.log.push(action.label);
                                }
                            }

                            ui.scroll_to_cursor(None);
                        });
                    });

                egui::TopBottomPanel::bottom("stats")
                    .resizable(true)
                    .show(ctx, |ui| {
                        for stat in self.stats.clone() {
                            ui.add(stat);
                        }
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("My egui Application");
                    ui.horizontal(|ui| {
                        let name_label = ui.label("Your name: ");
                        ui.text_edit_singleline(&mut self.name)
                            .labelled_by(name_label.id);
                    });
                    ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                    if ui.button("Increment").clicked() {
                        self.age += 1;
                    }
                    ui.label(format!("Hello '{}', age {}", self.name, self.age));

                    //     ui.image(egui::include_image!(
                    //         "../../../crates/egui/assets/ferris.png"
                    //     ));

                    let mut open = true;
                    self.map.show(ui, &mut open);
                });
            }
            Tabs::Mark => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.editor.ui(ui);
                });
            }
            Tabs::Table => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // egui::Frame::group(ui.style()).show(ui, |ui| {
                    self.db.ui(ui);
                    // });
                });
            }
        }
    }
}
