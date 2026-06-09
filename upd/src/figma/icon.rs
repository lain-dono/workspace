use bevy_egui::egui::{
    Color32, Painter, Shape, Ui,
    epaint::tessellator::path::add_circle_quadrant,
    epaint::tessellator::path::rounded_rectangle,
    epaint::{CornerRadiusF32, PathStroke, Stroke},
    {Pos2, Rect, Vec2, vec2},
};

#[derive(Clone, Copy, Debug)]
pub enum Icon {
    X,
    Y,
    W,
    H,

    FixW,
    FixH,
    HugW,
    HugH,

    MinW,
    MinH,
    MaxW,
    MaxH,

    Angle,

    ChevronUp,
    ChevronDown,

    IgnoreAutoLayout,
    ResizeToFit,

    EnableAutoLayout,
    DisableAutoLayout,

    HGap,
    VGap,

    Search,
    Add,
    Collapse,

    PadH,
    PadV,

    PadL,
    PadR,
    PadT,
    PadB,

    PadIndividual,

    AlignL,
    AlignH,
    AlignR,

    AlignT,
    AlignV,
    AlignB,

    Radius,

    RadiusNW,
    RadiusNE,
    RadiusSW,
    RadiusSE,
}

impl Icon {
    pub fn paint(self, ui: &Ui, color: Color32) {
        self.paint_at(ui.painter(), ui.available_rect_before_wrap(), color);
    }
    pub fn paint_at(self, painter: &Painter, frame: Rect, color: Color32) {
        let center = frame.center();

        let draw = Paint { painter, center };

        let show_viewport = false;
        if show_viewport {
            let r = Rect::from_center_size(center, vec2(24.0, 24.0));
            painter.rect_filled(r, 0, color.gamma_multiply(0.25));
        }

        let stroke = (1.0, color);
        let fill = color;

        match self {
            Self::X => {
                draw.line([vec2(-2.5, -3.5), vec2(2.5, 3.5)], stroke);
                draw.line([vec2(2.5, -3.5), vec2(-2.5, 3.5)], stroke);
            }
            Self::Y => {
                draw.line([vec2(-2.5, -3.5), vec2(0.0, 0.5)], stroke);
                draw.line([vec2(2.5, -3.5), vec2(0.0, 0.5)], stroke);
                draw.line([vec2(0.0, 4.0), vec2(0.0, 0.5)], stroke);
            }
            Self::W => {
                let points = [
                    vec2(4.5, -4.0),
                    vec2(2.5, 4.0),
                    vec2(0.0, -4.0),
                    vec2(-2.5, 4.0),
                    vec2(-4.5, -4.0),
                ];
                draw.line(points, stroke);
            }
            Self::H => {
                draw.vline(-2.5, -4.0, 4.0, stroke);
                draw.vline(2.5, -4.0, 4.0, stroke);
                draw.hline(-2.5, 2.5, 0.0, stroke);
            }

            Self::FixW => {
                draw.hline(-5.5, 5.0, 0.0, stroke);
                draw.vline(-5.0, -3.0, 3.0, stroke);
                draw.vline(5.0, -3.0, 3.0, stroke);
            }
            Self::FixH => {
                draw.vline(0.0, -5.5, 5.0, stroke);
                draw.hline(-3.0, 3.0, -5.0, stroke);
                draw.hline(-3.0, 3.0, 5.0, stroke);
            }
            Self::HugW => {
                let points = [vec2(-4.5, -3.0), vec2(-1.5, 0.0), vec2(-4.5, 3.0)];
                draw.line(points, stroke);

                let points = [vec2(4.5, -3.0), vec2(1.5, 0.0), vec2(4.5, 3.0)];
                draw.line(points, stroke);
            }
            Self::HugH => {
                let points = [vec2(-3.0, -4.5), vec2(0.0, -1.5), vec2(3.0, -4.5)];
                draw.line(points, stroke);

                let points = [vec2(-3.0, 4.5), vec2(0.0, 1.5), vec2(3.0, 4.5)];
                draw.line(points, stroke);
            }
            Self::MinW => {
                draw.vline(0.0, -5.5, 5.5, stroke);

                let points = [vec2(-5.5, -2.5), vec2(-3.0, 0.0), vec2(-5.5, 2.5)];
                draw.line(points, stroke);
                draw.hline(-7.5, -3.0, 0.0, stroke);

                let points = [vec2(5.5, -2.5), vec2(3.0, 0.0), vec2(5.5, 2.5)];
                draw.line(points, stroke);
                draw.hline(7.5, 3.0, 0.0, stroke);
            }
            Self::MinH => {
                draw.hline(-5.5, 5.5, 0.0, stroke);

                let points = [vec2(-2.5, -5.5), vec2(0.0, -3.0), vec2(2.5, -5.5)];
                draw.line(points, stroke);
                draw.vline(0.0, -7.5, -3.0, stroke);

                let points = [vec2(-2.5, 5.5), vec2(0.0, 3.0), vec2(2.5, 5.5)];
                draw.line(points, stroke);
                draw.vline(0.0, 7.5, 3.0, stroke);
            }
            Self::MaxW => {
                draw.vline(-6.5, -5.5, 5.5, stroke);
                draw.vline(6.5, -5.5, 5.5, stroke);

                let points = [vec2(-2.5, -2.5), vec2(-5.0, 0.0), vec2(-2.5, 2.5)];
                draw.line(points, stroke);
                let points = [vec2(2.5, -2.5), vec2(5.0, 0.0), vec2(2.5, 2.5)];
                draw.line(points, stroke);

                draw.hline(-5.0, 5.0, 0.0, stroke);
            }
            Self::MaxH => {
                draw.hline(-5.5, 5.5, -6.5, stroke);
                draw.hline(-5.5, 5.5, 6.5, stroke);

                let points = [vec2(-2.5, -2.5), vec2(0.0, -5.0), vec2(2.5, -2.5)];
                draw.line(points, stroke);
                let points = [vec2(-2.5, 2.5), vec2(0.0, 5.0), vec2(2.5, 2.5)];
                draw.line(points, stroke);

                draw.vline(0.0, -5.0, 5.0, stroke);
            }

            Self::Angle => {
                draw.circle_quadrant(vec2(-3.5, 3.5), 4.0, 3.0, stroke);
                draw.hline(-4.0, 4.0, 3.5, stroke);
                draw.vline(-3.5, -4.0, 4.0, stroke);
            }

            Self::ChevronUp => {
                draw.line([vec2(-2.0, 1.0), vec2(0.0, -1.0)], stroke);
                draw.line([vec2(2.5, 1.0), vec2(0.0, -1.0)], stroke);
            }

            Self::ChevronDown => {
                draw.line([vec2(-2.0, -1.0), vec2(0.0, 1.0)], stroke);
                draw.line([vec2(2.5, -1.0), vec2(0.0, 1.0)], stroke);
            }

            Self::IgnoreAutoLayout => {
                draw.stroke(vec2(-2.5, -2.5), vec2(2.5, 2.5), 0.5, stroke);

                draw.hline(-2.5, -5.5, -5.5, stroke);
                draw.vline(-5.5, -5.5, -2.5, stroke);

                draw.hline(2.5, 5.5, -5.5, stroke);
                draw.vline(5.5, -2.5, -5.5, stroke);

                draw.hline(2.5, 5.5, 5.5, stroke);
                draw.vline(5.5, 2.5, 5.5, stroke);

                draw.hline(-2.5, -5.5, 5.5, stroke);
                draw.vline(-5.5, 2.5, 5.5, stroke);
            }

            Self::ResizeToFit => {
                for [x, y] in [[1, 1], [-1, 1], [-1, -1], [1, -1]] {
                    let v = vec2(x as f32, y as f32);
                    let [min, max] = [v * 6.0, v * 2.5];
                    draw.hline(min.x, max.x, max.y, stroke);
                    draw.vline(max.x, min.y, max.y, stroke);
                    draw.line([min, max], stroke);
                }
            }

            Self::DisableAutoLayout => {
                draw.stroke(vec2(-5.5, 5.5), vec2(-1.5, -5.5), 1.5, stroke);
                draw.stroke(vec2(1.5, -1.5), vec2(5.5, -5.5), 1.5, stroke);

                draw.hline(1.5, 5.5, 3.5, stroke);
                draw.vline(3.5, 1.5, 5.5, stroke);
            }

            Self::EnableAutoLayout => {
                draw.stroke(vec2(-5.5, 5.5), vec2(-1.5, -5.5), 1.5, stroke);
                draw.stroke(vec2(1.5, -1.5), vec2(5.5, -5.5), 1.5, stroke);

                draw.line([vec2(1.5, 3.5), vec2(3.0, 5.0)], stroke);
                draw.line([vec2(5.5, 2.0), vec2(3.0, 5.0)], stroke);
            }

            Self::HGap => {
                draw.vline(0.0, -2.5, 2.5, stroke);
                let points = [
                    vec2(-4.0, -4.0),
                    vec2(-3.5, -4.0),
                    vec2(-3.0, -3.5),
                    vec2(-3.0, 3.5),
                    vec2(-3.5, 4.0),
                    vec2(-4.0, 4.0),
                ];
                draw.line(points, stroke);
                let points = [
                    vec2(4.0, -4.0),
                    vec2(3.5, -4.0),
                    vec2(3.0, -3.5),
                    vec2(3.0, 3.5),
                    vec2(3.5, 4.0),
                    vec2(4.0, 4.0),
                ];
                draw.line(points, stroke);
            }
            Self::VGap => {
                draw.hline(-2.5, 2.5, 0.0, stroke);
                let points = [
                    vec2(-4.0, -4.0),
                    vec2(-4.0, -3.5),
                    vec2(-3.5, -3.0),
                    vec2(3.5, -3.0),
                    vec2(4.0, -3.5),
                    vec2(4.0, -4.0),
                ];
                draw.line(points, stroke);
                let points = [
                    vec2(-4.0, 4.0),
                    vec2(-4.0, 3.5),
                    vec2(-3.5, 3.0),
                    vec2(3.5, 3.0),
                    vec2(4.0, 3.5),
                    vec2(4.0, 4.0),
                ];
                draw.line(points, stroke);
            }

            Self::Search => {
                draw.circle(-vec2(0.5, 0.5), 4.5, Color32::TRANSPARENT, stroke);
                draw.line([vec2(3.0, 3.0), vec2(6.5, 6.5)], stroke);
            }
            Self::Add => {
                draw.hline(-5.5, 5.5, 0.0, stroke);
                draw.vline(0.0, -5.5, 5.5, stroke);
            }
            Self::Collapse => {
                draw.hline(-6.0, 0.0, -4.5, stroke);
                draw.hline(-6.0, 0.0, -1.5, stroke);
                draw.line([vec2(1.5, -4.5), vec2(3.5, -2.5)], stroke);
                draw.line([vec2(5.5, -4.5), vec2(3.5, -2.5)], stroke);

                draw.hline(-6.0, 0.0, 1.5, stroke);
                draw.hline(-6.0, 0.0, 4.5, stroke);
                draw.line([vec2(1.5, 4.5), vec2(3.5, 2.5)], stroke);
                draw.line([vec2(5.5, 4.5), vec2(3.5, 2.5)], stroke);
            }

            Self::PadH => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.vline(-4.5, -4.5, 4.5, stroke);
                draw.vline(4.5, -4.5, 4.5, stroke);
            }
            Self::PadL => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.vline(-4.5, -4.5, 4.5, stroke);
            }
            Self::PadR => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.vline(4.5, -4.5, 4.5, stroke);
            }

            Self::PadV => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.hline(-4.5, 4.5, -4.5, stroke);
                draw.hline(-4.5, 4.5, 4.5, stroke);
            }
            Self::PadT => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.hline(-4.5, 4.5, -4.5, stroke);
            }
            Self::PadB => {
                draw.stroke(vec2(-1.5, -1.5), vec2(1.5, 1.5), 0.5, stroke);
                draw.hline(-4.5, 4.5, 4.5, stroke);
            }

            Self::PadIndividual => {
                draw.hline(-2.5, 2.5, 4.5, stroke);
                draw.hline(-2.5, 2.5, -4.5, stroke);

                draw.vline(4.5, -2.5, 2.5, stroke);
                draw.vline(-4.5, -2.5, 2.5, stroke);
            }

            Self::AlignL => {
                draw.vline(-6.0, -6.5, 6.5, stroke);
                draw.fill(vec2(-3.5, -3.5), vec2(6.5, -1.5), 0.5, fill);
                draw.fill(vec2(-3.5, 1.5), vec2(2.5, 3.5), 0.5, fill);
            }
            Self::AlignH => {
                draw.vline(0.0, -6.5, 6.5, stroke);
                draw.fill(vec2(-5.0, -3.5), vec2(5.0, -1.5), 0.5, fill);
                draw.fill(vec2(-3.0, 1.5), vec2(3.0, 3.5), 0.5, fill);
            }
            Self::AlignR => {
                draw.vline(6.0, -6.5, 6.5, stroke);
                draw.fill(vec2(-6.5, -3.5), vec2(3.5, -1.5), 0.5, fill);
                draw.fill(vec2(-2.5, 1.5), vec2(3.5, 3.5), 0.5, fill);
            }

            Self::AlignT => {
                draw.hline(-6.5, 6.5, -6.0, stroke);
                draw.fill(vec2(-3.5, -3.5), vec2(-1.5, 6.5), 0.5, fill);
                draw.fill(vec2(1.5, -3.5), vec2(3.5, 2.5), 0.5, fill);
            }
            Self::AlignV => {
                draw.hline(-6.5, 6.5, 0.0, stroke);
                draw.fill(vec2(-3.5, -5.0), vec2(-1.5, 5.0), 0.5, fill);
                draw.fill(vec2(1.5, -3.0), vec2(3.5, 3.0), 0.5, fill);
            }
            Self::AlignB => {
                draw.hline(-6.5, 6.5, 6.0, stroke);
                draw.fill(vec2(-3.5, -6.5), vec2(-1.5, 3.5), 0.5, fill);
                draw.fill(vec2(1.5, -2.5), vec2(3.5, 3.5), 0.5, fill);
            }

            Self::Radius => {
                // LT
                draw.hline(-4.5, -2.5, -5.5, stroke);
                draw.vline(-5.5, -4.5, -2.5, stroke);
                draw.circle_quadrant(vec2(-4.0, -4.0), 1.5, 2.0, stroke);

                // RT
                draw.hline(2.5, 4.5, -5.5, stroke);
                draw.vline(5.5, -4.5, -2.5, stroke);
                draw.circle_quadrant(vec2(4.0, -4.0), 1.5, 3.0, stroke);

                // LB
                draw.hline(-4.5, -2.5, 5.5, stroke);
                draw.vline(-5.5, 2.5, 4.5, stroke);
                draw.circle_quadrant(vec2(-4.0, 4.0), 1.5, 1.0, stroke);

                // RB
                draw.hline(2.5, 4.5, 5.5, stroke);
                draw.vline(5.5, 2.5, 4.5, stroke);
                draw.circle_quadrant(vec2(4.0, 4.0), 1.5, 0.0, stroke);
            }

            Self::RadiusNW => {
                // left-top
                draw.hline(-2.0, 3.5, -3.5, stroke);
                draw.vline(-3.5, -2.0, 3.5, stroke);
                draw.circle_quadrant(vec2(-1.0, -1.0), 2.5, 2.0, stroke);
            }
            Self::RadiusNE => {
                // right-top
                draw.hline(-3.5, 2.0, -3.5, stroke);
                draw.vline(3.5, -2.0, 3.5, stroke);
                draw.circle_quadrant(vec2(1.0, -1.0), 2.5, 3.0, stroke);
            }
            Self::RadiusSW => {
                // left-bottom
                draw.hline(-2.0, 3.5, 3.5, stroke);
                draw.vline(-3.5, -3.5, 1.0, stroke);
                draw.circle_quadrant(vec2(-1.0, 1.0), 2.5, 1.0, stroke);
            }
            Self::RadiusSE => {
                // right-bottom
                draw.hline(-3.5, 2.0, 3.5, stroke);
                draw.vline(3.5, -3.5, 2.0, stroke);
                draw.circle_quadrant(vec2(1.0, 1.0), 2.5, 0.0, stroke);
            }
        };
    }
}

fn rrect(painter: &Painter, rect: Rect, cr: impl Into<CornerRadiusF32>, color: Color32) {
    let mut points = vec![];
    rounded_rectangle(&mut points, rect, cr.into());
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

enum Draw {
    Line { min: Vec2, max: Vec2 },
    Hline { min_x: f32, max_x: f32, y: f32 },
    Vline { x: f32, min_y: f32, max_y: f32 },
    RRectFill { min: Vec2, max: Vec2, cr: f32 },
    RRectStroke { min: Vec2, max: Vec2, cr: f32 },
    CircleStroke { center: Vec2, radius: f32 },
    StrokeQuadrant { center: Vec2, radius: f32, q: f32 },
}

struct Paint<'a> {
    painter: &'a Painter,
    center: Pos2,
}

impl Paint<'_> {
    fn paint(
        &mut self,
        cmd: impl IntoIterator<Item = Draw>,
        fill: Color32,
        stroke: (f32, Color32),
    ) {
        for cmd in cmd.into_iter() {
            match cmd {
                Draw::Line { min, max } => self.line([min, max], stroke),
                Draw::Hline { min_x, max_x, y } => self.hline(min_x, max_x, y, stroke),
                Draw::Vline { x, min_y, max_y } => self.vline(x, min_y, max_y, stroke),
                Draw::RRectFill { min, max, cr } => self.fill(min, max, cr, fill),
                Draw::RRectStroke { min, max, cr } => self.stroke(min, max, cr, stroke),
                Draw::CircleStroke { center, radius } => {
                    self.circle(center, radius, Color32::TRANSPARENT, stroke)
                }
                Draw::StrokeQuadrant { center, radius, q } => {
                    self.circle_quadrant(center, radius, q, stroke)
                }
            }
        }
    }

    fn circle_quadrant(
        &self,
        center: Vec2,
        radius: f32,
        quadrant: f32,
        stroke: impl Into<PathStroke>,
    ) {
        let mut points = vec![];
        add_circle_quadrant(&mut points, self.center + center, radius, quadrant);
        self.painter.add(Shape::line(points, stroke));
    }

    fn circle(&self, center: Vec2, radius: f32, fill: Color32, stroke: impl Into<Stroke>) {
        self.painter
            .circle(self.center + center, radius, fill, stroke);
    }

    fn line<const N: usize>(&self, points: [Vec2; N], stroke: impl Into<PathStroke>) {
        let points = points.map(|v| self.center + v);
        self.painter.line(points.to_vec(), stroke);
    }

    fn hline(&self, x_start: f32, x_end: f32, y: f32, stroke: impl Into<PathStroke>) {
        let points = [vec2(x_start.min(x_end), y), vec2(x_start.max(x_end), y)];
        self.line(points, stroke);
    }
    fn vline(&self, x: f32, y_start: f32, y_end: f32, stroke: impl Into<PathStroke>) {
        let points = [vec2(x, y_start.min(y_end)), vec2(x, y_start.max(y_end))];
        self.line(points, stroke);
    }

    fn fill(&self, min: Vec2, max: Vec2, cr: impl Into<CornerRadiusF32>, fill: Color32) {
        self.rrect(min, max, cr.into(), fill, Stroke::NONE);
    }

    fn stroke(
        &self,
        min: Vec2,
        max: Vec2,
        cr: impl Into<CornerRadiusF32>,
        stroke: impl Into<PathStroke>,
    ) {
        self.rrect(min, max, cr.into(), Color32::TRANSPARENT, stroke);
    }

    fn rrect(
        &self,
        min: Vec2,
        max: Vec2,
        cr: CornerRadiusF32,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) {
        let rect = Rect::from_min_max(self.center + min, self.center + max);
        let mut points = vec![];
        rounded_rectangle(&mut points, rect, cr);
        self.painter
            .add(Shape::convex_polygon(points, fill, stroke));
    }
}
