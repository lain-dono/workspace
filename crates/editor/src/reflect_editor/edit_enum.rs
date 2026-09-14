use super::{errors, iter_all_eq, Arg, DefaultFor};
use bevy::reflect::{
    std_traits::ReflectDefault, DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant, Enum,
    EnumInfo, Reflect, ReflectMut, TypeInfo, TypeRegistry, VariantInfo, VariantType,
};
use std::{any::TypeId, borrow::Cow};

impl super::ReflectEditor<'_, '_, '_> {
    pub(crate) fn enum_ref(&mut self, Arg { id, ui, opt }: Arg, value: &dyn Enum) {
        ui.vertical(|ui| {
            ui.add_enabled_ui(false, |ui| {
                egui::ComboBox::from_id_source(id.with("select"))
                    .selected_text(value.variant_name())
                    .show_ui(ui, |_| {});
            });

            let len = value.field_len();
            let always_show_label = matches!(value.variant_type(), VariantType::Struct);
            maybe_grid_fold(len, ui, id, always_show_label, |index, ui, label| {
                if label {
                    let _ = match value.name_at(index) {
                        Some(name) => ui.label(name),
                        None => ui.label(index.to_string()),
                    };
                }

                // can panic with invalid reflect impl: field len
                let arg = Arg::variant_field(ui, id, opt, value.variant_index(), index);
                self.reflect_ref(arg, value.field_at(index).unwrap());
                ui.end_row();
                false
            });
        });
    }

    pub(crate) fn enum_mut(&mut self, Arg { id, ui, opt }: Arg, value: &mut dyn Enum) -> bool {
        let Some(type_info) = value.get_represented_type_info() else {
            ui.label("Unrepresentable");
            return false;
        };
        let TypeInfo::Enum(type_info) = type_info else {
            unreachable!("invalid reflect impl: type info mismatch")
        };

        let mut changed = false;

        ui.vertical(|ui| {
            let changed_variant =
                ui_for_enum_variant_select(self.registry, id, ui, value.variant_index(), type_info);
            if let Some((_new_variant, dynamic_enum)) = changed_variant {
                changed = true;
                value.apply(&dynamic_enum);
            }
            let variant_index = value.variant_index();

            let len = value.field_len();
            let always_show_label = matches!(value.variant_type(), VariantType::Struct);
            changed |= maybe_grid_fold(len, ui, id, always_show_label, |index, ui, label| {
                if let Some(name) = value.name_at(index).filter(|_| label) {
                    ui.label(name);
                } else if label {
                    ui.label(index.to_string());
                }

                // can panic with invalid reflect impl: field len
                let arg = Arg::variant_field(ui, id, opt, variant_index, index);
                let changed = self.reflect_mut(arg, value.field_at_mut(index).unwrap());
                ui.end_row();
                changed
            });
        });

        changed
    }

    pub(crate) fn enum_many(
        &mut self,
        Arg { id, ui, opt }: Arg,
        info: &EnumInfo,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let mut changed = false;

        let same_variant = {
            let iter = values.iter_mut();
            let iter = iter.map(|value| match projector(*value).reflect_mut() {
                ReflectMut::Enum(info) => info.variant_index(),
                _ => unreachable!(),
            });
            iter_all_eq(iter)
        };

        let Some(variant_index) = same_variant else {
            ui.label("enums have different selected variants, cannot multiedit");
            return changed;
        };

        let mut variant = info.variant_at(variant_index).unwrap();

        ui.vertical(|ui| {
            let variant_changed =
                ui_for_enum_variant_select(self.registry, id, ui, variant_index, info);

            if let Some((new_variant_index, dynamic_enum)) = variant_changed {
                changed = true;
                variant = info.variant_at(new_variant_index).unwrap();

                for value in values.iter_mut() {
                    projector(*value).apply(&dynamic_enum);
                }
            }

            let field_len = match variant {
                VariantInfo::Struct(info) => info.field_len(),
                VariantInfo::Tuple(info) => info.field_len(),
                VariantInfo::Unit(_) => 0,
            };

            let always_show_label = matches!(variant, VariantInfo::Struct(_));
            changed |= maybe_grid(field_len, ui, id, always_show_label, |ui, label| {
                let handle = |(index, name, type_id, type_name)| {
                    if label {
                        ui.label(name);
                    }

                    let values = values.iter_mut();
                    let values = values.map(|value| enum_field_at(projector(*value), index));

                    let mut values: Vec<&mut dyn Reflect> = values.collect();
                    let values = values.as_mut_slice();
                    let arg = Arg::variant_field(ui, id, opt, variant_index, index);
                    self.reflect_many(type_id, type_name, arg, values, &|a| a);

                    ui.end_row();

                    false
                };

                match variant {
                    VariantInfo::Struct(info) => info
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            (
                                index,
                                Cow::Borrowed(field.name()),
                                field.type_id(),
                                field.type_path(),
                            )
                        })
                        .map(handle)
                        .fold(false, or),
                    VariantInfo::Tuple(info) => info
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            (
                                index,
                                Cow::Owned(index.to_string()),
                                field.type_id(),
                                field.type_path(),
                            )
                        })
                        .map(handle)
                        .fold(false, or),
                    VariantInfo::Unit(_) => false,
                }
            });
        });

        changed
    }
}

fn or(a: bool, b: bool) -> bool {
    a || b
}

fn enum_field_at(reflect: &mut dyn Reflect, index: usize) -> &mut dyn Reflect {
    match reflect.reflect_mut() {
        ReflectMut::Enum(value) => value.field_at_mut(index).unwrap(),
        _ => unreachable!(),
    }
}

fn ui_for_enum_variant_select(
    registry: &TypeRegistry,
    id: egui::Id,
    ui: &mut egui::Ui,
    active_variant_idx: usize,
    info: &bevy::reflect::EnumInfo,
) -> Option<(usize, DynamicEnum)> {
    let mut changed_variant = None;

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_source(id.with("select"))
            .selected_text(info.variant_names()[active_variant_idx])
            .show_ui(ui, |ui| {
                for (i, variant) in info.iter().enumerate() {
                    let variant_name = variant.name();
                    let is_active_variant = i == active_variant_idx;

                    let variant_is_constructable = variant_constructable(registry, variant);

                    ui.add_enabled_ui(variant_is_constructable.is_ok(), |ui| {
                        let mut variant_label_response =
                            ui.selectable_label(is_active_variant, variant_name);

                        if let Err(fields) = variant_is_constructable {
                            variant_label_response =
                                variant_label_response.on_disabled_hover_ui(|ui| {
                                    errors::unconstructable_variant(
                                        ui,
                                        info.type_path(),
                                        variant_name,
                                        &fields,
                                    );
                                });
                        }

                        /*let res = variant_label_response.on_hover_ui(|ui| {
                            if !unconstructable_variants.is_empty() {
                                errors::unconstructable_variants(
                                    ui,
                                    info.type_name(),
                                    &unconstructable_variants,
                                );
                            }
                        });*/

                        if variant_label_response.clicked() {
                            if let Ok(dyn_enum) = construct_default_variant(registry, variant, ui) {
                                changed_variant = Some((i, dyn_enum));
                            }
                        }
                    });
                }

                false
            });
    });

    changed_variant
}

fn construct_default_variant(
    registry: &TypeRegistry,
    variant: &VariantInfo,
    ui: &mut egui::Ui,
) -> Result<DynamicEnum, ()> {
    let dynamic_variant = match variant {
        VariantInfo::Struct(struct_info) => {
            let mut dynamic_struct = DynamicStruct::default();
            for field in struct_info.iter() {
                let Some(field_default_value) = registry.default_for(field.type_id()) else {
                    errors::no_default_value(ui, field.type_path());
                    return Err(());
                };
                dynamic_struct.insert_boxed(field.name(), field_default_value);
            }
            DynamicVariant::Struct(dynamic_struct)
        }
        VariantInfo::Tuple(tuple_info) => {
            let mut dynamic_tuple = DynamicTuple::default();
            for field in tuple_info.iter() {
                let Some(field_default_value) = registry.default_for(field.type_id()) else {
                    errors::no_default_value(ui, field.type_path());
                    return Err(());
                };
                dynamic_tuple.insert_boxed(field_default_value);
            }
            DynamicVariant::Tuple(dynamic_tuple)
        }
        VariantInfo::Unit(_) => DynamicVariant::Unit,
    };

    Ok(DynamicEnum::new(variant.name(), dynamic_variant))
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
            .filter_map(|field| (!is_constructable(field.type_id())).then_some(field.type_path()))
            .collect(),
        VariantInfo::Tuple(variant) => variant
            .iter()
            .filter_map(|field| (!is_constructable(field.type_id())).then_some(field.type_path()))
            .collect(),
        VariantInfo::Unit(_) => return Ok(()),
    };

    if unconstructable_fields.is_empty() {
        Ok(())
    } else {
        Err(unconstructable_fields)
    }
}

fn maybe_grid<R: Default>(
    len: usize,
    ui: &mut egui::Ui,
    id_source: egui::Id,
    always_show_label: bool,
    mut f: impl FnMut(&mut egui::Ui, bool) -> R,
) -> R {
    match len {
        0 => R::default(),
        1 if !always_show_label => ui.vertical_centered_justified(|ui| f(ui, false)).inner,
        _ => {
            egui::Grid::new(id_source)
                .num_columns(2)
                .show(ui, |ui| f(ui, true))
                .inner
        }
    }
}

fn maybe_grid_fold(
    len: usize,
    ui: &mut egui::Ui,
    id_source: egui::Id,
    always_show_label: bool,
    mut f: impl FnMut(usize, &mut egui::Ui, bool) -> bool,
) -> bool {
    maybe_grid(len, ui, id_source, always_show_label, |ui, label| {
        let iter = (0..len).map(|index| f(index, ui, label));
        #[allow(clippy::unnecessary_fold)]
        iter.fold(false, |a, b| a || b)
    })
}
