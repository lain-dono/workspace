use super::item::{Consumable, ItemHandle, ReadConsumable, WriteConsumable};
use crate::{
    character::{CharacterController, FindAndMove, Inventory, DEFAULT_COLOR, FARM_COLOR},
    loading::ItemDatabase,
    state::InGame,
};
use bevy::prelude::*;
use big_brain::*;

#[derive(Component, Clone)]
pub struct Field;

pub struct FieldPlugin;

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (farm_action, FindAndMove::<Field>::system).in_set(BigBrainSet::Actions),
                work_need_scorer.in_set(BigBrainSet::Scorers),
            )
                .run_if(in_state(InGame)),
        );
    }
}

/// Farming 🚜
#[derive(Component, Clone, ActionSpawn)]
pub struct Farm {
    pub per_second: f32,
}

impl Farm {
    pub fn new(per_second: f32) -> Self {
        Self { per_second }
    }
}

pub fn farm_action(
    time: Res<Time<Virtual>>,
    mut actors: Query<(&Inventory, &mut CharacterController)>,
    mut query: Query<(ActionQuery, &Farm)>,

    items: Res<ItemDatabase>,
    children: Query<&Children>,
    mut consumable: WriteConsumable,
    mut commands: Commands,
) {
    for (mut action, farm) in &mut query {
        let (inventory, mut ctrl) = actors.get_mut(action.actor()).unwrap();
        let container = children.get(inventory.container).ok();

        if action.is_executing() {
            trace!("Farming...");
            let add = farm.per_second * time.delta_secs();

            let is_maximum = consumable.get(container, &items.raw_food, |mut cons| {
                cons.current = (cons.current + add).clamp(0.0, cons.maximum);
                cons.current == cons.maximum
            });

            if let Some(is_maximum) = is_maximum {
                ctrl.color = FARM_COLOR;
                if is_maximum {
                    debug!("Inventory full!");
                    ctrl.color = DEFAULT_COLOR;
                    action.success();
                }
            } else {
                // add empty raw_food and try next frame
                let food = commands.spawn(ItemHandle(items.raw_food.clone())).id();
                commands.entity(inventory.container).add_child(food);
            }
        }

        if action.is_cancelled() {
            debug!("Farming was interrupted. Still need to work.");
            ctrl.color = DEFAULT_COLOR;
            action.failure();
        }
    }
}

#[derive(Component, Clone, Default, ScorerSpawn)]
pub struct WorkNeedScorer;

pub fn work_need_scorer(
    actors: Query<&Inventory>,
    mut query: Query<ScorerQuery, With<WorkNeedScorer>>,

    children: Query<&Children>,
    items: Res<ItemDatabase>,
    consumable: ReadConsumable,
) {
    for mut score in &mut query {
        let inventory = actors.get(score.actor()).expect("actor");
        let children = children.get(inventory.container).ok();
        let full = consumable.get_or(&items.raw_food, children, false, Consumable::is_full);
        score.set(if !full { 0.6 } else { 0.0 });
    }
}
