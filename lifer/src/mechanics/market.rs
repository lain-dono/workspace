use super::item::{Consumable, ItemHandle, ReadConsumable, WriteConsumable};
use crate::{
    character::{FindAndMove, Inventory},
    loading::ItemDatabase,
    state::InGame,
};
use bevy::prelude::*;
use big_brain::*;

#[derive(Component, Clone)]
pub struct Market;

pub struct MarketPlugin;

impl Plugin for MarketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (sell_action, FindAndMove::<Market>::system).in_set(BigBrainSet::Actions),
                sell_need_scorer.in_set(BigBrainSet::Scorers),
            )
                .run_if(in_state(InGame)),
        );
    }
}

/// Selling 💰
#[derive(Component, Clone, ActionSpawn)]
pub struct Sell;

pub fn sell_action(
    mut commands: Commands,
    mut actors: Query<&Inventory>,
    mut query: Query<ActionQuery, With<Sell>>,

    items: Res<ItemDatabase>,
    children: Query<&Children>,
    mut consumable: WriteConsumable,
) {
    for mut action in &mut query {
        let inventory = actors.get_mut(action.actor()).unwrap();
        let container = children.get(inventory.container).ok();

        if action.is_executing() {
            fn sell_food(mut food: Mut<Consumable>, mut money: Mut<Consumable>) -> f32 {
                let amount = food.take();
                money.current += amount;
                amount
            }

            if let Some(amount) =
                consumable.transfer(container, &items.raw_food, &items.money, sell_food)
            {
                debug!("Sold! amount: {}", amount);
                action.success();
            } else {
                // add empty money and try next frame
                let food = commands.spawn(ItemHandle(items.money.clone())).id();
                commands.entity(inventory.container).add_child(food);
            }
        }

        if action.is_cancelled() {
            debug!("Selling was interrupted. Still need to work.");
            action.failure();
        }
    }
}

#[derive(Component, Clone, Default, ScorerSpawn)]
pub struct SellNeedScorer;

pub fn sell_need_scorer(
    actors: Query<&Inventory>,
    mut query: Query<ScorerQuery, With<SellNeedScorer>>,

    items: Res<ItemDatabase>,
    children: Query<&Children>,
    consumable: ReadConsumable,
) {
    for mut score in &mut query {
        let inventory = actors.get(score.actor()).expect("actor");
        let children = children.get(inventory.container).ok();
        let has_enough = consumable.get_or(&items.raw_food, children, false, Consumable::is_full);
        score.set(if has_enough { 0.6 } else { 0.0 });
    }
}
