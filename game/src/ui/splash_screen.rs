use crate::state::AppState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Splash), on_enter)
        .add_systems(
            EguiPrimaryContextPass,
            splash_ui.run_if(in_state(AppState::Splash)),
        )
        .add_systems(OnExit(AppState::Splash), on_exit);
}

#[derive(Resource)]
struct SplashState {
    timer: Timer,
    logo: egui::TextureId,
}

fn on_enter(mut commands: Commands, mut contexts: EguiContexts, assets: Res<AssetServer>) {
    // bevy.png just for now
    commands.insert_resource(SplashState {
        logo: contexts.add_image(EguiTextureHandle::Strong(assets.load("bevy.png"))),
        timer: Timer::from_seconds(2.0, TimerMode::Once),
    });
}

fn on_exit(mut commands: Commands) {
    // also free splash screen image
    commands.remove_resource::<SplashState>();
}

fn splash_ui(
    time: Res<Time>,
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<AppState>>,
    mut state: ResMut<SplashState>,
) -> Result {
    if state.timer.tick(time.delta()).is_finished() {
        game_state.set(AppState::Loading);
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("#SPLASH_SCREEN"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
        .show(ctx, |ui| {
            let max = 128.0;
            let base = 120.0;
            let delta = max - base;

            let (_, rect) = ui.allocate_space(egui::vec2(base, base));

            let amount = delta * state.timer.fraction();
            let scaled_size = [delta + amount; 2];

            let source = egui::load::SizedTexture::new(state.logo, scaled_size);
            let image = egui::widgets::Image::new(source);

            image.paint_at(ui, rect.expand(amount));
        });

    Ok(())
}
