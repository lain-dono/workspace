use crate::reflect_editor::{Arg, OptionsType, ReflectEditor};
use bevy::math::{prelude::*, EulerRot};
use bevy_egui::egui;
use std::any::Any;

#[derive(Default, Clone)]
#[non_exhaustive]
pub struct QuatOptions {
    pub display: QuatDisplay,
}

#[derive(Copy, Clone, Default)]
pub enum QuatDisplay {
    Raw,
    #[default]
    Euler,
    YawPitchRoll,
    AxisAngle,
}

impl OptionsType for bevy::math::Quat {
    type Derive = QuatOptions;
    type Options = QuatOptions;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        derive
    }
}

#[derive(Clone, Copy)]
struct Euler(Vec3);
#[derive(Clone, Copy)]
struct YawPitchRoll((f32, f32, f32));
#[derive(Clone, Copy)]
struct AxisAngle((Vec3, f32));

trait RotationEdit {
    fn from_quat(quat: Quat) -> Self;
    fn to_quat(self) -> Quat;

    fn ui(&mut self, ui: &mut egui::Ui, env: ReflectEditor) -> bool;
}

impl RotationEdit for Euler {
    fn from_quat(quat: Quat) -> Self {
        Euler(quat.to_euler(EulerRot::XYZ).into())
    }

    fn to_quat(self) -> Quat {
        Quat::from_euler(EulerRot::XYZ, self.0.x, self.0.y, self.0.z)
    }

    fn ui(&mut self, ui: &mut egui::Ui, mut env: ReflectEditor) -> bool {
        env.reflect_mut(Arg::null(ui), &mut self.0)
    }
}

impl RotationEdit for YawPitchRoll {
    fn from_quat(quat: Quat) -> Self {
        YawPitchRoll(quat.to_euler(EulerRot::YXZ))
    }

    fn to_quat(self) -> Quat {
        let (y, p, r) = self.0;
        Quat::from_euler(EulerRot::YXZ, y, p, r)
    }

    fn ui(&mut self, ui: &mut egui::Ui, _env: ReflectEditor) -> bool {
        let (yaw, pitch, roll) = &mut self.0;

        let mut changed = false;
        ui.vertical(|ui| {
            egui::Grid::new("ypr grid").show(ui, |ui| {
                ui.label("Yaw");
                changed |= ui.drag_angle(yaw).changed();
                ui.end_row();

                ui.label("Pitch").changed();
                changed |= ui.drag_angle(pitch).changed();
                ui.end_row();

                ui.label("Roll");
                changed |= ui.drag_angle(roll).changed();
                ui.end_row();
            });
        });
        changed
    }
}

impl RotationEdit for AxisAngle {
    fn from_quat(quat: Quat) -> Self {
        AxisAngle(quat.to_axis_angle())
    }

    fn to_quat(self) -> Quat {
        let (axis, angle) = self.0;
        let axis = axis.normalize();
        if axis.is_nan() {
            Quat::IDENTITY
        } else {
            Quat::from_axis_angle(axis.normalize(), angle)
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, mut env: ReflectEditor) -> bool {
        let (axis, angle) = &mut self.0;

        let mut changed = false;
        ui.vertical(|ui| {
            egui::Grid::new("axis-angle quat").show(ui, |ui| {
                ui.label("Axis");
                changed |= env.reflect_mut(Arg::null(ui), axis);
                ui.end_row();
                ui.label("Angle");
                changed |= ui.drag_angle(angle).changed();
                ui.end_row();
            });
        });
        changed
    }
}

pub fn quat_ref(env: ReflectEditor, args: Arg, value: &dyn Any) {
    let mut value = *value.downcast_ref::<Quat>().unwrap();
    args.ui.add_enabled_ui(false, |ui| {
        quat_mut(env, Arg { ui, ..args }, &mut value);
    });
}

pub fn quat_mut(mut env: ReflectEditor, Arg { ui, opt, .. }: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<Quat>().unwrap();
    let opt = opt.downcast_or_default::<QuatOptions>();

    ui.vertical(|ui| match opt.display {
        QuatDisplay::Raw => {
            let mut vec4 = Vec4::from(*value);
            let changed = env.reflect_mut(Arg::null(ui), &mut vec4);
            if changed {
                *value = Quat::from_vec4(vec4).normalize();
            }
            changed
        }
        QuatDisplay::Euler => quat_ui_kind::<Euler>(value, ui, env),
        QuatDisplay::YawPitchRoll => quat_ui_kind::<YawPitchRoll>(value, ui, env),
        QuatDisplay::AxisAngle => quat_ui_kind::<AxisAngle>(value, ui, env),
    })
    .inner
}

fn quat_ui_kind<T: Send + Sync + 'static + Copy + RotationEdit>(
    val: &mut Quat,
    ui: &mut egui::Ui,
    env: ReflectEditor,
) -> bool {
    let id = ui.id();
    let mut intermediate = ui.memory_mut(|memory| {
        *memory
            .data
            .get_temp_mut_or_insert_with(id, || T::from_quat(*val))
    });

    let externally_changed = !intermediate.to_quat().abs_diff_eq(*val, f32::EPSILON);
    if externally_changed {
        intermediate = T::from_quat(*val);
    }

    let changed = intermediate.ui(ui, env);
    if changed || externally_changed {
        *val = intermediate.to_quat();
        ui.memory_mut(|memory| memory.data.insert_temp(id, intermediate));
    }
    changed
}
