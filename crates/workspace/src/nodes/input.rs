use super::super::builder::{expr::*, *};
use super::super::{Data, Fragment, Node, Port, PreviewBuilder, Storage};
use egui::Rgba;
use egui::widgets::color_picker::color_edit_button_hsva;

pub struct InputFloat {
    x: Port,
    output: Port,
}

impl InputFloat {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Float", 80.0, |ctx, node| Self {
            x: ctx.input(node, "x", Fragment, Data::Float, None),
            output: ctx.output(node, "out", Fragment, Data::Float, None),
        })
    }
}

impl PreviewBuilder for InputFloat {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);
        let x = function.for_input_f32(self.x)?;
        x.emit(function)
    }
}

pub struct InputVector2 {
    x: Port,
    y: Port,
    output: Port,
}

impl InputVector2 {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Vector2", 80.0, |ctx, node| Self {
            x: ctx.input(node, "x", Fragment, Data::Float, None),
            y: ctx.input(node, "y", Fragment, Data::Float, None),
            output: ctx.output(node, "out", Fragment, Data::Vector2, None),
        })
    }
}

impl PreviewBuilder for InputVector2 {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);
        let x = function.for_input_f32(self.x)?;
        let y = function.for_input_f32(self.y)?;
        [x, y].emit(function)
    }
}

pub struct InputVector3 {
    x: Port,
    y: Port,
    z: Port,
    output: Port,
}

impl InputVector3 {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Vector3", 80.0, |ctx, node| Self {
            x: ctx.input(node, "x", Fragment, Data::Float, None),
            y: ctx.input(node, "y", Fragment, Data::Float, None),
            z: ctx.input(node, "z", Fragment, Data::Float, None),
            output: ctx.output(node, "out", Fragment, Data::Vector3, None),
        })
    }
}

impl PreviewBuilder for InputVector3 {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);
        let x = function.for_input_f32(self.x)?;
        let y = function.for_input_f32(self.y)?;
        let z = function.for_input_f32(self.z)?;
        [x, y, z].emit(function)
    }
}

pub struct InputVector4 {
    x: Port,
    y: Port,
    z: Port,
    w: Port,
    output: Port,
}

impl InputVector4 {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Vector4", 80.0, |ctx, node| Self {
            x: ctx.input(node, "x", Fragment, Data::Float, None),
            y: ctx.input(node, "y", Fragment, Data::Float, None),
            z: ctx.input(node, "z", Fragment, Data::Float, None),
            w: ctx.input(node, "w", Fragment, Data::Float, None),
            output: ctx.output(node, "out", Fragment, Data::Vector4, None),
        })
    }
}

impl PreviewBuilder for InputVector4 {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);
        let x = function.for_input_f32(self.x)?;
        let y = function.for_input_f32(self.y)?;
        let z = function.for_input_f32(self.z)?;
        let w = function.for_input_f32(self.w)?;
        [x, y, z, w].emit(function)
    }
}

pub struct Color {
    rgba: Rgba,
    port: Port,
}

impl Color {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Color", 80.0, |ctx, node| {
            let rgba = Rgba::WHITE;
            let port = ctx.output(node, "color", Fragment, Data::Vector4, None);
            Self { rgba, port }
        })
    }
}

impl PreviewBuilder for Color {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let alpha = egui::widgets::color_picker::Alpha::OnlyBlend;
        let mut hsva = self.rgba.into();
        ui.vertical_centered_justified(|ui| {
            color_edit_button_hsva(ui, &mut hsva, alpha);
        });
        self.rgba = hsva.into();
    }

    fn show_preview(&self) -> bool {
        false
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.port, output);

        let rgba = [
            F32(self.rgba.r()),
            F32(self.rgba.g()),
            F32(self.rgba.b()),
            F32(self.rgba.a()),
        ];

        rgba.emit(function)
    }
}

pub struct Slider {
    port: Port,
    value: f32,
    min: f32,
    max: f32,
}

impl Slider {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Slider", 140.0, |ctx, node| Self {
            port: ctx.output(node, "out", Fragment, Data::Float, None),
            value: 0.0,
            min: 0.0,
            max: 1.0,
        })
    }
}

impl PreviewBuilder for Slider {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let align = egui::Align::Min;
        let layout = egui::Layout::top_down(align).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            let range = self.min..=self.max;
            ui.add(egui::Slider::new(&mut self.value, range).fixed_decimals(4));

            let min = egui::DragValue::new(&mut self.min).range(f32::NEG_INFINITY..=self.max);
            ui.add(min.speed(0.01).prefix("min "));

            let max = egui::DragValue::new(&mut self.max).range(self.min..=f32::INFINITY);
            ui.add(max.speed(0.01).prefix("max "));
        });
    }

    fn show_preview(&self) -> bool {
        false
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.port, output);
        F32(self.value).emit(function)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum ConstantValue {
    PI,
    TAU,
    PHI,
    E,
    SQRT2,
}

pub struct Constant {
    selected: ConstantValue,
    port: Port,
}

impl Constant {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Const", 70.0, |ctx, node| Self {
            selected: ConstantValue::PI,
            port: ctx.output(node, "out", Fragment, Data::Float, None),
        })
    }
}

impl PreviewBuilder for Constant {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("Constant selector")
            .selected_text(format!("{:?}", self.selected))
            .width(62.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected, ConstantValue::PI, "PI");
                ui.selectable_value(&mut self.selected, ConstantValue::TAU, "TAU");
                ui.selectable_value(&mut self.selected, ConstantValue::PHI, "PHI");
                ui.selectable_value(&mut self.selected, ConstantValue::E, "E");
                ui.selectable_value(&mut self.selected, ConstantValue::SQRT2, "SQRT2");
            });
    }

    fn show_preview(&self) -> bool {
        false
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.port, output);

        let value = match self.selected {
            ConstantValue::PI => std::f32::consts::PI,
            ConstantValue::TAU => std::f32::consts::TAU,
            ConstantValue::PHI => (1.0 + f32::sqrt(5.0)) / 2.0,
            ConstantValue::E => std::f32::consts::E,
            ConstantValue::SQRT2 => std::f32::consts::SQRT_2,
        };

        F32(value).emit(function)
    }
}
