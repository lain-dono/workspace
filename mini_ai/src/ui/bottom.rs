use crate::time::Tick;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub const SPEED_1: f32 = 1.0;
pub const SPEED_2: f32 = 3.0;
pub const SPEED_3: f32 = 6.0;
pub const SPEED_4: f32 = 15.0;
// pub const SPEED_4: f32 = 1500.0;

pub fn panel(
    tick: Res<Tick>,
    mut contexts: EguiContexts,
    actors: Query<&ai::Actor>,
    mut time: ResMut<Time<Virtual>>,
) -> Result {
    let Ok(actor) = actors.single() else {
        return Ok(());
    };

    egui::Area::new(egui::Id::new("my_area"))
        // .pivot(egui::Align2::LEFT_BOTTOM)
        // .fixed_pos(egui::pos2(32.0, 32.0))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut()?, |ui| {
            main_ui(ui, &mut time, actor, *tick);
        });

    // return;

    let panel = egui::TopBottomPanel::bottom("bottom_panel");
    panel.show(contexts.ctx_mut()?, |ui| {
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
        ui.set_width(230.0);
        ui.vertical(|ui| {
            ui.set_width(230.0);

            ui.spacing_mut().item_spacing = egui::vec2(8.0, 2.0);
            ui.columns_const(|[l_ui, r_ui]| {
                progress(l_ui, &actor.motives, 0);
                progress(l_ui, &actor.motives, 1);
                progress(l_ui, &actor.motives, 2);
                progress(r_ui, &actor.motives, 3);
                progress(r_ui, &actor.motives, 4);
                progress(r_ui, &actor.motives, 5);
            });

            ui.spacing_mut().item_spacing = egui::vec2(12.0, 8.0);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let date = egui::RichText::new(tick.format_date()).strong();
                let time = egui::RichText::new(tick.format_time()).strong();
                let weather = egui::RichText::new("+15°C⛅").strong();
                let [date, time, weather] = [date, time, weather]
                    .map(|text| text.size(16.0).text_style(egui::TextStyle::Monospace));
                ui.add(egui::Label::new(date).selectable(false));
                ui.add(egui::Label::new(time).selectable(false));
                ui.add(egui::Label::new(weather).selectable(false));
            });
            // ui.heading("☔☁⛅☀🌙❄🌃 🎉🎄🎃");
            // ui.add_space(8.0);

            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

            ui.columns_const(|[ui0, ui1, ui2, ui3, ui4]| {
                btn(ui0, "⏸", time.is_paused(), || time.pause());
                btn(ui1, "▶", speed_1, || {
                    time.set_relative_speed(SPEED_1);
                    time.unpause();
                });
                btn(ui2, "▶▶", speed_2, || {
                    time.set_relative_speed(SPEED_2);
                    time.unpause();
                });
                btn(ui3, "▶▶▶", speed_3, || {
                    time.set_relative_speed(SPEED_3);
                    time.unpause();
                });
                btn(ui4, "▶▶▶▶", speed_4, || {
                    time.set_relative_speed(SPEED_4);
                    time.unpause();
                });
            });
        });
    });
}

fn progress(ui: &mut egui::Ui, motives: &[ai::Motive], index: usize) {
    ui.vertical(|ui| {
        let value = motives[index].current;

        let id = egui::Id::new(("#motive", index));
        let value = ui.ctx().animate_value_with_time(id, value, 0.10);

        let name = egui::RichText::new(crate::MOTIVE_NAMES[index]).strong();
        let bar = egui::ProgressBar::new(value / 100.0);

        let color = if value < 30.0 {
            palette::DANGER
        } else if value < 50.0 {
            palette::WARNING
        } else {
            palette::NORMAL
        };

        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        ui.add(egui::Label::new(name).selectable(false));
        ui.add(bar.corner_radius(0.0).desired_height(2.0).fill(color));
    });
}

fn btn(ui: &mut egui::Ui, text: &str, selected: bool, clicked: impl FnOnce()) {
    ui.vertical_centered_justified(|ui| {
        let text = egui::RichText::new(text).extra_letter_spacing(-8.0);
        let text = text.strong();
        if ui.add(egui::Button::selectable(selected, text)).clicked() {
            clicked();
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
            ui.add(egui::Label::new(egui::RichText::new(name).strong()).selectable(false));
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

        for week in 0..crate::time::WEEKS_PER_MONTH {
            ui.columns_const::<{ crate::time::DAYS_PER_WEEK as usize }, _>(|ui| {
                for (i, ui) in ui.iter_mut().enumerate() {
                    ui.set_width(14.0);

                    let day_index = week * crate::time::DAYS_PER_WEEK + i as u32;
                    let tick = crate::time::Tick::from_days(day_index);

                    let day = tick.num_day_of_month();
                    let is_current = day == current_day && month == current_month;
                    let text = egui::RichText::new(format!("{day}"));

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
