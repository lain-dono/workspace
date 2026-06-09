use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

#[derive(Message)]
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
            .add_message::<PersistSettings>()
            .add_systems(Startup, apply_settings)
            .add_systems(PostUpdate, persist);
    }
}

fn apply_settings(settings: Res<Settings>, mut windows: Query<&mut Window>) -> Result {
    if settings.is_changed() {
        let mut window = windows.single_mut()?;

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

    Ok(())
}

pub fn persist(
    settings: Res<Settings>,
    cfg: Res<SettingsPath>,
    ev: MessageReader<PersistSettings>,
) {
    if !ev.is_empty() {
        settings.save(&cfg.directory);
    }
}
