use crate::settings::{PersistSettings, Settings};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use bevy::winit::WinitWindows;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

const MAX_WIDTH: f32 = 500.0;

#[derive(Resource)]
pub struct SettingsMenu;

pub fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        editor.run_if(resource_exists::<SettingsMenu>),
    );
}

#[derive(SystemParam)]
struct SettingsParam<'w, 's> {
    settings: ResMut<'w, Settings>,
    writer: MessageWriter<'w, PersistSettings>,
    windows: Query<'w, 's, (Entity, &'static mut Window)>,
    winit: NonSend<'w, WinitWindows>,
}

fn editor(mut commands: Commands, mut contexts: EguiContexts, mut opt: SettingsParam) -> Result {
    let (mut window, monitor) = {
        let (entity, window) = opt.windows.single_mut()?;

        let monitor = opt
            .winit
            .get_window(entity)
            .and_then(|window| window.current_monitor());
        (window, monitor)
    };

    egui::Area::new(egui::Id::new("#SETTINGS"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(contexts.ctx_mut()?, |ui| {
            ui.set_max_width(MAX_WIDTH);
            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
                ui.spacing_mut().button_padding = egui::vec2(0.0, 32.0);

                ui.heading("Options");

                // egui::ComboBox::from_id_source("Select present mode")
                //     .width(MAX_WIDTH)
                //     .selected_text(format!("{:?}", window.present_mode))
                //     .show_ui(ui, |ui| {
                //         let current = &mut window.present_mode;
                //         ui.selectable_value(current, PresentMode::AutoVsync, "AutoVsync");
                //         ui.selectable_value(current, PresentMode::AutoNoVsync, "AutoNoVsync");
                //         ui.selectable_value(current, PresentMode::Fifo, "Fifo");
                //         ui.selectable_value(current, PresentMode::FifoRelaxed, "FifoRelaxed");
                //         ui.selectable_value(current, PresentMode::Immediate, "Immediate");
                //         ui.selectable_value(current, PresentMode::Mailbox, "Mailbox");
                //     });

                // egui::ComboBox::from_id_source("Select window mode")
                //     .width(MAX_WIDTH)
                //     .selected_text(format!("{:?}", window.mode))
                //     .show_ui(ui, |ui| {
                //         let current = &mut window.mode;
                //         ui.selectable_value(current, WindowMode::Windowed, "Windowed");
                //         ui.selectable_value(
                //             current,
                //             WindowMode::BorderlessFullscreen,
                //             "BorderlessFullscreen",
                //         );
                //         ui.selectable_value(
                //             current,
                //             WindowMode::SizedFullscreen,
                //             "SizedFullscreen",
                //         );
                //         ui.selectable_value(current, WindowMode::Fullscreen, "Fullscreen");
                //     });

                ui.horizontal(|ui| {
                    let mut vsync = matches!(window.present_mode, PresentMode::AutoVsync);
                    ui.checkbox(&mut vsync, "Vsync");
                    window.present_mode = if vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    };
                });

                ui.horizontal(|ui| {
                    let mut fullscreen = matches!(
                        window.mode,
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    );
                    ui.checkbox(&mut fullscreen, "Fullscreen");
                    window.mode = if fullscreen {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    } else {
                        WindowMode::Windowed
                    };
                });

                if let Some(monitor) = monitor {
                    let mut modes = monitor.video_modes().collect::<Vec<_>>();
                    modes.sort_by(|a, b| {
                        let width = b.size().width.cmp(&a.size().width);
                        let height = b.size().height.cmp(&a.size().height);
                        let rate = b
                            .refresh_rate_millihertz()
                            .cmp(&a.refresh_rate_millihertz());

                        width.then(height).then(rate)
                    });

                    let width = window.physical_width();
                    let height = window.physical_height();

                    egui::ComboBox::from_id_salt("Select window resolution")
                        .width(MAX_WIDTH)
                        .selected_text(format!("{width}x{height}"))
                        .show_ui(ui, |ui| {
                            for mode in modes {
                                let size = mode.size();
                                let text = format!("{}x{}", size.width, size.height);
                                let selected = size.width == width && size.height == height;
                                if ui.add(egui::Button::selectable(selected, text)).clicked() {
                                    opt.settings.width = size.width;
                                    opt.settings.height = size.height;
                                }
                            }
                        });
                }

                opt.settings.present_mode = window.present_mode;
                opt.settings.window_mode = window.mode;

                if ui.button("Close").clicked() {
                    opt.writer.write(PersistSettings);
                    commands.remove_resource::<SettingsMenu>();
                }
            });
        });

    Ok(())
}
