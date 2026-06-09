pub mod game;
pub mod loading;
pub mod main_menu;
pub mod settings;
pub mod splash_screen;
pub mod style;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        self::style::plugin,
        self::loading::plugin,
        self::splash_screen::plugin,
        self::main_menu::plugin,
        self::settings::plugin,
        self::game::plugin,
    ));
}
