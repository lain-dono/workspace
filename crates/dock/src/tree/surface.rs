use super::Tree;
use egui::{self, Id, Pos2, Rect, Vec2};

/// A [`Surface`] is the highest level component in a [`DockState`](crate::DockState). [`Surface`]s represent an area
/// in which nodes are placed.
///
/// Typically, you're only using one surface, which is the main surface. However, if you drag
/// a tab out in a way which creates a window, you also create a new surface in which nodes can appear.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Surface<Tab> {
    /// An empty surface, with nothing inside (practically, a null surface).
    Empty,

    /// The main surface of a [`DockState`](crate::DockState), only one should exist at surface index 0 at any one time.
    Main(Tree<Tab>),

    /// A windowed surface with a state.
    Window(Tree<Tab>, WindowState),
}

/// The state of a [`Surface::Window`](crate::Surface::Window).
///
/// Doubles as a handle for the surface, allowing the user to set its size and position.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WindowState {
    /// The [`Rect`] that this window was last taking up.
    pub(crate) screen_rect: Option<Rect>,

    /// Was this window dragged in the last frame?
    pub(crate) dragged: bool,

    /// The next position this window should be set to next frame.
    pub(crate) next_position: Option<Pos2>,

    /// The next size this window should be set to next frame.
    pub(crate) next_size: Option<Vec2>,

    /// The height of the window before it was fully collapsed
    pub(crate) expanded_height: Option<f32>,

    /// True the first frame this window is drawn.
    /// handles expanding after being fully collapsed, etc.
    pub(crate) new: bool,

    /// True if the window is minimized
    pub(crate) minimized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            screen_rect: None,
            dragged: false,
            next_position: None,
            next_size: None,
            expanded_height: None,
            new: true,
            minimized: false,
        }
    }
}

impl WindowState {
    /// Set the position for this window in screen coordinates.
    pub fn set_position(&mut self, position: Pos2) -> &mut Self {
        self.next_position = Some(position);
        self
    }

    /// Set the size of this window in egui points.
    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.next_size = Some(size);
        self
    }

    /// Get the [`Rect`] which this window occupies.
    /// If this window hasn't been shown before, this will be [`Rect::NOTHING`].
    pub fn rect(&self) -> Rect {
        // The reason why we're unwrapping an Option with a default value instead of
        // just storing Rect::NOTHING for the None variant is that deserializing Rect::NOTHING
        // with serde_json causes a panic, because f32::INFINITY serializes into null in JSON.
        self.screen_rect.unwrap_or(Rect::NOTHING)
    }

    /// Returns if this window is currently being dragged or not.
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    #[inline(always)]
    pub(crate) fn toggle_minimized(&mut self) {
        self.minimized = !self.minimized;
    }

    //the 'static in this case means that the `open` field is always `None`
    pub(crate) fn create_window(&mut self, id: Id, bounds: Rect) -> egui::Window<'static> {
        let mut window = egui::Window::new("").title_bar(false);
        window = window.id(id).constrain_to(bounds);

        if let Some(position) = self.next_position.take() {
            window = window.current_pos(position);
        }
        if let Some(size) = self.next_size.take() {
            window = window.fixed_size(size);
        }

        // Reset the height of the window if it is now expanded
        let new = self.new;
        if new && let Some(height) = self.expanded_height {
            window = window.max_height(height).min_height(height);
        }
        self.new = false;

        window
    }
}
