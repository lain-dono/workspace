use crate::reflect_editor::{Arg, ReflectEditor};
use bevy::render::view::RenderLayers;
use std::any::Any;

pub fn render_layers_ref(_: ReflectEditor, Arg { ui, .. }: Arg, value: &dyn Any) {
    let value = value.downcast_ref::<RenderLayers>().unwrap();

    for layer in value.iter() {
        ui.label(format!("- {layer}"));
    }
}

pub fn render_layers_mut(_: ReflectEditor, Arg { ui, id, .. }: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<RenderLayers>().unwrap();

    let mut new_value = None;
    egui::Grid::new(id).num_columns(2).show(ui, |ui| {
        for layer in value.iter() {
            let mut layer_copy = layer;
            let widget = egui::DragValue::new(&mut layer_copy)
                .clamp_range(0..=RenderLayers::TOTAL_LAYERS - 1);
            if ui.add(widget).changed() {
                new_value = Some(value.without(layer).with(layer_copy));
            }
            if ui.button("-").clicked() {
                new_value = Some(value.without(layer));
            }
            ui.end_row();
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Add").clicked() {
            let new_layer = value.iter().last().map_or(0, |last| last + 1);
            new_value = Some(value.with(new_layer));
        }
    });

    if let Some(new_value) = new_value {
        *value = new_value;
        true
    } else {
        false
    }
}
