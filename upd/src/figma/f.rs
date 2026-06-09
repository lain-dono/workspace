pub struct Frame {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Frame {
    pub fn dx(&self) -> f32 {
        self.max[0] - self.min[0]
    }

    pub fn dy(&self) -> f32 {
        self.max[1] - self.min[1]
    }
}

pub struct Padding {
    pub x: [f32; 2],
    pub y: [f32; 2],
}

pub enum Direction {
    X,
    Y,
}

/*
pub enum Constraint {
    Start,
    Center,
    End,
    Stretch,
    Scale,
}
*/

#[derive(Clone, Copy)]
pub struct Size {
    pub x: f32,
    pub y: f32,
}

impl Size {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn min(self, min: Self) -> Self {
        Self {
            x: self.x.min(min.x),
            y: self.x.min(min.y),
        }
    }

    #[must_use]
    pub fn max(self, max: Self) -> Self {
        Self {
            x: self.x.max(max.x),
            y: self.x.max(max.y),
        }
    }

    #[must_use]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.x.clamp(min.y, max.y),
        }
    }
}

pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub fn constrain(&self, size: impl Into<Size>) -> Size {
        size.into().clamp(self.min, self.max)
    }

    pub fn constrain_x(&self, x: f32) -> f32 {
        x.clamp(self.min.x, self.max.x)
    }

    pub fn constrain_y(&self, y: f32) -> f32 {
        y.clamp(self.min.y, self.max.y)
    }
}

pub enum Space {
    Auto,
    Between(f32),
}

pub struct AutoLayout {
    pub direction: Direction,
    pub padding: [f32; 4],
    pub primary_align: f32,
    pub spacing: f32,
    pub hug_content: f32,
}

impl AutoLayout {
    pub fn measure(&self, constraints: BoxConstraints, children: &[Child]) -> Size {
        fn measure_child<'a>(
            iter: impl Iterator<Item = &'a Child>,
            map: impl Fn(&'a Child) -> (Option<f32>, Option<f32>),
        ) -> (f32, f32, usize) {
            iter.map(map).fold(
                (0.0, 0.0, 0),
                |(acc_main, acc_cross, flex), (main, cross)| {
                    (
                        acc_main + main.unwrap_or(0.0),
                        acc_cross + cross.unwrap_or(0.0),
                        flex + main.is_some() as usize,
                    )
                },
            )
        }

        let max_main_size = match self.direction {
            Direction::X => constraints.max.x,
            Direction::Y => constraints.max.y,
        };

        let can_flex = max_main_size < f32::INFINITY;

        let mut total_flex = 0;
        let mut cross_size = 0.0;
        let mut allocated_size = 0.0;

        let mut last_flex_child = None;

        for (index, child) in children.iter().enumerate() {
            let flex = false;
            if flex {
                total_flex += 1;
                last_flex_child = Some(index);
            } else {
                let size = child.measure(BoxConstraints {
                    min: Size::new(0.0, 0.0),
                    max: match self.direction {
                        Direction::X => Size::new(f32::INFINITY, constraints.max.y),
                        Direction::Y => Size::new(constraints.max.x, f32::INFINITY),
                    },
                });
                let (main, cross) = match self.direction {
                    Direction::X => (size.x, size.y),
                    Direction::Y => (size.y, size.x),
                };
                allocated_size += main;
                cross_size = f32::max(cross_size, cross);
            }
        }

        let free_space = (if can_flex { max_main_size } else { 0.0 } - allocated_size).max(0.0);
        let mut allocated_flex_space = 0.0;

        /*
        let (main, cross, flex) = match self.axis {
            Direction::X => measure_child(children.iter(), |child| {
                (child.x_size.map(f32::abs), child.y_size.map(f32::abs))
            }),
            Direction::Y => measure_child(children.iter(), |child| {
                (child.y_size.map(f32::abs), child.x_size.map(f32::abs))
            }),
        };
        */

        /*
        match self.axis {
            Axis::X => Size {
                x: constraints.constrain_x(if flex { f32::INFINITY } else { allocated }),
            },
            Axis::Y => Size {
                y: constraints.constrain_y(if flex { f32::INFINITY } else { allocated }),
            },
        }
        */

        todo!()
    }

    /*
    pub fn layout(&self, constraints: Constraints, children: &mut [Child]) {

        // measure

        let (dx, dy) = (frame.dx(), frame.dy());
        let (x_size, x_reverse) = (dx.abs(), dx.is_sign_negative());
        let (y_size, y_reverse) = (dy.abs(), dy.is_sign_negative());
    }
    */
}

pub struct Child {
    pub size: Size,
    pub frame: Frame,

    pub x: Constraint,
    pub y: Constraint,
}

impl Child {
    fn measure(&self, constraints: BoxConstraints) -> Size {
        constraints.constrain(self.size)
    }
}

#[derive(Clone, Copy)]
pub enum Constraint {
    Sized { align: f32, size: f32 },
    Flexible { offset: (f32, f32) },
    //HugContent { align: f32 },
}

impl Constraint {
    pub const fn fill() -> Self {
        Self::Flexible { offset: (0.0, 0.0) }
    }

    pub const fn start(size: f32) -> Self {
        Self::Sized { align: -1.0, size }
    }

    pub const fn end(size: f32) -> Self {
        Self::Sized { align: 1.0, size }
    }

    pub const fn center(size: f32) -> Self {
        Self::Sized { align: 0.0, size }
    }

    pub fn perform(self, container: f32) -> (f32, f32) {
        match self {
            Self::Sized { align, size } => {
                let delta = (container - size) * 0.5;
                let aligned = align * delta;
                (aligned + delta, aligned - delta)
            }
            Self::Flexible { offset: (min, max) } => (min, -max),
        }
    }
}

pub struct Layout {
    pub direction: Direction,
    pub align_content: f32,
    pub spacing: f32,

    pub padding_x: [f32; 2],
    pub padding_y: [f32; 2],
    pub constraint_x: Constraint,
    pub constraint_y: Constraint,
}

pub struct Graphics {
    pub fills: Vec<Fill>,
    pub strokes: Vec<Stroke>,
}

pub struct Fill {
    pub color: Color,
}

pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub offset: f32, // -1 inside, 0 center, +1 outside
}

pub enum Effect {
    InnerShadow(Shadow),
    DrowShadow(Shadow),
    LayerBlur(f32),
    BackgroundBlur(f32),
}

pub struct Shadow {
    pub color: Color,
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
}

pub type Color = [f32; 4];

pub enum Interaction {
    Click, // on click
    Drag,  // on drag
    Hover, // while hovering
    Press, // while pressing

    Key, // on key
    Mouse { event: Mouse, delay: Option<f32> },
}

pub enum Mouse {
    Enter,
    Up,
    Down,
    Leave,
}
