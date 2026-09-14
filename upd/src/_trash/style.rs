use crate::{ui::tab::TabStyle, util::map_to_pixel};
use bevy::prelude::Resource;
use bevy_egui::egui::{
    self, Color32, CornerRadius, CursorIcon, Rect, Sense, Stroke, Ui, style::ScrollStyle,
};

#[derive(Clone, Resource)]
pub struct Style {
    pub separator_size: f32,
    pub separator_extra: f32,

    pub app_bg: Color32,

    pub top_base: Color32,
    pub top_text: Color32,
    pub top_active: Color32,
    pub top_disable: Color32,

    pub tab_rounding: CornerRadius,
    pub tab_bar: Color32,
    pub tab_base: Color32,
    pub tab_text: Color32,
    //pub tab_active: Color32,
    pub tab_outline: Color32,

    pub selection: Color32,
    pub selection_opaque: Color32,

    pub background: Color32,
    pub panel_fill: Color32,
    pub tabbar: Color32,
    pub separator: Color32,
    pub input_stroke: Color32,
    pub input_fill: Color32,
    pub input_text: Color32,
}

impl Default for Style {
    fn default() -> Self {
        let background = Color32::from_gray(0x00);
        let panel = Color32::from_gray(0x2B);
        let tabbar = Color32::from_gray(0x14);
        let separator = Color32::from_gray(0x1E);

        let input_stroke = Color32::from_gray(0x37);
        let input_fill = Color32::from_gray(0x1B);
        let input_text = Color32::from_gray(0xEE);

        Self {
            background,
            panel_fill: panel,
            tabbar,
            separator,
            input_stroke,
            input_text,
            input_fill,

            selection: Color32::from_rgba_unmultiplied(61, 133, 224, 30),
            selection_opaque: Color32::from_rgb(61, 133, 224),

            separator_size: 4.0,
            separator_extra: 80.0,

            //app_bg: Color32::from_gray(0x14),
            app_bg: background,

            top_base: Color32::from_gray(0x2d),
            top_text: Color32::from_gray(0x9d),
            top_active: Color32::from_gray(0x55),
            top_disable: Color32::from_gray(0x20),

            tab_rounding: CornerRadius {
                ne: 2,
                nw: 2,
                se: 0,
                sw: 0,
            },

            //tab_bar: Color32::from_gray(0x20),
            //tab_base: Color32::from_gray(0x30),
            tab_bar: tabbar,
            tab_base: panel,

            tab_text: Color32::from_gray(0x9d),
            //tab_active: Color32::from_gray(0x40),
            tab_outline: Color32::from_gray(0x1c),
        }
    }
}

impl Style {
    pub fn set_theme_visuals(&self, ui: &mut Ui) {
        self.set_theme_visuals_internal(ui.visuals_mut());
    }

    pub fn init(&self, context: &mut egui::Context) {
        use egui::FontFamily::{Monospace, Proportional};
        use egui::{FontId, Margin, TextStyle};

        context.style_mut(|style| {
            self.set_theme_visuals_internal(&mut style.visuals);
            //style.spacing.item_spacing = egui::vec2(0.0, 2.0);
            //style.spacing.button_padding = egui::vec2(2.0, 2.0);

            style.spacing = egui::style::Spacing {
                // item_spacing: egui::vec2(8.0, 3.0),
                item_spacing: egui::vec2(2.0, 2.0),
                window_margin: Margin::same(6),
                menu_margin: Margin::same(6),
                button_padding: egui::vec2(4.0, 1.0),
                indent: 18.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
                // interact_size: egui::vec2(40.0, 18.0),
                interact_size: egui::vec2(18.0, 18.0),
                slider_width: 100.0,
                combo_width: 100.0,
                text_edit_width: 280.0,
                icon_width: 14.0,
                icon_width_inner: 8.0,
                icon_spacing: 2.0,
                tooltip_width: 600.0,
                combo_height: 200.0,
                scroll: ScrollStyle {
                    // floating: todo!(),
                    bar_width: 8.0,
                    handle_min_length: 12.0,
                    bar_inner_margin: 0.0,
                    bar_outer_margin: 0.0,
                    // floating_width: todo!(),
                    // floating_allocated_width: todo!(),
                    // foreground_color: todo!(),
                    // dormant_background_opacity: todo!(),
                    // active_background_opacity: todo!(),
                    // interact_background_opacity: todo!(),
                    // dormant_handle_opacity: todo!(),
                    // active_handle_opacity: todo!(),
                    // interact_handle_opacity: todo!(),
                    ..Default::default()
                },

                indent_ends_with_horizontal_line: false,

                // slider_rail_height: todo!(),
                // default_area_size: todo!(),
                // menu_width: todo!(),
                // menu_spacing: todo!(),
                ..Default::default()
            };

            style.text_styles = std::collections::BTreeMap::from([
                (TextStyle::Small, FontId::new(9.0, Proportional)),
                (TextStyle::Body, FontId::new(10.5, Proportional)),
                (TextStyle::Button, FontId::new(10.5, Proportional)),
                (TextStyle::Heading, FontId::new(14.5, Proportional)),
                (TextStyle::Monospace, FontId::new(10.0, Monospace)),
            ]);
        });
    }

    fn set_theme_visuals_internal(&self, visuals: &mut egui::Visuals) {
        visuals.extreme_bg_color = self.input_fill;
        visuals.selection.bg_fill = self.selection_opaque;
        visuals.selection.stroke.color = Color32::WHITE;
        visuals.panel_fill = self.panel_fill;

        use egui::style::{WidgetVisuals, Widgets};

        let expansion = 0.0;
        let corner_radius = CornerRadius::same(2);
        let bg_fill = self.input_fill;
        let fg_color = self.input_text;

        visuals.widgets = Widgets {
            noninteractive: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(27),
                bg_fill,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines
                fg_stroke: Stroke::new(1.0, fg_color),               // normal text color
                corner_radius,
                expansion,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(60), // button background
                bg_fill,                              // checkbox background
                bg_stroke: Default::default(),
                fg_stroke: Stroke::new(1.0, fg_color), // button text
                corner_radius,
                expansion,
            },
            hovered: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(70),
                bg_fill,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, fg_color),
                corner_radius,
                expansion,
            },
            active: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(55),
                bg_fill,
                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                fg_stroke: Stroke::new(2.0, fg_color),
                corner_radius,
                expansion,
            },
            open: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(27),
                bg_fill,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                fg_stroke: Stroke::new(1.0, fg_color),
                corner_radius,
                expansion,
            },
        };
    }

    pub fn for_scrollbar(&self, ui: &mut Ui) {
        let spacing = ui.spacing_mut();
        spacing.scroll.bar_width = 4.0;

        let visuals = ui.visuals_mut();
        let corner_radius = CornerRadius::same(0);

        visuals.extreme_bg_color = self.panel_fill;
        visuals.clip_rect_margin = 0.0;
        visuals.widgets.noninteractive.corner_radius = corner_radius;
        visuals.widgets.inactive.corner_radius = corner_radius;
        visuals.widgets.hovered.corner_radius = corner_radius;
        visuals.widgets.active.corner_radius = corner_radius;
        visuals.widgets.open.corner_radius = corner_radius;
    }

    pub fn scrollarea(&self, ui: &mut Ui) {
        let visuals = ui.visuals_mut();
        let corner_radius = CornerRadius::same(2);

        visuals.extreme_bg_color = self.input_fill;
        visuals.widgets.noninteractive.corner_radius = corner_radius;
        visuals.widgets.inactive.corner_radius = corner_radius;
        visuals.widgets.hovered.corner_radius = corner_radius;
        visuals.widgets.active.corner_radius = corner_radius;
        visuals.widgets.open.corner_radius = corner_radius;
    }

    pub fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_size * 0.5;
        separator.max.x = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);

        {
            let delta = response.drag_delta().x;
            let range = rect.max.x - rect.min.x;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    pub fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator_size * 0.5;
        separator.max.y = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeVertical);

        {
            let delta = response.drag_delta().y;
            let range = rect.max.y - rect.min.y;

            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }

    pub fn tab_style(&self) -> TabStyle {
        TabStyle {
            corner_radius: self.tab_rounding,
            active_outline: self.tab_outline,
            active_fill: self.panel_fill,
            text_color: self.input_text,
            font_id: egui::FontId::proportional(12.0),
        }
    }
}

pub struct HierarchyItemStyle {
    pub level_indent: f32,
    pub item_height: f32,

    pub normal_color: egui::Color32,
    pub hover_color: egui::Color32,
    pub active_color: egui::Color32,

    pub children_icon_color: egui::Color32,
    pub children_icon_font: egui::FontId,
    pub children_icon_size: egui::Vec2,
    pub children_icon_offset: f32,

    pub custom_icon_color: egui::Color32,
    pub custom_icon_font: egui::FontId,
    pub custom_icon_size: egui::Vec2,
    pub custom_icon_offset: f32,

    pub label_color: egui::Color32,
    pub label_font: egui::FontId,
    pub label_offset: f32,

    pub visibility_icon_color: egui::Color32,
    pub visibility_icon_font: egui::FontId,
    pub visibility_icon_size: egui::Vec2,
    pub visibility_icon_offset: f32,
}

impl Default for HierarchyItemStyle {
    fn default() -> Self {
        let style_panel = egui::Color32::from_gray(0x2B);
        let style_input_text = egui::Color32::from_gray(0xEE);

        let base_height = 20.0;

        Self {
            level_indent: 14.0,
            item_height: base_height,

            //
            normal_color: style_panel,
            hover_color: egui::Color32::from_gray(0x3B),
            active_color: egui::Color32::from_rgb(0x33, 0x4d, 0x80),
            //active_color: style.selection,

            //
            children_icon_color: style_input_text.gamma_multiply(0.5),
            children_icon_font: egui::FontId::proportional(18.0),
            children_icon_size: egui::vec2(14.0, base_height),
            children_icon_offset: 0.0, // 8

            //
            custom_icon_color: egui::Color32::from_rgb(0xc4, 0x88, 0x57), // c48857
            custom_icon_font: egui::FontId::proportional(16.0),
            custom_icon_size: egui::vec2(16.0, base_height),
            custom_icon_offset: 14.0, // 22

            //
            label_color: style_input_text,
            label_font: egui::FontId::proportional(10.5),
            label_offset: 32.0, // 32

            //
            visibility_icon_color: style_input_text.gamma_multiply(0.5),
            visibility_icon_font: egui::FontId::proportional(18.0),
            visibility_icon_size: egui::vec2(14.0, base_height),
            visibility_icon_offset: 14.0 + 7.0,
        }
    }
}
