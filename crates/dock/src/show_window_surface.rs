use super::{
    dock_area::DockArea,
    draw::draw_chevron_right,
    state::State,
    style::Style,
    tab_viewer::TabViewer,
    tree::{Node, SurfaceIndex, TabRemoval},
    utils::{fade_visuals, rect_set_size_centered},
};
use egui::{
    Align, CornerRadius, CursorIcon, Frame, Layout, Rect, RichText, Sense, Ui, UiBuilder, Vec2,
    WidgetText, vec2,
};

impl<Tab> DockArea<'_, Tab> {
    pub(super) fn show_window_surface(
        &mut self,
        ui: &Ui,
        surface: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) {
        // Construct egui window
        let id = format!("window {surface:?}").into();
        let bounds = self.config.window_bounds.unwrap();
        let open = true;
        let (_, window) = self.state.window_mut(surface).unwrap();
        let window = window.create_window(id, bounds);

        // Calculate fading of the window (if any)
        let (fade_factor, fade_style) = match fade_style {
            Some((style, factor, surface_index)) if surface_index != surface => {
                (factor, Some((style, factor)))
            }
            _ => (1.0, None),
        };

        // Get galley of currently selected node as a window title
        let title = {
            let node_id = self.state[surface].focused_leaf().unwrap_or_else(|| {
                for node_index in self.state[surface].breadth_first_index_iter() {
                    if self.state[surface][node_index].is_leaf() {
                        return node_index;
                    }
                }
                unreachable!("a window surface should never be empty")
            });
            let leaf = self.state[surface][node_id].leaf_mut().unwrap();
            tab_viewer
                .title(&mut leaf.tabs[leaf.active.0])
                .color(ui.visuals().widgets.noninteractive.fg_stroke.color)
        };

        // Iterate through every node in dock_state[surf_index], and sum up the number of tabs in them
        let mut tab_count = 0;
        for node_index in self.state[surface].breadth_first_index_iter() {
            if self.state[surface][node_index].is_leaf() {
                tab_count += self.state[surface][node_index].num_tabs();
            }
        }

        // Fade window frame (if necessary)
        let mut frame = Frame::window(ui.style());
        if fade_factor != 1.0 {
            frame.fill = frame.fill.linear_multiply(fade_factor);
            frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
            frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
        }

        let tab_bar_height = self.style.tab_bar.height;
        let minimized = self.state.window(surface).unwrap().1.minimized;
        let window = if minimized {
            let height = tab_bar_height;
            window
                .resizable([true, false])
                .max_height(height)
                .min_height(height)
        } else if self.state[surface].is_collapsed() {
            let height = self.state[surface].collapsed_leaf_count() as f32 * tab_bar_height;
            window
                .resizable([true, false])
                .max_height(height)
                .min_height(height)
        } else {
            window
        };

        window.frame(frame).show(ui.ctx(), |ui| {
            // Fade inner ui (if necessary)
            if fade_factor != 1.0 {
                fade_visuals(ui.visuals_mut(), fade_factor);
            }
            if minimized {
                let fade_style = fade_style.map(|(style, _)| style);
                self.minimized_body(ui, surface, fade_style, title, tab_count);
            } else {
                self.render_nodes(ui, tab_viewer, state, surface, fade_style);
            }
        });

        if !open {
            self.to_remove.push(TabRemoval::Window(surface));
        }
    }

    fn minimized_body(
        &mut self,
        ui: &mut Ui,
        surface_index: SurfaceIndex,
        fade_style: Option<&Style>,
        title: WidgetText,
        tab_count: usize,
    ) {
        ui.horizontal(|ui| {
            let style = fade_style.unwrap_or(&self.style);
            let (tabbar_outer_rect, _) = ui.allocate_exact_size(
                vec2(Style::TAB_EXPAND_BUTTON_SIZE, style.tab_bar.height),
                Sense::hover(),
            );
            ui.painter().rect_filled(
                tabbar_outer_rect,
                style.tab_bar.corner_radius,
                style.tab_bar.bg_fill,
            );
            self.window_expand(ui, surface_index, tabbar_outer_rect, fade_style);
            ui.label(title);

            if tab_count > 1 {
                let text = RichText::new(format!("+{}", tab_count - 1));
                ui.label(text.color(ui.visuals().weak_text_color()));
            }
            ui.allocate_space(ui.available_size());
        });
    }

    /// Draws the expand window button.
    fn window_expand(
        &mut self,
        ui: &mut Ui,
        surface_index: SurfaceIndex,
        tabbar_outer_rect: Rect,
        fade_style: Option<&Style>,
    ) {
        let rect = tabbar_outer_rect;

        let id = (surface_index, "window_expand");
        let layout = Layout::left_to_right(Align::Center);
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect).layout(layout).id_salt(id));

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        response = response.on_hover_cursor(CursorIcon::PointingHand);

        let style = fade_style.unwrap_or(&self.style);
        let color = if response.hovered() || response.has_focus() {
            ui.painter().rect_filled(
                rect,
                CornerRadius::ZERO,
                style.buttons.minimize_window.bg_fill,
            );
            style.buttons.minimize_window.active_color
        } else {
            style.buttons.minimize_window.color
        };

        let mut arrow_rect = rect;

        rect_set_size_centered(&mut arrow_rect, Vec2::splat(Style::TAB_EXPAND_ARROW_SIZE));

        draw_chevron_right(ui, &mut response, style, color, arrow_rect);

        // Draw button right border.
        let px = ui.ctx().pixels_per_point().recip();
        ui.painter().vline(
            rect.right(),
            rect.y_range(),
            (px, style.buttons.minimize_window.border_color),
        );

        if response.clicked() {
            self.window_toggle_minimized(surface_index);
        }
    }

    pub(crate) fn window_toggle_minimized(&mut self, surf_index: SurfaceIndex) {
        let (surface, window_state) = self.state.window_mut(surf_index).unwrap();
        let root = surface.root_node();

        if root.is_some_and(Node::is_collapsed) {
            // The window is already fully collapsed,
            // so `expanded_height` has already been set.
            // We don't need to set `new` either.
            window_state.toggle_minimized();
        } else if window_state.minimized {
            window_state.new = true;
            window_state.minimized = false;
        } else {
            let surface_height = root.map_or(0.0, |node| node.rect().unwrap().height());

            window_state.expanded_height = Some(surface_height);
            window_state.minimized = true;
        }
    }
}
