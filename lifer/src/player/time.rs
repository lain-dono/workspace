use crate::{settings::SettingsMenu, state::AppState};
use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
// use bevy::window::{CursorGrabMode, PrimaryWindow};

pub fn time_ui(mut time: ResMut<Time<Virtual>>, mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("#TIME_HUD"))
        .anchor(egui::Align2::RIGHT_TOP, [-12.0, 12.0])
        .show(ctx, |ui| {
            ui.scope(|ui| {
                egui::Frame::side_top_panel(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let pause = egui::Button::new("\u{23F8}");
                        let speed_1 = egui::Button::new("x1");
                        let speed_2 = egui::Button::new("x2");
                        let speed_3 = egui::Button::new("x3");

                        let speed = time.effective_speed();

                        if ui.add_enabled(!time.is_paused(), pause).clicked() {
                            time.pause();
                        }
                        if ui.add_enabled(speed != 1.0, speed_1).clicked() {
                            time.set_relative_speed(1.0);
                            time.unpause();
                        }
                        if ui.add_enabled(speed != 2.0, speed_2).clicked() {
                            time.set_relative_speed(2.0);
                            time.unpause();
                        }
                        if ui.add_enabled(speed != 3.0, speed_3).clicked() {
                            time.set_relative_speed(3.0);
                            time.unpause();
                        }
                    });
                });
            });
        });

    Ok(())
}

pub fn pause_on_esc(mut time: ResMut<Time<Virtual>>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

pub fn pause_menu(
    mut time: ResMut<Time<Virtual>>,
    mut commands: Commands,
    mut contexts: EguiContexts,
    // mut ctx: MenuParam,
    // primary_window: Query<&Window, With<PrimaryWindow>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    /*
    let Ok(window) = primary_window.get_single() else {
        warn!("Primary window not found for `player_move`!");
        return;
    };

    if !matches!(window.cursor.grab_mode, CursorGrabMode::None) {
        return;
    }
    */

    if !time.is_paused() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::from("#PAUSE_MENU"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(ctx, |ui| {
            crate::menu_style(ui, |ui| {
                ui.set_max_width(300.0);

                ui.vertical_centered_justified(|ui| {
                    ui.heading("Pause");

                    if ui.button("Resume").clicked() {
                        time.unpause();
                    }

                    // if ui.button("Save Game").clicked() {}
                    // if ui.button("Load Game").clicked() {}

                    if ui.button("Options").clicked() {
                        commands.insert_resource(SettingsMenu);
                    }

                    if ui.button("Main Menu").clicked() {
                        next_state.set(AppState::MainMenu);
                    }

                    if ui.button("Quit Game").clicked() {
                        app_exit_events.write(AppExit::Success);
                    }
                });
            });
        });

    Ok(())
}
