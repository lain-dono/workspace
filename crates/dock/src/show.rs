use super::{
    dock_area::DockArea,
    drag_and_drop::{HoverData, TreeComponent},
    state::State,
    style::{OverlayType, Style},
    tab_viewer::{OnCloseResponse, TabViewer},
    tree::{Node, NodeIndex, SurfaceIndex, TabDestination, TabRemoval},
    utils::{expand_to_pixel, fade_dock_style, map_to_pixel},
};
use duplicate::duplicate;
use egui::{
    self, CentralPanel, Color32, Context, CornerRadius, CursorIcon, EventFilter, Frame, Key, Pos2,
    Rect, Sense, StrokeKind, Ui, Vec2,
};
use paste::paste;

impl<Tab> DockArea<'_, Tab> {
    /// Show the `DockArea` at the top level.
    ///
    /// This is the same as doing:
    ///
    /// ```
    /// # use egui_dock::{DockArea, DockState};
    /// # use egui::{CentralPanel, Frame};
    /// # struct TabViewer {}
    /// # impl egui_dock::TabViewer for TabViewer {
    /// #     type Tab = String;
    /// #     fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText { (&*tab).into() }
    /// #     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}
    /// # }
    /// # let mut tree: DockState<String> = DockState::new(vec![]);
    /// # let mut tab_viewer = TabViewer {};
    /// # egui::__run_test_ctx(|ctx| {
    /// CentralPanel::default()
    ///     .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
    ///     .show(ctx, |ui| {
    ///         DockArea::new(&mut tree).show_inside(ui, &mut tab_viewer);
    ///     });
    /// # });
    /// ```
    ///
    /// So you can't use the [`CentralPanel::show`] when using `DockArea`'s one.
    ///
    /// See also [`show_inside`](Self::show_inside).
    #[inline]
    pub fn show(self, ctx: &Context, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        let frame = Frame::central_panel(&ctx.style());
        let frame = frame.inner_margin(0).fill(Color32::TRANSPARENT);

        CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.show_inside(ui, tab_viewer);
        });
    }

    /// Shows the docking hierarchy inside a [`Ui`].
    ///
    /// See also [`show`](Self::show).
    pub fn show_inside(mut self, ui: &mut Ui, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        self.config
            .window_bounds
            .get_or_insert(ui.ctx().content_rect());

        let mut state = State::load(ui.ctx(), self.id);

        // Delay hover position one frame. On touch screens hover_pos() is None when any_released()
        if !ui.input(|i| i.pointer.any_released()) {
            state.last_hover_pos = ui.input(|i| i.pointer.hover_pos());
        }

        let (drag_data, hover_data) = ui.memory_mut(|mem| {
            (
                mem.data.remove_temp(self.id.with("drag_data")).flatten(),
                mem.data.remove_temp(self.id.with("hover_data")).flatten(),
            )
        });

        if let (Some(source), Some(hover)) = (drag_data, hover_data) {
            state.set_drag_and_drop(source, hover, ui.ctx(), &self.style);
            let tab_dst = self.show_drag_drop_overlay(ui, &mut state, tab_viewer);
            if ui.input(|i| i.pointer.primary_released())
                && let Some(destination) = tab_dst
            {
                let source = {
                    match state.dnd.as_ref().unwrap().drag.src {
                        TreeComponent::Tab(src_surf, src_node, src_tab) => {
                            (src_surf, src_node, src_tab)
                        }
                        _ => todo!(
                            "collections of tabs, like nodes and surfaces can't be docked (yet)"
                        ),
                    }
                };
                self.state.move_tab(source, destination);
            }
        }

        if ui.input(|i| i.pointer.primary_released()) {
            state.reset_drag();
        }

        let fade_surface = self.hovered_window_surface(
            &mut state,
            self.style.overlay.feel.fade_hold_time,
            ui.ctx(),
        );
        let fade_style = {
            fade_surface.is_some().then(|| {
                let mut fade_style = self.style.clone();
                fade_dock_style(&mut fade_style, self.style.overlay.surface_fade_opacity);
                (self.style.clone(), self.style.overlay.surface_fade_opacity)
            })
        };

        let indices: Box<[_]> = self.state.valid_surface_indices_iter().collect();
        for surface_index in indices {
            let fade_style = fade_style.as_ref().map(|(style, factor)| {
                (style, *factor, fade_surface.unwrap_or(SurfaceIndex::main()))
            });
            if surface_index.is_main() {
                let surf_index = SurfaceIndex::main();

                if self.state.main_surface().is_empty() {
                    let rect = ui.available_rect_before_wrap();
                    let response = ui.allocate_rect(rect, Sense::hover());
                    if response.contains_pointer() {
                        ui.memory_mut(|mem| {
                            let dst = TreeComponent::Surface(surf_index);
                            let tab = None;
                            let value = Some(HoverData { rect, dst, tab });
                            mem.data.insert_temp(self.id.with("hover_data"), value);
                        });
                    }
                    return;
                }

                self.render_nodes(ui, tab_viewer, &mut state, surf_index, None);
            } else {
                self.show_window_surface(ui, surface_index, tab_viewer, &mut state, fade_style);
            }
        }

        for removal in self.to_remove.drain(..).rev() {
            match removal {
                TabRemoval::ForcedTab(surface, node, tab) => {
                    self.state.remove_tab(surface, node, tab);
                }
                TabRemoval::Tab(surface, node, tab) => {
                    let leaf = &mut *self.state[surface][node].leaf_mut().unwrap();
                    match tab_viewer.on_close(&mut leaf.tabs[tab.0]) {
                        OnCloseResponse::Close => {
                            self.state.remove_tab(surface, node, tab);
                        }
                        OnCloseResponse::Focus => {
                            leaf.active = tab;
                            self.new_focused = Some((surface, node));
                        }
                        OnCloseResponse::Ignore => {} // no-op
                    }
                }
                TabRemoval::Node(surface, node) => {
                    let mut all_tabs_are_closable = true;
                    for tab in self.state[surface][node].tabs_mut() {
                        if !(tab_viewer.is_closeable(tab) && tab_viewer.on_close(tab).is_closing())
                        {
                            all_tabs_are_closable = false;
                        }
                    }
                    if all_tabs_are_closable {
                        self.state.remove_leaf((surface, node));
                    }
                }
                TabRemoval::Window(surface) => {
                    let mut all_tabs_are_closable = true;
                    for node in self.state[surface].iter_mut() {
                        for tab in node.tabs_mut() {
                            if !(tab_viewer.is_closeable(tab)
                                && tab_viewer.on_close(tab).is_closing())
                            {
                                all_tabs_are_closable = false;
                            }
                        }
                    }
                    if all_tabs_are_closable {
                        self.state.remove_surface(surface);
                    }
                }
            }
        }

        for (surface_index, node_index, tab_index) in self.to_detach.drain(..).rev() {
            let mouse_pos = state.last_hover_pos;
            self.state.detach_tab(
                (surface_index, node_index, tab_index),
                Rect::from_min_size(
                    mouse_pos.unwrap_or(Pos2::ZERO),
                    self.state[surface_index][node_index]
                        .rect()
                        .map_or(Vec2::new(100., 150.), |rect| rect.size()),
                ),
            );
        }

        if let Some(focused) = self.new_focused {
            self.state.set_focused_node_and_surface(focused);
        }

        state.store(ui.ctx(), self.id);
    }

    /// Returns some when windows are fading, and what surface index is being hovered over
    #[inline(always)]
    fn hovered_window_surface(
        &self,
        state: &mut State,
        hold_time: f32,
        ctx: &Context,
    ) -> Option<SurfaceIndex> {
        if let Some(dnd_state) = &state.dnd
            && dnd_state.is_locked(&self.style, ctx)
        {
            state.window_fade = Some((ctx.input(|i| i.time), dnd_state.hover.dst.surface()));
        }

        state.window_fade.and_then(|(time, surface)| {
            ctx.request_repaint();
            (hold_time > (ctx.input(|i| i.time) - time) as f32).then_some(surface)
        })
    }

    /// Resolve where a dragged tab would land given it's dropped this frame, returns `None` when the resulting drop is an invalid move.
    fn show_drag_drop_overlay(
        &mut self,
        ui: &Ui,
        state: &mut State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
    ) -> Option<TabDestination> {
        let drag_state = state.dnd.as_mut().unwrap();

        let deserted_node = {
            let (src_surf, src_node) = drag_state.drag.src.node_address();
            let (dst_surf, dst_node) = drag_state.hover.dst.node_address();

            src_node.zip(dst_node).is_some_and(|(src_node, dst_node)| {
                (src_surf == dst_surf && src_node == dst_node)
                    && self.state[src_surf][src_node].num_tabs() == 1
            })
        };

        // Not all scenarios can house all splits.
        let restrict = !(deserted_node || drag_state.hover.dst.is_surface());
        let allowed_splits = self.config.allowed_splits.filter(restrict);

        let allowed_in_window = match drag_state.drag.src {
            TreeComponent::Tab(surface, node, tab) => {
                let Node::Leaf(leaf) = &mut self.state[surface][node] else {
                    unreachable!("tab drags can only come from leaf nodes")
                };
                tab_viewer.allowed_in_windows(&mut leaf.tabs[tab.0])
            }
            _ => todo!("collections of tabs, like nodes or surfaces, can't be dragged! (yet)"),
        };

        if let Some(pointer) = state.last_hover_pos {
            drag_state.pointer = pointer;
        }

        let window_bounds = self.config.window_bounds.unwrap();
        match (
            self.style.overlay.overlay_type,
            drag_state.is_on_title_bar(),
        ) {
            (OverlayType::HighlightedAreas, _) | (_, true) => drag_state.resolve_traditional(
                ui,
                &self.style,
                allowed_splits,
                allowed_in_window,
                window_bounds,
            ),
            (OverlayType::Widgets, false) => drag_state.resolve_icon_based(
                ui,
                &self.style,
                allowed_splits,
                allowed_in_window,
                window_bounds,
            ),
        }
    }

    pub(super) fn render_nodes(
        &mut self,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        surface: SurfaceIndex,
        fade_style: Option<(&Style, f32)>,
    ) {
        // First compute all rect sizes in the node graph.
        let max_rect = self.allocate_area_for_root_node(ui, surface);
        for node in self.state[surface].breadth_first_index_iter() {
            if self.state[surface][node].is_parent() {
                self.compute_rect_sizes(ui, surface, node, max_rect);
            }
        }

        // Then, draw the bodies of each leaves.
        for node in self.state[surface].breadth_first_index_iter() {
            if self.state[surface][node].is_leaf() {
                self.show_leaf(ui, state, surface, node, tab_viewer, fade_style);
            }
        }

        // Finally, draw separators so that their "interaction zone" is above
        // bodies (see `SeparatorStyle::extra_interact_width`).
        let fade_style = fade_style.map(|(style, _)| style);
        for node in self.state[surface].breadth_first_index_iter() {
            if self.state[surface][node].is_parent() {
                self.show_separator(ui, surface, node, fade_style);
            }
        }
    }

    fn allocate_area_for_root_node(&mut self, ui: &mut Ui, surface: SurfaceIndex) -> Rect {
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = self.style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
        }

        ui.painter().rect_stroke(
            rect,
            self.style.main_surface_border_rounding,
            self.style.main_surface_border_stroke,
            StrokeKind::Inside,
        );
        if surface.is_main() {
            rect = rect.expand(-self.style.main_surface_border_stroke.width / 2.0);
        }
        ui.allocate_rect(rect, Sense::hover());

        if !self.state[surface].is_empty() {
            self.state.set_rect(surface, NodeIndex::root(), rect);
        }

        rect
    }

    fn compute_rect_sizes(
        &mut self,
        ui: &Ui,
        surface: SurfaceIndex,
        node: NodeIndex,
        max_rect: Rect,
    ) {
        assert!(self.state[surface][node].is_parent());

        let style = &self.style;
        let ppi = ui.ctx().pixels_per_point();

        let left_collapsed_count = self.state[surface][node.left()].collapsed_leaf_count();
        let right_collapsed_count = self.state[surface][node.right()].collapsed_leaf_count();
        let left_collapsed = self.state[surface][node.left()].is_collapsed();
        let right_collapsed = self.state[surface][node.right()].is_collapsed();

        if (left_collapsed || right_collapsed)
            && let Node::Vertical(split) = &mut self.state[surface][node]
        {
            debug_assert!(!split.rect.any_nan() && split.rect.is_finite());
            let rect = expand_to_pixel(split.rect, ppi);

            if left_collapsed {
                // EITHER only left collapsed OR left and right both collapsed
                let border_y = rect.min.y + (left_collapsed_count as f32) * style.tab_bar.height;
                let left_separator_border =
                    map_to_pixel(border_y - style.separator.width * 0.5, ppi, f32::round);
                let right_separator_border =
                    map_to_pixel(border_y + style.separator.width * 0.5, ppi, f32::round);
                let left = rect
                    .intersect(Rect::everything_above(left_separator_border))
                    .intersect(max_rect);
                let right = rect
                    .intersect(Rect::everything_below(right_separator_border))
                    .intersect(max_rect);
                self.state.set_rect(surface, node.left(), left);
                self.state.set_rect(surface, node.right(), right);
            } else {
                // Only right collapsed
                let border_y = rect.max.y - (right_collapsed_count as f32) * style.tab_bar.height;
                let left_separator_border =
                    map_to_pixel(border_y - style.separator.width * 0.5, ppi, f32::round);
                let right_separator_border =
                    map_to_pixel(border_y + style.separator.width * 0.5, ppi, f32::round);
                let left = rect
                    .intersect(Rect::everything_above(left_separator_border))
                    .intersect(max_rect);
                let right = rect
                    .intersect(Rect::everything_below(right_separator_border))
                    .intersect(max_rect);
                self.state.set_rect(surface, node.left(), left);
                self.state.set_rect(surface, node.right(), right);
            }
            return;
        }

        if let Node::Horizontal(split) = &mut self.state[surface][node] {
            let rect = split.rect;
            debug_assert!(!rect.any_nan() && rect.is_finite());
            let rect = expand_to_pixel(rect, ppi);

            let midpoint = rect.min.x + rect.width() * split.fraction;
            let w = style.separator.width * 0.5;
            let [min_separator_border, max_separator_border] =
                [midpoint - w, midpoint + w].map(|point| map_to_pixel(point, ppi, f32::round));

            let min = rect.intersect(Rect::everything_left_of(min_separator_border));
            let max = rect.intersect(Rect::everything_right_of(max_separator_border));

            self.state
                .set_rect(surface, node.left(), min.intersect(max_rect));
            self.state
                .set_rect(surface, node.right(), max.intersect(max_rect));
        }

        if let Node::Vertical(split) = &mut self.state[surface][node] {
            let rect = split.rect;
            debug_assert!(!rect.any_nan() && rect.is_finite());
            let rect = expand_to_pixel(rect, ppi);

            let midpoint = rect.min.y + rect.height() * split.fraction;
            let w = style.separator.width * 0.5;
            let [min_separator_border, max_separator_border] =
                [midpoint - w, midpoint + w].map(|point| map_to_pixel(point, ppi, f32::round));

            let min = rect.intersect(Rect::everything_above(min_separator_border));
            let max = rect.intersect(Rect::everything_below(max_separator_border));

            self.state
                .set_rect(surface, node.left(), min.intersect(max_rect));
            self.state
                .set_rect(surface, node.right(), max.intersect(max_rect));
        }
    }

    fn show_separator(
        &mut self,
        ui: &mut Ui,
        surface: SurfaceIndex,
        node: NodeIndex,
        fade_style: Option<&Style>,
    ) {
        assert!(self.state[surface][node].is_parent());

        // If either of the children is collapsed, we don't want the user to interact with the separator
        if (self.state[surface][node.left()].is_collapsed()
            || self.state[surface][node.right()].is_collapsed())
            && self.state[surface][node].is_vertical()
        {
            return;
        }

        let style = fade_style.unwrap_or(&self.style);
        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation   dim_point  dim_size;
                [Horizontal]  [x]        [width];
                [Vertical]    [y]        [height];
            ]
            if let Node::orientation(split) = &mut self.state[surface][node] {
                let rect = split.rect;

                let midpoint = rect.min.dim_point + rect.dim_size() * split.fraction;
                let min = midpoint - style.separator.width * 0.5;
                let max = midpoint + style.separator.width * 0.5;

                let mut separator = rect;
                separator.min.dim_point = min;
                separator.max.dim_point = max;

                let mut expand = Vec2::ZERO;
                expand.dim_point += style.separator.extra_interact_width / 2.0;
                let interact_rect = separator.expand2(expand);

                let cursor_icon = paste!{ CursorIcon::[<Resize orientation>]};
                let response = ui.allocate_rect(interact_rect, Sense::click_and_drag());
                let response = response.on_hover_and_drag_cursor(cursor_icon);

                let should_respond_to_arrow_keys = ui.input(|i| i.modifiers.command || i.modifiers.shift);

                if response.has_focus() {
                    // Prevent the default behaviour of removing focus from the separators when the
                    // arrow keys are pressed
                    ui.memory_mut(|m| m.set_focus_lock_filter(response.id, EventFilter {
                        horizontal_arrows: should_respond_to_arrow_keys,
                        vertical_arrows: should_respond_to_arrow_keys,
                        tab: false,
                        escape: false
                    }));
                }

                let arrow_key_offset = if response.has_focus() && should_respond_to_arrow_keys {
                    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                        Some(egui::vec2(0., -16.))
                    } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                        Some(egui::vec2(0., 16.))
                    } else if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
                        Some(egui::vec2(-16., 0.))
                    } else if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
                        Some(egui::vec2(16., 0.))
                    } else {
                        None
                    }
                } else {
                    None
                };

                separator.min.dim_point = map_to_pixel(min, pixels_per_point, f32::round);
                separator.max.dim_point = map_to_pixel(max, pixels_per_point, f32::round);

                let color = if response.dragged() {
                    style.separator.color_dragged
                } else if response.hovered() || response.has_focus() {
                    style.separator.color_hovered
                } else {
                    style.separator.color_idle
                };

                ui.painter().rect_filled(separator, CornerRadius::ZERO, color);

                // Update 'fraction' interaction after drawing separator,
                // otherwise it may overlap on other separator / bodies when
                // shrunk fast.
                if let Some(pos) = response.interact_pointer_pos().or(arrow_key_offset.map(|v| separator.center() + v)) {
                    let dim_point = pos.dim_point;
                    let delta = arrow_key_offset.unwrap_or(response.drag_delta()).dim_point;

                    if (delta > 0. && dim_point > midpoint && dim_point < rect.max.dim_point)
                        || (delta < 0. && dim_point < midpoint && dim_point > rect.min.dim_point)
                    {
                        let range = rect.max.dim_point - rect.min.dim_point;
                        let min = (style.separator.extra / range).min(1.0);
                        let max = 1.0 - min;
                        let (min, max) = (min.min(max), max.max(min));
                        split.fraction = (split.fraction + delta / range).clamp(min, max);
                    }
                }

                if response.double_clicked() {
                    split.fraction = 0.5;
                }
            }

        }
    }
}
