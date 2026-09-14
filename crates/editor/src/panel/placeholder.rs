use crate::ui::EditorStage;
use bevy::prelude::*;

#[derive(Default, Component)]
pub struct PlaceholderTab;

impl PlaceholderTab {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        super::run_panel_scrollbar::<With<Self>>(world, |_entity, world, ui| {
            let style = world.resource::<crate::ui::Style>();
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 0.0, style.panel_fill);
        });
    }
}
