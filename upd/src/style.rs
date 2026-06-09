use bevy_egui::egui;
use dock::style::{OverlayType, Style, TabBodyStyle};

pub fn init(ctx_style: std::sync::Arc<egui::Style>) -> Style {
    let mut style = Style::from_egui(ctx_style.as_ref());

    style.tab.tab_body = TabBodyStyle {
        inner_margin: style.tab.tab_body.inner_margin,
        stroke: egui::Stroke::default(),
        corner_radius: egui::CornerRadius::ZERO,
        bg_fill: style.tab.tab_body.bg_fill,
    };

    style.tab.spacing = 2.0;

    let tab = [
        &mut style.tab.active,
        &mut style.tab.inactive,
        &mut style.tab.focused,
        &mut style.tab.hovered,
        &mut style.tab.inactive_with_kb_focus,
        &mut style.tab.active_with_kb_focus,
        &mut style.tab.focused_with_kb_focus,
    ];

    // let bg = egui::Color32::GREEN;
    let bg = egui::Color32::TRANSPARENT;

    for tab in tab {
        tab.outline_color = bg;
    }

    // style.tab_bar.inner_margin = egui::Margin::symmetric(4, 0);
    style.tab_bar.bg_fill = bg;
    style.tab_bar.hline_color = bg;

    // let idle = style.separator.color_idle;
    // style.separator.color_idle = egui::Color32::BLACK;
    // style.separator.color_dragged = idle;
    // style.separator.color_hovered = idle;

    style.separator.color_idle = egui::Color32::BLACK;
    style.separator.color_hovered = ctx_style.visuals.widgets.noninteractive.bg_stroke.color;
    style.separator.color_dragged = ctx_style.visuals.widgets.noninteractive.fg_stroke.color;

    style.separator.width = 4.0;
    style.separator.extra_interact_width = 2.0;

    style.main_surface_border_stroke = egui::Stroke::default();
    style.dock_area_padding = Some(egui::Margin::symmetric(6, 4));

    let btn = [
        &mut style.buttons.close_tab,
        &mut style.buttons.add_tab,
        &mut style.buttons.close_all_tabs,
        &mut style.buttons.collapse_tabs,
        &mut style.buttons.minimize_window,
    ];

    for btn in btn {
        btn.border_color = bg;
        btn.bg_fill = bg;
    }

    style.overlay.overlay_type = OverlayType::Widgets;

    style
}
