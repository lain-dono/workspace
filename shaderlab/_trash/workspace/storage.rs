use super::NodeShape;
use crate::preview::Preview;
use egui::{layers::ShapeIdx, Pos2, Rect, Stroke, Vec2};
use slotmap::{new_key_type, SlotMap};

pub use crate::next::{Data, Direction, InputDefault, InputDefaultType, Stage};

new_key_type! {
    pub struct Node;
    pub struct Port;
    pub struct Link;
}

pub struct PortData {
    pub label: String,
    pub direction: Direction,
    pub stage: Stage,
    pub data: Data,

    pub node: Node,
    pub pos: Pos2,
    pub rect: Rect,
    pub shape_index: Option<ShapeIdx>,

    pub input_default: Option<InputDefault>,
}

pub struct LinkData {
    pub min: Port, // input
    pub max: Port, // output
    pub stroke: Stroke,
    pub shape: Option<ShapeIdx>,
}

impl LinkData {
    pub fn eq(&self, a: Port, b: Port) -> bool {
        self.has(a) && self.has(b)
    }

    pub fn has(&self, port: Port) -> bool {
        self.min == port || self.max == port
    }

    pub fn input_for(&self, output: Port) -> Option<Port> {
        self.has_output(output).then(|| self.min)
    }

    pub fn has_input(&self, port: Port) -> bool {
        self.min == port
    }

    pub fn output_for(&self, input: Port) -> Option<Port> {
        self.has_input(input).then(|| self.max)
    }
    pub fn has_output(&self, port: Port) -> bool {
        self.max == port
    }
}

pub struct NodeData {
    pub position: Pos2,
    pub width: f32,

    pub title: String,
    pub title_rect: Rect,
    pub rect: Rect,

    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,

    pub show_preview: bool,
    pub preview_is_valid: bool,

    pub shape: Option<NodeShape>,
    pub preview: Option<Preview>,
    pub builder: Option<Box<dyn crate::preview::PreviewBuilder>>,
}

#[derive(Default)]
pub struct Storage {
    pub nodes: SlotMap<Node, NodeData>,
    pub ports: SlotMap<Port, PortData>,
    pub links: SlotMap<Link, LinkData>,
}

macro_rules! derive_index {
    ($prop:ident: <$key:ident, $data:ident>) => {
        impl std::ops::Index<$key> for Storage {
            type Output = $data;

            fn index(&self, index: $key) -> &Self::Output {
                &self.$prop[index]
            }
        }

        impl std::ops::IndexMut<$key> for Storage {
            fn index_mut(&mut self, index: $key) -> &mut Self::Output {
                &mut self.$prop[index]
            }
        }
    };
}

derive_index!(nodes: <Node, NodeData>);
derive_index!(ports: <Port, PortData>);
derive_index!(links: <Link, LinkData>);
