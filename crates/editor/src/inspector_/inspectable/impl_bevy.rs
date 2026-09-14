use super::{add, InspectorUi};
use bevy::{
    asset::UntypedAssetId,
    prelude::Entity,
    reflect::{FromReflect, Reflect, TypeRegistry},
    render::color::Color,
};
use egui::{Id, Ui};

pub fn register(registry: &mut TypeRegistry) {
    add::<UntypedAssetId>(registry, handle_id);
    add::<Entity>(registry, entity);
    add::<Color>(registry, color);
}

fn handle_id(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let value = reflect.downcast_mut::<UntypedAssetId>().unwrap();
    ui.label(format!("{:?}", value));
    false
}

fn entity(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let value = reflect.downcast_mut::<Entity>().unwrap();
    ui.label(format!("{:?}", value));
    false
}

fn color(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    //let value = reflect.downcast_mut::<Color>().unwrap();

    let mut value = Color::from_reflect(reflect).unwrap();

    if let Color::Rgba {
        red,
        green,
        blue,
        alpha,
    } = &mut value
    {
        let mut rgba = egui::Rgba::from_rgba_premultiplied(*red, *green, *blue, *alpha);
        let alpha_mode = egui::widgets::color_picker::Alpha::Opaque;
        egui::widgets::color_picker::color_edit_button_rgba(ui, &mut rgba, alpha_mode);
        *red = rgba.r();
        *green = rgba.g();
        *blue = rgba.b();
        *alpha = rgba.a();
    }

    reflect.apply(&value);

    false
}
