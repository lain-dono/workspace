use super::{Action, ActionInit, Step, StepInit};
use bevy::ecs::{
    bundle::Bundle, component::Component, entity::Entity, observer::On, system::IntoSystem,
};

pub trait ActionApp {
    fn add_action_default<T: Component + Default>(&mut self) -> &mut Self {
        self.add_action(Action::default::<T>)
    }

    fn add_action<T: Component, B: Bundle, M>(
        &mut self,
        fun: impl IntoSystem<On<'static, 'static, ActionInit, T>, (Entity, B), M> + Send + 'static,
    ) -> &mut Self;
}

impl ActionApp for bevy::app::App {
    fn add_action<T: Component, B: Bundle, M>(
        &mut self,
        fun: impl IntoSystem<On<'static, 'static, ActionInit, T>, (Entity, B), M> + Send + 'static,
    ) -> &mut Self {
        self.world_mut().register_component::<T>();
        self.add_observer(fun.pipe(Action::init))
            .add_observer(Action::stop::<B>)
    }
}

pub trait StepApp {
    fn add_step_default<T: Component + Default>(&mut self) -> &mut Self {
        self.add_step(Step::default::<T>)
    }

    fn add_step<T: Component, B: Bundle, M>(
        &mut self,
        fun: impl IntoSystem<On<'static, 'static, StepInit, T>, (Entity, B), M> + Send + 'static,
    ) -> &mut Self;
}

impl StepApp for bevy::app::App {
    fn add_step<T: Component, B: Bundle, M>(
        &mut self,
        fun: impl IntoSystem<On<'static, 'static, StepInit, T>, (Entity, B), M> + Send + 'static,
    ) -> &mut Self {
        self.world_mut().register_component::<T>();
        self.add_observer(fun.pipe(Step::init))
            .add_observer(Step::stop::<B>)
    }
}
