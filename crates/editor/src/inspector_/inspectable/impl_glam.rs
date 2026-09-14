use super::{add, InspectorUi};
use bevy::math::{EulerRot, Quat, Vec2, Vec3, Vec4};
use bevy::reflect::{FromReflect, Reflect, TypeRegistry};
use egui::{DragValue, Id, Ui};

pub fn register(registry: &mut TypeRegistry) {
    add::<Vec2>(registry, vec2);
    add::<Vec3>(registry, vec3);
    add::<Vec4>(registry, vec4);
    add::<Quat>(registry, quat);
}

fn drag_f32(ui: &mut Ui, value: &mut f32) -> bool {
    ui.add(DragValue::new(value).speed(0.1)).changed()
}

fn vec2(i: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let mut value = Vec2::from_reflect(reflect).unwrap();

    let num = if i.wrapping { 2 } else { 1 };
    let changed = ui.columns(num, |ui| {
        let mut changed = false;
        changed |= drag_f32(&mut ui[0.min(num - 1)], &mut value.x);
        changed |= drag_f32(&mut ui[1.min(num - 1)], &mut value.y);
        changed
    });

    if changed {
        reflect.apply(&value);
    }
    changed
}

fn vec3(i: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    let mut value = Vec3::from_reflect(reflect).unwrap();

    let num = if i.wrapping { 3 } else { 1 };
    let changed = ui.columns(num, |ui| {
        let mut changed = false;
        changed |= drag_f32(&mut ui[0.min(num - 1)], &mut value.x);
        changed |= drag_f32(&mut ui[1.min(num - 1)], &mut value.y);
        changed |= drag_f32(&mut ui[2.min(num - 1)], &mut value.z);
        changed
    });

    if changed {
        reflect.apply(&value);
    }
    changed
}

fn vec4(i: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    dbg!("vec4");

    let value = reflect.downcast_mut::<Vec4>().unwrap();
    let num = if i.wrapping { 4 } else { 1 };
    ui.columns(num, |ui| {
        let mut changed = false;
        changed |= drag_f32(&mut ui[0.min(num - 1)], &mut value.x);
        changed |= drag_f32(&mut ui[1.min(num - 1)], &mut value.x);
        changed |= drag_f32(&mut ui[2.min(num - 1)], &mut value.z);
        changed |= drag_f32(&mut ui[4.min(num - 1)], &mut value.w);
        changed
    })
}

fn quat(i: &InspectorUi, ui: &mut Ui, _: Id, reflect: &mut dyn Reflect) -> bool {
    dbg!("quat");

    let value = reflect.downcast_mut::<Quat>().unwrap();

    let euler = EulerRot::YXZ;
    let (mut y, mut x, mut z) = value.to_euler(euler);

    let mut changed = false;
    let num = if i.wrapping { 3 } else { 1 };
    ui.columns(num, |ui| {
        changed |= ui[0.min(num - 1)].drag_angle(&mut y).changed();
        changed |= ui[1.min(num - 1)].drag_angle(&mut x).changed();
        changed |= ui[2.min(num - 1)].drag_angle(&mut z).changed();
    });

    *value = Quat::from_euler(euler, y, x, z);

    changed
}
