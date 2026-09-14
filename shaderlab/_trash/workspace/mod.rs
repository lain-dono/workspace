pub mod canvas;
pub mod storage;
pub mod style;

pub use self::{
    canvas::Canvas,
    storage::{
        Data, Direction, InputDefault, InputDefaultType, Link, LinkData, Node, NodeData, Port,
        PortData, Stage, Storage,
    },
    style::{NodeShape, Style},
};
pub use crate::graph::Graph;
pub use crate::next::LinkBezier;

use egui::InnerResponse;

#[derive(Debug)]
enum Interaction {
    None,
    Panning,
    BoxSelection(egui::Rect),
    NodeSelection,
    LinkCreation { min: Port, max: Option<Port> },
}

bitflags::bitflags! {
    struct ElementStateChange: u32 {
        const LINK_STARTED = 1 << 0;
        const LINK_DROPPED = 1 << 1;
        const LINK_CREATED = 1 << 2;
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct MouseButtonState {
    pub pressed: bool,
    pub released: bool,
    pub dragging: bool,
}

impl MouseButtonState {
    fn update(&mut self, down: bool) -> bool {
        let old = *self;
        self.released = (self.pressed || self.dragging) && !down;
        self.dragging = (self.pressed || self.dragging) && down;
        self.pressed = down && !(self.pressed || self.dragging);
        old != *self || self.dragging
    }
}

#[derive(Default)]
struct Mouse {
    pointer: egui::Pos2,
    delta: egui::Vec2,
    in_canvas: bool,

    left: MouseButtonState,
    right: MouseButtonState,
}

impl Mouse {
    fn update(&mut self, ui: &mut egui::Ui, response: &egui::Response) -> bool {
        let input = ui.ctx().input();

        let pointer = response.hover_pos().unwrap_or(self.pointer);
        self.delta = pointer - self.pointer;
        self.pointer = pointer;
        self.in_canvas = response.hover_pos().is_some();

        let primary = input.pointer.button_down(egui::PointerButton::Primary);
        let secondary = input.pointer.button_down(egui::PointerButton::Secondary);

        self.left.update(primary) || self.right.update(secondary)
    }
}

/// The Context that tracks the state of the node editor
pub struct Context {
    pub storage: Storage,
    pub dirty: bool,

    canvas: Canvas,
    style: Style,
    mouse: Mouse,
    interaction: Interaction,

    element_state_change: ElementStateChange,

    nodes_overlapping_with_mouse: Vec<Node>,
    hovered_node: Option<Node>,
    interactive_node: Option<Node>,
    selected_nodes: Vec<Node>,
    depth_order: Vec<Node>,
    depth_order_clone: Vec<Node>,

    hovered_port: Option<Port>,
    occluded_ports: Vec<Port>,

    hovered_link: Option<Link>,
    deleted_link: Option<Link>,
    snap_link: Option<Link>,

    graph: Graph<Node>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            nodes_overlapping_with_mouse: Vec::new(),
            occluded_ports: Vec::new(),

            canvas: Canvas::default(),
            style: Style::default(),
            mouse: Mouse::default(),
            dirty: true,

            hovered_node: None,
            interactive_node: None,
            hovered_link: None,
            hovered_port: None,

            deleted_link: None,
            snap_link: None,

            element_state_change: ElementStateChange::empty(),

            storage: Storage::default(),

            depth_order: Vec::new(),
            selected_nodes: Vec::new(),

            interaction: Interaction::None,

            graph: Graph::default(),

            depth_order_clone: Vec::new(),
        }
    }
}

impl Context {
    pub fn spawn<P, T, F, B>(&mut self, position: P, width: f32, title: T, preview: F) -> Node
    where
        P: Into<egui::Pos2>,
        T: Into<String>,
        B: crate::preview::PreviewBuilder + 'static,
        F: FnOnce(&mut Self, Node) -> B,
    {
        self.dirty = true;

        let position = self.canvas.screen_to_grid(position.into());

        let node = self.storage.nodes.insert(NodeData {
            position,
            width,
            title: title.into(),
            title_rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),
            rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),

            shape: None,

            show_preview: false,
            preview_is_valid: false,

            inputs: Vec::new(),
            outputs: Vec::new(),

            preview: None,
            builder: None,
        });

        self.storage[node].builder = Some(Box::new(preview(self, node)));
        self.depth_order.push(node);

        node
    }

    pub fn despawn(&mut self, node: Node) {
        self.dirty = true;

        if self.storage.nodes.remove(node).is_some() {
            self.storage.links.retain(|_, link| {
                self.storage.ports[link.min].node != node
                    && self.storage.ports[link.max].node != node
            });

            self.storage.ports.retain(|_, port| port.node != node);

            let index = self.depth_order.iter().position(|&n| n == node).unwrap();
            self.depth_order.remove(index);
        }
    }

    pub fn input_frag(&mut self, node: Node, label: impl Into<String>, data: Data) -> Port {
        let kind = Direction::Input;
        self.port(node, kind, Stage::Fragment, label.into(), data, None)
    }

    pub fn output_frag(&mut self, node: Node, label: impl Into<String>, data: Data) -> Port {
        let kind = Direction::Output;
        self.port(node, kind, Stage::Fragment, label.into(), data, None)
    }

    pub fn input_vert(&mut self, node: Node, label: impl Into<String>, data: Data) -> Port {
        let kind = Direction::Input;
        self.port(node, kind, Stage::Vertex, label.into(), data, None)
    }

    pub fn output_vert(&mut self, node: Node, label: impl Into<String>, data: Data) -> Port {
        let kind = Direction::Output;
        self.port(node, kind, Stage::Vertex, label.into(), data, None)
    }

    fn port(
        &mut self,
        node: Node,
        kind: Direction,
        shape: Stage,
        label: String,
        data: Data,
        input_default: Option<InputDefault>,
    ) -> Port {
        let input_default = input_default
            .or_else(|| match data {
                Data::Float => Some(InputDefault::float(0.0)),
                Data::Vector2 => Some(InputDefault::vector2(0.0, 0.0)),
                Data::Vector3 => Some(InputDefault::vector3(0.0, 0.0, 0.0)),
                Data::Vector4 => Some(InputDefault::vector4(0.0, 0.0, 0.0, 1.0)),

                Data::VectorAny => Some(InputDefault::vector4(0.0, 0.0, 0.0, 1.0)),
                Data::FloatOrVector => Some(InputDefault::float(0.0)),

                _ => None,
            })
            .filter(|_| kind.is_input());

        let key = self.storage.ports.insert(PortData {
            node,
            direction: kind,
            stage: shape,
            label,
            data,

            pos: egui::Pos2::ZERO,
            rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),

            shape_index: None,
            input_default,
        });

        match kind {
            Direction::Input => self.storage[node].inputs.push(key),
            Direction::Output => self.storage[node].outputs.push(key),
        }

        key
    }

    pub fn connect(
        &mut self,
        min: Port,
        max: Port,
        color: impl Into<Option<egui::Color32>>,
    ) -> Link {
        self.dirty = true;

        let (min, max) = match (self.storage[min].direction, self.storage[max].direction) {
            (Direction::Input, Direction::Output) => (min, max),
            (Direction::Output, Direction::Input) => (max, min),
            _ => return <Link as slotmap::Key>::null(),
        };

        let mut stroke = self.style.link_stroke;
        stroke.color = color.into().unwrap_or(stroke.color);

        println!("connect {:?} {:?}", min, max);

        self.storage.links.insert(LinkData {
            min,
            max,
            stroke,
            shape: None,
        })
    }
}

impl Context {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // cleanup
        {
            self.interactive_node = None;
            self.hovered_node = None;
            self.hovered_link = None;
            self.deleted_link = None;
            self.snap_link = None;

            self.nodes_overlapping_with_mouse.clear();
            self.element_state_change = ElementStateChange::empty();

            self.graph.clear();
        }

        let mut ui = self.canvas.start(ui);
        self.canvas.draw_background(&mut ui, &self.style);

        self.update_links(&mut ui);
        self.update_nodes(&mut ui);

        let response = ui.interact(
            self.canvas.rect,
            ui.id().with("Canvas Interact"),
            egui::Sense::click_and_drag(),
        );

        self.dirty = self.mouse.update(&mut ui, &response) || self.dirty;

        if self.mouse.in_canvas {
            self.resolve_occluded_ports();
            self.resolve_hovered_ports();

            //if self.hovered_port.is_none() {
            self.resolve_hovered_node();
            //}

            if self.hovered_node.is_none() {
                self.resolve_hovered_link();
            }
        }

        self.draw_nodes(&mut ui);
        self.draw_links(&mut ui);

        self.interaction(&mut ui);

        let stroke = self.style.grid_stroke;
        ui.painter().rect_stroke(self.canvas.rect, 0.0, stroke);

        response
    }

    fn update_links(&mut self, ui: &mut egui::Ui) {
        for (link_key, link) in &mut self.storage.links {
            link.shape.replace(ui.painter().add(egui::Shape::Noop));

            if let Interaction::LinkCreation { min, max, .. } = self.interaction {
                let a = min == link.min && max == Some(link.max);
                let b = min == link.max && max == Some(link.min);
                if a || b {
                    self.snap_link.replace(link_key);
                }
            }
        }
    }

    fn update_nodes(&mut self, ui: &mut egui::Ui) {
        fn add_port(
            ui: &mut egui::Ui,
            port: &mut PortData,
            align: egui::Align,
            has: bool,
        ) -> egui::Response {
            let desired_size = ui.available_size();

            let layout = egui::Layout::top_down(align);
            let text = &port.label;

            let content = |ui: &mut egui::Ui| {
                egui::Frame::none()
                    .margin(egui::style::Margin::symmetric(0.0, 2.0))
                    .show(ui, |ui| ui.label(text))
                    .inner
            };
            let response = ui.allocate_ui_with_layout(desired_size, layout, content);
            let InnerResponse { response, inner } = response;
            let response = response.union(inner);

            port.shape_index = Some(ui.painter().add(egui::Shape::Noop));
            port.rect = response.rect;

            if let Some(def) = port.input_default.as_mut().filter(|_| !has) {
                let egui::Pos2 { x, y } = port.rect.left_center();
                let max_rect = egui::Rect {
                    min: egui::pos2(x - 100.0, y - 10.0),
                    max: egui::pos2(x - 20.0, y + 10.0),
                };
                let mut ui = ui.child_ui(max_rect, egui::Layout::top_down(egui::Align::Max));

                let frame = egui::Frame {
                    //fill: self.style.node_base,
                    fill: egui::Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 0xEE),
                    ..egui::Frame::none()
                };

                let inner = frame.show(&mut ui, |ui| ui.horizontal(|ui| def.ui(ui)).response);

                response //.union(inner.response).union(inner.inner)
            } else {
                response
            }
        }

        let mut despawn = None;

        for &node_key in &self.depth_order {
            let Storage {
                nodes,
                ports,
                links,
            } = &mut self.storage;

            let node = &mut nodes[node_key];

            let origin = self.canvas.grid_to_screen(node.position);
            let origin = ui.painter().round_pos_to_pixels(origin);
            let max_rect = egui::Rect::from_min_size(origin, egui::vec2(node.width, 0.0));

            let layout = egui::Layout::top_down(egui::Align::Center);
            let mut ui = ui.child_ui_with_id_source(max_rect, layout, node_key);

            /*
            let layer_id = egui::LayerId::background();
            let mut ui = egui::Ui::new(
                ui.ctx().clone(),
                layer_id,
                egui::Id::new(node_key),
                max_rect,
                ui.clip_rect(),
            );
            */

            let InnerResponse { response, .. } = ui.allocate_ui_at_rect(max_rect, |ui| {
                node.shape = Some(NodeShape::nop(ui.painter()));

                {
                    let title = egui::Frame::none()
                        .margin(egui::style::Margin::symmetric(0.0, 4.0))
                        .show(ui, |ui| ui.label(&node.title));

                    let mark = if node.show_preview { "⏷" } else { "⏵" };
                    let close = "❌";

                    if node.preview_is_valid {
                        let center = title.response.rect.left_center() + egui::vec2(11.0, -4.0);
                        let rect = egui::Rect::from_center_size(center, egui::vec2(10.0, 10.0));
                        let btn = egui::Button::new(mark).frame(false);
                        if ui.put(rect, btn).clicked() {
                            node.show_preview = !node.show_preview;
                        }
                    }

                    {
                        let center = title.response.rect.right_center() + egui::vec2(-11.0, -4.0);
                        let rect = egui::Rect::from_center_size(center, egui::vec2(10.0, 10.0));
                        let btn = egui::Button::new(close).frame(false);
                        if ui.put(rect, btn).clicked() {
                            despawn = Some(node_key);
                        }
                    }

                    node.title_rect = title.response.rect
                };

                let has = |port| links.values().any(|link| link.has(port));

                //ui.add_space(self.style.node_padding.y);

                let mut inputs = node.inputs.iter();
                let mut outputs = node.outputs.iter();

                for _ in 0..usize::min(node.inputs.len(), node.outputs.len()) {
                    let &min_port = inputs.next().unwrap();
                    let &max_port = outputs.next().unwrap();

                    let mut rect = ui.available_rect_before_wrap();
                    rect.min.x += self.style.node_padding.x;
                    rect.max.x -= self.style.node_padding.x;

                    ui.allocate_ui_at_rect(rect, |ui| {
                        ui.horizontal(|ui| {
                            let a =
                                add_port(ui, &mut ports[min_port], egui::Align::Min, has(min_port));
                            let b =
                                add_port(ui, &mut ports[max_port], egui::Align::Max, has(max_port));

                            if a.is_pointer_button_down_on() || b.is_pointer_button_down_on() {
                                self.interactive_node.replace(node_key);
                            }
                        });
                    });
                }

                for &port in inputs {
                    let mut rect = ui.available_rect_before_wrap();
                    rect.min.x += self.style.node_padding.x;

                    ui.allocate_ui_at_rect(rect, |ui| {
                        let response = add_port(ui, &mut ports[port], egui::Align::Min, has(port));
                        if response.is_pointer_button_down_on() {
                            self.interactive_node.replace(node_key);
                        }
                    });
                }

                for &port in outputs {
                    let mut rect = ui.available_rect_before_wrap();
                    rect.max.x -= self.style.node_padding.x;

                    ui.allocate_ui_at_rect(rect, |ui| {
                        let response = add_port(ui, &mut ports[port], egui::Align::Max, has(port));
                        if response.is_pointer_button_down_on() {
                            self.interactive_node.replace(node_key);
                        }
                    });
                }

                if let Some(builder) = node.builder.as_mut() {
                    builder.ui(ui);
                    if node.show_preview && node.preview_is_valid && builder.show_preview() {
                        if let Some(preview) = node.preview.as_ref() {
                            ui.image(preview.texture_id, preview.size);
                        }
                    }
                }
            });

            nodes[node_key].rect = response.rect;

            if response.hovered() {
                self.nodes_overlapping_with_mouse.push(node_key);
            }
        }

        if let Some(despawn) = despawn {
            self.despawn(despawn);
        }

        let Storage { nodes, .. } = &mut self.storage;
        for (node_key, node) in nodes {
            let origin = self.canvas.grid_to_screen(node.position);
            let area = egui::Area::new((node_key, "area"));
            let area = area.current_pos(origin);
            area.show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.label(&node.title);
                })
            });
        }
    }

    fn draw_nodes(&mut self, ui: &mut egui::Ui) {
        self.depth_order_clone.clear();
        self.depth_order_clone
            .extend(self.depth_order.iter().copied());

        for node_key in self.depth_order_clone.drain(..) {
            let node = &mut self.storage.nodes[node_key];

            let is_hovered = self.hovered_node == Some(node_key);
            let is_selected = self.selected_nodes.contains(&node_key);

            self.style
                .draw_node(node, ui.painter(), is_selected || is_hovered);

            let ports = node.inputs.iter().chain(node.outputs.iter());

            for &port_key in ports {
                let port = &mut self.storage.ports[port_key];

                port.pos = self
                    .style
                    .screen_port_coords(node.rect, port.rect, port.direction);

                let is_hovered = self.hovered_port == Some(port_key);
                let shape_idx = port.shape_index.take().unwrap();

                if is_hovered
                    && self.mouse.left.pressed
                    && !self
                        .storage
                        .links
                        .values()
                        .any(|link| link.has_input(port_key))
                {
                    self.interaction = Interaction::LinkCreation {
                        min: port_key,
                        max: None,
                    };
                    self.element_state_change |= ElementStateChange::LINK_STARTED;
                }

                let is_creation = if let Interaction::LinkCreation { min, .. } = self.interaction {
                    min == port_key
                } else {
                    false
                };

                let filled = is_hovered
                    || is_creation
                    || self.storage.links.values().any(|link| link.has(port_key));

                self.style.draw_port_shape(
                    ui,
                    filled,
                    shape_idx,
                    port.pos,
                    port.stage,
                    port.data.color(),
                );
            }

            if is_hovered && self.mouse.left.pressed && self.interactive_node != Some(node_key) {
                if let Interaction::None = self.interaction {
                    self.interaction = Interaction::NodeSelection;

                    if !self.selected_nodes.contains(&node_key) {
                        self.selected_nodes.clear();
                        self.selected_nodes.push(node_key);

                        self.depth_order.retain(|&key| key != node_key);
                        self.depth_order.push(node_key);
                    }
                }
            }
        }
    }
}

impl Context {
    fn screen_port_coords(&self, port: &PortData) -> egui::Pos2 {
        let node = &self.storage[port.node];
        self.style
            .screen_port_coords(node.rect, port.rect, port.direction)
    }

    fn resolve_occluded_ports(&mut self) {
        self.occluded_ports.clear();
        if self.depth_order.len() < 2 {
            return;
        }

        for depth_idx in 0..(self.depth_order.len() - 1) {
            let node_below = &self.storage[self.depth_order[depth_idx]];
            let range = (depth_idx + 1)..self.depth_order.len();
            for &next_depth in &self.depth_order[range] {
                let rect_above = self.storage[next_depth].rect;
                for &port in node_below.inputs.iter() {
                    if rect_above.contains(self.storage[port].pos) {
                        self.occluded_ports.push(port);
                    }
                }
                for &port in node_below.outputs.iter() {
                    if rect_above.contains(self.storage[port].pos) {
                        self.occluded_ports.push(port);
                    }
                }
            }
        }
    }

    fn resolve_hovered_ports(&mut self) {
        self.hovered_port.take();
        let hover_radius = self.style.port_hover_radius * self.style.port_hover_radius;

        let mut smallest_distance = f32::MAX;
        for (key, port) in &self.storage.ports {
            if self.occluded_ports.contains(&key) {
                continue;
            }

            let distance = (port.pos - self.mouse.pointer).length_sq();
            if distance < hover_radius && distance < smallest_distance {
                smallest_distance = distance;
                self.hovered_port.replace(key);
            }
        }
    }

    fn resolve_hovered_node(&mut self) {
        match self.nodes_overlapping_with_mouse.len() {
            0 => self.hovered_node = None,
            1 => self.hovered_node = Some(self.nodes_overlapping_with_mouse[0]),
            _ => {
                let mut largest_depth_idx = None;

                for &node in self.nodes_overlapping_with_mouse.iter() {
                    for (depth_idx, &depth_node_idx) in self.depth_order.iter().enumerate() {
                        if depth_node_idx == node
                            && largest_depth_idx.map_or(true, |last| depth_idx > last)
                        {
                            largest_depth_idx = Some(depth_idx);
                            self.hovered_node.replace(node);
                        }
                    }
                }
            }
        }
    }

    fn resolve_hovered_link(&mut self) {
        let mut smallest_distance = f32::MAX;
        self.hovered_link.take();

        for (key, link) in &mut self.storage.links {
            if self.hovered_port == Some(link.min) || self.hovered_port == Some(link.max) {
                self.hovered_link.replace(key);
                return;
            }

            let min = &self.storage.ports[link.min];
            let max = &self.storage.ports[link.max];

            let bezier = LinkBezier::new(min.pos, max.pos);
            let rect = bezier.containing_rect_for_bezier_curve(self.style.link_hover_distance);

            if rect.contains(self.mouse.pointer) {
                let distance = bezier.distance_to_cubic_bezier(self.mouse.pointer);
                if distance < self.style.link_hover_distance && distance < smallest_distance {
                    smallest_distance = distance;
                    self.hovered_link.replace(key);
                }
            }
        }
    }

    fn draw_links(&mut self, ui: &mut egui::Ui) {
        for (link_key, link) in &mut self.storage.links {
            let min = &self.storage.ports[link.min];
            let max = &self.storage.ports[link.max];

            let bezier = LinkBezier::new(min.pos, max.pos);

            let link_shape = link.shape.take().unwrap();
            let is_hovered = self.hovered_link == Some(link_key)
                && !matches!(self.interaction, Interaction::BoxSelection(_));

            if is_hovered && self.mouse.left.pressed {
                let detach;

                if let Interaction::LinkCreation { .. } = &mut self.interaction {
                    let hovered_port = &self.storage.ports[self.hovered_port.unwrap()];
                    if hovered_port.direction.is_input() {
                        detach = self.hovered_port
                    } else {
                        detach = None;
                    }
                } else {
                    let to_min = self.storage.ports[link.min]
                        .pos
                        .distance(self.mouse.pointer);
                    let to_max = self.storage.ports[link.max]
                        .pos
                        .distance(self.mouse.pointer);
                    //let closest = if to_min < to_max { link.min } else { link.max };
                    let farthest = if to_min > to_max { link.min } else { link.max };
                    detach = Some(farthest);
                }

                if let Some(detach) = detach {
                    self.deleted_link.replace(link_key);
                    self.interaction = Interaction::LinkCreation {
                        min: detach,
                        max: None,
                    };
                }
            }

            if self.deleted_link != Some(link_key) {
                if let Some(bezier) = bezier.validate() {
                    let mut stroke = link.stroke;
                    stroke.color = min.data.color();
                    ui.painter().set(link_shape, bezier.draw(stroke));
                }
            }
        }

        if let Some(deleted_link) = self.deleted_link.take() {
            self.dirty = true;
            self.storage.links.remove(deleted_link);
        }
    }

    fn interaction(&mut self, ui: &mut egui::Ui) {
        if self.mouse.left.pressed || self.mouse.right.pressed {
            let any_ui_element_hovered = self.hovered_node.is_some()
                || self.hovered_link.is_some()
                || self.hovered_port.is_some();

            let mouse_not_in_canvas = !self.mouse.in_canvas;

            if !matches!(self.interaction, Interaction::None)
                || any_ui_element_hovered
                || mouse_not_in_canvas
            {
                return;
            }

            if self.mouse.right.pressed {
                self.interaction = Interaction::Panning;
            } else {
                let rect = egui::Rect::from_points(&[self.mouse.pointer]);
                self.interaction = Interaction::BoxSelection(rect);
            }
        }

        match self.interaction {
            Interaction::None => (),
            Interaction::Panning => {
                if self.mouse.right.dragging || self.mouse.right.pressed {
                    self.canvas.panning += self.mouse.delta;
                } else {
                    self.interaction = Interaction::None;
                }
            }
            Interaction::NodeSelection => {
                if self.mouse.left.dragging {
                    let delta = self.mouse.delta;
                    for &key in self.selected_nodes.iter() {
                        self.storage.nodes[key].position += delta;
                    }
                }
                if self.mouse.left.released {
                    self.interaction = Interaction::None;
                }
            }
            Interaction::BoxSelection(rect) => self.box_selection(ui, rect),
            Interaction::LinkCreation { min, max } => self.link_creation(ui, min, max),
        }
    }

    fn box_selection(&mut self, ui: &mut egui::Ui, mut rect: egui::Rect) {
        rect.max = self.mouse.pointer;

        {
            let fill_color = self.style.box_selector;
            let stroke = (1.0, self.style.box_selector_outline);

            let rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x.min(rect.max.x), rect.min.y.min(rect.max.y)),
                egui::pos2(rect.min.x.max(rect.max.x), rect.min.y.max(rect.max.y)),
            );

            self.selected_nodes.clear();
            for (key, node) in self.storage.nodes.iter() {
                if rect.intersects(node.rect) {
                    self.selected_nodes.push(key);
                }
            }

            ui.painter().rect(rect, 0.0, fill_color, stroke);
        }

        self.interaction = if self.mouse.left.released {
            let mut extend = Vec::with_capacity(self.selected_nodes.len());
            let selected_nodes = &self.selected_nodes;
            self.depth_order.retain(|&x| {
                let contains = selected_nodes.contains(&x);
                if contains {
                    extend.push(x);
                }
                !contains
            });
            self.depth_order.extend(extend);
            Interaction::None
        } else {
            Interaction::BoxSelection(rect)
        };
    }

    fn link_creation(&mut self, ui: &mut egui::Ui, mut min: Port, mut max: Option<Port>) {
        let maybe_duplicate_link = self.hovered_port.and_then(|max| {
            self.storage.links.iter().find_map(
                |(key, link)| {
                    if link.eq(min, max) {
                        Some(key)
                    } else {
                        None
                    }
                },
            )
        });

        let should_snap = self
            .hovered_port
            .map_or(false, |idx| self.should_link_snap_to_port(min, idx));

        let snapping_changed = max.map_or(false, |idx| self.hovered_port != Some(idx));

        if let Some(link_key) = self.snap_link {
            if snapping_changed && self.snap_link.is_some() {
                let link = &self.storage[link_key];
                min = if link.min == min { link.max } else { link.min };
                max = None;
            }
        }

        let start_port = &self.storage[min];
        let start_pos = self.screen_port_coords(start_port);

        let end_pos = if should_snap {
            self.screen_port_coords(&self.storage[self.hovered_port.unwrap()])
        } else {
            self.mouse.pointer
        };

        let bezier = {
            let (min, max) = if start_port.direction.is_output() {
                (start_pos, end_pos)
            } else {
                (end_pos, start_pos)
            };
            LinkBezier::new(min, max)
        };
        if let Some(bezier) = bezier.validate() {
            ui.painter().add(bezier.draw(self.style.link_stroke));
        }

        let link_creation_on_snap = self.hovered_port.is_some();

        if !should_snap {
            max = None;
        }

        let create_link = should_snap && (self.mouse.left.released || link_creation_on_snap);

        if create_link && maybe_duplicate_link.is_none() {
            if !self.mouse.left.released && max == self.hovered_port {
                return;
            }
            self.element_state_change |= ElementStateChange::LINK_CREATED;
            max = self.hovered_port;
        }

        if self.mouse.left.released && !create_link {
            self.element_state_change |= ElementStateChange::LINK_DROPPED;
        }

        self.interaction = if self.mouse.left.released {
            if let Some(max) = max {
                self.connect(min, max, None);
            }
            Interaction::None
        } else {
            Interaction::LinkCreation { min, max }
        };
    }

    fn should_link_snap_to_port(&mut self, current: Port, hovered: Port) -> bool {
        let start = &self.storage[current];
        let end = &self.storage[hovered];
        if start.node == end.node {
            return false;
        }

        if start.stage != end.stage {
            return false;
        }

        if !start.data.can_connect(end.data) {
            return false;
        }

        match (start.direction, end.direction) {
            (Direction::Input, Direction::Input) | (Direction::Output, Direction::Output) => {
                return false
            }
            (Direction::Output, Direction::Input) if self.any_link_with_port(hovered) => {
                return false
            }
            (Direction::Input, Direction::Output) if self.any_link_with_port(current) => {
                return false
            }
            _ => (),
        }

        self.graph.clear();

        for link in self.storage.links.values() {
            let input = self.storage[link.min].node;
            let output = self.storage[link.max].node;
            self.graph.add_edge(output, input);
        }

        let (input, output) = match (start.direction, end.direction) {
            (Direction::Input, Direction::Output) => (start, end),
            (Direction::Output, Direction::Input) => (end, start),
            _ => unreachable!(),
        };

        self.graph.add_edge(output.node, input.node);
        !self.graph.reachable(input.node, output.node)
    }

    fn any_link_with_port(&self, port: Port) -> bool {
        self.storage.links.values().any(|link| link.has(port))
    }
}

impl Canvas {
    /*
    pub fn set_node_pos_screen_space(&mut self, node_id: usize, screen_space_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = self.screen_space_to_grid_space(screen_space_pos);
    }

    pub fn set_node_pos_editor_space(&mut self, node_id: usize, editor_space_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = self.editor_space_to_grid_spcae(editor_space_pos);
    }

    pub fn set_node_pos_grid_space(&mut self, node_id: usize, grid_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = grid_pos;
    }

    pub fn set_node_draggable(&mut self, node_id: usize, draggable: bool) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].draggable = draggable;
    }

    pub fn node_pos_screen_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes
            .find(node_id)
            .map(|x| self.grid_space_to_screen_space(self.nodes.pool[x].origin))
    }

    pub fn node_pos_editor_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes
            .find(node_id)
            .map(|x| self.grid_space_to_editor_spcae(self.nodes.pool[x].origin))
    }

    pub fn node_pos_grid_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes.find(node_id).map(|x| self.nodes.pool[x].origin)
    }

    /// Check if there is a node that is hovered by the pointer
    pub fn node_hovered(&self) -> Option<usize> {
        self.hovered_node_index.map(|x| self.nodes.pool[x].id.0)
    }

    /// Check if there is a link that is hovered by the pointer
    pub fn link_hovered(&self) -> Option<usize> {
        self.hovered_link.map(|x| self.links.pool[x].id.0)
    }

    /// Check if there is a pin that is hovered by the pointer
    pub fn pin_hovered(&self) -> Option<usize> {
        self.hovered_pin.map(|x| self.pins.pool[x].id.0)
    }

    pub fn selected_nodes(&self) -> Vec<usize> {
        self.selected_nodes
            .iter()
            .map(|x| self.nodes.pool[*x].id.0)
            .collect()
    }

    pub fn clear_node_selection(&mut self) {
        self.selected_nodes.clear()
    }

    /// Has a new link been created from a pin?
    pub fn link_started(&self) -> Option<usize> {
        if self
            .element_state_change
            .contains(ElementStateChange::LINK_STARTED)
        {
            Some(self.ports.pool[self.link_creation.start_pin].id.0)
        } else {
            None
        }
    }

    /// Has a link been dropped? if including_detached_links then links that were detached then dropped are included
    pub fn link_dropped(&self, including_detached_links: bool) -> Option<usize> {
        if self
            .element_state_change
            .contains(ElementStateChange::LINK_DROPPED)
            && (including_detached_links || !self.link_creation.from_detach)
        {
            Some(self.ports.pool[self.link_creation.start_pin].id.0)
        } else {
            None
        }
    }

    /// Has a new link been created?
    /// -> Option<start_pin, end_pin created_from_snap>
    pub fn link_created(&self) -> Option<(usize, usize, bool)> {
        if !self
            .element_state_change
            .contains(ElementStateChange::LINK_CREATED)
        {
            return None;
        }

        let (start_pin_id, end_pin_id) = {
            let start_pin = &self.ports.pool[self.link_creation.start_pin];
            let end_pin = &self.ports.pool[self.link_creation.end_pin.unwrap()];
            if start_pin.kind.is_ouput() {
                (start_pin.id.0, end_pin.id.0)
            } else {
                (end_pin.id.0, start_pin.id.0)
            }
        };

        let created_from_snap = matches!(self.interaction, Interaction::LinkCreation);
        Some((start_pin_id, end_pin_id, created_from_snap))
    }

    /// Has a new link been created? Includes start and end node
    /// -> Option<start_pin, start_node, end_pin, end_node created_from_snap>
    pub fn link_created_node(&self) -> Option<(usize, usize, usize, usize, bool)> {
        if !self
            .element_state_change
            .contains(ElementStateChange::LINK_CREATED)
        {
            return None;
        }

        let (start_pin_id, start_node_id, end_pin_id, end_node_id) = {
            let start_pin = &self.ports.pool[self.link_creation.start_pin];
            let end_pin = &self.ports.pool[self.link_creation.end_pin.unwrap()];
            let start_node = &self.nodes.pool[start_pin.parent_node_idx];
            let end_node = &self.nodes.pool[end_pin.parent_node_idx];
            if start_pin.kind.is_output() {
                (start_pin.id.0, start_node.id.0, end_pin.id.0, end_node.id.0)
            } else {
                (end_pin.id.0, end_node.id.0, start_pin.id.0, start_node.id.0)
            }
        };

        let created_from_snap = matches!(self.interaction, Interaction::LinkCreation);
        Some((
            start_pin_id,
            start_node_id,
            end_pin_id,
            end_node_id,
            created_from_snap,
        ))
    }

    // Was an existing link detached?
    pub fn link_destroyed(&self) -> Option<usize> {
        self.deleted_link
    }

    pub fn node_dimensions(&self, id: usize) -> Option<egui::Vec2> {
        self.nodes.find(id).map(|x| self.nodes.pool[x].rect.size())
    }
    */
}
