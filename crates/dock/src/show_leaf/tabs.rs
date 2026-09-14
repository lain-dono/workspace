use super::super::{
    dock_area::DockArea,
    drag_and_drop::{DragData, DragDropState, HoverData, TreeComponent},
    draw::{draw_arrow, draw_chevron_down, draw_close_window_symbol},
    state::State,
    style::{ButtonStyle, Style, TabAddAlign, TabBodyStyle, TabInteractionStyle, TabStyle},
    tab_viewer::{OnCloseResponse, TabViewer},
    tree::{Node, NodeIndex, SurfaceIndex, TabIndex, TabRemoval},
    utils::{fade_visuals, rect_set_size_centered, rect_size_centered, rect_stroke_box},
};
use egui::{
    Align, Align2, Button, Color32, CornerRadius, CursorIcon, Frame, Id, Key, LayerId, Layout,
    NumExt, Order, Painter, Popup, PopupCloseBehavior, Rect, Response, ScrollArea, Sense,
    StrokeKind, TextStyle, Ui, UiBuilder, Vec2, WidgetText, emath::TSTransform, epaint::TextShape,
    lerp, pos2, vec2,
};

pub struct TabTitle {
    pub id: Id,
    pub label: WidgetText,

    pub is_focused: bool,
    pub is_active: bool,
    pub is_being_dragged: bool,

    pub preferred_width: f32,
    pub show_close_button: bool,
    pub draggable: bool,
}

impl TabTitle {
    /// * `active` means "the tab that is opened in the parent panel".
    /// * `focused` means "the tab that was last interacted with".
    ///
    /// Returns the main button response plus the response of the close button, if any.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        self,
        ui: &mut Ui,
        tab_style: &TabStyle,
        style: &Style,
    ) -> (Response, Option<Response>) {
        let galley = self
            .label
            .into_galley(ui, None, f32::INFINITY, TextStyle::Button);
        let x_spacing = 8.0;
        let text_width = galley.size().x + x_spacing;
        let close_button_size = if self.show_close_button {
            Style::TAB_CLOSE_BUTTON_SIZE.min(style.tab_bar.height)
        } else {
            0.0
        };

        // Compute total width of the tab bar.
        let min_width = tab_style.min_width.unwrap_or(0.0);
        let min_width = min_width.at_least(text_width + close_button_size);
        let tab_width = self.preferred_width.at_least(min_width);

        let (_, tab_rect) = ui.allocate_space(vec2(tab_width, ui.available_height()));
        let mut response = ui.interact(tab_rect, self.id, Sense::click_and_drag());
        if ui.ctx().dragged_id().is_none() && self.draggable {
            response = response.on_hover_cursor(CursorIcon::Grab);
        }

        let mut text_rect = tab_rect;
        text_rect.set_width(text_rect.width() - close_button_size);
        let text_rect = text_rect.with_min_x(text_rect.min.x + x_spacing);

        let center = tab_rect.with_min_x(text_rect.right()).center();
        let x_rect = Rect::from_center_size(center, Vec2::splat(close_button_size));

        let TabInteractionStyle {
            outline_color,
            corner_radius,
            bg_fill,
            text_color,
        } = if self.is_focused || self.is_being_dragged {
            if response.has_focus() {
                tab_style.focused_with_kb_focus
            } else {
                tab_style.focused
            }
        } else if self.is_active {
            if response.has_focus() {
                tab_style.active_with_kb_focus
            } else {
                tab_style.active
            }
        } else if response.hovered() {
            tab_style.hovered
        } else if response.has_focus() {
            tab_style.inactive_with_kb_focus
        } else {
            tab_style.inactive
        };

        let stroke = (1.0, outline_color);
        let stroke_rect = rect_stroke_box(tab_rect.min, tab_rect.max, 1.0);

        // Draw the full tab first and then the stroke on top to avoid the stroke mixing with the background color.
        ui.painter().rect_filled(tab_rect, corner_radius, bg_fill);
        ui.painter()
            .rect_stroke(stroke_rect, corner_radius, stroke, StrokeKind::Inside);

        let text_pos = Align2::CENTER_CENTER.pos_in_rect(&text_rect) - galley.size() / 2.0;

        ui.painter()
            .add(TextShape::new(text_pos, galley, text_color));

        let close_response = self.show_close_button.then(|| {
            let id = self.id.with("close-button");

            close_button(ui, id, x_rect, &style.buttons.close_tab)
        });

        (response, close_response)
    }
}

fn close_button(ui: &mut Ui, id: Id, rect: Rect, style: &ButtonStyle) -> Response {
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);

    let color = if response.hovered() || response.has_focus() {
        style.active_color
    } else {
        style.color
    };

    let rect = rect_size_centered(rect, Vec2::splat(Style::TAB_CLOSE_X_SIZE));
    x_icon(ui.painter(), rect, color);

    response
}

pub(crate) fn x_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let points = [rect.left_top(), rect.right_bottom()];
    painter.line_segment(points, (1.0, color));
    let points = [rect.right_top(), rect.left_bottom()];
    painter.line_segment(points, (1.0, color));
}
