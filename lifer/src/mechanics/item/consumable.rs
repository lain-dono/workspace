use super::{ReadContainer, WriteContainer};
use bevy::ecs::system::lifetimeless::{Read, Write};
use bevy::prelude::*;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Consumable {
    pub current: f32,
    pub maximum: f32,
}

impl Consumable {
    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.maximum
    }

    pub fn take(&mut self) -> f32 {
        let value = self.current;
        self.current = 0.0;
        value
    }
}

pub type ReadConsumable<'w, 's> = ReadContainer<'w, 's, Read<Consumable>>;
pub type WriteConsumable<'w, 's> = WriteContainer<'w, 's, Write<Consumable>>;
