// use super::style::named;
use crate::{ai, state::InGame, time::Tick, ui::style};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, panel.run_if(in_state(InGame)));
    app.add_systems(EguiPrimaryContextPass, example.run_if(in_state(InGame)));
}

fn example(mut contexts: EguiContexts) -> Result {
    egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
        ui.label("world");

        ui.heading("☔☁⛅☀🌙❄🌃");
        ui.heading(" 🎉🎄🎃");
        // ui.add_space(8.0);
    });
    Ok(())
}

pub const SPEED_1: f32 = 1.0;
pub const SPEED_2: f32 = 3.0;
pub const SPEED_3: f32 = 6.0;
pub const SPEED_4: f32 = 15.0;
// pub const SPEED_4: f32 = 1500.0;

pub static MOTIVE_NAMES: [&str; 6] = [
    "🚽 bladder",
    "🍴 hunger",
    "💤 energy",
    "🎮 fun",
    "💬 social",
    "🚿 hygiene",
];

pub fn panel(
    mut contexts: EguiContexts,
    tick: Res<Tick>,
    actors: Query<&ai::Actor>,
    mut time: ResMut<Time<Virtual>>,
) -> Result {
    let actor = actors.single()?;

    egui::Area::new(egui::Id::new("main_panel"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut()?, |ui| {
            main_ui(ui, &mut time, actor, *tick);
        });

    Ok(())
}

pub fn calendar_panel(tick: Res<Tick>, mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("calendar_panel").show(ctx, |ui| {
        let top_spacing_amount = ui.spacing().window_margin.top;
        ui.add_space(top_spacing_amount.into());

        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let size = ui.available_size_before_wrap();
        let layout =
            egui::Layout::left_to_right(egui::Align::Center).with_cross_align(egui::Align::Max);

        ui.allocate_ui_with_layout(size, layout, |ui| {
            // let frame = egui::Frame::group(ui.style());

            // main_ui(ui, &mut time, actor, *tick);

            ui.separator();
            ui.add_space(20.0);

            ui.spacing_mut().item_spacing = egui::vec2(20.0, 0.0);

            month(ui, 0, *tick);
            month(ui, 1, *tick);
            month(ui, 2, *tick);
            month(ui, 3, *tick);
        });
    });

    Ok(())
}

fn main_ui(ui: &mut egui::Ui, time: &mut Time<Virtual>, actor: &ai::Actor, tick: Tick) {
    let fill = ui.style().visuals.window_fill();
    let fill = fill.gamma_multiply_u8(0xAF);
    let frame = egui::Frame::NONE.inner_margin(6);
    let frame = frame.fill(fill);

    frame.show(ui, |ui| {
        let speed_1 = !time.is_paused() && time.relative_speed() == SPEED_1;
        let speed_2 = !time.is_paused() && time.relative_speed() == SPEED_2;
        let speed_3 = !time.is_paused() && time.relative_speed() == SPEED_3;
        let speed_4 = !time.is_paused() && time.relative_speed() == SPEED_4;

        // ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let width = 200.0;

        ui.set_width(width);
        ui.vertical(|ui| {
            ui.set_width(width);

            ui.spacing_mut().item_spacing = egui::vec2(8.0, 2.0);

            ui.label("lorem ".repeat(20));

            progress(ui, 80.0, "mood");

            ui.columns_const(|[l_ui, r_ui]| {
                progress(l_ui, actor.motives[0].current, MOTIVE_NAMES[0]);
                progress(l_ui, actor.motives[1].current, MOTIVE_NAMES[1]);
                progress(l_ui, actor.motives[2].current, MOTIVE_NAMES[2]);
                progress(r_ui, actor.motives[3].current, MOTIVE_NAMES[3]);
                progress(r_ui, actor.motives[4].current, MOTIVE_NAMES[4]);
                progress(r_ui, actor.motives[5].current, MOTIVE_NAMES[5]);
            });

            ui.spacing_mut().item_spacing = egui::vec2(12.0, 8.0);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let date = style::rich("clock", tick.format_date());
                let time = style::rich("clock", tick.format_time());
                let weather = style::rich("clock", "+15°C⛅");
                ui.add(egui::Label::new(date).selectable(false));
                ui.add(egui::Label::new(time).selectable(false));
                ui.add(egui::Label::new(weather).selectable(false));
            });

            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

            ui.columns_const(|[ui0, ui1, ui2, ui3, ui4]| {
                speed_btn(ui0, "⏸", time.is_paused(), || time.pause());
                speed_btn(ui1, "▶", speed_1, || {
                    time.set_relative_speed(SPEED_1);
                    time.unpause();
                });
                speed_btn(ui2, "▶▶", speed_2, || {
                    time.set_relative_speed(SPEED_2);
                    time.unpause();
                });
                speed_btn(ui3, "▶▶▶", speed_3, || {
                    time.set_relative_speed(SPEED_3);
                    time.unpause();
                });
                speed_btn(ui4, "▶▶▶▶", speed_4, || {
                    time.set_relative_speed(SPEED_4);
                    time.unpause();
                });
            });
        });
    });
}

fn progress(ui: &mut egui::Ui, value: f32, label: &str) {
    ui.vertical(|ui| {
        let id = egui::Id::new(("#progress", label));
        let value = ui.ctx().animate_value_with_time(id, value, 0.10);

        let bar = egui::ProgressBar::new(value / 100.0);

        let color = if value < 30.0 {
            palette::DANGER
        } else if value < 50.0 {
            palette::WARNING
        } else {
            palette::NORMAL
        };

        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        let label = style::rich("stat", label);
        ui.add(egui::Label::new(label).selectable(false));
        ui.add(bar.corner_radius(0.0).desired_height(2.0).fill(color));
    });
}

fn speed_btn(ui: &mut egui::Ui, text: &str, selected: bool, clicked: impl FnOnce()) {
    ui.vertical_centered_justified(|ui| {
        let text = style::rich("speed_btn", text);
        let btn = egui::Button::selectable(selected, text);
        if ui.add(btn.corner_radius(0)).clicked() {
            clicked()
        }
    });
}

fn month(ui: &mut egui::Ui, month: u32, tick: Tick) {
    let name = match month {
        0 => "Spring",
        1 => "Summer",
        2 => "Autumn",
        3 => "Winter",
        _ => unreachable!(),
    };

    ui.vertical(|ui| {
        let item = 14.0;
        let gap = 5.0;
        ui.set_max_width(item * 7.0 + gap * 6.0);
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 5.0);

        // Monday Tuesday Wednesday Thursday Friday Saturday Sunday

        ui.vertical_centered_justified(|ui| {
            let text = style::rich("calendar", name).strong();
            ui.add(egui::Label::new(text).selectable(false));
        });

        ui.add_space(4.0);

        // ui.columns_const::<{ crate::time::DAYS_PER_WEEK as usize }, _>(|ui| {
        //     let list = ["M", "T", "W", "T", "F", "S", "S"];
        //     for (i, ui) in ui.iter_mut().enumerate() {
        //         let text = egui::RichText::new(list[i]).small();
        //         let label = egui::Label::new(text).selectable(false);
        //         ui.vertical_centered_justified(|ui| ui.add(label));
        //     }
        // });

        let current_month = tick.month_of_year();
        let current_day = tick.num_day_of_month();

        for week in 0..4 {
            ui.columns_const::<{ crate::time::DAYS_PER_WEEK as usize }, _>(|ui| {
                for (i, ui) in ui.iter_mut().enumerate() {
                    ui.set_width(14.0);

                    let day_index = week * crate::time::DAYS_PER_WEEK + i as u32;
                    let tick = crate::time::Tick::from_days(day_index);

                    let day = tick.num_day_of_month();
                    let is_current = day == current_day && month == current_month;
                    let text = style::rich("calendar", format!("{day}"));

                    let bg_color = ui.style().visuals.widgets.noninteractive.bg_fill;
                    let fg_color = ui.style().visuals.widgets.noninteractive.fg_stroke.color;

                    let label = egui::Label::new(match (is_current, tick.is_weekend()) {
                        (true, _) => text.background_color(fg_color).color(bg_color),
                        (_, true) => text.strong(),
                        _ => text,
                    });

                    ui.vertical_centered_justified(|ui| ui.add(label.selectable(false)));
                }
            });
        }
    });
}

mod palette {
    #![allow(clippy::unusual_byte_groupings)]

    use bevy_egui::egui;

    const fn color_hex(color: u32) -> egui::Color32 {
        let [a, b, g, r] = color.to_le_bytes();
        egui::Color32::from_rgba_premultiplied(r, g, b, a)
    }

    pub const DANGER: egui::Color32 = color_hex(0xFF4136_FF);
    pub const WARNING: egui::Color32 = color_hex(0xFFDC00_FF);
    pub const NORMAL: egui::Color32 = color_hex(0x2ECC40_FF);
}

fn _spring_cd(
    velocity: &mut f32,

    current: f32,
    target: f32,

    dt: f32,
    speed: f32, // interp_speed
    max: Option<f32>,
) -> f32 {
    let diff = current - target;

    let a = *velocity - diff * (speed * speed * dt);
    let b = 1.0 + speed * dt;
    let next = a / (b * b);

    *velocity = max.map_or(next, |max| max.min(next));

    current + *velocity * dt
}
