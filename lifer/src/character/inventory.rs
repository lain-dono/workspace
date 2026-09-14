use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inventory>();
    }
}

#[derive(Component, Reflect)]
pub struct Inventory {
    pub container: Entity,
}
