use bevy::{
    ecs::{
        component::ComponentId,
        entity::Entity,
        event::{EntityEvent, Event, Trigger, trigger_entity_internal},
        observer::{CachedObservers, TriggerContext},
        world::DeferredWorld,
    },
    ptr::PtrMut,
};

/// An [`EntityEvent`] [`Trigger`] that, in addition to behaving like a normal [`EntityTrigger`], _also_ runs observers
/// that watch for components that match the slice of [`ComponentId`]s referenced in [`EntityComponentsTrigger`]. This includes
/// both _global_ observers of those components and "entity scoped" observers that watch the [`EntityEvent::event_target`].
///
/// This is used by Bevy's built-in [lifecycle events](crate::lifecycle).
pub struct EntityComponentTrigger(
    /// All of the components whose observers were triggered together for the target entity. For example,
    /// if components `A` and `B` are added together, producing the [`Add`](crate::lifecycle::Add) event, this will
    /// contain the [`ComponentId`] for both `A` and `B`.
    pub ComponentId,
);

impl Default for EntityComponentTrigger {
    fn default() -> Self {
        Self(ComponentId::new(usize::MAX))
    }
}

// SAFETY:
// - `E`'s [`Event::Trigger`] is constrained to [`EntityComponentsTrigger`]
unsafe impl<'a, E: EntityEvent + Event<Trigger<'a> = EntityComponentTrigger>> Trigger<E>
    for EntityComponentTrigger
{
    unsafe fn trigger(
        &mut self,
        world: DeferredWorld,
        observers: &CachedObservers,
        trigger_context: &TriggerContext,
        event: &mut E,
    ) {
        let entity = event.event_target();
        // SAFETY:
        // - `observers` come from `world` and match the event type `E`, enforced by the call to `trigger`
        // - the passed in event pointer comes from `event`, which is an `Event`
        // - `trigger_context`'s event_key matches `E`, enforced by the call to `trigger`
        unsafe {
            self.trigger_internal(world, observers, event.into(), entity, trigger_context);
        }
    }
}

impl EntityComponentTrigger {
    /// # Safety
    /// - `observers` must come from the `world` [`DeferredWorld`]
    /// - `event` must point to an [`Event`] whose [`Event::Trigger`] is [`EntityComponentsTrigger`]
    /// - `trigger_context`'s [`TriggerContext::event_key`] must correspond to the `event` type.
    #[inline(never)]
    unsafe fn trigger_internal(
        &mut self,
        mut world: DeferredWorld,
        observers: &CachedObservers,
        mut event: PtrMut,
        entity: Entity,
        trigger_context: &TriggerContext,
    ) {
        // SAFETY:
        // - `observers` come from `world` and match the event type `E`, enforced by the call to `trigger`
        // - the passed in event pointer comes from `event`, which is an `Event`
        // - `trigger` is a matching trigger type, as it comes from `self`, which is the Trigger for `E`
        // - `trigger_context`'s event_key matches `E`, enforced by the call to `trigger`
        unsafe {
            trigger_entity_internal(
                world.reborrow(),
                observers,
                event.reborrow(),
                self.into(),
                entity,
                trigger_context,
            );
        }

        // Trigger observers watching for a specific component

        if let Some(component_observers) = observers.component_observers().get(&self.0) {
            for (observer, runner) in component_observers.global_observers() {
                // SAFETY:
                // - `observers` come from `world` and match the `event` type, enforced by the call to `trigger_internal`
                // - the passed in event pointer is an `Event`, enforced by the call to `trigger_internal`
                // - `trigger` is a matching trigger type, enforced by the call to `trigger_internal`
                // - `trigger_context`'s event_key matches `E`, enforced by the call to `trigger_internal`
                unsafe {
                    (runner)(
                        world.reborrow(),
                        *observer,
                        trigger_context,
                        event.reborrow(),
                        self.into(),
                    );
                }
            }

            if let Some(map) = component_observers
                .entity_component_observers()
                .get(&entity)
            {
                for (observer, runner) in map {
                    // SAFETY:
                    // - `observers` come from `world` and match the `event` type, enforced by the call to `trigger_internal`
                    // - the passed in event pointer is an `Event`, enforced by the call to `trigger_internal`
                    // - `trigger` is a matching trigger type, enforced by the call to `trigger_internal`
                    // - `trigger_context`'s event_key matches `E`, enforced by the call to `trigger_internal`
                    unsafe {
                        (runner)(
                            world.reborrow(),
                            *observer,
                            trigger_context,
                            event.reborrow(),
                            self.into(),
                        );
                    }
                }
            }
        }
    }
}
