use crate::{
    character::Inventory,
    mechanics::item::{Consumable, ItemName},
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use big_brain::*;

pub fn inventory_ui(
    mut contexts: EguiContexts,
    query: Query<(&HasThinker, &Inventory), With<crate::player::Player>>,
    children_query: Query<&Children>,

    work_need_query: Query<&Score, With<crate::mechanics::WorkNeedScorer>>,
    sell_need_query: Query<&Score, With<crate::mechanics::SellNeedScorer>>,
    fatigue_query: Query<&Score, With<crate::mechanics::FatigueScorer>>,

    items: Query<(&ItemName, Option<&Consumable>)>,
) -> Result {
    let Ok((thinker, inventory)) = query.single() else {
        return;
    };

    let Ok(container_children) = children_query.get(inventory.container) else {
        return;
    };

    fn read_score<T: Component>(
        default: f32,
        entity: Entity,
        query: &Query<&Score, With<T>>,
    ) -> f32 {
        query
            .get(entity)
            .map(|score| score.get())
            .unwrap_or(default)
    }

    let mut work_need = 0.0;
    let mut sell_need = 0.0;
    let mut fatigue = 0.0;

    for entity in children_query.iter_descendants(thinker.entity()) {
        work_need = read_score(work_need, entity, &work_need_query);
        sell_need = read_score(sell_need, entity, &sell_need_query);
        fatigue = read_score(fatigue, entity, &fatigue_query);
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("#INVENTORY_HUD"))
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .show(ctx, |ui| {
            ui.scope(|ui| {
                egui::Frame::side_top_panel(ui.style()).show(ui, |ui| {
                    let _ = ui.allocate_space(egui::vec2(250.0, 0.0));

                    ui.heading("Inventory");

                    ui.group(|ui| {
                        for (index, child) in container_children.iter().enumerate() {
                            let num = index + 1;

                            if let Ok((ItemName(name), consumable)) = items.get(child) {
                                let mut text = format!("{:>3}: {}", num, name);

                                if let Some(Consumable { current, maximum }) = consumable {
                                    if maximum.is_finite() {
                                        text.push_str(&format!(" {:.0}/{:.0}", current, maximum));
                                    } else {
                                        text.push_str(&format!(" {:.0}", current));
                                    }
                                }

                                ui.label(text);
                            }
                        }
                    });

                    ui.heading("Scores:");
                    ui.label(format!("work_need: {work_need:?}"));
                    ui.label(format!("sell_need: {sell_need:?}"));
                    ui.label(format!("fatigue: {fatigue:?}"));
                });
            });
        });

    Ok(())
}
