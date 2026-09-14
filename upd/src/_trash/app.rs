pub use super::dock::{LeafNode, Node, NodeIndex, Split, SplitNode, Tab, TabIndex, Tree};
use crate::style::Style;
use bevy::ecs::system::{SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Rect};

use crate::ui::tab::TabTitle;

pub trait EditorTab: Component<Mutability = bevy::ecs::component::Mutable> {
    type Param: SystemParam;

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        query: &mut SystemParamItem<'_, '_, Self::Param>,
    );
}

struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    dst: NodeIndex,
    pointer: egui::Pos2,
}

impl HoverData {
    fn resolve(&self) -> (Option<Split>, Rect) {
        if let Some(tabs) = self.tabs {
            return (None, tabs);
        }

        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();
        let pts = [
            center.distance(pointer),
            rect.left_center().distance(pointer),
            rect.right_center().distance(pointer),
            rect.center_top().distance(pointer),
            rect.center_bottom().distance(pointer),
        ];

        let position = pts
            .into_iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| f32::total_cmp(lhs, rhs))
            .map(|(idx, _)| idx)
            .unwrap();

        let (target, other) = match position {
            0 => (None, Rect::EVERYTHING),
            1 => (Some(Split::Left), Rect::everything_left_of(center.x)),
            2 => (Some(Split::Right), Rect::everything_right_of(center.x)),
            3 => (Some(Split::Above), Rect::everything_above(center.y)),
            4 => (Some(Split::Below), Rect::everything_below(center.y)),
            _ => unreachable!(),
        };

        (target, rect.intersect(other))
    }
}

#[derive(Default, Resource)]
pub struct SharedData {
    drag: Option<(NodeIndex, usize)>,
    hover: Option<HoverData>,
}

#[allow(clippy::only_used_in_recursion)]
pub fn ui_root(
    mut drag_start: Local<Option<egui::Pos2>>,
    mut context: EguiContexts,
    mut tree: ResMut<Tree<crate::dock::Tab>>,
    style: Res<Style>,
    mut shared: ResMut<SharedData>,
) -> Result {
    let ctx = context.ctx_mut()?;

    let (rect, mut ui) = {
        {
            let mut style = egui::Style::clone(&ctx.style());

            let corner_radius = egui::CornerRadius::ZERO;

            style.visuals.widgets.noninteractive.corner_radius = corner_radius;
            style.visuals.widgets.inactive.corner_radius = corner_radius;
            style.visuals.widgets.hovered.corner_radius = corner_radius;
            style.visuals.widgets.active.corner_radius = corner_radius;
            style.visuals.widgets.open.corner_radius = corner_radius;
            style.visuals.window_corner_radius = corner_radius;

            ctx.set_style(style);
        }

        let rect = ctx.available_rect();
        let id = egui::Id::new("#_SHADERLAB_#");

        let ui_builder = egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(rect);

        (rect, egui::Ui::new(ctx.clone(), id, ui_builder))
    };

    let tabbar_bg_idx = ui.painter().add(egui::Shape::Noop);

    let topbar = {
        let egui::InnerResponse { response, .. } = egui::menu::bar(&mut ui, |ui| {
            ui.spacing_mut().button_padding.x += 2.0;
            ui.spacing_mut().button_padding.y += 4.0;

            let space = ui.spacing().button_padding.x;
            ui.add_space(space * 2.0);

            //style.theme(ui);
            {
                let mut visuals = ui.visuals().clone();
                visuals.widgets.noninteractive.bg_stroke.width = 0.0;
                visuals.widgets.inactive.bg_stroke.width = 0.0;
                visuals.widgets.hovered.bg_stroke.width = 0.0;
                visuals.widgets.active.bg_stroke.width = 0.0;
                visuals.widgets.open.bg_stroke.width = 0.0;

                visuals.popup_shadow = egui::epaint::Shadow::default();

                ui.ctx().set_visuals(visuals);
            }

            fn nested_menus(ui: &mut egui::Ui) {
                if ui.button("New...").clicked() {
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Open...").clicked() {
                    ui.close_kind(egui::UiKind::Menu);
                }

                ui.menu_button("Next", nested_menus);
            }

            ui.menu_button("File", nested_menus);
            ui.menu_button("Edit", nested_menus);
            ui.menu_button("Assets", nested_menus);
            ui.menu_button("Objects", nested_menus);
            ui.menu_button("Components", nested_menus);
            ui.menu_button("Window", nested_menus);
        });

        response.rect
    };

    style.set_theme_visuals(&mut ui);

    ui.painter().set(
        tabbar_bg_idx,
        egui::Shape::rect_filled(topbar, 0.0, style.app_bg),
    );

    if tree.is_empty() || tree[NodeIndex::root()].is_empty() {
        ui.painter().rect_filled(rect, 0.0, style.app_bg);
        // TODO: splash screen here?
        return Ok(());
    }

    let rect = {
        let rect = rect.intersect(Rect::everything_below(topbar.height()));
        let separator = style.separator_size;
        let corners = [
            rect.intersect(Rect::everything_above(rect.min.y + separator)),
            rect.intersect(Rect::everything_below(rect.max.y - separator)),
            rect.intersect(Rect::everything_left_of(rect.min.x + separator + 2.0)),
            rect.intersect(Rect::everything_right_of(rect.max.x - separator - 2.0)),
        ];
        for rect in corners {
            ui.painter().rect_filled(rect, 0.0, style.app_bg);
        }
        rect.shrink2(egui::vec2(separator + 2.0, separator))
    };

    tree[NodeIndex::root()].set_rect(rect);

    shared.drag = None;
    shared.hover = None;

    let pixels_per_point = ui.ctx().pixels_per_point();
    let px = pixels_per_point.recip();

    for tree_index in 0..tree.len() {
        let tree_index = NodeIndex(tree_index);
        match &mut tree[tree_index] {
            Node::Empty => (),

            Node::Horizontal(SplitNode { fraction, rect, .. }) => {
                let rect = crate::util::expand_to_pixel(*rect, pixels_per_point);
                ui.set_clip_rect(rect);

                let (left, separator, right) = style.hsplit(&mut ui, fraction, rect);
                ui.painter().rect_filled(separator, 0.0, style.app_bg);

                tree[tree_index.left()].set_rect(left);
                tree[tree_index.right()].set_rect(right);
            }

            Node::Vertical(SplitNode { fraction, rect, .. }) => {
                let rect = crate::util::expand_to_pixel(*rect, pixels_per_point);
                ui.set_clip_rect(rect);

                let (bottom, separator, top) = style.vsplit(&mut ui, fraction, rect);
                ui.painter().rect_filled(separator, 0.0, style.app_bg);

                tree[tree_index.left()].set_rect(bottom);
                tree[tree_index.right()].set_rect(top);
            }

            Node::Leaf(LeafNode {
                rect,
                tabs,
                active,
                viewport,
                ..
            }) => {
                let rect = *rect;
                ui.set_clip_rect(rect);

                let height_topbar = 24.0;

                let bottom_y = rect.min.y + height_topbar;
                let tabbar = rect.intersect(Rect::everything_above(bottom_y));

                let full_response = ui.allocate_rect(rect, egui::Sense::hover());
                let tabs_response = ui.allocate_rect(tabbar, egui::Sense::hover());

                // tabs
                {
                    ui.painter().rect_filled(tabbar, 0.0, style.app_bg);
                    ui.painter()
                        .rect_filled(tabbar, style.tab_rounding, style.tab_bar);

                    let a = egui::pos2(tabbar.min.x, tabbar.max.y - px);
                    let b = egui::pos2(tabbar.max.x, tabbar.max.y - px);
                    ui.painter().line_segment([a, b], (px, style.tab_outline));

                    let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(tabbar));
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.horizontal(|ui| {
                        for (tab_index, tab) in tabs.iter().enumerate() {
                            let widget = TabTitle {
                                label: tab.to_string(),
                                active: *active == TabIndex(tab_index),
                                style: &style,
                            };

                            let id = egui::Id::new((tree_index, tab_index, "tab"));
                            let is_being_dragged = ui.ctx().is_being_dragged(id);

                            if is_being_dragged {
                                let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
                                let ui_builder = egui::UiBuilder::new().layer_id(layer_id);
                                let response =
                                    ui.scope_builder(ui_builder, |ui| ui.add(widget)).response;

                                let sense = egui::Sense::click_and_drag();
                                let response = ui
                                    .interact(response.rect, id, sense)
                                    .on_hover_cursor(egui::CursorIcon::Grabbing);

                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let center = response.rect.center();
                                    let start = drag_start.unwrap_or(center);

                                    let delta = pointer_pos - start;
                                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                                        // ui.ctx().translate_layer(layer_id, delta);
                                        let transform =
                                            egui::emath::TSTransform::from_translation(delta);
                                        ui.ctx().transform_layer_shapes(layer_id, transform);

                                        shared.drag = Some((tree_index, tab_index));
                                    }
                                }

                                if response.clicked() {
                                    *active = TabIndex(tab_index);
                                }
                            } else {
                                let response = ui.scope(|ui| ui.add(widget)).response;
                                let sense = egui::Sense::click_and_drag();
                                let response = ui.interact(response.rect, id, sense);
                                if response.drag_started() {
                                    *drag_start = response.hover_pos();
                                } else if response.clicked() {
                                    *active = TabIndex(tab_index);
                                }
                            }
                        }
                    });
                }

                // tab body
                let top_y = rect.min.y + height_topbar;
                let rect = rect.intersect(Rect::everything_below(top_y));
                let rect = crate::util::expand_to_pixel(rect, pixels_per_point);
                *viewport = rect;

                let is_being_dragged = ui.ctx().dragged_id().is_some();
                if is_being_dragged && full_response.hovered() {
                    shared.hover = ui.input(|i| {
                        i.pointer.hover_pos().map(|pointer| HoverData {
                            rect,
                            dst: tree_index,
                            tabs: tabs_response.hovered().then_some(tabs_response.rect),
                            pointer,
                        })
                    });
                }
            }
        }
    }
    Ok(())
}

#[derive(Component, Default)]
pub struct EditorPanel {
    pub viewport: Option<Rect>,
}

pub fn ui_tabs(tree: Res<Tree<crate::dock::Tab>>, mut panels: Query<&mut EditorPanel>) {
    for mut panel in panels.iter_mut() {
        panel.viewport = None;
    }

    for node in tree.iter() {
        if let &Node::Leaf(LeafNode {
            ref tabs,
            active,
            viewport,
            ..
        }) = node
        {
            for (tab_index, tab) in tabs.iter().enumerate() {
                if let Ok(mut panel) = panels.get_mut(tab.entity) {
                    *panel = EditorPanel {
                        viewport: (TabIndex(tab_index) == active).then_some(viewport),
                        //node: NodeIndex(node_index),
                        //tab: tab_index,
                    }
                }
            }
        }
    }
}

pub fn ui_finish(
    mut context: EguiContexts,
    mut tree: ResMut<Tree<crate::dock::Tab>>,
    style: Res<Style>,
    shared: Res<SharedData>,
) -> Result {
    let ctx = context.ctx_mut()?;

    let (Some((src, tab_index)), Some(hover)) = (shared.drag, &shared.hover) else {
        return Ok(());
    };

    let dst = hover.dst;

    if !tree[src].is_leaf() || !tree[dst].is_leaf() {
        return Ok(());
    }

    let (target, helper) = hover.resolve();

    let id = egui::Id::new("helper");
    let layer_id = egui::LayerId::new(egui::Order::Foreground, id);
    let painter = ctx.layer_painter(layer_id);
    painter.rect_filled(helper, 0.0, style.selection);

    if ctx.input(|input| input.pointer.any_released()) {
        if let Node::Leaf(node) = &mut tree[src]
            && node.active.0 >= tab_index
        {
            node.active.0 = node.active.0.saturating_sub(1);
        }

        let tab = tree[src].remove_tab(tab_index).unwrap();

        if let Some(target) = target {
            tree.split(dst, target, 0.5, Node::leaf(vec![tab]));
        } else {
            tree[dst].append_tab(tab);
        }

        tree.remove_empty_leaf();
        for node in tree.iter_mut() {
            if let Node::Leaf(node) = node
                && node.active.0 >= node.tabs.len()
            {
                node.active.0 = 0;
            }
        }
    }

    Ok(())
}
