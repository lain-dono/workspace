use crate::reflect_editor::{iter_all_eq, AnyOptions, Arg, OptionsType, ReflectEditor};
use bevy::math::{prelude::*, DMat2, DMat3, DMat4, DVec2, DVec3, DVec4, Mat3A, Vec3A};
use bevy::reflect::Reflect;
use bevy_egui::egui;
use std::{any::Any, borrow::Cow, ops::Add, ops::AddAssign, ops::Sub};

macro_rules! vec_ui {
    ([$name_ref:ident $name_mut:ident $name_many:ident] $ty:ty[$elem_ty:ty]: $count:literal $($component:ident)*) => {
        vec_ui!([$name_ref $name_mut] $ty[$elem_ty]: $count $($component)*);
        vec_ui!($name_many $ty[$elem_ty]: $count $($component)*);
    };

    ($name_many:ident $ty:ty [$elem_ty:ty]: $count:literal $($component:ident)*) => {
        pub fn $name_many(
            _env: ReflectEditor,
            Arg { ui, id, ..}: Arg,
            values: &mut [&mut dyn Reflect],
            projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
        ) -> bool {
            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);
                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(
                            let same = iter_all_eq(values.iter_mut().map(|value| {
                                projector(*value).downcast_ref::<$ty>().unwrap().$component
                            }));

                            let id = id.with(stringify!($component));
                            changed |= change_slider($component, id, same, |change, overwrite| {
                                for value in values.iter_mut() {
                                    let value = projector(*value).downcast_mut::<$ty>().unwrap();
                                    if overwrite {
                                        value.$component = change;
                                    } else {
                                        value.$component += change;
                                    }

                                }
                            });
                        )*
                    }
                    _ => unreachable!(),
                });
            });
            changed
        }
    };

    ([$name_ref:ident $name_mut:ident] $ty:ty[$elem_ty:ty]: $count:literal $($component:ident)*) => {
        pub fn $name_ref(mut env: ReflectEditor, Arg { ui, ..}: Arg, value: &dyn Any) {
            let value = value.downcast_ref::<$ty>().unwrap();

            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(2.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(env.reflect_ref(Arg::null($component), &value.$component,);)*
                    }
                    _ => unreachable!(),
                });
            });
        }

        pub fn $name_mut(mut env: ReflectEditor, Arg { ui, opt, id }: Arg, value: &mut dyn Any) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();
            let opt = opt.downcast_or_default::<NumberOptions<$ty>>();

            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(2.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(
                            changed |= env.reflect_mut(
                                Arg {
                                    id: id.with(stringify!($component)),
                                    ui: $component,
                                    opt: AnyOptions::new(&opt.map(|vec| vec.$component))
                                },
                                &mut value.$component
                            );
                        )*
                    }
                    _ => unreachable!(),
                });
            });
            changed
        }

    };
}

macro_rules! mat_ui {
    ($name_ref:ident $name_mut:ident $ty:ty: $($component:ident)*) => {
        pub fn $name_ref(mut env: ReflectEditor, Arg { ui, .. }: Arg, value: &dyn Any) {
            ui.vertical(|ui| {
                let value = value.downcast_ref::<$ty>().unwrap();
                $(
                    env.reflect_ref(Arg::null(ui), &value.$component);
                )*
            });
        }

        pub fn $name_mut(mut env: ReflectEditor, Arg { ui, .. }: Arg, value: &mut dyn Any) -> bool {
            let mut changed = false;
            ui.vertical(|ui| {
                let value = value.downcast_mut::<$ty>().unwrap();
                $(
                    changed |= env.reflect_mut(Arg::null(ui), &mut value.$component);
                )*
            });
            changed
        }
    };
}

vec_ui!([vec2_ref  vec2_mut  vec2_many ] Vec2[f32]:  2 x y    );
vec_ui!([vec3_ref  vec3_mut  vec3_many ] Vec3[f32]:  3 x y z  );
vec_ui!([vec4_ref  vec4_mut  vec4_many ] Vec4[f32]:  4 x y z w);
vec_ui!([vec3a_ref vec3a_mut vec3a_many] Vec3A[f32]: 3 x y z  );

vec_ui!([uvec2_ref uvec2_mut uvec2_many] UVec2[u32]: 2 x y    );
vec_ui!([uvec3_ref uvec3_mut uvec3_many] UVec3[u32]: 3 x y z  );
vec_ui!([uvec4_ref uvec4_mut uvec4_many] UVec4[u32]: 4 x y z w);

vec_ui!([ivec2_ref ivec2_mut ivec2_many] IVec2[i32]: 2 x y    );
vec_ui!([ivec3_ref ivec3_mut ivec3_many] IVec3[i32]: 3 x y z  );
vec_ui!([ivec4_ref ivec4_mut ivec4_many] IVec4[i32]: 4 x y z w);

vec_ui!([dvec2_ref dvec2_mut dvec2_many] DVec2[f64]: 2 x y    );
vec_ui!([dvec3_ref dvec3_mut dvec3_many] DVec3[f64]: 3 x y z  );
vec_ui!([dvec4_ref dvec4_mut dvec4_many] DVec4[f64]: 4 x y z w);

vec_ui!([bvec2_ref bvec2_mut] BVec2[bool]: 2 x y    );
vec_ui!([bvec3_ref bvec3_mut] BVec3[bool]: 3 x y z  );
vec_ui!([bvec4_ref bvec4_mut] BVec4[bool]: 4 x y z w);

mat_ui!(mat2_ref  mat2_mut  Mat2:  x_axis y_axis              );
mat_ui!(mat3_ref  mat3_mut  Mat3:  x_axis y_axis z_axis       );
mat_ui!(mat4_ref  mat4_mut  Mat4:  x_axis y_axis z_axis w_axis);
mat_ui!(mat3a_ref mat3a_mut Mat3A: x_axis y_axis z_axis       );

mat_ui!(dmat2_ref dmat2_mut DMat2: x_axis y_axis              );
mat_ui!(dmat3_ref dmat3_mut DMat3: x_axis y_axis z_axis       );
mat_ui!(dmat4_ref dmat4_mut DMat4: x_axis y_axis z_axis w_axis);

impl super::EditorVTableBuilder<'_> {
    pub(crate) fn add_num<T: egui::emath::Numeric + Reflect + Add<T, Output = T> + AddAssign<T>>(
        &mut self,
    ) {
        self.add_many::<T>(number_ref::<T>, number_mut::<T>, number_many::<T>);
    }
}

pub fn number_ref<T>(_: ReflectEditor, arg: Arg, value: &dyn Any)
where
    T: egui::emath::Numeric,
{
    let value = value.downcast_ref::<T>().unwrap();
    let opt = arg.opt.downcast_or_default::<NumberOptions<T>>();
    let decimal_range = 0..=1usize;
    let number = egui::emath::format_with_decimals_in_range(value.to_f64(), decimal_range);
    let text = format!("{}{}{}", opt.prefix, number, opt.suffix);
    let text = egui::RichText::new(text).monospace();
    let sense = egui::Sense::hover();
    arg.ui.add(egui::Button::new(text).wrap(false).sense(sense));
}

pub fn number_mut<T>(_: ReflectEditor, arg: Arg, value: &mut dyn Any) -> bool
where
    T: egui::emath::Numeric,
{
    let value = value.downcast_mut::<T>().unwrap();
    let opt = arg.opt.downcast_or_default::<NumberOptions<T>>();
    display_number(value, &opt, arg.ui, 0.1)
}

fn display_number<T: egui::emath::Numeric>(
    value: &mut T,
    opt: &NumberOptions<T>,
    ui: &mut egui::Ui,
    default_speed: f32,
) -> bool {
    let mut changed = match opt.display {
        NumberDisplay::Drag => {
            let mut widget = egui::DragValue::new(value);
            if !opt.prefix.is_empty() {
                widget = widget.prefix(&opt.prefix);
            }
            if !opt.suffix.is_empty() {
                widget = widget.suffix(&opt.suffix);
            }
            match (opt.min, opt.max) {
                (Some(min), Some(max)) => widget = widget.clamp_range(min.to_f64()..=max.to_f64()),
                (Some(min), None) => widget = widget.clamp_range(min.to_f64()..=f64::MAX),
                (None, Some(max)) => widget = widget.clamp_range(f64::MIN..=max.to_f64()),
                (None, None) => {}
            }
            widget = widget.speed(if opt.speed != 0.0 {
                opt.speed
            } else {
                default_speed
            });
            ui.add(widget).changed()
        }
        NumberDisplay::Slider => {
            let min = opt.min.unwrap_or_else(|| T::from_f64(0.0));
            let max = opt.max.unwrap_or_else(|| T::from_f64(1.0));
            let range = min..=max;
            let widget = egui::Slider::new(value, range);
            ui.add(widget).changed()
        }
    };

    if let Some(min) = opt.min {
        let min = min.to_f64();
        if value.to_f64() < min {
            *value = T::from_f64(min);
            changed = true;
        }
    }
    if let Some(max) = opt.max {
        let max = max.to_f64();
        if value.to_f64() > max {
            *value = T::from_f64(max);
            changed = true;
        }
    }
    changed
}

pub fn number_many<T: Reflect + egui::emath::Numeric + Add<Output = T> + AddAssign<T>>(
    _: ReflectEditor,
    arg: Arg,
    values: &mut [&mut dyn Reflect],
    projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool {
    let iter = values.iter_mut();
    let iter = iter.map(|value| *projector(*value).downcast_ref::<T>().unwrap());
    let same = iter_all_eq(iter).map(T::to_f64);

    change_slider(arg.ui, arg.id, same, |change, overwrite| {
        for value in values.iter_mut() {
            let value = projector(*value).downcast_mut::<T>().unwrap();
            let change = T::from_f64(change);
            *value = if overwrite { change } else { *value + change };
        }
    })
}

fn change_slider<T: egui::emath::Numeric + Sub<Output = T> + Default + Send + Sync + 'static>(
    ui: &mut egui::Ui,
    id: egui::Id,
    same: Option<T>,
    f: impl FnOnce(T, bool),
) -> bool {
    let speed = if T::INTEGRAL { 1.0 } else { 0.1 };

    if let Some(mut same) = same {
        let widget = egui::DragValue::new(&mut same).speed(speed);
        let changed = ui.add(widget).changed();
        if changed {
            f(same, true);
        }
        changed
    } else {
        let old_change = ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default::<T>(id));
        let mut change = old_change;

        let widget = egui::DragValue::new(&mut change);
        let widget = widget.speed(speed).custom_formatter(|_, _| "-".to_string());

        let changed = ui.add(widget).changed();
        if changed {
            f(change - old_change, false);
        }

        ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default(id) = change);
        changed
    }
}

pub fn bool_ref(_: ReflectEditor, args: Arg, value: &dyn Any) {
    let mut copy = *value.downcast_ref::<bool>().unwrap();
    args.ui.add_enabled_ui(false, |ui| {
        ui.checkbox(&mut copy, "");
    });
}
pub fn bool_mut(_: ReflectEditor, Arg { ui, .. }: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<bool>().unwrap();
    ui.checkbox(value, "").changed()
}

pub fn string_ref(_: ReflectEditor, Arg { ui, .. }: Arg, value: &dyn Any) {
    let value = value.downcast_ref::<String>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(&mut value.as_str());
    } else {
        ui.text_edit_singleline(&mut value.as_str());
    }
}
pub fn string_mut(_: ReflectEditor, Arg { ui, .. }: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<String>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(value).changed()
    } else {
        ui.text_edit_singleline(value).changed()
    }
}

pub fn cow_str_mut(_: ReflectEditor, Arg { ui, .. }: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<Cow<str>>().unwrap();
    let mut clone = value.to_string();
    let changed = if value.contains('\n') {
        ui.text_edit_multiline(&mut clone).changed()
    } else {
        ui.text_edit_singleline(&mut clone).changed()
    };
    if changed {
        *value = Cow::Owned(clone);
    }
    changed
}
pub fn cow_str_ref(_: ReflectEditor, Arg { ui, .. }: Arg, value: &dyn Any) {
    let value = value.downcast_ref::<Cow<str>>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(&mut value.as_ref());
    } else {
        ui.text_edit_singleline(&mut value.as_ref());
    }
}

pub fn duration_mut(mut env: ReflectEditor, arg: Arg, value: &mut dyn Any) -> bool {
    let value = value.downcast_mut::<std::time::Duration>().unwrap();
    let mut seconds = value.as_secs_f64();
    let opt = NumberOptions {
        min: Some(0.0f64),
        suffix: "s".to_string(),
        ..Default::default()
    };

    let changed = env.reflect_mut(arg.with_opt(AnyOptions::new(&opt)), &mut seconds);
    if changed {
        *value = std::time::Duration::from_secs_f64(seconds);
    }
    changed
}

pub fn duration_ref(mut env: ReflectEditor, arg: Arg, value: &dyn Any) {
    let value = value.downcast_ref::<std::time::Duration>().unwrap();
    let seconds = value.as_secs_f64();
    let opt = NumberOptions {
        min: Some(0.0f64),
        suffix: "s".to_string(),
        ..Default::default()
    };
    env.reflect_ref(arg.with_opt(AnyOptions::new(&opt)), &seconds);
}

pub fn instant_mut(env: ReflectEditor, args: Arg, value: &mut dyn Any) -> bool {
    instant_ref(env, args, value);
    false
}

pub fn instant_ref(_: ReflectEditor, mut arg: Arg, value: &dyn Any) {
    let value = value.downcast_ref::<std::time::Instant>().unwrap();
    let mut secs = value.elapsed().as_secs_f32();
    arg.horizontal(|ui| {
        ui.add_enabled(false, egui::DragValue::new(&mut secs));
        ui.label("seconds ago");
    });
}

macro_rules! impl_num {
    ($($ty:ty),+) => {
        $(
            impl OptionsType for $ty {
                type Derive = NumberOptions<$ty>;
                type Options = NumberOptions<$ty>;

                fn options_from_derive(derive: Self::Derive) -> Self::Options {
                    derive
                }
            }
        )+
    };
}

impl_num!(f32, f64);
impl_num!(i8, i16, i32, i64, i128, isize);
impl_num!(u8, u16, u32, u64, u128, usize);

#[derive(Clone)]
#[non_exhaustive]
pub struct NumberOptions<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub speed: f32,
    pub prefix: String,
    pub suffix: String,
    pub display: NumberDisplay,
}

impl<T> Default for NumberOptions<T> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }
}

#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub enum NumberDisplay {
    #[default]
    Drag,
    Slider,
}

impl<T> NumberOptions<T> {
    pub fn between(min: T, max: T) -> NumberOptions<T> {
        Self {
            min: Some(min),
            max: Some(max),
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }

    pub fn at_least(min: T) -> NumberOptions<T> {
        Self {
            min: Some(min),
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }

    pub fn with_speed(self, speed: f32) -> NumberOptions<T> {
        Self { speed, ..self }
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> NumberOptions<U> {
        NumberOptions {
            #[allow(clippy::redundant_closure)] // false positive
            min: self.min.as_ref().map(|min| f(min)),
            max: self.max.as_ref().map(f),
            speed: self.speed,
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
            display: NumberDisplay::default(),
        }
    }
}

impl<T: egui::emath::Numeric> NumberOptions<T> {
    pub fn positive() -> NumberOptions<T> {
        Self {
            min: Some(T::from_f64(0.0)),
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }

    pub fn normalized() -> Self {
        Self {
            min: Some(T::from_f64(0.0)),
            max: Some(T::from_f64(1.0)),
            speed: 0.01,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }
}
