use super::icon::Icon;
use super::ui::{BtnIcon, FILL, Flow, Ui};
use bevy_egui::egui;

pub fn panel(mut ui: Ui, state: &mut super::State) {
    {
        ui.ui.add_space(95.0);
        ui.separator();
    }
    {
        let line = Flow::symmetric(8, 8, 4, 40.0, [34.0, 54.0, FILL, 24.0]);
        ui.line(line, |[mut a, mut b, _, mut icon]| {
            a.btn_center(true, true, "File"); // 34x24
            b.btn_center(false, true, "Assets"); // 54x24
            icon.btn_icon(Icon::Search, BtnIcon::FRAMELESS);
        });
        ui.separator();
    }

    let padding = egui::Margin {
        left: 0,
        right: 8,
        top: 8,
        bottom: 8,
    };
    let collapsable = Flow::new(padding, 0, 40.0, [16.0, FILL, 24.0]);

    {
        ui.line(collapsable, |[mut chevron, mut title, mut icon]| {
            chevron.btn_icon(Icon::ChevronDown, BtnIcon::FRAMELESS);
            title.btn_title(true, false, "Pages");
            icon.btn_icon(Icon::Add, BtnIcon::FRAMELESS);
        });

        let line = Flow::symmetric(8, 4, 0, 32.0, [FILL]);
        ui.line(line, |[mut ui]| {
            ui.btn_left(true, true, "Page 1");
        });
        ui.line(line, |[mut ui]| {
            ui.btn_left(true, true, "Page 2");
        });

        ui.ui.add_space(8.0);
        ui.separator();
    }
    {
        ui.line(collapsable, |[mut chevron, mut title, mut icon]| {
            chevron.btn_icon(Icon::ChevronDown, BtnIcon::FRAMELESS);
            title.btn_title(true, false, "Layers");
            icon.btn_icon(Icon::Collapse, BtnIcon::FRAMELESS);
        });

        let line = Flow::symmetric(8, 4, 0, 32.0, [FILL]);
        ui.line(line, |[mut ui]| {
            ui.btn_left(false, true, "L1");
        });
        ui.line(line, |[mut ui]| {
            ui.btn_left(false, true, "L2");
        });
        ui.line(line, |[mut ui]| {
            ui.btn_left(false, true, "L3");
        });

        // ui.btn_left(false, true, "L1");
        // ui.btn_left(false, true, "L2");
        // ui.btn_left(false, true, "L3");
    }
}
