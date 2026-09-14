use crate::coords::{ThreeD, TwoD};
use crate::landmass::CoordinateSystem;
use crate::landmass::nav_data::NodeRef;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::{component::Component, entity::Entity};
use bevy::math::Vec3;
use bevy::transform::{components::Transform, helper::TransformHelper};

pub type Character2d = Character<TwoD>;
pub type Character3d = Character<ThreeD>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct CharacterId(pub Entity);

/// A non-agent character.
///
/// While agents are "managed" by the archipelago,
/// characters are only as obstacles to be avoided by agents.
#[derive(Component, Debug)]
pub struct Character<T: CoordinateSystem> {
    /// The current position of the character.
    pub position: T::Coord,
    /// The current velocity of the character.
    pub velocity: T::Coord,
    /// The radius of the character.
    pub radius: f32,

    pub(crate) sampled_position: Option<(Vec3, NodeRef)>,
}

impl<T: CoordinateSystem> Default for Character<T> {
    fn default() -> Self {
        Self {
            position: T::Coord::default(),
            velocity: T::Coord::default(),
            radius: 0.0,
            sampled_position: None,
        }
    }
}

impl<T: CoordinateSystem> Character<T> {
    #[must_use]
    pub fn new(radius: f32) -> Self {
        Self {
            position: T::Coord::default(),
            velocity: T::Coord::default(),
            radius,
            sampled_position: None,
        }
    }

    #[must_use]
    pub fn with_position(mut self, position: T::Coord) -> Self {
        self.position = position;
        self
    }

    #[must_use]
    pub fn with_velocity(mut self, velocity: T::Coord) -> Self {
        self.velocity = velocity;
        self
    }
}

/// Copies Bevy character states to their associated landmass character.
pub fn sync_character<T: crate::coords::CoordinateSystem>(
    characters: Query<(Entity, &mut Character<T>), With<Transform>>,
    transform_helper: TransformHelper,
) {
    for (entity, mut character) in characters {
        if let Ok(transform) = transform_helper.compute_global_transform(entity) {
            character.position = T::from_bevy_position(transform.translation());
        }
    }
}
