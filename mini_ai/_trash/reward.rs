use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub struct Reward<T> {
    value: f32,
    marker: PhantomData<T>,
    // min: f32,
    // max: f32,
    // scale: Option<ComponentId>,
}

impl<T: TypePath> std::fmt::Debug for Reward<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Need<{}>", T::short_type_path()))
            .field(&self.value)
            .finish()
    }
}

impl<T> From<Reward<T>> for f32 {
    fn from(need: Reward<T>) -> Self {
        need.value
    }
}

impl<T> From<&Reward<T>> for f32 {
    fn from(need: &Reward<T>) -> Self {
        need.value
    }
}

impl<T> Reward<T> {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }
}
