use super::icon_btn::{add_button, down_button, remove_button, up_button};
use super::{errors, iter_all_eq, Arg, DefaultFor};
use bevy::reflect::{List, ListInfo, Reflect, ReflectMut, TypeInfo, TypeRegistry};

impl super::ReflectEditor<'_, '_, '_> {
    pub(crate) fn list_ref(&mut self, Arg { id, ui, opt }: Arg, list: &dyn List) {
        ui.vertical(|ui| {
            let len = list.len();
            for index in 0..len {
                let value = list.get(index).unwrap();
                ui.horizontal(|ui| self.reflect_ref(Arg::new_with(ui, id, opt, index), value));

                if index != len - 1 {
                    ui.separator();
                }
            }
        });
    }

    pub(crate) fn list_mut(&mut self, Arg { id, ui, opt }: Arg, list: &mut dyn List) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            let mut op = None;
            let len = list.len();
            if len == 0 && ui_for_empty_list(ui) {
                op = Some(ListOp::AddElement(0))
            }

            for index in 0..len {
                egui::Grid::new((id, index)).num_columns(2).show(ui, |ui| {
                    ui.label(index.to_string());
                    let value = list.get_mut(index).unwrap();
                    ui.horizontal(|ui| {
                        changed |= self.reflect_mut(Arg::new_with(ui, id, opt, index), value);
                    });
                    ui.end_row();

                    let item_op = ui_for_list_controls(ui, index, len);
                    if item_op.is_some() {
                        op = item_op;
                    }
                });

                if index != len - 1 {
                    ui.separator();
                }
            }

            let Some(TypeInfo::List(info)) = list.get_represented_type_info() else {
                return;
            };

            // Respond to control interaction
            if let Some(op) = op {
                let lists = std::iter::once(list);
                changed |= respond_to_list_op(op, self.registry, ui, id, lists);
            }

            let error_id = id.with("error");
            if ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id)) {
                errors::no_default_value(ui, info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
        });

        changed
    }

    pub(crate) fn list_many(
        &mut self,
        Arg { id, ui, opt }: Arg,
        info: &ListInfo,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        use ListOp::*;
        let mut changed = false;

        let iter = values.iter_mut();
        let iter = iter.map(|value| match projector(*value).reflect_mut() {
            ReflectMut::List(l) => l.len(),
            _ => unreachable!(),
        });
        let same_len = iter_all_eq(iter);

        let Some(len) = same_len else {
            ui.label("lists have different sizes, cannot multiedit");
            return changed;
        };

        ui.vertical(|ui| {
            let mut op = None;

            if len == 0 && ui_for_empty_list(ui) {
                op = Some(AddElement(0));
            }

            for index in 0..len {
                let mut items_at_i: Vec<&mut dyn Reflect> = values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::List(list) => list.get_mut(index).unwrap(),
                        _ => unreachable!(),
                    })
                    .collect();

                egui::Grid::new((id, index)).num_columns(2).show(ui, |ui| {
                    ui.label(index.to_string());
                    ui.horizontal(|ui| {
                        let type_id = info.item_type_id();
                        let name = info.type_path();
                        let values = items_at_i.as_mut_slice();
                        let args = Arg::new_with(ui, id, opt, index);
                        changed |= self.reflect_many(type_id, name, args, values, &|a| a);
                    });
                    ui.end_row();
                    let item_op = ui_for_list_controls(ui, index, len);
                    if item_op.is_some() {
                        op = item_op;
                    }
                });

                if index != len - 1 {
                    ui.separator();
                }
            }

            let error_id = id.with("error");
            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
            if let Some(op) = op {
                let lists = values
                    .iter_mut()
                    .map(|l| match projector(*l).reflect_mut() {
                        ReflectMut::List(value) => value,
                        _ => unreachable!(),
                    });
                changed |= respond_to_list_op(op, self.registry, ui, id, lists);
            }
        });

        changed
    }
}

/// Mutate one or more lists based on a [`ListOp`], generated by some user interaction.
fn respond_to_list_op<'lists>(
    op: ListOp,
    registry: &TypeRegistry,
    ui: &mut egui::Ui,
    id: egui::Id,
    lists: impl Iterator<Item = &'lists mut dyn List>,
) -> bool {
    let mut changed = false;
    let error_id = id.with("error");

    for list in lists {
        let Some(TypeInfo::List(info)) = list.get_represented_type_info() else {
            continue;
        };
        changed = match op {
            ListOp::AddElement(index) => {
                let default = registry.default_for(info.item_type_id());
                let default = default.or_else(|| list.get(index).map(Reflect::clone_value));
                if let Some(new_value) = default {
                    list.insert(index, new_value);
                } else {
                    ui.data_mut(|data| data.insert_temp::<bool>(error_id, true));
                }
                true
            }
            ListOp::RemoveElement(index) => {
                list.remove(index);
                true
            }
            ListOp::MoveElementUp(index) => {
                if let Some(prev_index) = index.checked_sub(1) {
                    // Clone this element and insert it at its index - 1.
                    if let Some(element) = list.get(index) {
                        let clone = element.clone_value();
                        list.insert(prev_index, clone);
                    }
                    // Remove the original, now at its index + 1.
                    list.remove(index + 1);
                    true
                } else {
                    false
                }
            }
            ListOp::MoveElementDown(index) => {
                // Clone the next element and insert it at this index.
                if let Some(next_element) = list.get(index + 1) {
                    let next_clone = next_element.clone_value();
                    list.insert(index, next_clone);
                }
                // Remove the original, now at i + 2.
                list.remove(index + 2);
                true
            }
        };
    }
    changed
}

fn ui_for_empty_list(ui: &mut egui::Ui) -> bool {
    let mut add = false;
    ui.vertical_centered(|ui| {
        ui.label("(Empty List)");
        if add_button(ui).on_hover_text("Add element").clicked() {
            add = true;
        }
    });
    add
}

enum ListOp {
    AddElement(usize),
    RemoveElement(usize),
    MoveElementUp(usize),
    MoveElementDown(usize),
}

fn ui_for_list_controls(ui: &mut egui::Ui, index: usize, len: usize) -> Option<ListOp> {
    let mut op = None;
    ui.horizontal(|ui| {
        if add_button(ui).on_hover_text("Add element").clicked() {
            op = Some(ListOp::AddElement(index));
        }
        if remove_button(ui).on_hover_text("Remove element").clicked() {
            op = Some(ListOp::RemoveElement(index));
        }
        let up_enabled = index > 0;
        ui.add_enabled_ui(up_enabled, |ui| {
            if up_button(ui).on_hover_text("Move element up").clicked() {
                op = Some(ListOp::MoveElementUp(index));
            }
        });
        let down_enabled = len.checked_sub(1).map(|l| index < l).unwrap_or(false);
        ui.add_enabled_ui(down_enabled, |ui| {
            if down_button(ui).on_hover_text("Move element down").clicked() {
                op = Some(ListOp::MoveElementDown(index));
            }
        });
    });
    op
}
