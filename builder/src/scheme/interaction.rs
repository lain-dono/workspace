use bevy::{ecs::query::QueryData, prelude::*};

pub trait ExtOnEvent {
    fn on_event<E, D, T>(
        &mut self,
        input: T,
        action: impl Fn(T, D::Item<'_, '_>) + Send + Sync + 'static,
    ) -> &mut Self
    where
        E: std::fmt::Debug + Clone + Reflect,
        D: QueryData + 'static,
        T: Clone + Send + Sync + 'static;
}

impl ExtOnEvent for EntityCommands<'_> {
    fn on_event<E, D, T>(
        &mut self,
        input: T,
        action: impl Fn(T, D::Item<'_, '_>) + Send + Sync + 'static,
    ) -> &mut Self
    where
        E: std::fmt::Debug + Clone + Reflect,
        D: QueryData + 'static,
        T: Clone + Send + Sync + 'static,
    {
        self.observe(move |trigger: On<Pointer<E>>, mut query: Query<D>| {
            if let Ok(ground) = query.get_mut(trigger.entity) {
                action(input.clone(), ground)
            }
        })
    }
}
