use bevy::{ecs::component::ComponentId, prelude::*};
use std::collections::VecDeque;

fn spawn(mut commands: Commands) {
    let person = commands.spawn((Name::new("person"),));
    let person = person.id();

    commands
        .spawn(Name::new("kitchen"))
        .with_child((
            Name::new("kitchen frige"),
            Actions {
                actions: vec![Action::new("take food")],
            },
        ))
        .with_child((
            Name::new("kitchen board"),
            Actions {
                actions: vec![Action::new("prepare food")],
            },
        ))
        .with_child((
            Name::new("kitchen stove"),
            Actions {
                actions: vec![Action::new("cook food")],
            },
        ));
}

struct Bucket {
    score: f32,
    list: Vec<(Entity, ComponentId, f32)>,
}

#[derive(Component)]
struct Actions {
    actions: Vec<Action>,
}

struct Action {
    name: String,
}

impl Action {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Component)]
struct Agent {
    actions: VecDeque<()>,
}

fn operate(agents: Query<&mut Agent>, interactions: Query<&Actions>) {
    for mut agent in agents {
        // While there are actions in the queue,
        // pop the next one off, perform it,
        // and maybe get a reward
        if let Some(action) = agent.actions.pop_front() {
            //
        }

        // If you run out of actions,
        // perform action selection based on current needs,
        // to find more actions
        if agent.actions.is_empty() {
            // 1. Examine objects around you, and find out what they advertise

            let mut environment = vec![];

            for Actions { actions } in interactions.iter() {
                for ad in actions.iter() {
                    environment.push(ad.name.as_str());
                }
            }

            // 2. Score each advertisement based on your current needs

            // 3. Pick the best advertisement, get its action sequence

            // 4. Push the action sequence on your queue

            // If you still have nothing to do, do some fallback actions
        }
    }
}

pub struct Hunger(pub f32);

pub struct Mood(pub f32);

struct Person {
    personality: Personality,

    skills: Skills,
    // rel: Relationship,
}

// Sloppy - Neat
// Shy - Outgoing
// Serious - Playful
// Lazy - Active
// Mean - Nice
struct Personality;

// Cooking
// Mechanical
// Logic
// Body
// etc.
struct Skills;
