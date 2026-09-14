use super::{ActionInit, ActionStop, CurrentAction, EntityComponentTrigger, RequestAction, Target};
use bevy::ecs::{
    bundle::Bundle,
    component::{Component, ComponentId, Components},
    entity::Entity,
    event::EntityEvent,
    observer::On,
    system::{Commands, In, Query},
};

#[derive(EntityEvent)]
#[entity_event(trigger = EntityComponentTrigger)]
pub struct StepInit {
    #[event_target]
    pub caller: Entity,
    pub target: Option<Target>,
}

#[derive(EntityEvent)]
#[entity_event(trigger = EntityComponentTrigger)]
pub struct StepStop {
    #[event_target]
    pub caller: Entity,
}

#[derive(EntityEvent)]
pub struct RequestStep {
    #[event_target]
    pub caller: Entity,
}

#[derive(Component)]
pub struct Script {
    pub code: Vec<ComponentId>,
}

impl Script {
    #[must_use]
    pub fn new(code: Vec<ComponentId>) -> Self {
        Self { code }
    }
}

#[derive(Component)]
pub struct Step {
    code: Vec<ComponentId>,
    step: usize,
}

impl Step {
    pub fn init_observer(
        trigger: On<ActionInit, Self>,
        query: Query<&Script>,
        mut commands: Commands,
    ) {
        let caller = trigger.caller;

        bevy::log::debug!("init action {caller} {}", std::any::type_name::<Self>());

        let interaction = trigger.event().target.unwrap().interaction;
        let code = query.get(interaction).unwrap().code.clone();
        commands.entity(caller).insert(Self { code, step: 0 });
        commands.trigger(RequestStep { caller });
    }

    pub fn request_observer(
        trigger: On<RequestStep>,
        components: &Components,
        mut commands: Commands,
        mut query: Query<(&mut Self, &CurrentAction)>,
    ) {
        let caller = trigger.event_target();
        if let Ok((mut runtime, current_action)) = query.get_mut(caller) {
            if let Some(&next) = runtime.code.get(runtime.step) {
                commands.trigger_with(
                    StepInit {
                        caller,
                        target: current_action.target,
                    },
                    EntityComponentTrigger(next),
                );
                runtime.step += 1;
            } else {
                let id = components.component_id::<Self>().unwrap();
                commands.trigger_with(ActionStop { caller }, EntityComponentTrigger(id));
                commands.trigger(RequestAction {
                    caller,
                    target: None,
                });
            }
        }
    }

    #[must_use]
    pub fn default<T: Component + Default>(trigger: On<StepInit, T>) -> (Entity, T) {
        (trigger.caller, T::default())
    }

    pub fn init<B: Bundle>(In((caller, bundle)): In<(Entity, B)>, mut commands: Commands) {
        bevy::log::debug!("init step {caller} {}", std::any::type_name::<B>());
        commands.entity(caller).insert(bundle);
    }

    pub fn stop<B: Bundle>(trigger: On<StepStop, B>, mut commands: Commands) {
        let caller = trigger.caller;
        bevy::log::debug!("stop step {caller} {}", std::any::type_name::<B>());
        commands.entity(caller).remove::<B>();
    }
}
