use crate::ai::Score;
use crate::interaction::{InteractionTarget, InteractiveEntity};
use crate::need::Need;
use crate::reward::Reward;
use bevy::prelude::*;

mod raw;

macro_rules! create_item {
    ($($need:ident),* $(,)?) => {
        $(
            #[derive( Default)]
            pub struct $need;
        )*
    };
}

create_item! {
    Hunger,
    Bladder,
    Hygiene,
    Energy,
}
create_item! {
    Comfort,
    Fun,
    Social,
    Room,
}

#[derive(Component)]
struct Dirtiness(pub f32);

fn bathroom_system(mut commands: Commands) {
    commands.spawn((
        Name::new("actor"),
        (
            Need::<Hunger>::new(0.0),
            Need::<Bladder>::new(0.0),
            Need::<Hygiene>::new(0.0),
            Need::<Energy>::new(0.0),
        ),
        (
            Need::<Comfort>::new(0.0),
            Need::<Fun>::new(0.0),
            Need::<Social>::new(0.0),
            Need::<Room>::new(0.0),
        ),
    ));
}

fn spawn_choices<T: Send + Sync + 'static>(
    mut commands: Commands,
    actors: Query<Entity, With<Need<T>>>,
    actions: Query<(Entity, &InteractionTarget), With<Reward<T>>>,
) {
    for actor in actors {
        for (option, &InteractionTarget { target }) in actions.iter() {
            let bundle = (Choice { target, option }, Score::MIN);
            let child = commands.spawn(bundle).id();
            commands.entity(actor).add_child(child);
        }
    }
}

fn measure(
    mut commands: Commands,
    choices: Query<(Entity, &Choice, &ChildOf, &mut Score)>,
    actors: Query<&Name>,
    targets: Query<&Name>,
    options: Query<&Name>,
) {
    for (entity, &Choice { target, option }, &ChildOf(actor), mut score) in choices {
        let entities = (actor, target, option);

        let actor = actors.get(actor);
        let target = targets.get(target);
        let option = options.get(option);

        let (Ok(actor), Ok(target), Ok(option)) = (actor, target, option) else {
            bevy::log::warn!("measure {entities:?} {actor:?} {target:?} {option:?} despawn");
            commands.entity(entity).despawn();
            continue;
        };

        bevy::log::debug!("measure {entities:?} {actor} {target} {option}");

        score.set(0.0);
    }
}

#[derive(Component, Clone, Copy)]
struct Choice {
    pub target: Entity,
    pub option: Entity,
}

#[derive(Component)]
struct Scorer;

#[derive(Component)]
struct Action;
