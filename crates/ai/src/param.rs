use super::{
    Action, ActionStop, EntityComponentTrigger, RequestAction, RequestStep, StepStop, Target,
};
use bevy::ecs::{
    component::{Component, Components},
    entity::Entity,
    error::Result,
    system::{Commands, SystemParam},
};

#[derive(SystemParam)]
pub struct ActionParam<'w, 's, Marker: Component + 'static> {
    pub commands: Commands<'w, 's>,
    pub components: &'w Components,
    marker: std::marker::PhantomData<Marker>,
}

impl<Marker: Component + 'static> ActionParam<'_, '_, Marker> {
    pub fn reborrow(&mut self) -> ActionParam<'_, '_, Marker> {
        ActionParam {
            commands: self.commands.reborrow(),
            components: self.components,
            marker: self.marker,
        }
    }

    pub fn stop(&mut self, caller: Entity) {
        let id = self.components.component_id::<Marker>().unwrap();
        self.commands
            .trigger_with(ActionStop { caller }, EntityComponentTrigger(id));
        self.request_action(caller, None);
    }

    pub fn request_action(&mut self, caller: Entity, next: Option<(Action, Target)>) {
        self.commands.trigger(RequestAction {
            caller,
            target: next,
        });
    }

    pub fn request_step(&mut self, caller: Entity) {
        self.commands.trigger(RequestStep { caller });
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct StepParam<'w, 's, Marker: Component + 'static> {
    pub commands: Commands<'w, 's>,
    pub components: &'w Components,
    marker: std::marker::PhantomData<Marker>,
}

impl<Marker: Component + 'static> StepParam<'_, '_, Marker> {
    pub fn reborrow(&mut self) -> StepParam<'_, '_, Marker> {
        StepParam {
            commands: self.commands.reborrow(),
            components: self.components,
            marker: self.marker,
        }
    }

    pub fn stop(&mut self, caller: Entity) {
        let id = self.components.component_id::<Marker>().unwrap();
        self.commands
            .trigger_with(StepStop { caller }, EntityComponentTrigger(id));
        self.commands.trigger(RequestStep { caller });
    }
}
