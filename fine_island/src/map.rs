use eframe::egui::{self, Color32, Frame, Pos2, Rect, Sense, Stroke, Ui, Window, emath, vec2};

mod canvas;
mod path;
mod shape;
mod vector;

pub use self::path::Path;
pub use self::shape::Shape;
pub use self::vector::Vector;

pub struct Map {
    /// in 0-1 normalized coordinates
    pub lines: Vec<Vec<Pos2>>,
    pub stroke: Stroke,

    pub canvas: canvas::Canvas,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            canvas: canvas::Canvas::new(0.0, 0.0),
        }
    }
}

impl Map {
    pub fn show(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        self.ui_control(ui);

        ui.drag_angle(&mut self.canvas.camera.angle);
        ui.add(egui::DragValue::new(&mut self.canvas.camera.scale).range(1.0..=200.0));

        ui.add(
            egui::DragValue::new(&mut self.canvas.color_difference)
                .range(0.01..=1.0)
                .speed(0.01),
        );

        ui.label("Paint with your mouse/touch!");
        Frame::canvas(ui.style()).show(ui, |ui| self.ui_content(ui));
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            ui.add(&mut self.stroke);
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
            response.mark_changed();
        }

        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        {
            let r = response.rect;
            let x = r.min.x + r.width() / 2.0;
            let y = r.min.y + r.height() * 0.7;

            self.canvas.camera.x = x;
            self.canvas.camera.y = y;
            self.canvas.transformation = self.canvas.camera.transformation();

            // let mut iso = canvas::Canvas::new(x, y);
            let ctx = &mut self.canvas;
            // var Shape = Isomer.Shape;
            // var Point = Isomer.Point;
            // var Color = Isomer.Color;
            let base = egui::Color32::from_rgb(120, 120, 120);
            let red = egui::Color32::from_rgb(160, 60, 50);
            let blue = egui::Color32::from_rgb(50, 60, 160);

            ctx.add_shape(Shape::prism(Vector::ZERO, 3.0, 3.0, 1.0), base);
            ctx.add_shape(
                Shape::pyramid(Vector::new(0.0, 2.0, 1.0), 1.0, 1.0, 1.0),
                red,
            );
            ctx.add_shape(
                Shape::prism(Vector::new(2.0, 0.0, 1.0), 1.0, 1.0, 1.0),
                blue,
            );

            // for p in iso.data {
            painter.extend(ctx.data.drain(..))
            // }
        }

        response
    }
}
