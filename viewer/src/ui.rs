use bevy::{
    post_process::bloom::{Bloom, BloomCompositeMode},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::ops::RangeInclusive;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, (ui_example_system, ui_bloom));
}

fn ui_example_system(mut contexts: EguiContexts) -> Result {
    Ok(())
}

fn ui_bloom(
    mut contexts: EguiContexts,
    camera: Single<(Entity, Option<&mut Bloom>), With<Camera>>,
) -> Result {
    let (_entity, bloom) = camera.into_inner();
    let Some(mut bloom) = bloom else {
        return Ok(());
    };

    egui::Window::new("Bloom").show(contexts.ctx_mut()?, |ui| {
        ui.label("Intensity");
        drag(ui, &mut bloom.intensity, 0.0..=1.0, 0.01);

        ui.label("Low frequency boost");
        drag(ui, &mut bloom.low_frequency_boost, 0.0..=1.0, 0.01);
        ui.label("Low frequency boost curvative");
        drag(
            ui,
            &mut bloom.low_frequency_boost_curvature,
            0.0..=1.0,
            0.01,
        );
        ui.label("High pass frequency");
        drag(ui, &mut bloom.high_pass_frequency, 0.0..=1.0, 0.01);

        let checked = bloom.composite_mode == BloomCompositeMode::Additive;
        if ui.selectable_label(checked, "Additive").clicked() {
            bloom.composite_mode = BloomCompositeMode::Additive;
        }
        let checked = bloom.composite_mode == BloomCompositeMode::EnergyConserving;
        if ui.selectable_label(checked, "EnergyConserving").clicked() {
            bloom.composite_mode = BloomCompositeMode::EnergyConserving;
        }

        ui.label("Prefilter threshold");
        drag(ui, &mut bloom.prefilter.threshold, 0.0..=10.0, 0.1);
        ui.label("Prefilter threshold softness");
        drag(ui, &mut bloom.prefilter.threshold_softness, 0.0..=1.0, 0.01);

        ui.label("Scale x");
        drag(ui, &mut bloom.scale.x, 0.0..=8.0, 0.1);
        ui.label("Scale y");
        drag(ui, &mut bloom.scale.y, 0.0..=8.0, 0.1);
    });

    Ok(())
}

fn drag(ui: &'_ mut egui::Ui, value: &'_ mut f32, range: RangeInclusive<f32>, speed: f32) {
    ui.add(egui::DragValue::new(value).range(range).speed(speed));
}
