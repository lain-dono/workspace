use super::{Link, Node};
use ahash::AHashSet;
use egui::NumExt;

slotmap::new_key_type! {
    pub struct Port;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Data {
    Boolean,

    Float,
    Vector2,
    Vector3,
    Vector4,
    VectorAny,

    Matrix2,
    Matrix3,
    Matrix4,
    MatrixAny,

    VectorOrMatrix,
    FloatOrVector,
    FloatOrVectorOrMatrix,

    Image(naga::ImageClass),
    VirtualTexture,
    Gradient,
    Sampler,
}

impl Data {
    pub fn can_connect(self, other: Self) -> bool {
        use Data::*;
        match (self, other) {
            (Image(a), Image(b)) => a == b,
            (VirtualTexture, VirtualTexture) => true,
            (Gradient, Gradient) => true,
            (Sampler, Sampler) => true,
            (Boolean, Boolean) => true,

            (
                Float | FloatOrVector | FloatOrVectorOrMatrix,
                Float | FloatOrVector | FloatOrVectorOrMatrix,
            ) => true,

            (
                Vector2 | Vector3 | Vector4 | VectorAny | FloatOrVector | FloatOrVectorOrMatrix,
                Vector2 | Vector3 | Vector4 | VectorAny | FloatOrVector | FloatOrVectorOrMatrix,
            ) => true,

            (
                Matrix2 | Matrix3 | Matrix4 | MatrixAny | FloatOrVectorOrMatrix,
                Matrix2 | Matrix3 | Matrix4 | MatrixAny | FloatOrVectorOrMatrix,
            ) => true,

            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    pub fn is_input(self) -> bool {
        matches!(self, Self::Input)
    }

    pub fn is_output(self) -> bool {
        matches!(self, Self::Output)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Vertex,
    Fragment,
}

impl Stage {
    pub fn is_fragment(self) -> bool {
        matches!(self, Self::Fragment)
    }

    pub fn is_vertex(self) -> bool {
        matches!(self, Self::Vertex)
    }

    pub fn draw(
        self,
        painter: &egui::Painter,
        center: egui::Pos2,
        fill_color: egui::Color32,
        linked: bool,
    ) {
        let stroke = (1.5, fill_color);
        let radius = 4.0;
        let rect = egui::Rect::from_center_size(center, [7.0; 2].into());

        match self {
            Self::Fragment => {
                painter.add(egui::Shape::circle_stroke(center, radius, stroke));
                if linked {
                    painter.add(egui::Shape::circle_filled(center, radius - 2.0, fill_color));
                }
            }
            Self::Vertex => {
                painter.add(egui::Shape::rect_stroke(
                    rect,
                    0.0,
                    stroke,
                    egui::StrokeKind::Middle,
                ));
                if linked {
                    painter.add(egui::Shape::rect_filled(rect.shrink(2.0), 0.0, fill_color));
                }
            }
        }
    }
}

pub struct PortData {
    pub label: String,
    pub direction: Direction,
    pub stage: Stage,
    pub data: Data,
    pub node: Node,

    pub position: egui::Pos2,
    pub rect: egui::Rect,
    pub links: AHashSet<Link>,

    pub input_default: Option<InputDefault>,
}

impl PortData {
    pub fn input(node: Node, label: impl Into<String>, stage: Stage, data: Data) -> Self {
        Self {
            label: label.into(),
            direction: Direction::Input,
            stage,
            data,
            node,

            position: egui::Pos2::ZERO,
            rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),
            links: AHashSet::default(),

            input_default: None,
        }
    }

    pub fn output(node: Node, label: impl Into<String>, stage: Stage, data: Data) -> Self {
        Self {
            label: label.into(),
            direction: Direction::Output,
            stage,
            data,
            node,

            position: egui::Pos2::ZERO,
            rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),
            links: AHashSet::default(),

            input_default: None,
        }
    }

    pub fn is_input(&self) -> bool {
        matches!(self.direction, Direction::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self.direction, Direction::Output)
    }

    pub fn widget(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let font_id = egui::FontId::proportional(10.0);

        let output = self.is_output();
        let linked = !self.links.is_empty();

        let halign = if output {
            egui::Align::Max
        } else {
            egui::Align::Min
        };

        let layout = egui::Layout::top_down_justified(halign);

        let egui::InnerResponse { inner, response } = ui.with_layout(layout, |ui| {
            let sense = egui::Sense::drag();

            //let padding = ui.spacing().button_padding;
            let padding = egui::Vec2::new(0.0, 0.0);
            let total_extra = padding + padding;
            let wrap_width = ui.available_width() - total_extra.x;

            let text = {
                let text = &self.label;
                let format = egui::TextFormat::simple(font_id, egui::Color32::WHITE);

                let job = egui::text::LayoutJob {
                    sections: vec![egui::text::LayoutSection {
                        leading_space: 0.0,
                        byte_range: 0..text.len(),
                        format,
                    }],
                    text: text.clone(),
                    halign,
                    wrap: egui::text::TextWrapping {
                        max_width: wrap_width,
                        ..Default::default()
                    },
                    break_on_newline: false,
                    ..Default::default()
                };

                ui.painter().layout_job(job)
            };

            /*
            icon_width: 14.0,
            icon_width_inner: 8.0,
            icon_spacing: 4.0,
            */

            //let icon_width = ui.spacing().icon_width;
            let icon_width = 10.0;
            let icon_spacing = 2.0;

            let mut desired_size = text.size() + total_extra;
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

            desired_size.x += icon_width + icon_spacing;
            desired_size.y = desired_size.y.max(icon_width + total_extra.y);

            let (rect, response) = ui.allocate_at_least(desired_size, sense);
            // ? response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text.text()));

            let ext_hover = {
                let pos = ui.ctx().input(|input| input.pointer.hover_pos());
                pos.filter(|&pos| response.rect.contains(pos))
                    .and_then(|pos| ui.ctx().layer_id_at(pos))
                    .is_some_and(|layer| layer == ui.painter().layer_id())
            };

            if ui.is_rect_visible(rect) {
                let egui::Rect { min, max } = rect;
                let visuals = ui.style().interact(&response);

                let pad = padding.x + icon_width + icon_spacing;
                let x = if output { max.x - pad } else { min.x + pad };
                let text_pos = egui::pos2(x, rect.center().y - 0.5 * text.size().y);

                // text.paint_with_visuals(ui.painter(), text_pos, visuals);
                ui.painter()
                    .galley_with_override_text_color(text_pos, text, visuals.text_color());

                let pad = icon_width * 0.5;
                let x = if output { max.x - pad } else { min.x + pad };
                let center = egui::pos2(x, rect.center().y);

                let linked = linked || response.hovered() || response.dragged() || ext_hover;
                let fill_color = self.data.color();
                self.stage.draw(ui.painter(), center, fill_color, linked);

                self.position = center;
            }

            response
        });

        let response = response.union(inner);
        self.rect = response.rect;
        response
    }
}

pub enum InputDefaultType {
    Marker(String),
    Bool,
    F32,
    F32x2,
    F32x3,
    F32x4,
}

pub struct InputDefault {
    pub kind: InputDefaultType,
    pub width: Option<f32>,
    pub layer_id: Option<egui::LayerId>,

    pub checked: bool,

    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl InputDefault {
    pub const fn new(kind: InputDefaultType, x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            kind,
            width: None,
            layer_id: None,
            checked: false,
            x,
            y,
            z,
            w,
        }
    }

    pub fn marker(marker: impl Into<String>) -> Self {
        Self::new(InputDefaultType::Marker(marker.into()), 0.0, 0.0, 0.0, 1.0)
    }

    pub const fn bool() -> Self {
        Self::new(InputDefaultType::Bool, 0.0, 0.0, 0.0, 1.0)
    }

    pub const fn f32(x: f32) -> Self {
        Self::new(InputDefaultType::F32, x, 0.0, 0.0, 1.0)
    }

    pub const fn f32x2(x: f32, y: f32) -> Self {
        Self::new(InputDefaultType::F32x2, x, y, 0.0, 1.0)
    }

    pub const fn f32x3(x: f32, y: f32, z: f32) -> Self {
        Self::new(InputDefaultType::F32x3, x, y, z, 1.0)
    }

    pub const fn f32x4(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self::new(InputDefaultType::F32x4, x, y, z, w)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        fn drag_value(ui: &mut egui::Ui, label: &str, value: &mut f32) -> egui::Response {
            let widget = egui::DragValue::new(value);
            ui.add(widget.speed(0.01).prefix(label))
        }

        match self.kind {
            InputDefaultType::Marker(ref marker) => ui.label(marker),
            InputDefaultType::Bool => ui.checkbox(&mut self.checked, ""),
            InputDefaultType::F32 => drag_value(ui, "x ", &mut self.x),
            InputDefaultType::F32x2 => {
                let x = drag_value(ui, "y ", &mut self.y);
                let y = drag_value(ui, "x ", &mut self.x);
                x.union(y)
            }
            InputDefaultType::F32x3 => {
                let x = drag_value(ui, "z ", &mut self.z);
                let y = drag_value(ui, "y ", &mut self.y);
                let z = drag_value(ui, "x ", &mut self.x);
                x.union(y).union(z)
            }
            InputDefaultType::F32x4 => {
                let x = drag_value(ui, "w ", &mut self.w);
                let y = drag_value(ui, "z ", &mut self.z);
                let z = drag_value(ui, "y ", &mut self.y);
                let w = drag_value(ui, "x ", &mut self.x);
                x.union(y).union(z).union(w)
            }
        }
    }
}

impl super::builder::expr::Emit for InputDefault {
    fn emit(&self, function: &mut super::builder::FnBuilder) -> super::builder::EmitResult {
        use super::builder::expr::{Bool, F32};
        let [x, y, z, w] = [F32(self.x), F32(self.y), F32(self.z), F32(self.w)];
        match self.kind {
            InputDefaultType::Bool => Bool(self.checked).emit(function),
            InputDefaultType::F32 => x.emit(function),
            InputDefaultType::F32x2 => [x, y].emit(function),
            InputDefaultType::F32x3 => [x, y, z].emit(function),
            InputDefaultType::F32x4 => [x, y, z, w].emit(function),
            _ => unimplemented!(),
        }
    }
}
