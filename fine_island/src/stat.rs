use eframe::egui;

pub fn stat(name: impl Into<String>, value: f32, label: impl Into<String>) -> Stat {
    Stat {
        name: name.into(),
        value,
        label: label.into(),
    }
}

#[derive(Clone)]
pub struct Stat {
    pub name: String,
    pub value: f32,
    pub label: String,
}

impl egui::Widget for Stat {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut size = ui.available_size_before_wrap();
        size.y = 18.0;

        let valign = egui::Align::Min;
        let layout = egui::Layout::left_to_right(valign).with_cross_align(egui::Align::Max);

        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.label(self.name);
            ui.label(self.label);
        });

        let bar = egui::ProgressBar::new(self.value);
        ui.add(bar.desired_height(2.0).corner_radius(0))
    }
}

pub fn stats() -> Vec<Stat> {
    vec![
        stat("Pain", 0.25, "Tears run down your face"),
        stat("Arousal", 0.25, "You feel aroused"),
        stat("Will power", 0.25, "You are wavering"),
        stat("Fatigue", 0.25, "You are alert"),
        stat("Stress", 0.25, "You are tense"),
        stat("Trauma", 0.25, "You are uneasy"),
        stat("Control", 0.25, "You are anxious"),
        stat("Allure", 1.00, "People lust after you"),
    ]
}
