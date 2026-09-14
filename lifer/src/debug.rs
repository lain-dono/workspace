use bevy::prelude::*;
use bevy::{color::palettes::css::*, render::view::ColorGrading};
use bevy_egui::{egui, EguiContexts};

pub fn plugin(app: &mut App) {
    app.init_gizmo_group::<DebugGizmos>()
        .add_systems(Update, ui_color_grading)
        .add_systems(PostUpdate, (update_config, draw_gizmos).chain());
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DebugGizmos {}

fn draw_gizmos(
    mut gizmos: Gizmos<DebugGizmos>,
    characters: Query<(
        &Transform,
        &crate::character::CachedFinder,
        Has<crate::player::Player>,
    )>,
    targets: Query<&Transform>,
) {
    for (ch, target, is_player) in &characters {
        let color = if is_player { RED } else { YELLOW };
        if let Some(target) = target.target().and_then(|entity| targets.get(entity).ok()) {
            gizmos.line(ch.translation, target.translation, color);
            gizmos
                .sphere(target.to_isometry(), 0.2, color)
                .resolution(12);
        }
    }
}

fn update_config(mut config_store: ResMut<GizmoConfigStore>) {
    let depth = true;
    let (config, _) = config_store.config_mut::<DebugGizmos>();
    config.depth_bias = if depth { -1. } else { 0. };
}

fn ui_color_grading(mut contexts: EguiContexts, mut query: Query<Mut<ColorGrading>>) -> Result {
    let mut color_grading = query.single_mut()?;

    fn label(ui: &mut egui::Ui, label: &str) {
        ui.add_sized(egui::vec2(100.0, 18.0), egui::Label::new(label));
    }

    struct ColorGradingEditor<'a, 'b> {
        color_grading: &'a mut Mut<'b, ColorGrading>,
    }

    impl ColorGradingEditor<'_, '_> {
        fn drag(
            &mut self,
            ui: &mut egui::Ui,
            f: impl FnOnce(&mut ColorGrading) -> &mut f32,
            prefix: &str,
        ) -> egui::Response {
            let mut value = self.color_grading.reborrow().map_unchanged(f);
            let mut new_value = *value;

            let widget = egui::DragValue::new(&mut new_value);
            let max_size = egui::vec2(100.0, 18.0);
            let response =
                ui.add_sized(max_size, widget.prefix(format!("{prefix}: ")).speed(0.001));

            if new_value != *value {
                *value = new_value;
            }

            response
        }

        fn drag_hue(
            &mut self,
            ui: &mut egui::Ui,
            f: impl FnOnce(&mut ColorGrading) -> &mut f32,
        ) -> egui::Response {
            let mut radians = self.color_grading.reborrow().map_unchanged(f);

            let mut degrees = radians.to_degrees();
            let widget = egui::DragValue::new(&mut degrees);
            let widget = widget.prefix("hue: ").suffix("°");
            let max_size = egui::vec2(100.0, 18.0);
            let mut response = ui.add_sized(max_size, widget.speed(0.1));

            // only touch `*radians` if we actually changed the degree value
            if degrees != radians.to_degrees() {
                *radians = degrees.to_radians();
                response.mark_changed();
            }

            response
        }
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::from("#COLOR_GRADING"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [0.0; 2])
        .show(ctx, |ui| {
            egui::Frame::side_top_panel(ui.style()).show(ui, |ui| {
                let mut editor = ColorGradingEditor {
                    color_grading: &mut color_grading,
                };

                ui.horizontal(|ui| {
                    label(ui, "global");
                    editor.drag(ui, |cg| &mut cg.global.exposure, "exposure");
                    editor.drag(ui, |cg| &mut cg.global.temperature, "temperature");
                    editor.drag(ui, |cg| &mut cg.global.tint, "tint");
                    editor.drag_hue(ui, |cg| &mut cg.global.hue);
                });

                ui.horizontal(|ui| {
                    label(ui, "highlights");
                    editor.drag(ui, |cg| &mut cg.highlights.saturation, "saturation");
                    editor.drag(ui, |cg| &mut cg.highlights.contrast, "contrast");
                    editor.drag(ui, |cg| &mut cg.highlights.gamma, "gamma");
                    editor.drag(ui, |cg| &mut cg.highlights.gain, "gain");
                    editor.drag(ui, |cg| &mut cg.highlights.lift, "lift");
                });
                ui.horizontal(|ui| {
                    label(ui, "midtones");
                    editor.drag(ui, |cg| &mut cg.midtones.saturation, "saturation");
                    editor.drag(ui, |cg| &mut cg.midtones.contrast, "contrast");
                    editor.drag(ui, |cg| &mut cg.midtones.gamma, "gamma");
                    editor.drag(ui, |cg| &mut cg.midtones.gain, "gain");
                    editor.drag(ui, |cg| &mut cg.midtones.lift, "lift");
                });
                ui.horizontal(|ui| {
                    label(ui, "shadows");
                    editor.drag(ui, |cg| &mut cg.shadows.saturation, "saturation");
                    editor.drag(ui, |cg| &mut cg.shadows.contrast, "contrast");
                    editor.drag(ui, |cg| &mut cg.shadows.gamma, "gamma");
                    editor.drag(ui, |cg| &mut cg.shadows.gain, "gain");
                    editor.drag(ui, |cg| &mut cg.shadows.lift, "lift");
                });
            });
        });

    Ok(())
}
