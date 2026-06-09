use super::icon::Icon;
use bevy_egui::egui::{
    self, CursorIcon, Direction, Layout, Response, RichText, Sense, TextStyle, UiBuilder,
    WidgetText,
    ecolor::Color32,
    emath::{
        Align, Align2, NumExt, Rect, Vec2, format_with_decimals_in_range, pos2, round_to_decimals,
        smart_aim, vec2,
    },
    epaint::{CornerRadius, Margin, Stroke, TextShape},
};

pub const FILL: f32 = f32::INFINITY;

pub const PANEL_BG: Color32 = Color32::from_gray(0x2c);
pub const CANVAS_BG: Color32 = Color32::from_gray(0x1e);

pub const BORDER: Color32 = Color32::from_gray(0x44);

pub const BG_SECONDARY: Color32 = Color32::from_gray(0x38);
pub const BG_SECONDARY_ACTIVE: Color32 = Color32::from_gray(0x42);

pub const CHEVRON_COLOR: Color32 = Color32::from_gray(0x80);

pub const TEXT_COLOR: Color32 = Color32::from_gray(0xFF);
pub const TEXT_COLOR_SECONDARY: Color32 =
    Color32::from_rgba_unmultiplied_const(0xFF, 0xFF, 0xFF, 0xb3);

pub const RADIUS_MEDIUM: u8 = 5; // 0.3125

pub const SELECT_ROOT: Color32 = Color32::from_rgb(0x4a, 0x58, 0x78);
pub const SELECT_GAP: Color32 = Color32::from_rgb(0x39, 0x43, 0x60);
pub const SELECT_HOVER: Color32 = Color32::from_rgb(0x53, 0x63, 0x83);

pub struct Ui {
    pub ui: egui::Ui,
}

impl Ui {
    pub fn new(ui: &mut egui::Ui, salt: impl std::hash::Hash) -> Self {
        let layout = Layout::top_down_justified(Align::Min);

        let rect = ui.available_rect_before_wrap();
        let id = ui.id().with(salt);
        let ui = ui.new_child(UiBuilder::new().max_rect(rect).layout(layout).id(id));
        Self { ui }
    }
}

impl Ui {
    pub fn separator(&mut self) {
        self.ui.add(hline_ui(0.0, 0.0, (1.0, BORDER)));
    }

    pub fn label(&mut self, text: impl Into<String>) {
        let text = egui::RichText::new(text).color(TEXT_COLOR);
        self.ui.add(egui::Label::new(text));
    }

    pub fn edit(&mut self, icon: Icon, value: &mut f32) {
        self.edit_value(icon, value, 0, Some(2));
    }

    pub fn edit_value(
        &mut self,
        icon: Icon,
        value: &mut f32,
        min_decimals: usize,
        max_decimals: Option<usize>,
    ) {
        let old_value = *value;

        let speed: f64 = 1.0;
        let range = f64::NEG_INFINITY..=f64::INFINITY;

        self.fill(4, 4, 4, 4, BG_SECONDARY);

        let mut value_text = String::new();

        let id = self.ui.next_auto_id();

        let edit = Flow::symmetric(0, 0, 0, 24.0, [24.0, FILL]);
        edit.show(&mut self.ui, |[mut icon_ui, mut text_ui]| {
            {
                let ui = &mut icon_ui.ui;

                let aim_radius = ui.input(|i| i.aim_radius() as f64);

                let auto_decimals = aim_radius / speed.abs();
                let auto_decimals = auto_decimals.log10().ceil().clamp(0.0, 15.0) as usize;

                let max_decimals = max_decimals.unwrap_or(auto_decimals + 2);
                let auto_decimals =
                    auto_decimals.clamp(min_decimals, max_decimals.at_least(min_decimals));

                value_text =
                    format_with_decimals_in_range(*value as f64, auto_decimals..=max_decimals);

                icon.paint(ui, TEXT_COLOR);
                let rect = ui.available_rect_before_wrap();

                let cursor_icon = CursorIcon::ResizeHorizontal;

                let response = ui.interact(rect, id, Sense::drag());
                let response = response.on_hover_cursor(cursor_icon);

                if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
                    // Reset memory of preciely dagged value.
                    ui.data_mut(|data| data.remove::<f64>(id));
                }

                if response.dragged() {
                    ui.ctx().set_cursor_icon(cursor_icon);

                    let mdelta = response.drag_delta();
                    let delta = mdelta.x - mdelta.y; // Increase to the right and up
                    let delta_value = delta as f64 * speed;

                    if delta_value.abs() > f64::EPSILON {
                        // Since we round the value being dragged, we need to store the full precision value in memory:
                        let precise_value = ui.data_mut(|data| data.get_temp::<f64>(id));
                        let precise_value = precise_value.unwrap_or(*value as f64) + delta_value;

                        let aim_delta = aim_radius * speed;
                        let rounded_new_value = smart_aim::best_in_range_f64(
                            precise_value - aim_delta,
                            precise_value + aim_delta,
                        );

                        // Dragging will always clamp the value to the range.
                        let rounded_new_value = clamp_value_to_range(
                            round_to_decimals(rounded_new_value, auto_decimals),
                            range.clone(),
                        );

                        // set(&mut get_set_value, rounded_new_value);
                        *value = rounded_new_value as f32;
                        value_text = value.to_string();

                        ui.data_mut(|data| data.insert_temp::<f64>(id, precise_value));
                    }
                }
            }

            text_ui.text_edit(&mut value_text);
        });

        if let Ok(new_value) = value_text.parse::<f32>() {
            *value = new_value;
        }
    }

    pub fn text_edit(&mut self, text: &mut dyn egui::TextBuffer) {
        let text = egui::TextEdit::singleline(text)
            .horizontal_align(Align::Min)
            .vertical_align(Align::Center)
            .frame(false)
            .text_color(TEXT_COLOR);
        self.ui.add(text);
    }

    pub fn fill(&self, nw: u8, ne: u8, sw: u8, se: u8, color: Color32) {
        let corner_radius = CornerRadius { nw, ne, sw, se };
        let rect = self.ui.available_rect_before_wrap();
        self.ui.painter().rect_filled(rect, corner_radius, color);
    }

    pub fn header<const N: usize>(&mut self, children: [f32; N], callback: impl FnOnce([Ui; N])) {
        let line = Flow::symmetric(8, 8, 4, 40.0, children);
        self.line(line, callback);
    }

    pub fn line<const N: usize>(&mut self, line: Flow<N>, callback: impl FnOnce([Ui; N])) {
        line.show(&mut self.ui, callback);
    }

    pub fn btn_icon(&mut self, icon: Icon, style: BtnIcon) -> Response {
        self.ui.add(style.widget(icon))
    }

    pub fn btn_title(&mut self, active: bool, frame: bool, text: impl Into<String>) -> Response {
        let style = BtnText::new(frame, Align::LEFT, 0.0, "title");
        self.ui.add(style.widget(active, text))
    }

    pub fn btn_header(&mut self, active: bool, frame: bool, text: impl Into<String>) -> Response {
        let style = BtnText::new(frame, Align::LEFT, 8.0, "title");
        self.ui.add(style.widget(active, text))
    }

    pub fn btn_center(&mut self, active: bool, frame: bool, text: impl Into<String>) -> Response {
        let style = BtnText::new(frame, Align::Center, 8.0, "regular");
        self.ui.add(style.widget(active, text))
    }

    pub fn btn_left(&mut self, active: bool, frame: bool, text: impl Into<String>) -> Response {
        let style = BtnText::new(frame, Align::LEFT, 8.0, "regular");
        self.ui.add(style.widget(active, text))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BtnIcon {
    radius: egui::CornerRadius,
    frame: bool,
}

impl BtnIcon {
    pub const FRAMELESS: Self = Self {
        radius: egui::CornerRadius {
            nw: 4,
            ne: 4,
            sw: 4,
            se: 4,
        },
        frame: false,
    };

    pub const LEFT: Self = Self {
        radius: egui::CornerRadius {
            nw: 4,
            ne: 0,
            sw: 4,
            se: 0,
        },
        frame: true,
    };

    pub const RIGHT: Self = Self {
        radius: egui::CornerRadius {
            nw: 0,
            ne: 4,
            sw: 0,
            se: 4,
        },
        frame: true,
    };

    pub const CENTER: Self = Self {
        radius: egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: 0,
            se: 0,
        },
        frame: true,
    };

    fn widget(self, icon: Icon) -> impl FnOnce(&mut egui::Ui) -> Response {
        move |ui| {
            let size = ui.available_size_before_wrap();
            let (rect, response) = ui.allocate_exact_size(size, Sense::click());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);

            let touch = response.is_pointer_button_down_on();
            let hover = response.hovered();

            let fill_color = if self.frame && hover || touch {
                BG_SECONDARY_ACTIVE
            } else if self.frame || hover {
                BG_SECONDARY
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, self.radius, fill_color);
            icon.paint_at(ui.painter(), rect, TEXT_COLOR);
            response
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BtnText {
    pub frame: bool,
    pub align: Align,
    pub spacing: f32,
    pub font: &'static str,
}

impl BtnText {
    const fn new(frame: bool, align: Align, spacing: f32, font: &'static str) -> Self {
        Self {
            frame,
            align,
            spacing,
            font,
        }
    }

    fn widget(
        self,
        active: bool,
        text: impl Into<String>,
    ) -> impl FnOnce(&mut egui::Ui) -> Response {
        let style = self;
        let text = text.into();
        move |ui| {
            let text_style = egui::TextStyle::Name(style.font.into());
            // let line_height = Some(16.0);
            // let height = 24.0;

            let height = ui.available_height().min(24.0);
            let corner_radius = CornerRadius::same(RADIUS_MEDIUM);

            let id = ui.id().with(&text); //Id::new(&text);

            // let label = RichText::new(text).size(font_size).family(font_family);
            let label = RichText::new(text).text_style(text_style);
            let label = WidgetText::from(label);
            let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

            let width = galley.size().x + 2.0 * style.spacing;

            let (_, rect) = ui.allocate_space(vec2(width, height));
            let response = ui.interact(rect, id, Sense::hover());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);

            let (fg_color, bg_color) = match (active, response.hovered()) {
                (true, _) => (TEXT_COLOR, BG_SECONDARY),
                (_, true) => (TEXT_COLOR_SECONDARY, BG_SECONDARY),
                (_, false) => (TEXT_COLOR_SECONDARY, Color32::TRANSPARENT),
            };

            let text_size = galley.size();
            let text_frame = rect.shrink2(vec2(style.spacing, 0.0));
            let text_pos = match style.align {
                Align::Min => {
                    Align2::LEFT_CENTER.pos_in_rect(&text_frame) - vec2(0.0, text_size.y) * 0.5
                }
                Align::Center => Align2::CENTER_CENTER.pos_in_rect(&text_frame) - text_size * 0.5,
                Align::Max => todo!(),
            };

            let painter = ui.painter();

            if style.frame {
                painter.rect_filled(rect, corner_radius, bg_color);
            }
            painter.add(TextShape::new(text_pos, galley, fg_color));

            response
        }
    }
}

fn horizontal(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    let initial_size = vec2(
        ui.available_size_before_wrap().x,
        ui.spacing().interact_size.y, // Assume there will be something interactive on the horizontal layout
    );

    let layout = Layout::left_to_right(Align::Center).with_main_wrap(false);
    ui.allocate_ui_with_layout(initial_size, layout, content);
}

#[derive(Clone, Copy)]
pub struct Flow<const N: usize> {
    pub padding: Margin,
    pub gap: f32,
    pub size: f32,
    pub children: [f32; N],
    pub child_layout: Layout,
    pub axis: Axis,
}

impl<const N: usize> Flow<N> {
    pub const fn new(padding: Margin, gap: i8, size: f32, children: [f32; N]) -> Self {
        Self {
            padding,
            gap: gap as f32,
            size,
            children,
            child_layout: Layout {
                main_dir: Direction::LeftToRight,
                main_wrap: false,
                main_align: Align::Min,
                main_justify: true,
                cross_align: Align::Center,
                cross_justify: true,
            },
            axis: Axis::X,
        }
    }

    pub const fn symmetric(x: i8, y: i8, gap: i8, size: f32, children: [f32; N]) -> Self {
        let padding = Margin::symmetric(x, y);
        Self::new(padding, gap, size, children)
    }

    pub fn show(&self, ui: &mut egui::Ui, callback: impl FnOnce([Ui; N])) {
        callback(self.alloc(ui).map(|rect| {
            let ui_builder = UiBuilder::new().max_rect(rect);
            let ui = ui.new_child(ui_builder.layout(self.child_layout));
            Ui { ui }
        }))
    }

    pub fn alloc(&self, ui: &mut egui::Ui) -> [Rect; N] {
        let size = match self.axis {
            Axis::X => vec2(ui.available_width(), self.size),
            Axis::Y => vec2(self.size, ui.available_height()),
        };

        let (_id, frame) = ui.allocate_space(size);
        let Rect { min, max } = frame - self.padding;

        match self.axis {
            Axis::X => flow(min.x, self.gap, max.x, self.children).map(|(min_x, max_x)| Rect {
                min: pos2(min_x.min(max_x), min.y.min(max.y)),
                max: pos2(min_x.max(max_x), min.y.max(max.y)),
            }),
            Axis::Y => flow(min.y, self.gap, max.y, self.children).map(|(min_y, max_y)| Rect {
                min: pos2(min.x.min(max.x), min_y.min(max_y)),
                max: pos2(min.x.max(max.x), min_y.max(max_y)),
            }),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
}

fn flow<const N: usize, I>(mut min: f32, gap: f32, max: f32, children: I) -> [(f32, f32); N]
where
    I: IntoIterator<Item = f32> + Copy,
{
    let allocated = children.into_iter().filter(|c| c.is_finite()).sum::<f32>();
    let available = (max - min) - allocated - gap * N.saturating_sub(1) as f32;
    let fraction = available / children.into_iter().filter(|c| !c.is_finite()).count() as f32;
    let mut output = [(0.0, 0.0); N];
    for (width, range) in children.into_iter().zip(&mut output) {
        *range = (min, min + if width.is_finite() { width } else { fraction });
        min = range.1 + gap;
    }
    output
}

fn hline_ui(
    spacing: f32,
    grow: f32,
    stroke: impl Into<Stroke>,
) -> impl Fn(&mut egui::Ui) -> egui::Response {
    let stroke = stroke.into();
    move |ui| -> Response {
        let available_space = if ui.is_sizing_pass() {
            Vec2::ZERO
        } else {
            ui.available_size_before_wrap()
        };

        let size = vec2(available_space.x, spacing);
        let (rect, response) = ui.allocate_at_least(size, Sense::hover());
        if ui.is_rect_visible(response.rect) {
            let x = (rect.left() - grow)..=(rect.right() + grow);
            ui.painter().hline(x, rect.center().y, stroke);
        }
        response
    }
}

/// Clamp the given value with careful handling of negative zero, and other corner cases.
pub(crate) fn clamp_value_to_range(x: f64, range: std::ops::RangeInclusive<f64>) -> f64 {
    use std::cmp::Ordering;

    let (mut min, mut max) = (*range.start(), *range.end());

    if min.total_cmp(&max) == Ordering::Greater {
        (min, max) = (max, min);
    }

    match x.total_cmp(&min) {
        Ordering::Less | Ordering::Equal => min,
        Ordering::Greater => match x.total_cmp(&max) {
            Ordering::Greater | Ordering::Equal => max,
            Ordering::Less => x,
        },
    }
}
