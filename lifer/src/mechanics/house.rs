use crate::{
    character::{CharacterController, FindAndMove, DEFAULT_COLOR, SLEEP_COLOR},
    state::InGame,
};
use bevy::prelude::*;
use big_brain::*;

#[derive(Component, Clone)]
pub struct House;

pub struct HousePlugin;

impl Plugin for HousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fatigue_system).add_systems(
            PreUpdate,
            (
                (sleep_action, FindAndMove::<House>::system).in_set(BigBrainSet::Actions),
                fatigue_scorer.in_set(BigBrainSet::Scorers),
            )
                .run_if(in_state(InGame)),
        );
    }
}

/// Sleepiness 😴
#[derive(Component)]
pub struct Fatigue {
    pub change: f32,
    pub current: f32,
}

pub fn fatigue_system(time: Res<Time<Virtual>>, mut parameters: Query<&mut Fatigue>) {
    for mut param in &mut parameters {
        param.current = (param.current + param.change * time.delta_secs()).clamp(0.0, 100.0);
    }
}

#[derive(Component, Clone, ActionSpawn)]
pub struct Sleep {
    pub until: f32,
    pub per_second: f32,
}

impl Sleep {
    pub fn new(until: f32, per_second: f32) -> Self {
        Self { until, per_second }
    }
}

pub fn sleep_action(
    time: Res<Time<Virtual>>,
    mut actors: Query<(&mut Fatigue, &mut CharacterController)>,
    mut query: Query<(ActionQuery, &Sleep)>,
) {
    for (mut action, sleep) in &mut query {
        let (mut fatigue, mut ctrl) = actors.get_mut(action.actor()).unwrap();

        if action.is_executing() {
            if !ctrl.is_sleeping {
                debug!("Time to sleep!");
                ctrl.is_sleeping = true;
            }

            trace!("Sleeping...");

            fatigue.current -= sleep.per_second * time.delta_secs();
            ctrl.color = SLEEP_COLOR;

            if fatigue.current <= sleep.until {
                debug!("Woke up well-rested!");
                ctrl.color = DEFAULT_COLOR;
                ctrl.is_sleeping = false;
                action.success();
            }
        }

        if action.is_cancelled() {
            debug!("Sleep was interrupted. Still tired.");
            ctrl.color = DEFAULT_COLOR;
            ctrl.is_sleeping = false;
            action.failure();
        }
    }
}

#[derive(Component, Clone, Default, ScorerSpawn)]
pub struct FatigueScorer {
    last_score: Option<f32>,
}

pub fn fatigue_scorer(
    actors: Query<(&Fatigue, &CharacterController)>,
    mut query: Query<(ScorerQuery, &mut FatigueScorer)>,
) {
    for (mut score, mut scorer) in &mut query {
        if let Ok((fatigue, sleeping)) = actors.get(score.actor()) {
            let new_score = fatigue.current / 100.0;

            if sleeping.is_sleeping {
                score.set(*scorer.last_score.get_or_insert(new_score));
            } else {
                scorer.last_score.take();
                score.set(new_score);
                if new_score >= 0.8 {
                    trace!("Fatigue above threshold! Score: {}", new_score)
                }
            }
        }
    }
}
