use super::{add, InspectorUi};
use bevy::reflect::{Reflect, TypeRegistry};
use egui::{DragValue, Id, Ui};
use std::borrow::Cow;

pub fn register(registry: &mut TypeRegistry) {
    add::<u8>(registry, num_drag::<u8>);
    add::<u16>(registry, num_drag::<u16>);
    add::<u32>(registry, num_drag::<u32>);
    add::<u64>(registry, num_drag::<u64>);
    add::<usize>(registry, num_drag::<usize>);

    add::<i8>(registry, num_drag::<i8>);
    add::<i16>(registry, num_drag::<i16>);
    add::<i32>(registry, num_drag::<i32>);
    add::<i64>(registry, num_drag::<i64>);
    add::<isize>(registry, num_drag::<isize>);

    add::<f32>(registry, num_drag::<f32>);
    add::<f64>(registry, num_drag::<f64>);

    add::<bool>(registry, bool_checkbox);

    add::<Cow<'static, str>>(registry, cow_str_mut);
    add::<String>(registry, string_mut);

    // TODO: std::time::Duration, bevy::utils::Instant
}

fn bool_checkbox(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let value = reflect.downcast_mut::<bool>().unwrap();
    ui.columns(1, |ui| ui[0].checkbox(value, "").changed())
}

fn num_drag<T: Reflect + egui::emath::Numeric>(
    _: &InspectorUi,
    ui: &mut Ui,
    _: Id,
    reflect: &mut dyn Reflect,
) -> bool {
    let value = reflect.downcast_mut::<T>().unwrap();
    ui.columns(1, |ui| {
        let widget = DragValue::new(value).speed(0.1);
        ui[0].add(widget).changed()
    })
}

fn cow_str_mut(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let value = reflect.downcast_mut::<Cow<'static, str>>().unwrap();
    ui.columns(1, |ui| ui[0].text_edit_singleline(value.to_mut()).changed())
}

fn string_mut(_: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let value = reflect.downcast_mut::<String>().unwrap();
    ui.columns(1, |ui| ui[0].text_edit_singleline(value).changed())
}
