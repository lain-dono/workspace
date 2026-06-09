use crate::state;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui::{FontFamily, FontFamily::Monospace, FontFamily::Proportional, FontId, TextStyle};

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(state::AppState::Splash), init_style);
}

pub fn rich(style: &str, text: impl Into<String>) -> egui::RichText {
    let text = egui::RichText::new(text).text_style(TextStyle::Name(style.into()));
    match style {
        "clock" => text.strong(),
        "stat" => text.strong(),
        "speed_btn" => text.strong().extra_letter_spacing(-8.0),
        _ => text,
    }
}

fn init_style(mut contexts: EguiContexts) -> Result {
    fn named(name: &str, size: f32, family: FontFamily) -> (TextStyle, FontId) {
        (TextStyle::Name(name.into()), FontId::new(size, family))
    }

    contexts.ctx_mut()?.all_styles_mut(move |style| {
        let visuals = [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ];

        for widget in visuals {
            widget.corner_radius = (0.0).into();
            widget.expansion = 0.0;
            widget.bg_stroke = (0.0, widget.bg_stroke.color).into();
        }

        style.text_styles = [
            // default
            (TextStyle::Small, FontId::new(9.0, Proportional)),
            (TextStyle::Body, FontId::new(12.5, Proportional)),
            (TextStyle::Button, FontId::new(12.5, Proportional)),
            (TextStyle::Heading, FontId::new(18.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            //
            named("loading", 34.0, Proportional),
            named("menu", 24.0, Proportional),
            named("Heading2", 25.0, Proportional),
            named("Context", 23.0, Proportional),
            //
            named("clock", 12.0, Monospace),
            named("calendar", 12.5, Proportional),
            named("stat", 12.5, Proportional),
            named("speed_btn", 12.5, Proportional),
        ]
        .into()
    });
    Ok(())
}
