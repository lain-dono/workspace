use bevy::app::{App, Plugin};
use bevy::ecs::{
    bundle::Bundle,
    component::{Component, Components},
    entity::Entity,
    event::Event,
    observer::Trigger,
    query::{QueryItem, ReadOnlyQueryData},
    relationship::{Relationship, RelationshipTarget, SourceIter},
    schedule::{InternedScheduleLabel, IntoScheduleConfigs, ScheduleLabel, SystemSet},
    system::{Commands, Query},
};

mod picker;

pub use self::picker::{
    ActionTarget, CurrentAction, FirstToScore, Highest, Picker, RequestAction, WeightedRandom,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]

enum AiSet {
    Thinker,
    Picker,
}

pub struct AiPlugin {
    pub schedule: InternedScheduleLabel,
}

impl Default for AiPlugin {
    fn default() -> Self {
        Self {
            schedule: bevy::app::PostUpdate.intern(),
        }
    }
}

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(self.schedule, (AiSet::Thinker, AiSet::Picker).chain())
            .add_systems(self.schedule, FirstToScore::system.in_set(AiSet::Thinker))
            .add_systems(self.schedule, Highest::system.in_set(AiSet::Thinker))
            .add_systems(self.schedule, WeightedRandom::system.in_set(AiSet::Thinker))
            .add_systems(self.schedule, Picker::system.in_set(AiSet::Picker))
            .add_observer(Picker::observer);
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Score {
    value: f32,
}

impl std::fmt::Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Score").field(&self.value).finish()
    }
}

impl From<Score> for f32 {
    fn from(Score { value }: Score) -> Self {
        value
    }
}

impl From<&Score> for f32 {
    fn from(&Score { value }: &Score) -> Self {
        value
    }
}

impl Score {
    pub const MIN: Self = Self { value: 0.0 };
    pub const MAX: Self = Self { value: 1.0 };

    pub const fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

#[derive(Event, Clone, Debug)]
pub enum ActionEvent {
    Initialized(Option<ActionTarget>),
    Cancelled,
    Successed,
}

pub trait ActionObserver {
    type ActorQuery: ReadOnlyQueryData;
    type Bundle: Bundle;

    fn create<'w>(
        target: Option<ActionTarget>,
        query: QueryItem<'w, Self::ActorQuery>,
    ) -> Self::Bundle;

    fn observer(
        trigger: Trigger<ActionEvent, Self>,
        mut cmd: Commands,
        actors: Query<Self::ActorQuery>,
    ) where
        Self: Component + Default + Sized,
    {
        let mut actor = cmd.entity(trigger.target());
        match trigger.event() {
            &ActionEvent::Initialized(target) => {
                if let Ok(query) = actors.get(actor.id()) {
                    actor.insert(Self::create(target, query));
                } else {
                    actor.remove::<CurrentAction>();
                }
            }
            ActionEvent::Successed | ActionEvent::Cancelled => {
                actor.remove::<(Self::Bundle, CurrentAction)>();
            }
        }
    }
}

impl<T: Component + Default> ActionObserver for T {
    type ActorQuery = ();
    type Bundle = Self;

    fn create<'w>(
        _target: Option<ActionTarget>,
        _: QueryItem<'w, Self::ActorQuery>,
    ) -> Self::Bundle {
        if let Some(target) = _target {
            dbg!(target);
        }
        Self::default()
    }
}

pub fn success<T: Component>(actor: Entity, mut cmd: Commands, components: &Components) {
    let id = components.component_id::<T>().unwrap();
    cmd.trigger_targets(ActionEvent::Successed, (actor, id));
}

pub fn cancel<T: Component>(actor: Entity, mut cmd: Commands, components: &Components) {
    let id = components.component_id::<T>().unwrap();
    cmd.trigger_targets(ActionEvent::Cancelled, (actor, id));
}

pub trait RelationScorer: Component + Sized {
    type Query: ReadOnlyQueryData;

    fn score<'w>(&self, query: QueryItem<'w, Self::Query>) -> Score;

    fn related<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: Relationship,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .related(entity)
                .and_then(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn relationship_sources<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: RelationshipTarget,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .relationship_sources(entity)
                .find_map(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn root_ancestor<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: Relationship,
    {
        for (entity, scorer, mut score) in scorers {
            *score = query
                .get(relation.root_ancestor(entity))
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn iter_leaves<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: RelationshipTarget,
        for<'w> SourceIter<'w, R>: DoubleEndedIterator,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .iter_leaves(entity)
                .find_map(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn iter_descendants<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: RelationshipTarget,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .iter_descendants(entity)
                .find_map(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn iter_descendants_depth_first<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: RelationshipTarget,
        for<'w> SourceIter<'w, R>: DoubleEndedIterator,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .iter_descendants_depth_first(entity)
                .find_map(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }

    fn iter_ancestors<R>(
        scorers: Query<(Entity, &Self, &mut Score)>,
        relation: Query<&R>,
        query: Query<Self::Query>,
    ) where
        R: Relationship,
    {
        for (entity, scorer, mut score) in scorers {
            *score = relation
                .iter_ancestors(entity)
                .find_map(|entity| query.get(entity).ok())
                .map_or(Score::MIN, |item| scorer.score(item));
        }
    }
}
