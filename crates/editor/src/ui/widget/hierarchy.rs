use crate::ui::{icon, style::HierarchyItemStyle};

#[must_use]
#[derive(Clone, Debug)]
pub struct HierarchyItemState {
    pub is_open: bool,
}

impl Default for HierarchyItemState {
    fn default() -> Self {
        Self { is_open: true }
    }
}

pub struct HierarchyItemResponse {
    pub show_children: bool,
    pub just_selected: bool,
    pub is_visible: Option<bool>,
}

pub struct HierarchyItemWidget<'a> {
    pub id: egui::Id,
    pub level: usize,
    pub style: &'a HierarchyItemStyle,

    pub is_selected: bool,
    pub has_children: bool,

    pub hover_text: Option<String>,

    pub icon: Option<u32>,
    pub name: String,
    pub is_visible: Option<bool>,
}

impl<'a> HierarchyItemWidget<'a> {
    pub fn ui(self, ui: &mut egui::Ui) -> HierarchyItemResponse {
        let desired_size = egui::vec2(ui.available_width(), self.style.item_height);
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        let (max_rect, mut response_bg) =
            ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let mut ui = ui.child_ui_with_id_source(max_rect, layout, self.id);

        let mut state: HierarchyItemState = ui
            .ctx()
            .data(|data| data.get_temp(self.id))
            .unwrap_or_default();

        let mut item_response = HierarchyItemResponse {
            is_visible: self.is_visible,
            show_children: false,
            just_selected: false,
        };

        let is_hovered = response_bg.hovered();

        if let Some(hover_text) = self.hover_text {
            response_bg = response_bg.on_hover_text_at_pointer(hover_text);
        }

        let bg_color = if self.is_selected {
            self.style.active_color
        } else if is_hovered {
            self.style.hover_color
        } else {
            self.style.normal_color
        };

        ui.painter().rect_filled(response_bg.rect, 0.0, bg_color);

        let offset = egui::vec2(self.style.level_indent * self.level as f32, 0.0);
        let from_left = max_rect.left_top();
        let from_right = max_rect.right_top();

        let children_icon_rect = egui::Rect::from_min_size(
            from_left + egui::vec2(self.style.children_icon_offset, 0.0) + offset,
            self.style.children_icon_size,
        );

        let custom_icon_rect = egui::Rect::from_min_size(
            from_left + egui::vec2(self.style.custom_icon_offset, 0.0) + offset,
            self.style.custom_icon_size,
        );
        let label_pos = max_rect.left_center() + egui::vec2(self.style.label_offset, 0.0) + offset;

        let visibility_icon_rect = egui::Rect::from_min_size(
            from_right - egui::vec2(self.style.visibility_icon_offset, 0.0),
            self.style.visibility_icon_size,
        );

        let cursor = egui::CursorIcon::PointingHand;
        let sense = egui::Sense::click();

        if let Some(icon) = self.icon {
            let response = ui.allocate_rect(custom_icon_rect, sense);
            let response = response.on_hover_cursor(cursor);

            ui.painter().text(
                response.rect.center() - egui::vec2(0.0, self.style.custom_icon_font.size / 2.0),
                egui::Align2::CENTER_TOP,
                char::from_u32(icon).unwrap(),
                self.style.custom_icon_font.clone(),
                self.style.custom_icon_color,
            );
        }

        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            self.name,
            self.style.label_font.clone(),
            self.style.label_color,
        );

        if let Some(is_visible) = item_response.is_visible.as_mut() {
            let response = ui.allocate_rect(visibility_icon_rect, sense);
            let response = response.on_hover_cursor(cursor);

            if response.clicked() {
                *is_visible = !*is_visible;
            }

            let hide_icon = if *is_visible {
                icon::HIDE_OFF
            } else {
                icon::HIDE_ON
            };

            ui.painter().text(
                response.rect.center()
                    - egui::vec2(0.0, self.style.visibility_icon_font.size / 2.0),
                egui::Align2::CENTER_TOP,
                hide_icon,
                self.style.visibility_icon_font.clone(),
                self.style.visibility_icon_color,
            );
        }

        if self.has_children {
            let response = ui.allocate_rect(children_icon_rect, sense);
            let response = response.on_hover_cursor(cursor);

            if response.clicked() {
                state.is_open = !state.is_open;
                ui.ctx().request_repaint();
            }

            let triangle_icon = if state.is_open {
                icon::DISCLOSURE_TRI_DOWN
            } else {
                icon::DISCLOSURE_TRI_RIGHT
            };

            ui.painter().text(
                response.rect.center() - egui::vec2(0.0, self.style.children_icon_font.size / 2.0),
                egui::Align2::CENTER_TOP,
                triangle_icon,
                self.style.children_icon_font.clone(),
                self.style.children_icon_color,
            );
        }

        item_response.show_children = state.is_open && self.has_children;

        let response_bg = response_bg.interact(egui::Sense::click());

        if response_bg.clicked() {
            item_response.just_selected = true;
        }

        if response_bg.double_clicked() && self.has_children {
            state.is_open = !state.is_open;
            ui.ctx().request_repaint();
        }

        // store state
        ui.ctx().data_mut(|data| data.insert_temp(self.id, state));

        item_response
    }
}
