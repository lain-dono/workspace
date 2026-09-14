use crate::ai::{ActionObserver, RelationScorer, Score};
use crate::interaction::{InteractionTarget, InteractiveEntity};
use crate::need::{Need, NeedDecay};
use crate::reward::Reward;

use bevy::{
    app::ScheduleRunnerPlugin,
    ecs::{component::Components, query::QueryItem},
    log::LogPlugin,
    prelude::*,
};
use std::time::Duration;

pub mod ai;
pub mod interaction;
pub mod need;
pub mod reward;
pub mod sample;
pub mod sims;

fn main() {
    let speed = Duration::from_secs_f64(1.0 / 5.0);
    // let speed = Duration::from_secs_f64(1.0);
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(speed)))
        .add_plugins(LogPlugin {
            filter: "mini_ai=debug".to_string(),
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                ThirstDecay::system::<With<Idle>>,
                ThirstyScorer::iter_ancestors::<ChildOf>,
                Drinking::quench_thirst,
                print_need::<Thirst>,
            )
                .chain(),
        )
        .add_plugins(ai::AiPlugin::default())
        .add_observer(Idle::observer)
        .add_observer(Drinking::observer)
        .run();
}

trait SpawnEntity {
    fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity;
}

impl SpawnEntity for Commands<'_, '_> {
    fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity {
        self.spawn(bundle).id()
    }
}

fn setup(mut commands: Commands, components: &Components) {
    info!("setup");

    let target = commands.spawn_entity(Name::new("water bottle"));

    let interaction = commands.spawn_entity((
        Name::new("drink"),
        Action(components.component_id::<Drinking>().unwrap()),
        ThirstyScorer2,
        // Choice::new(thirsty, components.component_id::<Drinking>().unwrap()),
        Reward::<Thirst>::new(40.0),
    ));

    commands.entity(target).add_interaction(interaction);

    let drink_target = ai::ActionTarget {
        target,
        interaction,
    };

    let thirsty_interaction = ScorerTarget {
        target,
        interaction,
    };

    let thirsty = commands.spawn_entity((ThirstyScorer, Score::default(), thirsty_interaction));

    let mut actor = commands.spawn((
        Name::new("Actor"),
        Need::<Thirst>::new(0.0),
        ThirstDecay::new(40.0),
        ai::WeightedRandom { threshold: 0.5 },
        ai::Picker::new(components.component_id::<Idle>().unwrap()).with_target(
            thirsty,
            components.component_id::<Drinking>().unwrap(),
            drink_target,
        ),
    ));

    actor.add_child(thirsty);

    commands
        .spawn(Name::new("bathroom"))
        .with_children(|commands| {
            commands
                .spawn((Name::new("toilet"), Dirtiness(0.5)))
                .with_interactions(|target| {
                    target.spawn((Name::new("urinate"), Reward::<Bladder>::new(40.0)));
                    target.spawn((Name::new("clean"), Reward::<Room>::new(30.0)));
                });

            commands
                .spawn((Name::new("bathtub"), Dirtiness(0.5)))
                .with_interactions(|target| {
                    target.spawn((Name::new("take bath"), Reward::<Hygiene>::new(40.0)));
                    target.spawn((Name::new("clean"), Reward::<Room>::new(30.0)));
                });
        });
}

fn spawn_scorers(
    mut commands: Commands,
    actors: Query<(Entity, &mut ai::Picker)>,
    interactions: Query<(Entity, &InteractionTarget, &Action), With<ThirstyScorer2>>,
) {
    for (actor, mut picker) in actors {
        for (entity, _) in picker.choices.drain() {
            // todo despawn_related
            commands.entity(entity).despawn();
        }

        for (interaction, &InteractionTarget { target }, &Action(action)) in interactions {
            let scorer = ScorerTarget {
                target,
                interaction,
            };

            let scorer = commands.spawn_entity((ThirstyScorer, scorer));
            commands.entity(actor).add_child(scorer);

            let target = ai::ActionTarget {
                target,
                interaction,
            };

            picker.choices.insert(scorer, (action, Some(target)));
        }
    }
}

fn thirsty_scorer(
    scorers: Query<(&ScorerTarget, &ChildOf, &mut Score), With<ThirstyScorer>>,
    actors: Query<&Need<Thirst>>,
    //targets
    interactions: Query<&Reward<Thirst>>,
) {
    for (scorer, &ChildOf(actor), score) in scorers {
        let need = f32::from(actors.get(actor).unwrap());
        // let interaction = ;
        //
    }
}

#[derive(Component)]
pub struct Action(pub bevy::ecs::component::ComponentId);

#[derive(Component)]
#[require(Score)]
pub struct ScorerTarget {
    pub target: Entity,
    pub interaction: Entity,
}

#[derive(Component)]
pub struct Choice {
    pub scorer: Entity,
    pub action: bevy::ecs::component::ComponentId,
}

impl Choice {
    const fn new(scorer: Entity, action: bevy::ecs::component::ComponentId) -> Self {
        Self { action, scorer }
    }
}

fn print_need<T: TypePath + Send + Sync>(query: Query<&Need<T>>) {
    for need in query {
        info!("{need:.0?}");
    }
}

macro_rules! create_item {
    ($($need:ident),* $(,)?) => {
        $( #[derive(Component, Default)] pub struct $need; )*
    };
}

create_item! { Hunger, Bladder, Hygiene, Energy }
create_item! { Comfort, Fun, Social, Room }

#[derive(Component)]
struct Dirtiness(pub f32);

type ThirstDecay = NeedDecay<Thirst>;

#[derive(Reflect)]
pub struct Thirst;

#[derive(Component)]
pub struct ThirstyScorer2;

#[derive(Component)]
#[require(Score)]
pub struct ThirstyScorer;

impl ai::RelationScorer for ThirstyScorer {
    type Query = &'static Need<Thirst>;

    fn score<'w>(&self, thirst: QueryItem<'w, Self::Query>) -> Score {
        Score::new(thirst.get() / 100.0)
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Idle;

#[derive(Component, Clone, Debug)]
pub struct Drinking {
    pub until: f32,
    pub speed: f32,
}

impl Default for Drinking {
    fn default() -> Self {
        Self {
            until: 10.0,
            speed: 80.0,
        }
    }
}

impl Drinking {
    fn quench_thirst(
        time: Res<Time>,
        mut drinking: Query<(Entity, &mut Need<Thirst>, &Self)>,
        mut commands: Commands,
        components: &Components,
    ) {
        for (actor, mut thirst, drink) in drinking.iter_mut() {
            info!("drink");
            *thirst -= drink.speed * time.delta_secs();
            if f32::from(&*thirst) <= drink.until {
                thirst.set(drink.until);
                ai::success::<Self>(actor, commands.reborrow(), components);
            }
        }
    }
}
