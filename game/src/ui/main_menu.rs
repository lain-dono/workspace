use crate::state::AppState;
use crate::ui::{settings::SettingsMenu, style};
use bevy::{app::AppExit, prelude::*};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        main_menu_ui.run_if(in_state(AppState::MainMenu).and(not(resource_exists::<SettingsMenu>))),
    );
}

fn main_menu_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut app_exit_events: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::from("#MAIN_MENU"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
            ui.spacing_mut().button_padding = egui::vec2(0.0, 32.0);
            ui.set_max_width(300.0);

            ui.vertical_centered_justified(|ui| {
                if ui.button(style::rich("menu", "New game")).clicked() {
                    next_state.set(AppState::InGame { paused: false });
                }
                if ui.button(style::rich("menu", "Settings")).clicked() {
                    commands.insert_resource(SettingsMenu);
                }
                if ui.button(style::rich("menu", "Quit")).clicked() {
                    app_exit_events.write(AppExit::Success);
                }
            });
        });

    Ok(())
}
