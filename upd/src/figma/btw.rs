fn run() {
    let mut root = Widget {
        x: Constraint::offset(20.0, 20.0),
        y: Constraint::min_offset(150.0, 20.0),
        auto: Some(AutoLayout::default()),
        children: vec![
            Widget {
                x: Constraint::center(350.0),
                y: Constraint::center(120.0),
                ..Widget::simple(egui::Color32::from_rgba_unmultiplied(200, 0, 0, 127))
            },
            Widget {
                x: Constraint::fill(),
                y: Constraint::fill(),
                ..Widget::simple(egui::Color32::from_rgba_unmultiplied(0, 200, 0, 127))
            },
        ],
        ..Widget::simple(egui::Color32::from_rgb(230, 230, 230))
    };

    root.frame = frame;

    layout_children(frame, &mut root.children);
    draw_children(ui.ui, frame, &root.children);
}

fn draw_children(ui: &mut egui::Ui, frame: egui::Rect, children: &[Widget]) {
    for child in children {
        // let base = child.frame.translate(frame.min.to_vec2());
        let base = child.frame;
        // draw_frame(ctx, base, &child.color);
        ui.painter().rect_filled(base, 0, child.color);
        draw_children(ui, base, &child.children);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug)]
pub struct AutoLayout {
    pub axis: Axis,
    pub align: f32,
    pub gap: f32, // None is auto?
                  // pub padding: egui::Vec2,
}

impl Default for AutoLayout {
    fn default() -> Self {
        Self {
            axis: Axis::X,
            gap: 0.0,
            align: 0.0,
        }
    }
}

fn layout_children(frame: egui::Rect, children: &mut [Widget]) {
    use egui::{Rect, Vec2, pos2};

    let Vec2 { x: dx, y: dy } = frame.size();

    for base in children.iter_mut() {
        let frame = {
            let (min_x, max_x) = base.x.perform(0.0, dx);
            let (min_y, max_y) = base.y.perform(0.0, dy);
            Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y))
        };

        base.frame = frame;

        if let Some(auto) = base.auto {
            let (main_sum, cross_max, flex_count) = match auto.axis {
                Axis::X => measure(&base.children, |child| (child.x.size(), child.y.size())),
                Axis::Y => measure(&base.children, |child| (child.y.size(), child.x.size())),
            };

            layout_children(frame, &mut base.children);

            let mut offset = Vec2::ZERO;
            match auto.axis {
                Axis::X => {
                    for child in &mut base.children {
                        child.frame.min += offset;
                        child.frame.max += offset;
                        // if child.x.is_flex()
                    }
                }
                Axis::Y => {
                    todo!()
                }
            }
        } else {
            layout_children(frame, &mut base.children);
        }
    }
}

fn measure<'a>(
    children: &'a [Widget],
    map: impl Fn(&'a Widget) -> (Option<f32>, Option<f32>),
) -> (f32, f32, usize) {
    children.iter().map(map).fold(
        (0.0, 0.0, 0),
        |(main_sum, cross_max, flex), (main, cross)| {
            (
                main_sum + main.unwrap_or(0.0),
                cross_max.max(cross.unwrap_or(0.0)),
                flex + main.is_some() as usize,
            )
        },
    )
}

pub struct Widget {
    pub frame: egui::Rect,
    pub x: Constraint,
    pub y: Constraint,
    pub auto: Option<AutoLayout>,
    pub children: Vec<Self>,

    pub color: egui::Color32,
}

impl Widget {
    pub fn new(frame: egui::Rect, color: egui::Color32) -> Self {
        Self {
            frame,
            color,
            children: Vec::new(),
            x: Constraint::Flexible { offset: (0.0, 0.0) },
            y: Constraint::Flexible { offset: (0.0, 0.0) },
            auto: None,
        }
    }

    pub fn simple(color: egui::Color32) -> Self {
        Self::new(egui::Rect::NOTHING, color)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Constraint {
    Align { size: f32, align: f32 },
    MinOffset { size: f32, offset: f32 },
    MaxOffset { size: f32, offset: f32 },
    Flexible { offset: (f32, f32) },
    // HugContent { align: f32 },
}

impl Constraint {
    pub const fn fill() -> Self {
        Self::Flexible { offset: (0.0, 0.0) }
    }

    pub const fn align(size: f32, align: f32) -> Self {
        Self::Align { size, align }
    }

    pub const fn offset(start: f32, end: f32) -> Self {
        let offset = (start, end);
        Self::Flexible { offset }
    }

    pub const fn min_offset(size: f32, offset: f32) -> Self {
        Self::MinOffset { size, offset }
    }

    pub const fn max_offset(size: f32, offset: f32) -> Self {
        Self::MaxOffset { size, offset }
    }

    pub const fn start(size: f32) -> Self {
        Self::Align { align: -1.0, size }
    }

    pub const fn end(size: f32) -> Self {
        Self::Align { align: 1.0, size }
    }

    pub const fn center(size: f32) -> Self {
        Self::Align { align: 0.0, size }
    }

    pub fn size(self) -> Option<f32> {
        match self {
            Self::Align { size, .. } => Some(size),
            Self::MinOffset { size, .. } => Some(size),
            Self::MaxOffset { size, offset } => Some(size),
            Self::Flexible { offset: (min, max) } => None,
            // Self::HugContent => None,
        }
    }

    pub fn is_flex(self) -> bool {
        matches!(self, Self::Flexible { .. })
    }

    pub fn perform(self, start: f32, end: f32) -> (f32, f32) {
        match self {
            Self::Align { align, size } => {
                let delta = (end - start - size) * 0.5;
                let aligned = align * delta;
                (start + aligned + delta, end + aligned - delta)
            }
            Self::MinOffset { size, offset } => (start + offset, start + offset + size),
            Self::MaxOffset { size, offset } => (end - offset, end - offset + size),
            Self::Flexible { offset: (min, max) } => (start + min, end - max),
        }
    }
}
