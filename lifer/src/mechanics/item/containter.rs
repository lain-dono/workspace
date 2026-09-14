use super::{ItemAsset, ItemHandle};
use bevy::ecs::{
    query::{QueryData, ReadOnlyQueryData},
    system::{lifetimeless::Read, SystemParam},
};
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Container;

#[derive(Bundle)]
pub struct ContainerBundle {
    pub container: Container,

    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,

    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
}

impl Default for ContainerBundle {
    fn default() -> Self {
        Self {
            container: Container,

            visibility: Visibility::Hidden,
            inherited_visibility: InheritedVisibility::HIDDEN,
            view_visibility: ViewVisibility::HIDDEN,

            transform: Transform::IDENTITY,
            global_transform: GlobalTransform::IDENTITY,
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct PutInContainer {
    pub item: Entity,
    pub to: Entity,
}

pub fn put_in_container(mut commands: Commands, mut events: ResMut<Events<PutInContainer>>) {
    for PutInContainer { item, to } in events.drain() {
        commands.entity(to).add_child(item);
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MoveBetweenContainers {
    pub item: Entity,
    pub from: Entity,
    pub to: Entity,
}

pub fn move_between(mut commands: Commands, mut events: ResMut<Events<MoveBetweenContainers>>) {
    for MoveBetweenContainers { item, from, to } in events.drain() {
        commands.entity(to).add_child(item);
        commands.entity(from).remove_children(&[item]);
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct TakeOutOfContainer {
    pub item: Entity,
    pub from: Entity,
}

pub fn take_out_of_container(
    mut commands: Commands,
    mut events: ResMut<Events<TakeOutOfContainer>>,
) {
    for TakeOutOfContainer { item, from } in events.drain() {
        commands.entity(from).remove_children(&[item]);
    }
}

#[derive(SystemParam)]
pub struct ReadContainer<'w, 's, Q: ReadOnlyQueryData + 'static> {
    pub query: Query<'w, 's, (Read<ItemHandle>, Q)>,
}

impl<Q: ReadOnlyQueryData + 'static> ReadContainer<'_, '_, Q> {
    pub fn get<R>(
        &self,
        item: &Handle<ItemAsset>,
        container: Option<&Children>,
        map: impl FnOnce(Q::Item<'_>) -> R,
    ) -> Option<R> {
        for &child_entity in container? {
            if let Ok((ItemHandle(asset), consumable)) = self.query.get(child_entity) {
                if asset == item {
                    return Some(map(consumable));
                }
            }
        }
        None
    }

    pub fn get_or<R>(
        &self,
        item: &Handle<ItemAsset>,
        container: Option<&Children>,
        default: R,
        map: impl FnOnce(Q::Item<'_>) -> R,
    ) -> R {
        self.get(item, container, map).unwrap_or(default)
    }
}

#[derive(SystemParam)]
pub struct WriteContainer<'w, 's, Q: QueryData + 'static> {
    pub query: Query<'w, 's, (Read<ItemHandle>, Q)>,
}

impl<'s, Q: QueryData + 'static> WriteContainer<'_, 's, Q> {
    pub fn as_readonly(&self) -> ReadContainer<'_, 's, Q::ReadOnly> {
        ReadContainer {
            query: self.query.as_readonly(),
        }
    }

    pub fn get<R>(
        &mut self,
        container: Option<&Children>,
        item: &Handle<ItemAsset>,
        map: impl FnOnce(Q::Item<'_>) -> R,
    ) -> Option<R> {
        for &child_entity in container? {
            if let Ok((ItemHandle(asset), consumable)) = self.query.get_mut(child_entity) {
                if asset == item {
                    return Some(map(consumable));
                }
            }
        }
        None
    }

    pub fn transfer<R>(
        &mut self,
        container: Option<&Children>,
        from: &Handle<ItemAsset>,
        to: &Handle<ItemAsset>,
        map: impl FnOnce(Q::Item<'_>, Q::Item<'_>) -> R,
    ) -> Option<R> {
        let mut from_entity = None;
        let mut to_entity = None;

        for &entity in container? {
            if let Ok((ItemHandle(asset), _)) = self.query.get_mut(entity) {
                if asset == from {
                    from_entity = Some(entity);
                }
                if asset == to {
                    to_entity = Some(entity);
                }
            }
        }

        from_entity.zip(to_entity).map(|(from, to)| {
            let [(_, from), (_, to)] = self.query.get_many_mut([from, to]).unwrap();
            map(from, to)
        })
    }
}
