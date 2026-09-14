use super::component::ProxyMeta;
use super::context::{ReflectEntityGetters, SceneEditorContext};
use super::{InspectorState, SceneMapping};
use crate::ui::widget::hierarchy::{HierarchyItemStyle, HierarchyItemWidget, ProxyMetaState};
use crate::ui::{EditorTab, Style};
use bevy::ecs::system::lifetimeless::{SRes, SResMut};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::ReflectRef;
use std::borrow::Cow;
use std::collections::VecDeque;

#[derive(Default, Component)]
pub struct Hierarchy {
    search: String,
}

impl EditorTab for Hierarchy {
    type Param = (
        SceneEditorContext<'static, 'static>,
        SResMut<SceneMapping>,
        SRes<Style>,
        SResMut<InspectorState>,
    );

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (ctx, mapping, style, selection): &mut SystemParamItem<'_, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            style.set_theme_visuals(ui);

            /*
            ui.horizontal(|ui| {
                let frame = egui::Frame::none().inner_margin(egui::Margin::symmetric(3.0, 3.0));
                frame.fill(style.tab_base).show(ui, |ui| {
                    let text = egui::TextEdit::singleline(&mut self.search);
                    ui.add(text.desired_width(f32::INFINITY));
                });
            });
            */

            style.for_scrollbar(ui);

            let scroll = egui::ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.id_source("hierarchy scroll").show(ui, |ui| {
                style.scrollarea(ui);

                let style = HierarchyItemStyle::new(style);

                struct Item {
                    level: usize,
                    entity: usize,
                }

                let root_indices = 0..ctx.scene_mut().entities.len();

                let mut nodes: VecDeque<Item> = root_indices
                    .into_iter()
                    .filter_map(|entity| {
                        if ctx.get(entity).unwrap().without::<Parent>() {
                            let level = 0;
                            Some(Item { level, entity })
                        } else {
                            None
                        }
                    })
                    .collect();

                while let Some(Item { level, entity }) = nodes.pop_front() {
                    let editor = ctx.get(entity).unwrap();

                    let widget = HierarchyItemWidget {
                        id: egui::Id::new((entity, "#hierarchy_item")),
                        level,
                        style: &style,
                        is_selected: selection.lock_or(None) == Some(entity),
                        has_children: editor
                            .children()
                            .map_or(false, |children| !children.is_empty()),
                        meta: editor
                            .component_ref::<ProxyMeta>()
                            .and_then(|meta| match meta {
                                ReflectRef::Struct(s) => Some(ProxyMetaState {
                                    icon: s.get_field("icon").copied()?,
                                    name: s
                                        .get_field::<Cow<'static, str>>("name")
                                        .map(ToString::to_string)?,
                                    is_visible: s.get_field("is_visible").copied()?,
                                }),
                                _ => None,
                            }),
                    };

                    let response = widget.ui(ui);

                    if response.show_children {
                        if let Some(children) = editor.children() {
                            let children: Vec<u32> = children
                                .iter()
                                .map(|e| e.downcast_ref::<Entity>().unwrap().index())
                                .collect();

                            let level = level + 1;
                            for entity in children.iter().rev() {
                                let entity = mapping.entity[entity];
                                nodes.push_front(Item { level, entity })
                            }
                        }
                    }

                    if let Some(is_visible) = response.is_visible {
                        let dst = editor
                            .entity
                            .field_mut::<ProxyMeta, bool>("is_visible")
                            .unwrap();

                        *dst = is_visible;
                    }

                    if response.just_selected {
                        selection.select(entity);
                    }
                }
            });
        });
    }
}
