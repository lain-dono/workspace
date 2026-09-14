use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;

pub mod hierarchy;
pub mod outliner;
pub mod placeholder;
pub mod scene;
pub mod view_style;
// pub mod workspace;

pub use self::hierarchy::HierarchyTab;
pub use self::outliner::OutlinerTab;
pub use self::placeholder::PlaceholderTab;
pub use self::scene::SceneTab;
pub use self::view_style::ViewStyleTab;
// pub use self::workspace::WorkspaceTab;

#[must_use]
pub fn start_panel<F: ReadOnlyWorldQuery>(world: &mut World) -> Option<(Entity, egui::Ui)> {
    let mut context = world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
    let mut context = context.get_single(world).cloned().ok()?;

    let (entity, viewport) = world
        .query_filtered::<(Entity, &crate::ui::EditorPanel), F>()
        .iter(world)
        .find_map(|(entity, panel)| panel.viewport.map(|rect| (entity, rect)))?;

    let ctx = context.get_mut().clone();
    let layer_id = egui::LayerId::background();
    let id = egui::Id::new(entity);

    Some((entity, egui::Ui::new(ctx, layer_id, id, viewport, viewport)))
}

pub(crate) fn run_panel_scrollbar<F: ReadOnlyWorldQuery + 'static>(
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
    style.for_scrollbar(&mut ui);

    let _output = egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_width(rect.width())
        .id_source((entity, std::any::TypeId::of::<F>()))
        .show(&mut ui, |ui| {
            style.set_theme_visuals(ui);
            contents(entity, world, ui)
        });
}

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
