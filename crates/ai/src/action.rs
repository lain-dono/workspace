use super::{CurrentAction, EntityComponentTrigger, Target};
use bevy::ecs::{
    bundle::Bundle,
    component::{Component, ComponentId, Components},
    entity::Entity,
    event::EntityEvent,
    observer::On,
    system::{Commands, In},
};

#[derive(EntityEvent)]
#[entity_event(trigger = EntityComponentTrigger)]
pub struct ActionInit {
    #[event_target]
    pub caller: Entity,
    pub target: Option<Target>,
}

#[derive(EntityEvent)]
#[entity_event(trigger = EntityComponentTrigger)]
pub struct ActionStop {
    #[event_target]
    pub caller: Entity,
}

#[derive(EntityEvent)]
pub struct RequestAction {
    #[event_target]
    pub caller: Entity,
    pub target: Option<(Action, Target)>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Action(pub ComponentId);

impl Action {
    pub fn new<T: Component>(components: &Components) -> Self {
        Self(components.component_id::<T>().unwrap())
    }

    #[must_use]
    pub fn default<T: Component + Default>(trigger: On<ActionInit, T>) -> (Entity, T) {
        (trigger.caller, T::default())
    }

    pub fn init<B: Bundle>(In((caller, bundle)): In<(Entity, B)>, mut commands: Commands) {
        bevy::log::debug!("init action {caller} {}", std::any::type_name::<B>());
        commands.entity(caller).insert(bundle);
    }

    pub fn stop<B: Bundle>(trigger: On<ActionStop, B>, mut commands: Commands) {
        let caller = trigger.caller;
        bevy::log::debug!("stop action {caller} {}", std::any::type_name::<B>());
        commands.entity(caller).remove::<(B, CurrentAction)>();
    }
}
