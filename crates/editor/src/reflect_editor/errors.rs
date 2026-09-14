use bevy::asset::UntypedAssetId;
use bevy::ecs::entity::Entity;
use bevy::reflect::TypeRegistry;
use std::{any::TypeId, borrow::Cow};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    ResourceDoesNotExist(TypeId),
    ComponentDoesNotExist(Entity, TypeId),
    NoComponentId(TypeId),
    NoTypeRegistration(TypeId),
    NoTypeData(TypeId, &'static str),
}

impl Error {
    pub fn show_typed<T>(self, ui: &mut egui::Ui) {
        self.show(ui, &pretty_type_name::pretty_type_name::<T>())
    }

    pub fn show(self, ui: &mut egui::Ui, type_name: &str) {
        match self {
            Self::ComponentDoesNotExist(entity, _) => {
                ui.label(layout_job(&[
                    proportional("Component "),
                    monospace(type_name),
                    proportional(" does not exist on entity "),
                    monospace(&format!("{entity:?}")),
                    proportional("."),
                ]));
            }
            Self::ResourceDoesNotExist(_) => {
                ui.label(layout_job(&[
                    proportional("Resource "),
                    monospace(type_name),
                    proportional(" does not exist in the world."),
                ]));
            }
            Self::NoComponentId(_type_id) => {
                ui.label(layout_job(&[
                    monospace(type_name),
                    proportional(" has no associated "),
                    monospace("ComponentId"),
                    proportional("."),
                ]));
            }
            Self::NoTypeRegistration(_type_id) => {
                ui.label(layout_job(&[
                    monospace(type_name),
                    proportional(" is not registered in the "),
                    monospace("TypeRegistry"),
                ]));
            }
            Self::NoTypeData(_type_id, type_data) => no_type_data(ui, type_name, type_data),
        }
    }
}

fn layout_job(text: &[(egui::FontId, &str)]) -> egui::epaint::text::LayoutJob {
    let mut job = egui::epaint::text::LayoutJob::default();
    for (font_id, text) in text {
        let format = egui::TextFormat::simple(font_id.clone(), egui::Color32::GRAY);
        job.append(text, 0.0, format);
    }
    job
}

const fn monospace(text: &str) -> (egui::FontId, &str) {
    (egui::FontId::monospace(9.0), text)
}

const fn proportional(text: &str) -> (egui::FontId, &str) {
    (egui::FontId::proportional(10.0), text)
}

pub fn reflect_value_no_impl(ui: &mut egui::Ui, type_name: &str) {
    let name = pretty_type_name::pretty_type_name_str(type_name);
    ui.label(layout_job(&[
        monospace(type_name),
        proportional(" is "),
        monospace("#[reflect_value]"),
        proportional(", but has no "),
        monospace("InspectorEguiImpl"),
        proportional(" registered in the "),
        monospace("TypeRegistry"),
        proportional(" .\n"),
        proportional("Try calling "),
        monospace(&format!(".register_type::<{}>", name)),
        proportional(" or add the "),
        monospace("DefaultInspectorConfigPlugin"),
        proportional(" for builtin types."),
    ]));
}

pub fn no_default_value(ui: &mut egui::Ui, type_name: &str) {
    ui.label(layout_job(&[
        monospace(type_name),
        proportional(" has no "),
        monospace("ReflectDefault"),
        proportional(" type data, so no value of it can be constructed."),
    ]));
}

pub fn unconstructable_variant(
    ui: &mut egui::Ui,
    type_name: &str,
    variant: &str,
    unconstructable_field_types: &[&str],
) {
    let mut vec = Vec::with_capacity(2 + unconstructable_field_types.len() * 2 + 4);

    let qualified_variant = format!(
        "{}::{}",
        pretty_type_name::pretty_type_name_str(type_name),
        variant
    );
    vec.extend([
        monospace(qualified_variant.as_str()),
        proportional(" has unconstructable fields.\nConsider adding "),
        monospace("#[reflect(Default)]"),
        proportional(" to\n\n"),
    ]);
    vec.extend(
        unconstructable_field_types
            .iter()
            .flat_map(|variant| [proportional("- "), monospace(variant)]),
    );

    let job = layout_job(&vec);

    ui.label(job);
}

pub fn no_multiedit(ui: &mut egui::Ui, type_name: &str) {
    ui.label(layout_job(&[
        monospace(type_name),
        proportional(" doesn't support multi-editing."),
    ]));
}

pub fn no_type_data(ui: &mut egui::Ui, type_name: &str, type_data: &str) {
    ui.label(layout_job(&[
        monospace(type_name),
        proportional(" has no "),
        monospace(type_data),
        proportional(" type data, so it cannot be displayed"),
    ]));
}

pub fn entity_does_not_exist(ui: &mut egui::Ui, entity: Entity) {
    ui.label(layout_job(&[
        proportional("Entity "),
        monospace(&format!("{entity:?}")),
        proportional(" does not exist."),
    ]));
}

pub fn no_world_in_context(ui: &mut egui::Ui, type_name: &str) {
    ui.label(layout_job(&[
        monospace(type_name),
        proportional(" needs the bevy world in the "),
        monospace("InspectorUi"),
        proportional(" context to provide meaningful information."),
    ]));
}

pub fn dead_asset_handle(ui: &mut egui::Ui, handle: UntypedAssetId) {
    ui.label(layout_job(&[
        proportional("Handle "),
        monospace(&format!("{handle:?}")),
        proportional(" points to no asset."),
    ]));
}

pub fn state_does_not_exist(ui: &mut egui::Ui, name: &str) {
    ui.label(layout_job(&[
        proportional("State "),
        monospace(name),
        proportional(" does not exist. Did you forget to call "),
        monospace(&format!(".add_state::<{name}>(..)")),
        proportional("?"),
    ]));
}

pub fn no_type_id(ui: &mut egui::Ui, component_name: &str) {
    ui.label(layout_job(&[
        monospace(component_name),
        proportional(" is not backed by a rust type, so it cannot be displayed."),
    ]));
}

pub fn name_of_type(type_id: TypeId, registry: &'_ TypeRegistry) -> Cow<'_, str> {
    registry.get(type_id).map_or_else(
        || Cow::Owned(format!("{type_id:?}")),
        |reg| Cow::Borrowed(reg.type_info().type_path_table().short_path()),
    )
}
