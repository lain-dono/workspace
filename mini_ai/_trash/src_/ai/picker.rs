use super::{ActionEvent, Score};
use bevy::ecs::{
    component::{Component, ComponentId},
    entity::{Entity, EntityHashMap},
    event::Event,
    hierarchy::Children,
    observer::Trigger,
    query::With,
    system::{Commands, Local, Query},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ActionTarget {
    pub target: Entity,
    pub interaction: Entity,
}

#[derive(Event, Clone, Debug)]
pub struct RequestAction(pub Option<(ComponentId, Option<ActionTarget>)>);

#[derive(Component)]
pub struct CurrentAction(pub ComponentId, pub Option<ActionTarget>);

#[derive(Component)]
pub struct Picker {
    pub current: (ComponentId, Option<ActionTarget>),
    pub choices: EntityHashMap<(ComponentId, Option<ActionTarget>)>,
    pub default: ComponentId,
}

impl Picker {
    pub fn new(default: ComponentId) -> Self {
        Self {
            current: (default, None),
            choices: EntityHashMap::default(),
            default,
        }
    }

    pub fn with(mut self, scorer: Entity, action: ComponentId) -> Self {
        self.choices.insert(scorer, (action, None));
        self
    }

    pub fn with_target(
        mut self,
        scorer: Entity,
        action: ComponentId,
        target: ActionTarget,
    ) -> Self {
        self.choices.insert(scorer, (action, Some(target)));
        self
    }

    pub fn pick(&mut self, scorer: Option<Entity>) -> ComponentId {
        let next = scorer.and_then(|entity| self.choices.get(&entity).copied());
        self.current = next.unwrap_or((self.default, None));
        self.current.0
    }

    pub fn system(
        mut commads: Commands,
        actors: Query<(Entity, &Self, Option<&CurrentAction>), With<Picker>>,
    ) {
        for (actor, &Self { default, .. }, current) in actors {
            if current.is_some_and(|action| default == action.0) || current.is_none() {
                commads.trigger_targets(RequestAction(None), actor);
            }
        }
    }

    pub fn observer(
        trigger: Trigger<RequestAction>,
        mut commands: Commands,
        mut actors: Query<(&Picker, Option<&CurrentAction>)>,
    ) {
        let actor = trigger.target();
        if let Ok((picker, current_action)) = actors.get_mut(actor) {
            let curr = current_action.map(|action| (action.0, action.1));
            let next = trigger.event().0.unwrap_or(picker.current);

            if let Some(curr) = curr {
                if next == curr {
                    return;
                }
                commands.trigger_targets(ActionEvent::Cancelled, (actor, curr.0));
            }

            commands.entity(actor).insert(CurrentAction(next.0, next.1));
            commands.trigger_targets(ActionEvent::Initialized(next.1), (actor, next.0));
        }
    }
}

#[derive(Component)]
pub struct FirstToScore {
    pub threshold: f32,
}

impl FirstToScore {
    pub fn system(
        pickers: Query<(&Children, &mut Picker, &Self)>,
        scores: Query<(Entity, &Score)>,
    ) {
        for (children, mut picker, &Self { threshold }) in pickers {
            let mut scorer = None;

            for (entity, score) in scores.iter_many(children) {
                if f32::from(score) >= threshold {
                    scorer = Some(entity);
                    break;
                }
            }

            picker.pick(scorer);
            // picker.pick(default, scorer, choices_query.iter_many(choices));
        }
    }
}

#[derive(Component)]
pub struct Highest;

impl Highest {
    pub fn system(
        pickers: Query<(&Children, &mut Picker), With<Self>>,
        scores: Query<(Entity, &Score)>,
    ) {
        for (children, mut picker) in pickers {
            let mut scorer: Option<(Entity, &Score)> = None;
            for (entity, score) in scores.iter_many(children) {
                if let Some((_, highest_score)) = scorer {
                    if f32::from(score) > f32::from(highest_score) {
                        scorer = Some((entity, score));
                    }
                } else {
                    scorer = Some((entity, score));
                }
            }

            let scorer = scorer.map(|(entity, _)| entity);
            picker.pick(scorer);
        }
    }
}

#[derive(Component)]
pub struct WeightedRandom {
    pub threshold: f32,
}

impl WeightedRandom {
    pub fn system(
        mut cumulative_weights: Local<Vec<(Entity, f32)>>,
        pickers: Query<(&Children, &mut Picker, &Self)>,
        scores: Query<(Entity, &Score)>,
    ) {
        use rand::distr::Distribution;
        use rand::distr::uniform::Uniform;

        let mut rng = rand::rng();

        for (children, mut picker, &Self { threshold }) in pickers {
            let mut iter = scores.iter_many(children).filter_map(|(entity, score)| {
                let score = f32::from(score);
                (score > threshold).then_some((entity, score))
            });

            let scorer = if let Some((first, mut total_weight)) = iter.next() {
                for (entity, score) in iter {
                    cumulative_weights.push((entity, total_weight));
                    total_weight += score;
                }

                let weight_distribution = Uniform::new(0.0, total_weight).unwrap();
                let chosen_weight = weight_distribution.sample(&mut rng);
                let index = cumulative_weights.partition_point(|&(_, w)| w <= chosen_weight);
                let index = index.checked_sub(1);

                cumulative_weights.clear();

                Some(index.map_or(first, |index| cumulative_weights[index].0))
            } else {
                None
            };

            picker.pick(scorer);
        }
    }
}
