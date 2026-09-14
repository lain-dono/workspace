use bevy::prelude::*;

mod asset;
mod consumable;
mod containter;
mod spawn;

pub use self::{
    asset::{ItemAsset, ItemAssetLoader, ItemAssetLoaderError, ItemHandle},
    consumable::{Consumable, ReadConsumable, WriteConsumable},
    containter::{Container, ContainerBundle, ReadContainer, WriteContainer},
    spawn::ItemSpawnError,
};

#[derive(Component)]
pub struct Item;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ItemName(pub String);

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ItemAsset>()
            .init_asset_loader::<ItemAssetLoader>()
            .add_event::<self::containter::PutInContainer>()
            .add_event::<self::containter::MoveBetweenContainers>()
            .add_event::<self::containter::TakeOutOfContainer>()
            .register_type::<Consumable>()
            .register_type::<ItemName>()
            .add_systems(SpawnScene, self::spawn::spawn_items_system)
            .add_systems(
                PostUpdate,
                (
                    self::containter::put_in_container,
                    self::containter::move_between,
                    self::containter::take_out_of_container,
                )
                    .chain(),
            );
    }
}
