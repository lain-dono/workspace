use crate::ui::EditorStage;
use crate::workspace::Workspace;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;

#[derive(Default, Component)]
pub struct WorkspaceTab(pub Workspace);

impl WorkspaceTab {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn panel(world: &mut World) {
        let mut context = world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
        let Ok(mut context) = context.get_single(world).cloned() else {
            return;
        };

        let Some((_entity, viewport, mut this)) = world
            .query::<(Entity, &crate::ui::EditorPanel, &mut Self)>()
            .iter_mut(world)
            .find_map(|(entity, panel, this)| panel.viewport.map(|rect| (entity, rect, this)))
        else {
            return;
        };

        let ctx = context.get_mut();
        this.0.draw(ctx, viewport);
    }
}
