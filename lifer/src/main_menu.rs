use crate::{settings::SettingsMenu, state::AppState};
use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        main_menu_ui.run_if(in_state(AppState::MainMenu).and(not(resource_exists::<SettingsMenu>))),
    );
}

fn main_menu_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut app_exit_events: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::from("#MAIN_MENU"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(ctx, |ui| {
            crate::menu_style(ui, |ui| {
                ui.set_max_width(300.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.button("New game").clicked() {
                        next_state.set(AppState::InGame);
                    }
                    if ui.button("Settings").clicked() {
                        commands.insert_resource(SettingsMenu);
                    }
                    if ui.button("Quit").clicked() {
                        app_exit_events.write(AppExit::Success);
                    }
                });
            });
        });

    Ok(())
}
