/// Groups together labels from different elements of the [`DockArea`](crate::DockArea).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Translations {
    // Specifies text in buttons displayed in the context menu displayed upon right-clicking on a tab.
    // Text overrides for buttons in tab context menus.
    //
    /// Button that closes the tab.
    pub tab_context_menu_close_button: String,
    // Specifies text in buttons displayed in the context menu displayed upon right-clicking on a tab.
    // Text overrides for buttons in tab context menus.
    //
    /// Button that undocks the tab into a new window.
    pub tab_context_menu_eject_button: String,

    /// Message in the tooltip shown while hovering over a grayed out X button of a leaf
    /// containing non-closable tabs.
    pub leaf_close_button_disabled_tooltip: String,
    /// Button that closes the entire window.
    pub leaf_close_all_button: String,
    /// Message in the tooltip shown while hovering over an X button of a window.
    /// Used when the secondary buttons are accessible from the context menu.
    pub leaf_close_all_button_menu_hint: String,
    /// Message in the tooltip shown while hovering over an X button of a window.
    /// Used when the secondary buttons are accessible using modifiers.
    pub leaf_close_all_button_modifier_hint: String,
    /// Message in the tooltip shown while hovering over an X button of a window.
    /// Used when the secondary buttons are accessible using modifiers and from the context menu.
    pub leaf_close_all_button_modifier_menu_hint: String,
    /// Message in the tooltip shown while hovering over a grayed out close window button of a window
    /// containing non-closable tabs.
    pub leaf_close_all_button_disabled_tooltip: String,
    /// Button that minimizes the window.
    pub leaf_minimize_button: String,
    /// Message in the tooltip shown while hovering over a collapse button of a leaf.
    /// Used when the secondary buttons are accessible from the context menu.
    pub leaf_minimize_button_menu_hint: String,
    /// Message in the tooltip shown while hovering over a collapse button of a leaf.
    /// Used when the secondary buttons are accessible using modifiers.
    pub leaf_minimize_button_modifier_hint: String,
    /// Message in the tooltip shown while hovering over a collapse button of a leaf.
    /// Used when the secondary buttons are accessible using modifiers and from the context menu.
    pub leaf_minimize_button_modifier_menu_hint: String,
}

impl Translations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            tab_context_menu_close_button: String::from("Close"),
            tab_context_menu_eject_button: String::from("Eject"),
            leaf_close_button_disabled_tooltip: String::from(
                "This leaf contains non-closable tabs.",
            ),
            leaf_close_all_button: String::from("Close window"),
            leaf_close_all_button_menu_hint: String::from("Right click to close this window."),
            leaf_close_all_button_modifier_hint: String::from(
                "Press modifier keys (Shift by default) to close this window.",
            ),
            leaf_close_all_button_modifier_menu_hint: String::from(
                "Press modifier keys (Shift by default) or right click to close this window.",
            ),
            leaf_close_all_button_disabled_tooltip: String::from(
                "This window contains non-closable tabs.",
            ),
            leaf_minimize_button: String::from("Minimize window"),
            leaf_minimize_button_menu_hint: String::from("Right click to minimize this window."),
            leaf_minimize_button_modifier_hint: String::from(
                "Press modifier keys (Shift by default) to minimize this window.",
            ),
            leaf_minimize_button_modifier_menu_hint: String::from(
                "Press modifier keys (Shift by default) or right click to minimize this window.",
            ),
        }
    }
}
