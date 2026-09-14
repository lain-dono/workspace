use eframe::egui;

use super::time::Clock;

pub fn action(
    key: impl Into<Option<usize>>,
    label: impl Into<String>,
    time: Clock,
) -> ActionButton {
    let key = key.into();
    let label = label.into();
    ActionButton { key, label, time }
}

#[derive(Clone)]
pub struct ActionButton {
    pub key: Option<usize>,
    pub label: String,
    pub time: Clock,
}

impl egui::Widget for ActionButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let key = self.key.map_or(String::new(), |key| format!("{}.", key));

        let key = egui::RichText::new(key).color(egui::Color32::from_gray(170));
        let label = egui::RichText::new(self.label).color(egui::Color32::from_gray(238));

        let h = self.time.hours();
        let m = self.time.minutes();
        let time = egui::RichText::new(format!("({h:}:{m:02})"))
            .color(egui::Color32::from_rgb(102, 170, 255));
        btn_justify(ui, (key, label, time))
    }
}

fn justify<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.scope_builder(
        egui::UiBuilder::new()
            .layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true)),
        add_contents,
    )
}

fn btn_justify<'a>(ui: &mut egui::Ui, atoms: impl egui::IntoAtoms<'a>) -> egui::Response {
    justify(ui, |ui| ui.button(atoms)).inner
}
