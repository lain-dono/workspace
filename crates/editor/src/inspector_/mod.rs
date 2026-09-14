use bevy::prelude::*;
use bevy::reflect::{
    Array, DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant, Enum, EnumInfo, List, Map,
    Reflect, ReflectMut, ReflectRef, Struct, Tuple, TupleStruct, TypeInfo, TypeRegistry,
    TypeRegistryArc, VariantInfo,
};
use egui::{Color32, Id, Ui, WidgetText};
use std::any::{Any, TypeId};

mod errors;
mod inspectable;

pub use self::inspectable::{InspectMutFn, InspectRefFn, InspectableImpl};

const WRAP_WIDTH: f32 = 235.0;
pub const PERCENT: f32 = 0.35;

const ERR_COLOR: Color32 = Color32::GRAY;

pub struct InspectorPlugin;

impl bevy::app::Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<Self>() {
            panic!();
            return;
        }

        let registry = app.world.resource::<AppTypeRegistry>();
        let mut registry = registry.write();

        self::inspectable::impl_bevy::register(&mut registry);
        self::inspectable::impl_std::register(&mut registry);
        self::inspectable::impl_glam::register(&mut registry);

        /*
        for t in registry.iter() {
            dbg!(t.type_name());
        }
        */
    }
}

pub fn ui_for_reflect(ui: &mut Ui, registry: &TypeRegistryArc, reflect: &mut dyn Reflect) -> bool {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        let registry = registry.read();
        let wrapping = ui.available_width() > WRAP_WIDTH;
        InspectorUi::new(&registry, wrapping).reflect_mut(ui, Id::null(), reflect)
    })
    .inner
}

#[derive(Clone, Copy)]
pub struct InspectorUi<'registry> {
    registry: &'registry TypeRegistry,

    level: usize,
    wrapping: bool,
}

impl<'registry> InspectorUi<'registry> {
    fn new(registry: &'registry TypeRegistry, wrapping: bool) -> Self {
        Self {
            registry,
            level: 0,
            wrapping,
        }
    }

    fn next_level(self) -> Self {
        Self {
            level: self.level + 1,
            ..self
        }
    }
}

impl<'registry> InspectorUi<'registry> {
    #[must_use]
    fn reflect_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Reflect) -> bool {
        let type_id = Any::type_id(reflect);
        if let Some(s) = self.registry.get_type_data::<InspectableImpl>(type_id) {
            dbg!(reflect.type_name());
            return s.inspect_mut(self, ui, id, reflect);
        }

        let type_id = {
            self.registry
                .get_with_name(reflect.type_name())
                .map(|r| r.type_id())
        };

        if let Some(type_id) = type_id {
            if let Some(s) = self.registry.get_type_data::<InspectableImpl>(type_id) {
                dbg!(reflect.type_name());
                return s.inspect_mut(self, ui, id, reflect);
            }
        }

        match reflect.reflect_mut() {
            ReflectMut::Struct(reflect) => self.reflect_struct_mut(ui, id, reflect),
            ReflectMut::TupleStruct(reflect) => self.reflect_tuple_struct_mut(ui, id, reflect),
            ReflectMut::Tuple(reflect) => self.reflect_tuple_mut(ui, id, reflect),
            ReflectMut::List(reflect) => self.reflect_list_mut(ui, id, reflect),
            ReflectMut::Array(reflect) => self.reflect_array_mut(ui, id, reflect),
            ReflectMut::Map(reflect) => self.reflect_map_mut(ui, id, reflect),
            ReflectMut::Enum(reflect) => self.reflect_enum_mut(ui, id, reflect),
            ReflectMut::Value(reflect) => {
                self::errors::reflect_value_no_impl(ui, reflect.type_name());
                false
            }
        }
    }

    fn reflect_struct_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Struct) -> bool {
        ui_list(ui, reflect.field_len(), |ui, index| {
            let label = reflect.name_at(index).map(WidgetText::from).unwrap();
            self.field_pair_mut(ui, label, id, reflect.field_at_mut(index).unwrap())
        })
    }

    fn reflect_tuple_struct_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn TupleStruct) -> bool {
        ui_list(ui, reflect.field_len(), |ui, index| {
            self.indexed_field_mut(ui, index, id, reflect.field_mut(index).unwrap())
        })
    }

    fn reflect_tuple_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Tuple) -> bool {
        ui_list(ui, reflect.field_len(), |ui, index| {
            self.indexed_field_mut(ui, index, id, reflect.field_mut(index).unwrap())
        })
    }

    fn reflect_list_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn List) -> bool {
        ui_list(ui, reflect.len(), |ui, index| {
            self.indexed_field_mut(ui, index, id, reflect.get_mut(index).unwrap())
        })
    }

    fn reflect_array_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Array) -> bool {
        ui_list(ui, reflect.len(), |ui, index| {
            self.indexed_field_mut(ui, index, id, reflect.get_mut(index).unwrap())
        })
    }

    fn reflect_map_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Map) -> bool {
        ui_list(ui, reflect.len(), |ui, index| {
            let id = id.with(index);
            let (key, _) = reflect.get_at(index).unwrap();
            let key = unsafe { crate::util::fuck_ref(key) };
            let field = reflect.get_mut(key).unwrap();
            self.next_level().reflect_mut(ui, id, field)
        })
    }

    fn reflect_enum_mut(&self, ui: &mut Ui, id: Id, reflect: &mut dyn Enum) -> bool {
        let Some(type_info) = reflect.get_represented_type_info() else {
            ui.label("Unrepresentable");
            return false;
        };
        let TypeInfo::Enum(info) = type_info else {
            unreachable!("invalid reflect impl: type info mismatch")
        };

        let mut changed = false;

        let mut resp = ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            let active = reflect.variant_index();

            if let Some((_variant, mut new_value)) = self.enum_variant_select(id, ui, active, info)
            {
                changed = true;
                new_value.set_represented_type(Some(type_info));
                reflect.apply(&new_value);
                return;
            }

            let id = id.with(reflect.variant_index());

            changed |= ui_list(ui, reflect.field_len(), |ui, index| {
                let id = id.with(index);
                let label = reflect
                    .name_at(index)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| index.to_string());

                let field = match reflect.field_at_mut(index) {
                    Some(field) => field,
                    None => reflect.field_mut(&label).unwrap(),
                };

                self.field_pair_mut(ui, label, id, field)
            });
        });

        if changed {
            resp.response.mark_changed();
        }

        changed
    }

    fn enum_variant_select(
        &self,
        id: Id,
        ui: &mut Ui,
        active: usize,
        info: &EnumInfo,
    ) -> Option<(usize, DynamicEnum)> {
        let mut changed_variant = None;

        egui::ComboBox::new(id.with("select"), "")
            .selected_text(info.variant_names()[active])
            .show_ui(ui, |ui| {
                for (index, variant) in info.iter().enumerate() {
                    let variant_name = variant.name();
                    let is_active_variant = index == active;

                    let variant_is_constructable = variant_constructable(self.registry, variant);

                    ui.add_enabled_ui(variant_is_constructable.is_ok(), |ui| {
                        let mut variant_label_response =
                            ui.selectable_label(is_active_variant, variant_name);

                        if let Err(fields) = variant_is_constructable {
                            variant_label_response =
                                variant_label_response.on_disabled_hover_ui(|ui| {
                                    self::errors::unconstructable_variant(
                                        ui,
                                        info.type_name(),
                                        variant_name,
                                        &fields,
                                    );
                                });
                        }

                        /*let res = variant_label_response.on_hover_ui(|ui| {
                            if !unconstructable_variants.is_empty() {
                                self::errors::unconstructable_variants(
                                    ui,
                                    info.type_name(),
                                    &unconstructable_variants,
                                );
                            }
                        });*/

                        if variant_label_response.clicked() {
                            if let Ok(dynamic_enum) =
                                construct_default_variant(self.registry, index, variant, ui)
                            {
                                changed_variant = Some((index, dynamic_enum));
                            };
                        }
                    });
                }

                false
            });

        changed_variant
    }

    #[must_use]
    fn indexed_field_mut(
        &self,
        ui: &mut Ui,
        index: usize,
        id: Id,
        reflect: &mut dyn Reflect,
    ) -> bool {
        self.field_pair_mut(ui, index.to_string(), id.with(index), reflect)
    }

    #[must_use]
    fn field_pair_mut(
        &self,
        ui: &mut Ui,
        label: impl Into<WidgetText>,
        id: Id,
        reflect: &mut dyn Reflect,
    ) -> bool {
        let width = ui.available_width();
        let height = 18.0;
        let pad = 16.0;

        let run_label = |ui: &mut egui::Ui| {
            let indent = pad * self.level as f32;
            let _ = ui.allocate_space(egui::vec2(indent, height));
            let (id, space) = ui.allocate_space(egui::vec2(width * PERCENT - indent, height));
            let layout = egui::Layout::left_to_right(egui::Align::Center);
            let mut ui = ui.child_ui_with_id_source(space, layout, id);
            ui.label(label);
        };

        if self.wrapping && matches!(reflect.reflect_ref(), ReflectRef::Value(_)) {
            ui.spacing_mut().interact_size.y = height;
            ui.horizontal(|ui| {
                run_label(ui);
                self.reflect_mut(ui, id, reflect)
            })
            .inner
        } else {
            ui.horizontal(|ui| run_label(ui));
            self.next_level().reflect_mut(ui, id, reflect)
        }
    }
}

fn ui_list(ui: &mut Ui, len: usize, mut element: impl FnMut(&mut Ui, usize) -> bool) -> bool {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), move |ui| {
        (0..len)
            .map(move |index| element(ui, index))
            .fold(false, |acc, x| acc | x)
    })
    .inner
}

fn variant_constructable<'a>(
    registry: &TypeRegistry,
    variant: &'a VariantInfo,
) -> Result<(), Vec<&'a str>> {
    let is_constructable =
        |type_id: TypeId| registry.get_type_data::<ReflectDefault>(type_id).is_some();

    let unconstructable_fields: Vec<&'a str> = match variant {
        VariantInfo::Struct(variant) => variant
            .iter()
            .filter_map(|field| (!is_constructable(field.type_id())).then_some(field.type_name()))
            .collect(),
        VariantInfo::Tuple(variant) => variant
            .iter()
            .filter_map(|field| (!is_constructable(field.type_id())).then_some(field.type_name()))
            .collect(),
        VariantInfo::Unit(_) => return Ok(()),
    };

    if unconstructable_fields.is_empty() {
        Ok(())
    } else {
        Err(unconstructable_fields)
    }
}

fn default_value_for(registry: &TypeRegistry, type_id: TypeId) -> Option<Box<dyn Reflect>> {
    registry
        .get_type_data::<ReflectDefault>(type_id)
        .map(|reflect| reflect.default())
}

fn construct_default_variant(
    registry: &TypeRegistry,
    index: usize,
    variant: &VariantInfo,
    ui: &mut egui::Ui,
) -> Result<DynamicEnum, ()> {
    let dynamic_variant = match variant {
        VariantInfo::Struct(struct_info) => {
            let mut dynamic_struct = DynamicStruct::default();
            for field in struct_info.iter() {
                let field_default_value = match default_value_for(registry, field.type_id()) {
                    Some(value) => value,
                    None => {
                        self::errors::no_default_value(ui, field.type_name());
                        return Err(());
                    }
                };
                dynamic_struct.insert_boxed(field.name(), field_default_value);
            }
            DynamicVariant::Struct(dynamic_struct)
        }
        VariantInfo::Tuple(tuple_info) => {
            let mut dynamic_tuple = DynamicTuple::default();
            for field in tuple_info.iter() {
                let field_default_value = match default_value_for(registry, field.type_id()) {
                    Some(value) => value,
                    None => {
                        self::errors::no_default_value(ui, field.type_name());
                        return Err(());
                    }
                };
                dynamic_tuple.insert_boxed(field_default_value);
            }
            DynamicVariant::Tuple(dynamic_tuple)
        }
        VariantInfo::Unit(_) => DynamicVariant::Unit,
    };

    Ok(DynamicEnum::new_with_index(
        index,
        variant.name(),
        dynamic_variant,
    ))
}
