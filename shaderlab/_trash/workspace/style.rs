#![allow(dead_code)]

use super::{Data, Direction, NodeData, Stage};
use egui::{layers::ShapeIdx, Color32};

pub struct NodeShape {
    pub outline: ShapeIdx,
    pub titlebar: ShapeIdx,
    pub separator: ShapeIdx,
    pub background: ShapeIdx,
}

impl NodeShape {
    pub fn nop(painter: &egui::Painter) -> Self {
        Self {
            outline: painter.add(egui::Shape::Noop),
            separator: painter.add(egui::Shape::Noop),
            background: painter.add(egui::Shape::Noop),
            titlebar: painter.add(egui::Shape::Noop),
        }
    }
}

#[derive(Debug)]
pub struct Style {
    pub node_base: Color32,
    pub node_active: Color32,
    pub node_outline: Color32,
    pub node_padding: egui::Vec2,

    pub port_radius: f32,
    pub port_side: f32,
    pub port_thickness: f32,
    pub port_hover_radius: f32,
    pub port_offset: f32,

    pub link_hover_distance: f32,
    pub link_stroke: egui::Stroke,

    pub box_selector: Color32,
    pub box_selector_outline: Color32,

    pub grid_spacing: f32,
    pub grid_background: Color32,
    pub grid_stroke: egui::Stroke,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            node_base: NODE_BASE,
            node_active: NODE_ACTIVE,
            node_outline: NODE_BORDER,
            node_padding: NODE_PADDING,

            port_radius: 4.0,
            port_side: 7.0,
            port_thickness: 1.5,
            port_hover_radius: 16.0,
            port_offset: -8.0,

            link_hover_distance: 10.0,
            link_stroke: egui::Stroke {
                width: 2.0,
                color: PORT_VECTOR_1,
            },

            box_selector: Color32::from_rgba_unmultiplied(61, 133, 224, 30),
            box_selector_outline: Color32::from_rgba_unmultiplied(61, 133, 224, 150),

            grid_spacing: 32.0,
            grid_background: WORKSPACE_BG,
            grid_stroke: egui::Stroke {
                width: 1.0,
                color: WORKSPACE_GRID,
            },
        }
    }
}

impl Style {
    pub fn screen_port_coords(
        &self,
        node: egui::Rect,
        attr: egui::Rect,
        kind: Direction,
    ) -> egui::Pos2 {
        let x = match kind {
            Direction::Input => node.min.x - self.port_offset,
            Direction::Output => node.max.x + self.port_offset,
        };
        egui::pos2(x, 0.5 * (attr.min.y + attr.max.y))
    }

    pub fn draw_port_shape(
        &self,
        ui: &mut egui::Ui,
        filled: bool,
        idx: egui::layers::ShapeIdx,
        center: egui::Pos2,
        shape: Stage,
        color: Color32,
    ) {
        let stroke = (self.port_thickness, color);
        let rect = egui::Rect::from_center_size(center, [self.port_side; 2].into());

        let outer_shape = match shape {
            Stage::Fragment => egui::Shape::circle_stroke(center, self.port_radius, stroke),
            Stage::Vertex => egui::Shape::rect_stroke(rect, 0.0, stroke),
        };

        let shape = if filled {
            let rect = rect.shrink(2.0);
            let radius = self.port_radius - 2.0;
            let shape = match shape {
                Stage::Fragment => egui::Shape::circle_filled(center, radius, color),
                Stage::Vertex => egui::Shape::rect_filled(rect, 0.0, color),
            };
            egui::Shape::Vec(vec![shape, outer_shape])
        } else {
            outer_shape
        };
        ui.painter().set(idx, shape);
    }

    pub fn draw_node(&self, node: &mut NodeData, painter: &egui::Painter, is_active: bool) {
        let node_shape = if let Some(node_shape) = node.shape.take() {
            node_shape
        } else {
            return;
        };

        let fill_color = if is_active {
            self.node_active
        } else {
            self.node_base
        };

        let mut rect = node.rect;

        let title_rect = node.title_rect;
        //let title_rect = title_rect.expand2(egui::vec2(0.0, crate::style::NODE_PADDING.y));
        let mut title_rect = egui::Rect::from_min_size(
            title_rect.min,
            egui::vec2(node.rect.width(), title_rect.height()),
        );

        rect.min.y = title_rect.max.y + 0.5;
        title_rect.max.y -= 0.5;

        let shape = egui::Shape::rect_filled(rect, 0.0, fill_color);
        painter.set(node_shape.background, shape);

        if node.title_rect.height() > 0.0 {
            let shape = egui::Shape::rect_filled(title_rect, 0.0, fill_color);
            painter.set(node_shape.titlebar, shape);
        }

        {
            let stroke = (NODE_BORDER_THICKNESS, self.node_outline);
            let rect = node.rect.expand(NODE_BORDER_THICKNESS / 2.0);
            let shape = egui::Shape::rect_stroke(rect, 1.0, stroke);
            painter.set(node_shape.outline, shape);
        }

        {
            let stroke = (NODE_SEPARATOR_THICKNESS, self.node_outline);
            let min = egui::pos2(title_rect.min.x, title_rect.max.y + 0.5);
            let max = egui::pos2(title_rect.max.x, title_rect.max.y + 0.5);
            let shape = egui::Shape::line_segment([min, max], stroke);
            painter.set(node_shape.separator, shape);
        }
    }
}

impl Data {
    pub fn color(self) -> egui::Color32 {
        use Data::*;
        match self {
            Boolean => PORT_BOOL,
            Float | FloatOrVector | VectorAny => PORT_VECTOR_1,
            Vector2 => PORT_VECTOR_2,
            Vector3 => PORT_VECTOR_3,
            Vector4 => PORT_VECTOR_4,
            FloatOrVectorOrMatrix | VectorOrMatrix => PORT_MATRIX,
            Matrix2 | Matrix3 | Matrix4 | MatrixAny => PORT_MATRIX,
            Image(_) => PORT_IMAGE,
            VirtualTexture | Gradient | Sampler => PORT_STRUCT,
        }
    }
}

pub const PORT_BOOL: Color32 = Color32::from_rgb(0x94, 0x81, 0xe6);
pub const PORT_STRUCT: Color32 = Color32::from_rgb(0xc8, 0xc8, 0xc8);
pub const PORT_IMAGE: Color32 = Color32::from_rgb(0xff, 0x8b, 0x8b);
pub const PORT_MATRIX: Color32 = Color32::from_rgb(0x8f, 0xc1, 0xdf);

pub const PORT_VECTOR_1: Color32 = Color32::from_rgb(0x84, 0xe4, 0xe7);
pub const PORT_VECTOR_2: Color32 = Color32::from_rgb(0x9a, 0xef, 0x92);
pub const PORT_VECTOR_3: Color32 = Color32::from_rgb(0xf6, 0xff, 0x9a);
pub const PORT_VECTOR_4: Color32 = Color32::from_rgb(0xfb, 0xcb, 0xf4);

pub const WORKSPACE_BG: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
//pub const WORKSPACE_GRID: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1C);
pub const WORKSPACE_GRID: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);

//pub const NODE_BACKGROUND: Color32 = rgba(0x333333, 0.95);
pub const NODE_TEXT: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub const NODE_BASE: Color32 = Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 0xEE); // 0xE6
pub const NODE_ACTIVE: Color32 = Color32::from_rgba_premultiplied(0x44, 0x44, 0x44, 0xEE);
pub const NODE_BORDER: Color32 = Color32::from_rgba_premultiplied(0x18, 0x18, 0x18, 0xEE);
pub const NODE_BORDER_THICKNESS: f32 = 2.0;
pub const NODE_SEPARATOR_THICKNESS: f32 = 1.0;

pub const TITLE_PADDING: egui::Vec2 = egui::vec2(8.0, 4.0);
pub const NODE_PADDING: egui::Vec2 = egui::vec2(16.0, 4.0);

pub const SLIDER_RAIL: Color32 = Color32::from_rgb(0x5e, 0x5e, 0x5e);
pub const SLIDER_HANDLE_ACTIVE: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);
pub const SLIDER_HANDLE_HOVER: Color32 = Color32::from_rgb(0xEA, 0xEA, 0xEA);

pub const FONT_SIZE: u16 = 14;

pub const PORT_COLOR: Color32 = PORT_VECTOR_1;
pub const PORT_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 0xe6);

pub const SELECTION: Color32 = Color32::from_rgb(0x44, 0xc0, 0xff);

pub const SIDEBAR_BG: Color32 = Color32::from_rgb(39, 39, 39);
