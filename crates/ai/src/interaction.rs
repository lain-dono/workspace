use bevy::ecs::{
    bundle::Bundle,
    component::Component,
    entity::{Entity, EntityHashSet, hash_set},
    prelude::{ReflectComponent, ReflectFromWorld},
    relationship::RelatedSpawnerCommands,
    system::EntityCommands,
    world::{FromWorld, World},
};
use bevy::reflect::Reflect;

#[derive(Clone, Copy, Debug)]
pub struct Ad {
    pub idx: usize,
    pub fix: usize,
    pub min: f32,
    pub max: f32,
}

impl Ad {
    #[must_use]
    pub const fn new(idx: usize, min: f32, max: f32) -> Self {
        let fix = usize::MAX;
        Self { idx, fix, min, max }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Interaction {
    pub advertising: Vec<Ad>,
    pub attenuation: f32,
}

impl Interaction {
    #[must_use]
    pub fn single(attenuation: f32, ad: Ad) -> Self {
        Self {
            advertising: vec![ad],
            attenuation,
        }
    }
}

#[derive(Component, Clone, PartialEq, Eq, Debug, Reflect)]
#[reflect(Component, PartialEq, Debug, FromWorld, Clone)]
#[relationship(relationship_target = Interactive)]
pub struct InteractionTarget {
    pub target: Entity,
}

// TODO: We need to impl either FromWorld or Default so Interaction can be registered as Reflect.
// This is because Reflect deserialize by creating an instance and apply a patch on top.
// However Ineraction should only ever be set with a real user-defined entity.  Its worth looking into
// better ways to handle cases like this.
impl FromWorld for InteractionTarget {
    #[inline]
    fn from_world(_world: &mut World) -> Self {
        Self {
            target: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Default)]
#[relationship_target(relationship = InteractionTarget, linked_spawn)]
pub struct Interactive(EntityHashSet);

#[allow(clippy::into_iter_without_iter)]
impl<'a> IntoIterator for &'a Interactive {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = hash_set::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub trait InteractiveEntity {
    fn with_interaction(&mut self, bundle: impl Bundle) -> &mut Self;

    fn with_interactions(
        &mut self,
        func: impl FnOnce(&mut RelatedSpawnerCommands<'_, InteractionTarget>),
    ) -> &mut Self;

    fn add_interaction(&mut self, entity: Entity) -> &mut Self;
    fn add_interactions(&mut self, entities: &[Entity]) -> &mut Self;
}

impl InteractiveEntity for EntityCommands<'_> {
    fn with_interaction(&mut self, bundle: impl Bundle) -> &mut Self {
        self.with_related::<InteractionTarget>(bundle)
    }

    fn with_interactions(
        &mut self,
        func: impl FnOnce(&mut RelatedSpawnerCommands<'_, InteractionTarget>),
    ) -> &mut Self {
        self.with_related_entities::<InteractionTarget>(func)
    }

    fn add_interaction(&mut self, entity: Entity) -> &mut Self {
        self.add_one_related::<InteractionTarget>(entity)
    }

    fn add_interactions(&mut self, entities: &[Entity]) -> &mut Self {
        self.add_related::<InteractionTarget>(entities)
    }
}
