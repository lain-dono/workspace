use bevy::prelude::*;

pub trait SpawnEntity {
    fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity;

    fn spawn_many<const N: usize, B: Bundle>(&mut self, bundles: [B; N]) -> [Entity; N];
}

impl SpawnEntity for Commands<'_, '_> {
    fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity {
        self.spawn(bundle).id()
    }

    fn spawn_many<const N: usize, B: Bundle>(&mut self, bundles: [B; N]) -> [Entity; N] {
        bundles.map(|bundle| self.spawn(bundle).id())
    }
}
