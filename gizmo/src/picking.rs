use crate::plugin::GizmoTarget;
use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_mod_picking::{
    picking_core::PickingPluginsSettings, prelude::*, selection::SelectionPluginSettings,
};

/// Integrates picking with gizmo and highlighting.
pub fn picking_plugin(app: &mut bevy::prelude::App) {
    app.add_plugins(DefaultPickingPlugins.build())
        .add_plugins(OutlinePlugin)
        .insert_resource(SelectionPluginSettings {
            click_nothing_deselect_all: false,
            ..default()
        })
        .add_systems(PreUpdate, toggle_picking_enabled)
        .add_systems(Update, update_picking);
}

fn toggle_picking_enabled(
    targets: Query<&GizmoTarget>,
    mut settings: ResMut<PickingPluginsSettings>,
) {
    // Picking is disabled when any of the gizmos is focused or active.
    settings.is_enabled = targets.iter().all(|t| !t.is_focused() && !t.is_active());
}

fn update_picking(
    mut commands: Commands,
    mut targets: Query<(
        Entity,
        &PickSelection,
        Option<&mut OutlineVolume>,
        Has<GizmoTarget>,
    )>,
) {
    // Continuously update entities based on their picking state
    for (entity, pick_interaction, outline, has_gizmo_target) in &mut targets {
        let mut entity_cmd = commands.entity(entity);

        if pick_interaction.is_selected {
            if !has_gizmo_target {
                entity_cmd.insert(GizmoTarget::default());
            }
        } else {
            entity_cmd.remove::<GizmoTarget>();
        }

        if let Some(mut outline) = outline {
            outline.visible = pick_interaction.is_selected;
        }
    }
}
