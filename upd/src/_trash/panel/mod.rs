use crate::{EditorPanel, Style};
use bevy::ecs::{query::QueryFilter, system::SystemState};
use bevy::prelude::*;
use bevy_egui::egui;

mod placeholder;

pub use self::placeholder::PlaceholderTab;

#[must_use]
pub fn start_panel<F: QueryFilter>(world: &mut World) -> Option<(Entity, egui::Ui)> {
    let (entity, viewport) = world
        .query_filtered::<(Entity, &EditorPanel), F>()
        .iter(world)
        .find_map(|(entity, panel)| panel.viewport.map(|rect| (entity, rect)))?;

    let mut system_state: SystemState<bevy_egui::EguiContexts> = SystemState::new(world);
    let ctx = system_state.get_mut(world).ctx_mut().ok()?.clone();

    let layer_id = egui::LayerId::background();
    let id = egui::Id::new(entity);
    let ui_builder = egui::UiBuilder::new().layer_id(layer_id).max_rect(viewport);

    Some((entity, egui::Ui::new(ctx, id, ui_builder)))
}

pub(crate) fn run_panel_scrollbar<F: QueryFilter + 'static>(
    world: &mut World,
    contents: impl FnOnce(Entity, &mut World, &mut egui::Ui),
) {
    let Some((entity, mut ui)) = start_panel::<F>(world) else {
        return;
    };

    let style = world.resource::<Style>();

    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, 0.0, style.panel_fill);

    let style = Style::default();
    style.set_theme_visuals(&mut ui);
    style.for_scrollbar(&mut ui);

    let _output = egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_width(rect.width())
        .id_salt((entity, std::any::TypeId::of::<F>()))
        .show(&mut ui, |ui| {
            style.set_theme_visuals(ui);
            contents(entity, world, ui)
        });
}

/*
pub(crate) fn run_panel<F: ReadOnlyWorldQuery + 'static>(
    world: &mut World,
    contents: impl FnOnce(Entity, &mut World, &mut egui::Ui),
) {
    let Some((entity, mut ui)) = start_panel::<F>(world) else {
        return;
    };

    let style = world.resource::<crate::ui::Style>();

    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, 0.0, style.panel_fill);

    let style = crate::ui::Style::default();
    style.set_theme_visuals(&mut ui);
    contents(entity, world, &mut ui)
}
*/
