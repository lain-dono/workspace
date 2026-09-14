use super::Style;
use egui::{Align, Layout, Pos2, Rect, Ui, Vec2};

pub struct Canvas {
    pub origin: Vec2,  // screen space
    pub panning: Vec2, // screen space
    pub rect: Rect,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            panning: Vec2::ZERO,
        }
    }
}

impl Canvas {
    #[inline]
    pub fn screen_to_grid(&self, pt: impl Into<Pos2>) -> Pos2 {
        pt.into() - self.origin - self.panning
    }

    #[inline]
    pub fn grid_to_screen(&self, pt: impl Into<Pos2>) -> Pos2 {
        pt.into() + self.origin + self.panning
    }

    /*
    #[inline]
    pub fn grid_to_editor(&self, pt: impl Into<Pos2>) -> Pos2 {
        pt.into() + self.panning
    }

    #[inline]
    pub fn editor_to_grid(&self, pt: impl Into<Pos2>) -> Pos2 {
        pt.into() - self.panning
    }
    */

    #[inline]
    pub fn editor_to_screen(&self, pt: impl Into<Pos2>) -> Pos2 {
        pt.into() + self.origin
    }

    pub fn start(&mut self, ui: &mut Ui) -> Ui {
        let rect = ui.available_rect_before_wrap();
        self.rect = rect;
        self.origin = self.rect.min.to_vec2();

        let screen_rect = ui.ctx().input().screen_rect();

        ui.set_clip_rect(rect.intersect(screen_rect));
        ui.set_min_size(rect.size());

        ui.child_ui(self.rect, Layout::top_down(Align::Min))
    }

    pub fn draw_background(&self, ui: &mut Ui, style: &Style) {
        let fill_color = style.grid_background;
        ui.painter().rect_filled(self.rect, 0.0, fill_color);

        let size = self.rect.size();

        let mut x = self.panning.x.rem_euclid(style.grid_spacing);
        while x < size.x {
            let a = self.editor_to_screen([x, 0.0]);
            let b = self.editor_to_screen([x, size.y]);
            ui.painter().line_segment([a, b], style.grid_stroke);
            x += style.grid_spacing;
        }

        let mut y = self.panning.y.rem_euclid(style.grid_spacing);
        while y < size.y {
            let a = self.editor_to_screen([0.0, y]);
            let b = self.editor_to_screen([size.x, y]);
            ui.painter().line_segment([a, b], style.grid_stroke);
            y += style.grid_spacing;
        }
    }
}
