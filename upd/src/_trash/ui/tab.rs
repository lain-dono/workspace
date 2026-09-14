use crate::style::Style;
use bevy_egui::egui;

pub struct TabStyle {
    pub corner_radius: egui::CornerRadius,
    pub active_outline: egui::Color32,
    pub active_fill: egui::Color32,
    pub text_color: egui::Color32,
    pub font_id: egui::FontId,
}

pub struct TabTitle<'a> {
    pub label: String,
    pub active: bool,
    pub style: &'a Style,
}

impl egui::Widget for TabTitle<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let style = self.style.tab_style();

        let px = ui.ctx().pixels_per_point().recip();

        let galley = ui
            .painter()
            .layout_no_wrap(self.label, style.font_id, style.text_color);

        let tab_width = galley.size().x + 10.0 * 2.0;

        let size = egui::vec2(tab_width, 24.0);

        let (id, tab_rect) = ui.allocate_space(size);
        let response = ui.interact(tab_rect, id, egui::Sense::hover());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        let painter = ui.painter();

        if self.active {
            let rect = tab_rect.expand2(egui::vec2(px, 0.0));
            painter.rect_filled(rect, style.corner_radius, style.active_outline);

            let rect = rect.shrink2(egui::vec2(px, 0.0));
            painter.rect_filled(rect, style.corner_radius, style.active_fill);
        }

        let anchor = egui::Align2::LEFT_TOP;
        let anchor_rect = anchor.anchor_rect(tab_rect.shrink2(egui::vec2(8.0, 5.0)));

        painter.galley(anchor_rect.min, galley, egui::Color32::PLACEHOLDER);

        response
    }
}
