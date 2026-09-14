use super::style::Style;
use egui::{Color32, Rect, Response, Shape, Stroke, Ui};

pub(crate) fn draw_chevron_right(
    ui: &mut Ui,
    response: &mut Response,
    style: &Style,
    color: Color32,
    rect: Rect,
) {
    // Arrow pointing rightwards.
    let points = vec![rect.left_top(), rect.center(), rect.left_bottom()];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));

    // Chevron pointing rightwards.
    let points = vec![rect.center_top(), rect.right_center(), rect.center_bottom()];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));
    let color = if response.hovered() || response.has_focus() {
        style.buttons.minimize_window.bg_fill
    } else {
        style.tab_bar.bg_fill
    };
    let points = vec![
        rect.center_top().lerp(rect.center_bottom(), 0.25),
        rect.center().lerp(rect.right_center(), 0.5),
        rect.center_top().lerp(rect.center_bottom(), 0.75),
    ];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub(crate) fn draw_chevron_down(ui: &mut Ui, style: &Style, color: Color32, rect: Rect) {
    // Arrow pointing downwards.
    let points = vec![rect.left_top(), rect.right_top(), rect.center()];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));

    // Chevron pointing downwards.
    let points = vec![
        rect.left_center(),
        rect.right_center(),
        rect.center_bottom(),
    ];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));

    let color = style.buttons.minimize_window.bg_fill;
    let points = vec![
        rect.left_center().lerp(rect.right_center(), 0.25),
        rect.left_center().lerp(rect.right_center(), 0.75),
        rect.center().lerp(rect.center_bottom(), 0.5),
    ];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub(crate) fn draw_arrow(collapsed: bool, ui: &mut Ui, color: Color32, rect: Rect) {
    let points = if collapsed {
        // Arrow pointing rightwards.
        vec![rect.left_top(), rect.right_center(), rect.left_bottom()]
    } else {
        // Arrow pointing downwards.
        vec![rect.left_top(), rect.right_top(), rect.center_bottom()]
    };
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub(crate) fn draw_close_window_symbol(ui: &mut Ui, stroke_color: Color32, rect: Rect) {
    let stroke = (1.0, stroke_color);
    let points = vec![
        rect.right_center().lerp(rect.right_bottom(), 0.5),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
        rect.center_top().lerp(rect.left_top(), 0.5),
    ];
    ui.painter().add(Shape::line(points, stroke));

    let points = [rect.center_top(), rect.right_center()];
    ui.painter().line_segment(points, stroke);

    let points = [rect.center(), rect.right_top()];
    ui.painter().line_segment(points, stroke);
}
