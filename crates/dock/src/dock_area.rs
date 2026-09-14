use super::{
    style::Style,
    translations::Translations,
    tree::{DockState, NodeIndex, SurfaceIndex, TabIndex, TabRemoval},
};
use egui::{Id, Modifiers, emath::Rect};

pub struct DockAreaConfig {
    /// Shows or hides the add button popup.
    /// By default it's `false`.
    pub show_add_popup: bool,

    /// Shows or hides the tab add buttons.
    /// By default it's `false`.
    pub show_add_buttons: bool,

    /// Shows or hides the tab close buttons.
    /// By default it's `true`.
    pub show_close_buttons: bool,

    /// Whether tabs show a context menu when right-clicked.
    /// By default it's `true`.
    pub tab_context_menus: bool,

    /// Whether tabs can be dragged between nodes and reordered on the tab bar.
    /// By default it's `true`.
    pub draggable_tabs: bool,

    /// Enables or disables the close all tabs button on tab bars.
    /// By default it's `true`.
    pub show_leaf_close_all_buttons: bool,

    /// Enables or disables the collapse tabs button on tab bars.
    /// By default it's `true`.
    pub show_leaf_collapse_buttons: bool,

    /// Whether tooltip hints are shown for secondary buttons on tab bars.
    /// By default it's `true`.
    pub show_secondary_button_hint: bool,

    /// The key combination used to activate secondary buttons on tab bars.
    /// By default it's [`Modifiers::SHIFT`].
    pub secondary_button_modifiers: Modifiers,

    /// Whether the secondary buttons on tab bars are activated by the modifier key.
    /// By default it's `true`.
    pub secondary_button_on_modifier: bool,

    /// Whether the secondary buttons on tab bars are activated from a context value by right-clicking primary buttons.
    /// By default it's `true`.
    pub secondary_button_context_menu: bool,

    /// What directions can a node be split in: left-right, top-bottom, all, or none.
    /// By default it's all.
    pub allowed_splits: AllowedSplits,

    /// The bounds for any windows inside the [`DockArea`]. Defaults to the screen rect.
    /// By default it's set to [`egui::Context::screen_rect`].
    pub window_bounds: Option<Rect>,
}

impl Default for DockAreaConfig {
    fn default() -> Self {
        Self {
            show_add_popup: false,
            show_add_buttons: false,

            show_close_buttons: true,
            tab_context_menus: true,
            draggable_tabs: true,

            show_leaf_close_all_buttons: true,
            show_leaf_collapse_buttons: true,
            show_secondary_button_hint: true,
            secondary_button_modifiers: Modifiers::SHIFT,
            secondary_button_on_modifier: true,
            secondary_button_context_menu: true,

            allowed_splits: AllowedSplits::default(),
            window_bounds: None,
        }
    }
}

/// What directions can this dock be split in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AllowedSplits {
    /// Don't allow splits at all.
    None = 0b00,

    /// Only allow splits in a vertical directions.
    Vertical = 0b01,

    /// Only allow split in a horizontal directions.
    Horizontal = 0b10,

    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All = 0b11,
}

impl AllowedSplits {
    pub fn filter(self, pass: bool) -> Self {
        if pass { self } else { Self::None }
    }
}

/// Displays a [`DockState`] in `egui`.
pub struct DockArea<'tree, Tab> {
    pub(crate) id: Id,
    pub(crate) state: &'tree mut DockState<Tab>,
    pub(crate) style: Style,
    pub(crate) config: DockAreaConfig,

    /// Contains translations of text shown in [`DockArea`](crate::DockArea).
    pub translations: Translations,

    pub(crate) to_remove: Vec<TabRemoval>,
    pub(crate) to_detach: Vec<(SurfaceIndex, NodeIndex, TabIndex)>,
    pub(crate) new_focused: Option<(SurfaceIndex, NodeIndex)>,
    pub(crate) tab_hover_rect: Option<(Rect, TabIndex)>,
}

// Builder
impl<'tree, Tab> DockArea<'tree, Tab> {
    /// Creates a new [`DockArea`] from the provided [`DockState`].
    #[inline(always)]
    pub fn new(
        state: &'tree mut DockState<Tab>,
        style: Style,
        config: DockAreaConfig,
    ) -> DockArea<'tree, Tab> {
        Self {
            id: Id::new("egui_dock::DockArea"),
            state,
            style,
            config,

            to_remove: Vec::new(),
            to_detach: Vec::new(),
            new_focused: None,
            tab_hover_rect: None,
            translations: Translations::english(),
        }
    }

    /// Sets translations of text later displayed in [`DockArea`](crate::DockArea).
    pub fn with_translations(mut self, translations: Translations) -> Self {
        self.translations = translations;
        self
    }

    /// Sets the [`DockArea`] ID. Useful if you have more than one [`DockArea`].
    #[inline(always)]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

impl<Tab> std::fmt::Debug for DockArea<'_, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockArea").finish_non_exhaustive()
    }
}
