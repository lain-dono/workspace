use super::component::{reflect_component_editor, ComponentEditor, ReflectComponentEditor};
use super::context::SceneEditorContext;
use crate::ui::{EditorTab, Style};
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::{FromType, Reflect, TypeInfo, TypeRegistry, TypeRegistryArc};
use egui::style::Margin;
use egui::*;

#[derive(Default, Resource)]
pub struct InspectorState {
    entity: Option<usize>,
}

impl InspectorState {
    pub fn select(&mut self, entity: usize) {
        self.entity = Some(entity);
    }

    pub fn deselect(&mut self) {
        self.entity = None;
    }

    pub fn lock_or(&self, lock: Option<usize>) -> Option<usize> {
        lock.or(self.entity)
    }
}

#[derive(Default, Component)]
pub struct Inspector {
    lock: Option<usize>,
}

impl EditorTab for Inspector {
    type Param = (
        SceneEditorContext<'static, 'static>,
        SRes<Style>,
        SRes<InspectorState>,
        SRes<AppTypeRegistry>,
    );

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (ctx, style, selection, type_registry): &mut SystemParamItem<'_, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        let entity = selection
            .lock_or(self.lock)
            .and_then(|index| ctx.get(index));

        let entity = match entity {
            Some(entity) => entity,
            None => {
                Frame::none()
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| ui.label("Select something..."));
                    });
                return;
            }
        };

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            style.set_theme_visuals(ui);
            style.for_scrollbar(ui);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                style.scrollarea(ui);

                let frame = Frame::none();
                frame.fill(style.panel).show(ui, |ui| {
                    for component in entity.entity.components.iter_mut().map(AsMut::as_mut) {
                        let type_info = component.get_represented_type_info().unwrap();

                        {
                            use bevy::prelude::*;
                            add_custom_editor_if::<Parent>(entity.types, type_name);
                            add_custom_editor_if::<Children>(entity.types, type_name);
                        }

                        let registry = entity.types.read();
                        let registration = match registry.get_with_name(type_name) {
                            Some(registration) => registration,
                            None => continue,
                        };

                        if let Some(editor) = registration.data::<ReflectComponentEditor>() {
                            if editor.skip() {
                                continue;
                            }
                            editor.ui(ui, style, type_registry, component);
                        } else {
                            let name = registration.short_name();
                            reflect_component_editor(
                                ui,
                                style,
                                type_registry,
                                component,
                                ' ',
                                name,
                            );
                        }

                        let width = ui.available_width();
                        let (_, separator) = ui.allocate_space(vec2(width, 1.0));
                        ui.painter().rect_filled(separator, 0.0, style.separator);
                    }
                });
            });
        });
    }
}

fn add_custom_editor_if<T: ComponentEditor + Reflect + 'static>(
    types: &TypeRegistryArc,
    type_info: &TypeInfo,
) {
    if type_info.is::<T>() {
        let mut registry = types.write();
        if let Some(registration) = registry.get_with_type_path_mut(type_info.type_path()) {
            let data: ReflectComponentEditor = FromType::<T>::from_type();
            registration.insert(data);
        }
    }
}
