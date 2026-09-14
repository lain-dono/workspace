use super::icon_btn::{add_button, remove_button};
use super::{errors, Arg, DefaultFor};
use bevy::reflect::{Map, MapInfo, Reflect, TypeInfo};

impl super::ReflectEditor<'_, '_, '_> {
    pub(crate) fn map_ref(&mut self, Arg { id, ui, .. }: Arg, map: &dyn Map) {
        egui::Grid::new(id).show(ui, |ui| {
            for (index, (key, value)) in map.iter().enumerate() {
                self.reflect_ref(Arg::id(id, ui).with("K").with(index), key);
                self.reflect_ref(Arg::id(id, ui).with("K").with(index), value);
                ui.end_row();
            }
        });
    }

    pub(crate) fn map_mut(&mut self, Arg { id, ui, .. }: Arg, map: &mut dyn Map) -> bool {
        let mut changed = false;
        let map_draft_id = id.with("map_draft");
        if map.is_empty() {
            ui.label("(Empty Map)");
            ui.end_row();
        }

        let draft_clone = ui.data_mut(|data| {
            data.get_temp_mut_or_default::<Option<DraftKeyValue>>(map_draft_id)
                .to_owned()
        });

        let mut to_delete: Option<usize> = None;

        egui::Grid::new(id).show(ui, |ui| {
            for index in 0..map.len() {
                if let Some((key, value)) = map.get_at_mut(index) {
                    let kid = id.with("K").with(index);
                    let vid = id.with("V").with(index);

                    self.reflect_ref(Arg::id(kid, ui), key);
                    changed |= self.reflect_mut(Arg::id(vid, ui), value);
                    if remove_button(ui).on_hover_text("Remove element").clicked() {
                        to_delete = Some(index);
                    }
                    ui.end_row();
                }
            }

            ui.separator();
            ui.end_row();

            ui.label("New element");

            if let Some(DraftKeyValue(mut key, mut value)) = draft_clone {
                ui.end_row();

                // Show controls for editing our draft element.

                let kid = id.with("K").with("draft");
                let vid = id.with("V").with("draft");

                let key_changed = self.reflect_mut(Arg::id(kid, ui), key.as_mut());
                let value_changed = self.reflect_mut(Arg::id(vid, ui), value.as_mut());

                // If the clone changed, update the data in UI state.
                if key_changed || value_changed {
                    let next_draft = DraftKeyValue(key, value);
                    ui.data_mut(|data| data.insert_temp(map_draft_id, Some(next_draft)));
                }

                // Show controls to insert the draft into the map, or remove it.
                if ui.button("Insert").clicked() {
                    let draft =
                        ui.data_mut(|data| data.get_temp::<Option<DraftKeyValue>>(map_draft_id));
                    if let Some(DraftKeyValue(key, value)) = draft.flatten() {
                        map.insert_boxed(key, value);
                        ui.data_mut(|data| data.remove_by_type::<Option<DraftKeyValue>>());
                    }
                    changed = true;
                }

                if ui.button("Cancel").clicked() {
                    ui.data_mut(|data| data.remove_by_type::<Option<DraftKeyValue>>());
                    changed = true;
                }

                ui.end_row();
            } else {
                // If no draft element exists, show a button to create one.
                if add_button(ui).clicked() {
                    // Insert a temporary 'draft' key-value pair into UI state.
                    if let Some(TypeInfo::Map(map_info)) = map.get_represented_type_info() {
                        let key = self.registry.default_for(map_info.key_type_id());
                        let value = self.registry.default_for(map_info.value_type_id());
                        let op = DraftKeyValue::new(key, value);
                        if op.is_some() {
                            ui.data_mut(|data| data.insert_temp(map_draft_id, op));
                        }
                    }
                }

                ui.end_row();
            }
        });

        if let Some(index) = to_delete {
            // Can't have both an immutable borrow of the map's key,
            // and mutably borrow the map to delete the element.
            if let Some(cloned_key) = map.get_at(index).map(|(key, _)| key.clone_value()) {
                map.remove(cloned_key.as_ref());
            }
        }

        changed
    }

    pub(crate) fn map_many(
        &mut self,
        Arg { ui, .. }: Arg,
        info: &MapInfo,
        _: &mut [&mut dyn Reflect],
        _: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let type_name = pretty_type_name::pretty_type_name_str(info.type_path());
        errors::no_multiedit(ui, &type_name);
        false
    }
}

struct DraftKeyValue(Box<dyn Reflect>, Box<dyn Reflect>);

impl DraftKeyValue {
    fn new(key: Option<Box<dyn Reflect>>, value: Option<Box<dyn Reflect>>) -> Option<Self> {
        Some(Self(key?, value?))
    }
}

impl Clone for DraftKeyValue {
    fn clone(&self) -> Self {
        Self(self.0.clone_value(), self.1.clone_value())
    }
}
