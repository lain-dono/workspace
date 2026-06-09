use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

mod drag_value;
mod f;
mod hierarchy;
mod icon;
mod inspector;
mod ui;

use self::icon::Icon;
use self::ui::{FILL, Flow, Ui};

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, main_ui);
    app.insert_resource(State {
        rect: egui::Rect::NOTHING,

        rotation: 0.0,

        pos: egui::vec2(240.0, 160.0),
        size: egui::vec2(240.0, 160.0),
        min_size: egui::vec2(24.0, 16.0),
        max_size: egui::vec2(2400.0, 1600.0),
        gap: egui::vec2(16.0, 16.0),

        padding: Padding {
            l: 8.0,
            t: 4.0,
            r: 8.0,
            b: 4.0,
        },

        radius: Radius {
            nw: 0.0,
            ne: 0.0,
            sw: 0.0,
            se: 0.0,
        },
    });
}

#[derive(Resource)]
pub struct State {
    pub rect: egui::Rect,

    pub rotation: f32,

    pub pos: egui::Vec2,
    pub size: egui::Vec2,
    pub min_size: egui::Vec2,
    pub max_size: egui::Vec2,
    pub gap: egui::Vec2,
    pub padding: Padding,
    pub radius: Radius,
}

pub struct Radius {
    pub nw: f32,
    pub ne: f32,
    pub sw: f32,
    pub se: f32,
}

pub struct Padding {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}

pub fn main_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<State>,
    mut skipped_loading_fonts: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    if !*skipped_loading_fonts {
        *skipped_loading_fonts = true;
        return Ok(());
    }

    let canvas_frame = egui::Frame {
        fill: ui::CANVAS_BG,
        inner_margin: egui::Margin::ZERO,
        ..egui::Frame::NONE
    };

    let panel_frame = egui::Frame {
        fill: ui::PANEL_BG,
        inner_margin: egui::Margin::ZERO,
        ..egui::Frame::NONE
    };

    let l = egui::SidePanel::left("#figma_hierarchy");
    let r = egui::SidePanel::right("#figma_inspector");
    let c = egui::CentralPanel::default().frame(canvas_frame);

    let [l, r] = [l, r].map(|panel| panel.frame(panel_frame).min_width(240.0));

    l.show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        self::hierarchy::panel(Ui::new(ui, "left"), &mut state);
        ui.take_available_space();
    });
    r.show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        self::inspector::panel(Ui::new(ui, "right"), &mut state);
        ui.take_available_space();
    });
    c.show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        center_panel(Ui::new(ui, "center"), &mut state);
        ui.take_available_space();
    });

    Ok(())
}

fn center_panel(mut ui: Ui, state: &mut State) {
    let line = Flow::symmetric(8, 8, 4, 40.0, [34.0, 54.0, FILL, 24.0]);
    line.show(&mut ui.ui, |[mut a, mut b, mut c, mut d]| {
        a.btn_center(true, true, "a");
        b.btn_center(true, true, "b");
        c.btn_center(true, true, "c");
        d.btn_center(true, true, "d");
    });

    let line = Flow::symmetric(8, 8, 4, 40.0, [24.0; 11]);
    line.show(&mut ui.ui, |ui| {
        use Icon::*;
        let icons = [PadH, PadV, PadL, PadR, PadT, PadB, PadIndividual];
        for (mut ui, icon) in ui.into_iter().zip(icons) {
            ui.btn_icon(icon, ui::BtnIcon::FRAMELESS);
        }
    });

    line.show(&mut ui.ui, |ui| {
        use Icon::*;
        let icons = [AlignL, AlignH, AlignR, AlignT, AlignV, AlignB];
        for (mut ui, icon) in ui.into_iter().zip(icons) {
            ui.btn_icon(icon, ui::BtnIcon::FRAMELESS);
        }
    });

    line.show(&mut ui.ui, |ui| {
        use Icon::*;
        let icons = [HGap, VGap, Angle, X, Y, W, H];
        for (mut ui, icon) in ui.into_iter().zip(icons) {
            ui.btn_icon(icon, ui::BtnIcon::FRAMELESS);
        }
    });

    line.show(&mut ui.ui, |ui| {
        use Icon::*;
        let icons = [FixW, FixH, HugW, HugH, MinW, MinH, MaxW, MaxH];
        for (mut ui, icon) in ui.into_iter().zip(icons) {
            ui.btn_icon(icon, ui::BtnIcon::FRAMELESS);
        }
    });

    line.show(&mut ui.ui, |ui| {
        use Icon::*;
        let icons = [Radius, RadiusNW, RadiusNE, RadiusSW, RadiusSE];
        for (mut ui, icon) in ui.into_iter().zip(icons) {
            ui.btn_icon(icon, ui::BtnIcon::FRAMELESS);
        }
    });
}
