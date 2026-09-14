use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use bevy::winit::WinitWindows;
use bevy_egui::{egui, EguiContexts};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

const MAX_WIDTH: f32 = 500.0;

#[derive(Resource)]
pub struct SettingsMenu;

#[derive(Event)]
pub struct PersistSettings;

#[derive(Resource, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub present_mode: PresentMode,
    pub window_mode: WindowMode,
    pub width: u32,
    pub height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            present_mode: PresentMode::AutoVsync,
            window_mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            width: 1920,
            height: 1080,
        }
    }
}

impl Settings {
    fn load(directory: &Path) -> Self {
        let path = directory.join("settings.toml");
        if !path.exists() {
            Settings::default()
        } else {
            bevy::log::info!("loading settings {path:?}");

            std::fs::read_to_string(path)
                .ok()
                .and_then(|data| toml::from_str(&data).ok())
                .unwrap_or_default()
        }
    }

    fn save(&self, directory: &Path) {
        let contents = toml::to_string(self).expect("Couldn't serialize the settings to toml");
        std::fs::create_dir_all(directory)
            .expect("Couldn't create the folders for the settings file");
        std::fs::write(directory.join("settings.toml"), contents)
            .expect("couldn't persist the settings while trying to write the string to disk");
    }
}

#[derive(Resource, Debug)]
pub struct SettingsPath {
    pub directory: PathBuf,
}

pub struct SettingsPlugin {
    pub qualifier: String,
    pub organization: String,
    pub application: String,
}

impl SettingsPlugin {
    pub fn new(
        qualifier: impl Into<String>,
        organization: impl Into<String>,
        application: impl Into<String>,
    ) -> Self {
        Self {
            qualifier: qualifier.into(),
            organization: organization.into(),
            application: application.into(),
        }
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let local_save = Path::new("save").is_dir().then_some(PathBuf::from("save"));

        let directory = local_save.unwrap_or_else(|| {
            ProjectDirs::from(&self.qualifier, &self.organization, &self.application)
                .expect("Couldn't find a folder to store the settings")
                .config_dir()
                .to_path_buf()
        });

        let settings = Settings::load(&directory);

        app.insert_resource(settings)
            .insert_resource(SettingsPath { directory })
            .add_event::<PersistSettings>()
            .add_systems(Startup, apply_settings)
            .add_systems(
                Update,
                (
                    editor.run_if(resource_exists::<SettingsMenu>),
                    persist.after(editor),
                ),
            );
    }
}

fn apply_settings(settings: Res<Settings>, mut windows: Query<&mut Window>) {
    if settings.is_changed() {
        let mut window = windows.single_mut();

        window.present_mode = settings.present_mode;
        window.mode = settings.window_mode;

        let (width, height) = (settings.width, settings.height);
        // window.resolution.set_physical_resolution(width, height);
        window.resolution.set(width as f32, height as f32);

        // // window.resolution.set(mode.width as f32, mode.height as f32);
        // window
        //     .resolution
        //     .set_physical_resolution(mode.width, mode.height);

        // window.resolution.set_scale_factor_override(Some(1.0));
    }
}

fn persist(settings: Res<Settings>, cfg: Res<SettingsPath>, ev: EventReader<PersistSettings>) {
    if !ev.is_empty() {
        settings.save(&cfg.directory);
    }
}

#[derive(SystemParam)]
struct SettingsParam<'w, 's> {
    settings: ResMut<'w, Settings>,
    writer: EventWriter<'w, PersistSettings>,
    windows: Query<'w, 's, (Entity, &'static mut Window)>,
    winit: NonSend<'w, WinitWindows>,
}

fn editor(mut commands: Commands, mut contexts: EguiContexts, mut opt: SettingsParam) {
    egui::Area::new(egui::Id::new("#SETTINGS"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(contexts.ctx_mut(), |ui| {
            ui.set_max_width(MAX_WIDTH);
            ui.vertical_centered_justified(|ui| {
                crate::menu_style(ui, |ui| {
                    ui.heading("Options");

                    let (mut window, monitor) = {
                        let (entity, window) = opt.windows.single_mut();

                        let monitor = opt
                            .winit
                            .get_window(entity)
                            .and_then(|window| window.current_monitor());
                        (window, monitor)
                    };

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
                            .selected_text(format!("{}x{}", width, height))
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
                        opt.writer.send(PersistSettings);
                        commands.remove_resource::<SettingsMenu>();
                    }
                });
            });
        });
}
