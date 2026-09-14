use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

mod action;
mod app;
mod arrays;
mod interaction;
mod motive;
mod param;
mod score;
mod script;

mod trigger;

pub use self::action::{Action, ActionInit, ActionStop, RequestAction};
pub use self::app::{ActionApp, StepApp};
pub use self::interaction::{Ad, Interaction, InteractionTarget, Interactive, InteractiveEntity};
pub use self::motive::{Curve, Motive};
pub use self::param::{ActionParam, StepParam};
pub use self::score::Score;
pub use self::script::{RequestStep, Script, Step, StepInit, StepStop};

pub use self::trigger::EntityComponentTrigger;

pub mod prelude {
    pub use super::{ActionApp as _, InteractiveEntity as _, StepApp as _};
}

pub fn plugin(app: &mut App) {
    app.world_mut().register_component::<Step>();
    app.add_observer(Actor::request_observer)
        .add_observer(Step::request_observer)
        .add_observer(Step::init_observer)
        .add_observer(Action::stop::<Step>);
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Target {
    pub target: Entity,
    pub interaction: Entity,
}

impl Target {
    #[must_use]
    pub const fn new(target: Entity, interaction: Entity) -> Self {
        Self {
            target,
            interaction,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct CurrentAction {
    pub action: Action,
    pub target: Option<Target>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct OccupedBy(pub Entity);

#[derive(Component)]
pub struct Actor {
    pub motives: smallvec::SmallVec<[Motive; 8]>,
    pub modificators: Vec<f32>,
    pub threshold: f32,
    pub default: Action,
}

impl Actor {
    pub fn request_observer(
        trigger: On<RequestAction>,
        mut runtime: Local<ActorRuntime>,
        mut commands: Commands,
        actors: Query<(&Self, Option<&GlobalTransform>, Option<&CurrentAction>)>,
        targets: Query<(Entity, &Interactive, Option<&GlobalTransform>), Without<OccupedBy>>,
        interactions: Query<(Entity, &Interaction, &Action)>,
    ) {
        let actor_entity = trigger.caller;
        let &RequestAction { target: next, .. } = trigger.event();

        let Ok((actor, transform, current)) = actors.get(actor_entity) else {
            return;
        };

        let position = transform.map(GlobalTransform::translation);
        let choice = next.or_else(|| runtime.find_next(actor, position, targets, interactions));
        let (action, target) = choice.unzip();
        let action = action.unwrap_or(actor.default);
        if let Some(current) = current {
            if action == current.action && target == current.target {
                return;
            }
            commands.trigger_with(
                ActionStop {
                    caller: actor_entity,
                },
                EntityComponentTrigger(current.action.0),
            );
        }

        let current = CurrentAction { action, target };
        commands.entity(actor_entity).insert(current);
        commands.trigger_with(
            ActionInit {
                caller: actor_entity,
                target,
            },
            EntityComponentTrigger(action.0),
        );
    }

    #[must_use]
    pub fn score_interaction(&self, interaction: &Interaction, distance: f32) -> Score {
        let ads = interaction.advertising.iter();
        let scores = ads.filter_map(|&Ad { idx, min, max, fix }| {
            let motive = self.motives.get(idx)?;
            let reward = max * self.modificators.get(fix).unwrap_or(&1.0);
            (min <= 0.0 || motive.current <= min).then(|| motive.score(reward))
        });
        Score::new(scores.sum::<f32>() / (1.0 + distance * interaction.attenuation))
    }
}

#[derive(Clone, Copy, Debug)]
struct Choice {
    score: Score,
    target: Target,
    action: Action,
}

#[derive(Clone, Debug, Default)]
pub struct ActorRuntime {
    actions: Vec<Choice>,
    weights: Vec<(Choice, f32)>,
}

impl ActorRuntime {
    pub fn find_next<F: QueryFilter>(
        &mut self,
        actor: &Actor,
        position: Option<Vec3>,
        targets: Query<(Entity, &Interactive, Option<&GlobalTransform>), F>,
        interactions: Query<(Entity, &Interaction, &Action)>,
    ) -> Option<(Action, Target)> {
        use rand::distr::{Distribution, uniform::Uniform};

        self.find_actions(actor, position, targets, interactions);

        let mut rng = rand::rng();
        let mut iter = self.actions.iter().copied();

        if let Some(first) = iter.next() {
            let mut total = f32::from(first.score);
            for item in iter {
                self.weights.push((item, total));
                total += f32::from(item.score);
            }

            let chosen = Uniform::new(0.0, total).unwrap().sample(&mut rng);
            let index = self.weights.partition_point(|&(_, w)| w <= chosen);
            let index = index.checked_sub(1);
            let choice = index.map_or(first, |index| self.weights[index].0);

            self.weights.clear();

            Some((choice.action, choice.target))
        } else {
            None
        }
    }

    pub fn find_best<F: QueryFilter>(
        &mut self,
        actor: &Actor,
        position: Option<Vec3>,
        targets: Query<(Entity, &Interactive, Option<&GlobalTransform>), F>,
        interactions: Query<(Entity, &Interaction, &Action)>,
    ) -> Option<(Action, Target)> {
        self.find_actions(actor, position, targets, interactions);
        let choice = self.actions.iter().max_by_key(|choice| choice.score);
        choice.map(|choice| (choice.action, choice.target))
    }

    fn find_actions<F: QueryFilter>(
        &mut self,
        actor: &Actor,
        position: Option<Vec3>,
        targets: Query<(Entity, &Interactive, Option<&GlobalTransform>), F>,
        interactions: Query<(Entity, &Interaction, &Action)>,
    ) {
        self.actions.clear();
        self.actions.extend(
            targets
                .into_iter()
                .flat_map(|(target, entities, transform)| {
                    let pair = position.zip(transform.map(GlobalTransform::translation));
                    let distance = pair.map_or(0.0, |(actor, transform)| actor.distance(transform));
                    interactions
                        .iter_many(entities)
                        .map(move |(interaction, scorer, &action)| Choice {
                            score: actor.score_interaction(scorer, distance),
                            target: Target::new(target, interaction),
                            action,
                        })
                        .filter(move |action| f32::from(action.score) >= actor.threshold)
                }),
        );
    }
}
