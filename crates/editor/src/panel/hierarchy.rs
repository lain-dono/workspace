use crate::inspector::guess_entity_name;
use crate::ui::{style::HierarchyItemStyle, EditorStage};
use bevy::ecs::prelude::*;
use bevy::prelude::*;
use std::collections::VecDeque;

pub type ContextMenu<T> = dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T);
pub type CustomEntity<T> = dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T) -> bool;

#[derive(Component, Default)]
pub struct HierarchyTab;

impl HierarchyTab {
    pub fn add_panel(app: &mut App) {
        app.init_resource::<Selection>();
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    /// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
    pub fn panel(world: &mut World) {
        super::run_panel_scrollbar::<With<Self>>(world, |_, world, ui| {
            world.resource_scope(|world, mut selected: Mut<Selection>| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                struct Item {
                    level: usize,
                    entity: Entity,
                    siblings: Vec<Entity>,
                }

                let mut root = world.query_filtered::<Entity, Without<Parent>>();
                let mut root: Vec<Entity> = root.iter(world).collect();

                root.sort();

                let mut nodes: VecDeque<Item> = root
                    .iter()
                    .copied()
                    .map(|entity| Item {
                        level: 0,
                        entity,
                        siblings: root.clone(),
                    })
                    .collect();

                let style = HierarchyItemStyle::default();

                while let Some(Item {
                    level,
                    entity,
                    siblings,
                }) = nodes.pop_front()
                {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    use crate::ui::widget::hierarchy::*;

                    let is_selected = selected.contains(entity);

                    let entity_name = guess_entity_name(world.as_unsafe_world_cell(), entity);
                    //let mut name = egui::RichText::new(entity_name.clone());

                    let children = world.get::<Children>(entity);
                    let has_children = !children.is_none_or(|children| children.is_empty());

                    let widget = HierarchyItemWidget {
                        id: egui::Id::new((entity, "#hierarchy")),
                        level,
                        style: &style,
                        is_selected,
                        has_children,
                        hover_text: Some(format!(
                            "Entity.index = {}\nEntity.generation = {}",
                            entity.index(),
                            entity.generation()
                        )),

                        name: entity_name,
                        icon: Some(crate::ui::icon::MESH_CUBE.into()),
                        is_visible: None,
                    };

                    let response = widget.ui(ui);

                    if response.just_selected {
                        let selection_mode = ui.input(|input| {
                            SelectionMode::from_ctrl_shift(
                                input.modifiers.ctrl,
                                input.modifiers.shift,
                            )
                        });

                        let extend_with = |from, to| {
                            // PERF: this could be done in one scan
                            let from = siblings.iter().position(|&entity| entity == from);
                            let to = siblings.iter().position(|&entity| entity == to);
                            from.zip(to)
                                .map(|(from, to)| {
                                    let (min, max) =
                                        if from < to { (from, to) } else { (to, from) };
                                    siblings[min..=max].iter().copied()
                                })
                                .into_iter()
                                .flatten()
                        };

                        selected.select(selection_mode, entity, extend_with);
                    }

                    if response.show_children {
                        if let Some(children) = children {
                            let siblings: Vec<Entity> = children.iter().copied().collect();
                            let level = level + 1;
                            for &entity in children.iter().rev() {
                                nodes.push_front(Item {
                                    level,
                                    entity,
                                    siblings: siblings.clone(),
                                })
                            }
                        }
                    }
                }
            });
        });
    }
}

/// Kind of selection modifier
#[derive(Debug, Clone, Copy)]
pub enum SelectionMode {
    /// No modifiers
    Replace,
    /// `Ctrl`
    Add,
    /// `Shift`
    Extend,
}

impl SelectionMode {
    pub fn from_ctrl_shift(ctrl: bool, shift: bool) -> SelectionMode {
        match (ctrl, shift) {
            (true, _) => SelectionMode::Add,
            (false, true) => SelectionMode::Extend,
            (false, false) => SelectionMode::Replace,
        }
    }
}

/// Collection of currently selected entities
#[derive(Resource, Default, Debug)]
pub struct Selection {
    pub entities: Vec<Entity>,
    pub last_action: Option<(SelectionMode, Entity)>,
}

impl Selection {
    pub fn select_replace(&mut self, entity: Entity) {
        self.entities.clear();
        self.entities.push(entity);
        self.last_action = Some((SelectionMode::Replace, entity));
    }

    pub fn select_maybe_add(&mut self, entity: Entity, add: bool) {
        let mode = match add {
            true => SelectionMode::Add,
            false => SelectionMode::Replace,
        };
        self.select(mode, entity, |_, _| std::iter::empty());
    }

    pub fn select<I: IntoIterator<Item = Entity>>(
        &mut self,
        mode: SelectionMode,
        entity: Entity,
        extend_with: impl Fn(Entity, Entity) -> I,
    ) {
        match mode {
            _ if self.is_empty() => self.insert_unchecked(entity),
            SelectionMode::Replace => {
                self.entities.clear();
                self.entities.push(entity);
            }
            SelectionMode::Add => {
                if self.remove(entity).is_none() {
                    self.entities.push(entity);
                }
            }
            SelectionMode::Extend => {
                if let Some((last_mode, last_entity)) = self.last_action {
                    if let SelectionMode::Add | SelectionMode::Replace = last_mode {
                        self.clear()
                    }
                    for entity in extend_with(entity, last_entity) {
                        self.insert_unchecked(entity);
                    }

                    // extending doesn't update last action
                    return;
                } else {
                    self.insert_unchecked(entity)
                }
            }
        }

        self.last_action = Some((mode, entity));
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    fn insert_unchecked(&mut self, entity: Entity) {
        if !self.contains(entity) {
            self.entities.push(entity);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let index = self.entities.iter().position(|&e| e == entity);
        index.map(|index| self.entities.remove(index))
    }

    pub fn last_action(&self) -> Option<(SelectionMode, Entity)> {
        self.last_action
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn retain(&mut self, f: impl Fn(Entity) -> bool) {
        self.entities.retain(|&entity| f(entity));
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }

    pub fn as_slice(&self) -> &[Entity] {
        self.entities.as_slice()
    }
}
