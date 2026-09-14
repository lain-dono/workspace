use bevy::{ecs::query::QueryFilter, prelude::*};
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub struct Need<T> {
    value: f32,
    marker: PhantomData<T>,
}

impl<T: TypePath> std::fmt::Debug for Need<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Need<{}>", T::short_type_path()))
            .field(&self.value)
            .finish()
    }
}

impl<T> From<Need<T>> for f32 {
    fn from(need: Need<T>) -> Self {
        need.value
    }
}

impl<T> From<&Need<T>> for f32 {
    fn from(need: &Need<T>) -> Self {
        need.value
    }
}

impl<T> From<&mut Need<T>> for f32 {
    fn from(need: &mut Need<T>) -> Self {
        need.value
    }
}

impl<T> Need<T> {
    pub const fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 100.0),
            marker: PhantomData,
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(0.0, 100.0);
    }
}

impl<T> std::ops::AddAssign<f32> for Need<T> {
    fn add_assign(&mut self, rhs: f32) {
        self.set(self.value + rhs);
    }
}

impl<T> std::ops::SubAssign<f32> for Need<T> {
    fn sub_assign(&mut self, rhs: f32) {
        self.set(self.value - rhs);
    }
}

#[derive(Component)]
pub struct NeedDecay<T> {
    decay: f32,
    marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> NeedDecay<T> {
    pub fn new(decay: f32) -> Self {
        Self {
            decay,
            marker: PhantomData,
        }
    }

    pub fn system<F: QueryFilter>(time: Res<Time>, mut thirsts: Query<(&mut Need<T>, &Self), F>) {
        for (mut need, &Self { decay, .. }) in thirsts.iter_mut() {
            *need += decay * time.delta_secs();
        }
    }
}
