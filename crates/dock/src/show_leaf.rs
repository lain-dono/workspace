use super::{
    dock_area::DockArea,
    drag_and_drop::{DragData, DragDropState, HoverData, TreeComponent},
    draw::{draw_arrow, draw_chevron_down, draw_close_window_symbol},
    state::State,
    style::{Style, TabAddAlign, TabBodyStyle, TabStyle},
    tab_viewer::{OnCloseResponse, TabViewer},
    tree::{Node, NodeIndex, SurfaceIndex, TabIndex, TabRemoval},
    utils::{fade_visuals, rect_set_size_centered, rect_stroke_box},
};
use egui::{
    Align, Align2, Button, Color32, CornerRadius, CursorIcon, Frame, Id, Key, LayerId, Layout,
    NumExt, Order, Popup, PopupCloseBehavior, Rect, Response, ScrollArea, Sense, StrokeKind,
    TextStyle, Ui, UiBuilder, Vec2, WidgetText, emath::TSTransform, epaint::TextShape, lerp, pos2,
    vec2,
};

mod tabs;

use self::tabs::TabTitle;

#[allow(clippy::too_many_arguments)]
impl<Tab> DockArea<'_, Tab> {
    pub(super) fn show_leaf(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        fade_style: Option<(&Style, f32)>,
    ) {
        let leaf = &mut self.state[surface][node];

        assert!(leaf.is_leaf());
        let collapsed = leaf.is_collapsed();

        let rect = leaf.rect().expect("This node must be a leaf");
        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down_justified(Align::Min))
                .id_salt((node, "node")),
        );
        let spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.set_clip_rect(rect);

        if leaf.num_tabs() == 0 {
            return;
        }
        let tabbar_rect = self.tab_bar(
            ui,
            state,
            surface,
            node,
            tab_viewer,
            fade_style.map(|(style, _)| style),
            collapsed,
        );
        self.tab_body(
            ui,
            state,
            surface,
            node,
            tab_viewer,
            spacing,
            tabbar_rect,
            fade_style,
            collapsed,
        );

        let tabs = self.state[surface][node]
            .get_tabs_mut()
            .expect("This node must be a leaf here");
        for (tab, state) in tabs.iter_mut().enumerate() {
            if tab_viewer.force_close(state) {
                self.to_remove
                    .push(TabRemoval::ForcedTab(surface, node, TabIndex(tab)));
            }
        }
    }

    fn tab_bar(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        fade_style: Option<&Style>,
        collapsed: bool,
    ) -> Rect {
        let leaf = &mut self.state[surface][node];
        assert!(leaf.is_leaf());

        let style = fade_style.unwrap_or(&self.style);
        let (tabbar_outer_rect, tabbar_response) = ui.allocate_exact_size(
            vec2(ui.available_width(), style.tab_bar.height),
            Sense::hover(),
        );
        ui.painter().rect_filled(
            tabbar_outer_rect,
            style.tab_bar.corner_radius,
            style.tab_bar.bg_fill,
        );

        let tabbar_outer_rect = tabbar_outer_rect - style.tab_bar.inner_margin;

        let mut available_width = tabbar_outer_rect.width();
        let scroll_bar_width = available_width;
        if available_width == 0.0 {
            return tabbar_outer_rect;
        }

        // Reserve space for the buttons at the ends of the tab bar.

        if self.config.show_add_buttons {
            available_width -= Style::TAB_ADD_BUTTON_SIZE;
        }

        if self.config.show_leaf_close_all_buttons {
            available_width -= Style::TAB_CLOSE_ALL_BUTTON_SIZE;
        }

        if self.config.show_leaf_collapse_buttons {
            available_width -= Style::TAB_COLLAPSE_BUTTON_SIZE;
        }

        let (actual_width, tab_hovered) = {
            let leaf = leaf.leaf_mut().expect("This node must be a leaf");

            let x = leaf.scroll
                + if self.config.show_leaf_collapse_buttons {
                    Style::TAB_COLLAPSE_BUTTON_SIZE
                } else {
                    0.0
                };
            let tabbar_inner_rect = tabbar_outer_rect.translate(vec2(x, 0.0));

            let tabs_ui = &mut ui.new_child(
                UiBuilder::new()
                    .id_salt("tabs")
                    .max_rect(tabbar_inner_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );

            let mut clip_rect = tabbar_outer_rect;
            clip_rect.set_width(available_width);
            if self.config.show_leaf_collapse_buttons {
                clip_rect = clip_rect.translate(vec2(Style::TAB_COLLAPSE_BUTTON_SIZE, 0.0));
            }
            tabs_ui.set_clip_rect(clip_rect);

            // Desired size for tabs in "expanded" mode.
            let preferred_width = if style.tab_bar.fill_tab_bar {
                available_width / (leaf.tabs.len() as f32)
            } else {
                0.0
            };

            let tab_hovered = self.tabs(
                tabs_ui,
                state,
                surface,
                node,
                tab_viewer,
                tabbar_outer_rect,
                preferred_width,
                fade_style,
            );

            // Draw hline from tab end to edge of tab bar.
            let px = ui.ctx().pixels_per_point().recip();
            let style = fade_style.unwrap_or(&self.style);

            ui.painter().hline(
                tabs_ui.min_rect().right().min(clip_rect.right())..=tabbar_outer_rect.right(),
                tabbar_outer_rect.bottom() - px,
                (px, style.tab_bar.hline_color),
            );

            // Add button at the ends of the tab bar.
            if self.config.show_add_buttons {
                let offset = match style.buttons.add_tab_align {
                    TabAddAlign::Left => {
                        (clip_rect.width() - tabs_ui.min_rect().width()).at_least(0.0)
                    }
                    TabAddAlign::Right => 0.0,
                } + if self.config.show_leaf_close_all_buttons {
                    Style::TAB_CLOSE_ALL_BUTTON_SIZE
                } else {
                    0.0
                };
                self.tab_plus(
                    ui,
                    surface,
                    node,
                    tab_viewer,
                    tabbar_outer_rect,
                    offset,
                    fade_style,
                );
            }

            if self.config.show_leaf_close_all_buttons {
                // Current leaf contains non-closable tabs.
                let disabled = self.state[surface][node]
                    .leaf_mut()
                    .map(|leaf| !leaf.tabs.iter_mut().all(|tab| tab_viewer.is_closeable(tab)))
                    .expect("This node must be a leaf");

                // Current window contains non-closable tabs.
                let close_window_disabled = disabled
                    || !self.state[surface].iter_mut().all(|node| {
                        node.leaf_mut().is_none_or(|leaf| {
                            leaf.tabs.iter_mut().all(|tab| tab_viewer.is_closeable(tab))
                        })
                    });

                self.tab_close_all(
                    ui,
                    surface,
                    node,
                    tabbar_outer_rect,
                    fade_style,
                    disabled,
                    close_window_disabled,
                );
            }

            if self.config.show_leaf_collapse_buttons {
                self.tab_collapse(ui, surface, node, tabbar_outer_rect, fade_style, collapsed);
            }

            (tabs_ui.min_rect().width(), tab_hovered)
        };

        self.tab_bar_scroll(
            ui,
            state,
            surface,
            node,
            actual_width,
            available_width,
            scroll_bar_width,
            &tabbar_response,
            tab_hovered,
            fade_style,
        );

        tabbar_outer_rect
    }

    fn tabs(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        tabbar_outer_rect: Rect,
        preferred_width: f32,
        fade: Option<&Style>,
    ) -> bool {
        let mut tab_hovered = false;

        assert!(self.state[surface][node].is_leaf());

        let focused = self.state.focused_leaf();
        let tabs_len = self.state[surface][node].tabs().count();

        for tab in (0..tabs_len).map(TabIndex) {
            let id = self.id.with((surface, "surface", node, "node", tab, "tab"));

            let is_being_dragged = ui.ctx().is_being_dragged(id)
                && ui.input(|i| i.pointer.is_decidedly_dragging())
                && self.config.draggable_tabs;

            if is_being_dragged {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
            }

            let (is_active, label, tab_style, closeable) = {
                let leaf = self.state[surface][node]
                    .leaf_mut()
                    .expect("This node must be a leaf");
                let style = fade.unwrap_or(&self.style);
                let tab_style = tab_viewer.tab_style_override(&leaf.tabs[tab.0], &style.tab);
                (
                    leaf.active == tab || is_being_dragged,
                    tab_viewer.title(&mut leaf.tabs[tab.0]),
                    tab_style.unwrap_or(style.tab.clone()),
                    tab_viewer.is_closeable(&leaf.tabs[tab.0]),
                )
            };

            let show_close_button = self.config.show_close_buttons && closeable;

            if tab.0 != 0 {
                ui.allocate_space(vec2(tab_style.spacing, 0.0));
            }

            let tab_title = TabTitle {
                id,
                label,
                is_focused: is_active && Some((surface, node)) == focused,
                is_active,
                is_being_dragged,
                preferred_width,
                show_close_button,
                draggable: self.config.draggable_tabs,
            };

            let (response, title_id) = if is_being_dragged {
                let layer_id = LayerId::new(Order::Tooltip, id);
                let response = ui
                    .scope_builder(UiBuilder::new().layer_id(layer_id), |ui| {
                        tab_title.show(ui, &tab_style, fade.unwrap_or(&self.style))
                    })
                    .response;

                let title_id = response.id;

                let response =
                    ui.interact(response.rect, id.with("dragged"), Sense::click_and_drag());

                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                    let start = *state.drag_start.get_or_insert(pointer_pos);
                    let delta = pointer_pos - start;
                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                        ui.ctx()
                            .transform_layer_shapes(layer_id, TSTransform::new(delta, 1.0));

                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(
                                self.id.with("drag_data"),
                                Some(DragData {
                                    src: TreeComponent::Tab(surface, node, tab),
                                    rect: self.state[surface][node].rect().unwrap(),
                                }),
                            );
                        });
                    }
                }

                (response, title_id)
            } else {
                let (response, close_response) =
                    tab_title.show(ui, &tab_style, fade.unwrap_or(&self.style));
                let title_id = response.id;
                let close_clicked = close_response.is_some_and(|res| res.clicked());
                let is_lonely_tab = self.state[surface].tabs().count() == 1;

                if self.config.tab_context_menus {
                    let eject_button =
                        Button::new(&self.translations.tab_context_menu_eject_button);
                    let close_button =
                        Button::new(&self.translations.tab_context_menu_close_button);

                    response.context_menu(|ui| {
                        let leaf = self.state[surface][node]
                            .leaf_mut()
                            .expect("This node must be a leaf");
                        let leaf_tab = &mut leaf.tabs[tab.0];

                        tab_viewer.context_menu(ui, leaf_tab, surface, node);
                        if (surface.is_main() || !is_lonely_tab)
                            && tab_viewer.allowed_in_windows(leaf_tab)
                            && ui.add(eject_button).clicked()
                        {
                            self.to_detach.push((surface, node, tab));
                            ui.close();
                        }
                        if show_close_button && ui.add(close_button).clicked() {
                            match tab_viewer.on_close(leaf_tab) {
                                OnCloseResponse::Close => {
                                    self.to_remove.push(TabRemoval::Tab(surface, node, tab));
                                }
                                OnCloseResponse::Focus => {
                                    leaf.active = tab;
                                    self.new_focused = Some((surface, node));
                                }
                                OnCloseResponse::Ignore => (),
                            }
                            ui.close();
                        }
                    });
                }

                if close_clicked {
                    self.to_remove.push(TabRemoval::Tab(surface, node, tab));
                }

                if let Some(pos) = state.last_hover_pos {
                    // Use response.rect.contains instead of response.hovered as the dragged tab covers the underlying tab
                    if state.drag_start.is_some() && response.rect.contains(pos) {
                        self.tab_hover_rect = Some((response.rect, tab));
                    }
                }

                (response, title_id)
            };

            if response.hovered() {
                tab_hovered = true;
            }

            // Paint hline below each tab unless its active (or option says otherwise).
            let leaf = self.state[surface][node].leaf_mut().unwrap();
            let leaf_tab = &mut leaf.tabs[tab.0];
            let style = fade.unwrap_or(&self.style);
            let tab_style = tab_viewer.tab_style_override(leaf_tab, &style.tab);
            let tab_style = tab_style.as_ref().unwrap_or(&style.tab);

            if !is_active || tab_style.hline_below_active_tab_name {
                let px = ui.ctx().pixels_per_point().recip();
                ui.painter().hline(
                    response.rect.x_range(),
                    tabbar_outer_rect.bottom() - px,
                    (px, style.tab_bar.hline_color),
                );
            }

            if response.clicked()
                || (ui.memory(|m| m.has_focus(title_id))
                    && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)))
            {
                leaf.active = tab;
                self.new_focused = Some((surface, node));
            }

            tab_viewer.on_tab_button(leaf_tab, &response);

            if self.config.show_close_buttons
                && tab_viewer.is_closeable(leaf_tab)
                && response.middle_clicked()
            {
                self.to_remove.push(TabRemoval::Tab(surface, node, tab));
            }
        }

        tab_hovered
    }

    /// Draws the tab add button.
    fn tab_plus(
        &mut self,
        ui: &mut Ui,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        tabbar_outer_rect: Rect,
        offset: f32,
        fade_style: Option<&Style>,
    ) {
        let rect = Rect::from_min_max(
            tabbar_outer_rect.right_top() - vec2(Style::TAB_ADD_BUTTON_SIZE + offset, 0.0),
            tabbar_outer_rect.right_bottom() - vec2(offset, 2.0),
        );

        let layout = Layout::left_to_right(Align::Center);
        let id = (node, "tab_add");
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect).layout(layout).id_salt(id));

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        response = response.on_hover_cursor(CursorIcon::PointingHand);

        let style = fade_style.unwrap_or(&self.style);
        let color = if response.hovered() || response.has_focus() {
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, style.buttons.add_tab.bg_fill);
            style.buttons.add_tab.active_color
        } else {
            style.buttons.add_tab.color
        };

        let mut plus_rect = rect;

        rect_set_size_centered(&mut plus_rect, Vec2::splat(Style::TAB_ADD_PLUS_SIZE));

        ui.painter().line_segment(
            [plus_rect.center_top(), plus_rect.center_bottom()],
            (1.0, color),
        );
        ui.painter().line_segment(
            [plus_rect.right_center(), plus_rect.left_center()],
            (1.0, color),
        );

        // Draw button left border.
        ui.painter().vline(
            rect.left(),
            rect.y_range(),
            (
                ui.ctx().pixels_per_point().recip(),
                style.buttons.add_tab.border_color,
            ),
        );

        let popup_id = ui.id().with("tab_add_popup");
        if self.config.show_add_popup {
            Popup::from_toggle_button_response(&response)
                .id(popup_id)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    tab_viewer.add_popup(ui, surface, node);
                });
        }

        if response.clicked() {
            tab_viewer.on_add(surface, node);
        }
    }

    /// Draws the close all button.
    fn tab_close_all(
        &mut self,
        ui: &mut Ui,
        surface: SurfaceIndex,
        node: NodeIndex,
        tabbar_outer_rect: Rect,
        fade_style: Option<&Style>,
        disabled: bool,
        close_window_disabled: bool,
    ) {
        let rect = Rect::from_min_max(
            tabbar_outer_rect.right_top() - vec2(Style::TAB_CLOSE_ALL_BUTTON_SIZE, 0.0),
            tabbar_outer_rect.right_bottom() - vec2(0.0, 2.0),
        );

        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center))
                .id_salt((node, "tab_close_all")),
        );

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        let style = fade_style.unwrap_or(&self.style);

        // Whether we're on "secondary button mode" due to modifier keys
        let on_secondary_button = self.is_on_secondary_button(surface, ui, &response);

        let mut stroke_color = if disabled {
            style.buttons.close_all_tabs.disabled_color
        } else if response.hovered() || response.has_focus() {
            if !(close_window_disabled && on_secondary_button) {
                let fill = style.buttons.close_all_tabs.bg_fill;
                ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
            }
            style.buttons.close_all_tabs.active_color
        } else {
            style.buttons.close_all_tabs.color
        };

        let mut close_all_rect = rect;

        rect_set_size_centered(&mut close_all_rect, Vec2::splat(Style::TAB_CLOSE_ALL_SIZE));

        if !disabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }

        if on_secondary_button {
            // Close the entire window
            if close_window_disabled {
                stroke_color = style.buttons.close_all_tabs.disabled_color;
                response = response
                    .on_hover_cursor(CursorIcon::NotAllowed)
                    .on_hover_text(
                        self.translations
                            .leaf_close_all_button_disabled_tooltip
                            .as_str(),
                    );
            }
            draw_close_window_symbol(ui, stroke_color, close_all_rect);
        } else {
            // Close all tabs in this leaf
            if disabled {
                response = response
                    .on_hover_cursor(CursorIcon::NotAllowed)
                    .on_hover_text(
                        self.translations
                            .leaf_close_button_disabled_tooltip
                            .as_str(),
                    );
            } else if !surface.is_main() && self.config.secondary_button_context_menu {
                response.context_menu(|ui| {
                    ui.add_enabled_ui(!close_window_disabled, |ui| {
                        if ui
                            .button(&self.translations.leaf_close_all_button)
                            .on_disabled_hover_text(
                                self.translations
                                    .leaf_close_all_button_disabled_tooltip
                                    .as_str(),
                            )
                            .clicked()
                        {
                            self.to_remove.push(TabRemoval::Window(surface));
                        }
                    });
                });
            }

            if response.clicked() {
                if on_secondary_button {
                    if !close_window_disabled {
                        self.to_remove.push(TabRemoval::Window(surface));
                    }
                } else if !disabled {
                    self.to_remove.push(TabRemoval::Node(surface, node));
                }
            }

            self::tabs::x_icon(ui.painter(), close_all_rect, stroke_color);
        }

        // Draw button left border.
        let px = ui.ctx().pixels_per_point().recip();
        ui.painter().vline(
            rect.left(),
            rect.y_range(),
            (px, style.buttons.close_all_tabs.border_color),
        );

        if !disabled && !on_secondary_button {
            _ = self.show_tooltip_hints(surface, response);
        }
    }

    /// Draws the collapse button.
    fn tab_collapse(
        &mut self,
        ui: &mut Ui,
        surface: SurfaceIndex,
        node: NodeIndex,
        tabbar_outer_rect: Rect,
        fade_style: Option<&Style>,
        collapsed: bool,
    ) {
        let rect = Rect::from_min_max(
            tabbar_outer_rect.left_top(),
            tabbar_outer_rect.left_bottom() + vec2(Style::TAB_COLLAPSE_BUTTON_SIZE, 0.0),
        );

        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center))
                .id_salt((node, "tab_collapse")),
        );

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        response = response.on_hover_cursor(CursorIcon::PointingHand);

        let style = fade_style.unwrap_or(&self.style);

        // Whether we're on "secondary button mode" due to modifier keys
        let on_secondary_button = self.is_on_secondary_button(surface, ui, &response);

        let color = if response.hovered() || response.has_focus() {
            ui.painter()
                .rect_filled(rect, 0, style.buttons.collapse_tabs.bg_fill);
            style.buttons.collapse_tabs.active_color
        } else {
            style.buttons.collapse_tabs.color
        };

        let mut arrow_rect = rect;
        rect_set_size_centered(&mut arrow_rect, Vec2::splat(Style::TAB_COLLAPSE_ARROW_SIZE));

        if on_secondary_button {
            // Collapse the entire window
            draw_chevron_down(ui, style, color, arrow_rect);
        } else {
            // Draw arrow.
            draw_arrow(collapsed, ui, color, arrow_rect);
        }

        // Draw button right border.
        let px = ui.ctx().pixels_per_point().recip();
        ui.painter().vline(
            rect.right(),
            rect.y_range(),
            (px, style.buttons.collapse_tabs.border_color),
        );

        if response.clicked() {
            if on_secondary_button {
                self.window_toggle_minimized(surface);
            } else {
                self.state[surface][node].set_collapsed(!collapsed);
                self.state[surface].node_update_collapsed(node);
                self.window_update_collapsed(surface, node);
            }
        }

        if !surface.is_main() && self.config.secondary_button_context_menu {
            response.context_menu(|ui| {
                if ui.button(&self.translations.leaf_minimize_button).clicked() {
                    ui.close();
                    self.window_toggle_minimized(surface);
                }
            });
        }

        if !on_secondary_button {
            self.show_tooltip_hints(surface, response);
        }
    }

    fn show_tooltip_hints(&mut self, surface: SurfaceIndex, response: Response) -> Response {
        if !surface.is_main()
            && self.config.show_secondary_button_hint
            && (self.config.secondary_button_context_menu
                || self.config.secondary_button_on_modifier)
        {
            let hint = if self.config.secondary_button_context_menu
                && self.config.secondary_button_on_modifier
            {
                &self.translations.leaf_minimize_button_modifier_menu_hint
            } else if self.config.secondary_button_context_menu {
                &self.translations.leaf_minimize_button_menu_hint
            } else {
                &self.translations.leaf_minimize_button_modifier_hint
            };
            response.on_hover_text(hint)
        } else {
            response
        }
    }

    fn is_on_secondary_button(
        &self,
        surface: SurfaceIndex,
        ui: &mut Ui,
        response: &Response,
    ) -> bool {
        !surface.is_main()
            && self.config.secondary_button_on_modifier
            && ui.input(|i| {
                i.modifiers
                    .matches_logically(self.config.secondary_button_modifiers)
            })
            && (response.hovered() || response.has_focus() || response.is_pointer_button_down_on())
    }

    /// Updates the collapsed state of the node and its parents.
    fn window_update_collapsed(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        let tree = &mut self.state[surface];
        let collapsed = tree[node].is_collapsed();
        if !collapsed {
            if let Some((_, window_state)) = self.state.window_mut(surface) {
                window_state.new = true;
            }
        } else if tree.root_node().is_some_and(Node::is_collapsed) {
            let surface_height = if tree.root_node().is_some() {
                tree[NodeIndex::root()].rect().unwrap().height()
            } else {
                0.0
            };
            if let Some((_, window_state)) = self.state.window_mut(surface) {
                window_state.expanded_height = Some(surface_height);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tab_bar_scroll(
        &mut self,
        ui: &mut Ui,
        state: &State,
        surface: SurfaceIndex,
        node: NodeIndex,
        actual_width: f32,
        available_width: f32,
        scroll_bar_width: f32,
        tabbar_response: &Response,
        tab_hovered: bool,
        fade_style: Option<&Style>,
    ) {
        assert_ne!(available_width, 0.0);

        let leaf = self.state[surface][node]
            .leaf_mut()
            .expect("This node must be a leaf");

        let overflow = (actual_width - available_width).at_least(0.0);
        let style = fade_style.unwrap_or(&self.style);

        // Compare to 1.0 and not 0.0 to avoid drawing a scroll bar due
        // to floating point precision issue during tab drawing.
        if overflow > 1.0 {
            if style.tab_bar.show_scroll_bar_on_overflow {
                // Draw scroll bar
                let bar_height = 7.5;
                let (scroll_bar_rect, _scroll_bar_response) = ui.allocate_exact_size(
                    vec2(scroll_bar_width, bar_height),
                    Sense::click_and_drag(),
                );

                // Compute scroll bar handle position and size.
                let overflow_ratio = actual_width / available_width;
                let scroll_ratio = -leaf.scroll / overflow;

                let scroll_bar_handle_size = overflow_ratio.recip() * scroll_bar_rect.width();
                let scroll_bar_handle_start = lerp(
                    scroll_bar_rect.left()..=scroll_bar_rect.right() - scroll_bar_handle_size,
                    scroll_ratio,
                );
                let scroll_bar_handle_rect = Rect::from_min_size(
                    pos2(scroll_bar_handle_start, scroll_bar_rect.min.y),
                    vec2(scroll_bar_handle_size, bar_height),
                );

                let scroll_bar_handle_response = ui.interact(
                    scroll_bar_handle_rect,
                    self.id.with((node, "node")),
                    Sense::drag(),
                );

                // Coefficient to apply to input displacements so that we move the scroll by the correct amount.
                let points_to_scroll_coefficient =
                    overflow / (scroll_bar_rect.width() - scroll_bar_handle_size);

                leaf.scroll -=
                    scroll_bar_handle_response.drag_delta().x * points_to_scroll_coefficient;

                if let Some(pos) = state.last_hover_pos
                    && scroll_bar_rect.contains(pos)
                {
                    leaf.scroll += ui.input(|i| i.smooth_scroll_delta.y + i.smooth_scroll_delta.x)
                        * points_to_scroll_coefficient;
                }

                // Draw the bar.
                ui.painter()
                    .rect_filled(scroll_bar_rect, 0.0, ui.visuals().extreme_bg_color);

                let fill = ui
                    .visuals()
                    .widgets
                    .style(&scroll_bar_handle_response)
                    .bg_fill;
                ui.painter()
                    .rect_filled(scroll_bar_handle_rect, bar_height / 2.0, fill);
            }

            // Handle user input.
            if tabbar_response.hovered() || tab_hovered {
                leaf.scroll += ui.input(|i| i.smooth_scroll_delta.y + i.smooth_scroll_delta.x);
            }
        }

        leaf.scroll = leaf.scroll.clamp(-overflow, 0.0);
    }

    #[allow(clippy::too_many_arguments)]
    fn tab_body(
        &mut self,
        ui: &mut Ui,
        state: &State,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        spacing: Vec2,
        tabbar: Rect,
        fade: Option<(&Style, f32)>,
        collapsed: bool,
    ) {
        let (body_rect, _body_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

        let leaf = self.state[surface][node]
            .leaf_mut()
            .expect("This node must be a leaf");

        let (style, fade_factor) = fade.unwrap_or((&self.style, 1.0));

        if !collapsed && let Some(tab) = leaf.tabs.get_mut(leaf.active.0) {
            if leaf.viewport != body_rect {
                leaf.viewport = body_rect;
                tab_viewer.on_rect_changed(tab);
            }

            if ui.input(|i| i.pointer.any_click())
                && let Some(pos) = state.last_hover_pos
                && body_rect.contains(pos)
                && Some(ui.layer_id()) == ui.ctx().layer_id_at(pos)
            {
                self.new_focused = Some((surface, node));
            }

            let tabs_styles = tab_viewer.tab_style_override(tab, &style.tab);
            let tabs_style = tabs_styles.as_ref().unwrap_or(&style.tab);

            let mut ui = new_body_ui(ui, body_rect, self.id.with(tab_viewer.id(tab)));

            body_ui(
                tab_viewer,
                &mut ui,
                spacing,
                tab,
                tabs_style,
                fade_factor,
                body_rect,
            );
        }

        // Change hover destination
        let leaf = self.state[surface][node].leaf_mut().unwrap();

        if let Some(pointer) = state.last_hover_pos {
            // Prevent borrow checker issues.
            let rect = leaf.rect;

            // if the dragged tab isn't allowed in a window,
            // it's unnecessary to change the hover state
            let is_dragged_valid = match &state.dnd {
                Some(DragDropState {
                    drag: DragData { src, .. },
                    ..
                }) => match *src {
                    TreeComponent::Tab(d_surf, d_node, d_tab) => {
                        if let Node::Leaf(node) = &mut self.state[d_surf][d_node] {
                            tab_viewer.allowed_in_windows(&mut node.tabs[d_tab.0])
                                || surface == SurfaceIndex::main()
                        } else {
                            true
                        }
                    }
                    _ => unreachable!("collections of nodes can't be dragged (yet)"),
                },
                _ => true,
            };

            // Use rect.contains instead of response.hovered as the dragged tab covers
            // the underlying responses.
            if state.drag_start.is_some() && rect.contains(pointer) && is_dragged_valid {
                let on_title_bar = tabbar.contains(pointer);
                let (dst, tab) = {
                    match self.tab_hover_rect {
                        Some((rect, tab)) => (TreeComponent::Tab(surface, node, tab), Some(rect)),
                        None => (
                            TreeComponent::Node(surface, node),
                            on_title_bar.then_some(tabbar),
                        ),
                    }
                };

                ui.memory_mut(|mem| {
                    let id = self.id.with("hover_data");
                    let value = HoverData { rect, dst, tab };
                    mem.data.insert_temp(id, Some(value));
                });
            }
        }
    }
}

/// Tab content wrapper.
fn body_ui<Tab>(
    viewer: &mut impl TabViewer<Tab = Tab>,
    ui: &mut Ui,
    spacing: Vec2,
    tab: &mut Tab,
    style: &TabStyle,
    fade_factor: f32,
    rect: Rect,
) {
    let &TabBodyStyle {
        inner_margin,
        stroke,
        corner_radius,
        bg_fill,
    } = &style.tab_body;

    if viewer.clear_background(tab) {
        ui.painter().rect_filled(rect, corner_radius, bg_fill);
    }

    // Use initial spacing for ui.
    ui.spacing_mut().item_spacing = spacing;

    // Offset the background rectangle up to hide the top border behind the clip rect.
    // To avoid anti-aliasing lines when the stroke width is not divisible by two, we
    // need to calculate the effective anti-aliased stroke width.
    let effective_stroke_width = (stroke.width / 2.0).ceil() * 2.0;
    let min = ui.clip_rect().min - vec2(0.0, effective_stroke_width);
    let rect = rect_stroke_box(min, ui.clip_rect().max, stroke.width);
    ui.painter()
        .rect_stroke(rect, corner_radius, stroke, StrokeKind::Inside);

    ScrollArea::new(viewer.scroll_bars(tab)).show(ui, |ui| {
        Frame::new().inner_margin(inner_margin).show(ui, |ui| {
            if fade_factor < 1.0 {
                fade_visuals(ui.visuals_mut(), fade_factor);
            }
            ui.expand_to_include_rect(ui.available_rect_before_wrap());
            viewer.ui(ui, tab);
        });
    });
}

// Construct a new ui with the correct tab id.
//
// We are forced to use `Ui::new` because other methods (eg: push_id) always mix
// the provided id with their own which would cause tabs to change id when moved
// from node to node.
fn new_body_ui(ui: &mut Ui, rect: Rect, id: Id) -> Ui {
    ui.ctx().check_for_id_clash(id, rect, "a tab with id");

    let ui_builder = UiBuilder::new().max_rect(rect).layer_id(ui.layer_id());
    let mut ui = Ui::new(ui.ctx().clone(), id, ui_builder);
    ui.set_clip_rect(Rect::from_min_max(ui.cursor().min, ui.clip_rect().max));
    ui
}
