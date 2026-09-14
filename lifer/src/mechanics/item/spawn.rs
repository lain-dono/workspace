use super::{Item, ItemAsset, ItemHandle};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use thiserror::Error;

pub type ItemQuery<'a> = Option<QueryState<(Entity, &'a ItemHandle), Without<Item>>>;

pub fn spawn_items_system(
    world: &mut World,
    mut query: Local<ItemQuery>,
    mut entities: Local<Vec<(Entity, AssetId<ItemAsset>)>>,
) {
    let query =
        query.get_or_insert_with(|| world.query_filtered::<(Entity, &ItemHandle), Without<Item>>());

    query.update_archetypes(world);

    entities.extend(query.iter(world).map(|(e, h)| (e, h.0.id())));

    let registry = world.resource::<AppTypeRegistry>().clone();
    let registry = registry.read();

    world.resource_scope(|world, assets: Mut<Assets<ItemAsset>>| {
        for (entity, id) in entities.drain(..) {
            let mut entity = world.entity_mut(entity);

            entity.insert(Item);

            let entity_id = entity.id();

            if let Err(err) = spawn_item(&registry, &assets, id, entity) {
                error!("spawning entities err: {:?}", err);
            }

            debug!("load item for {:?}", entity_id);
        }
    });
}

fn spawn_item(
    registry: &TypeRegistry,
    assets: &Assets<ItemAsset>,
    id: AssetId<ItemAsset>,
    mut entity: EntityWorldMut<'_>,
) -> Result<(), ItemSpawnError> {
    let err = ItemSpawnError::NonExistentItem { id };
    let asset = assets.get(id).ok_or(err)?;

    for reflect in &asset.components {
        let type_info = reflect.get_represented_type_info().ok_or_else(|| {
            ItemSpawnError::NoRepresentedType {
                type_path: reflect.reflect_type_path().to_string(),
            }
        })?;

        let registration = registry.get(type_info.type_id()).ok_or_else(|| {
            ItemSpawnError::UnregisteredButReflectedType {
                type_path: type_info.type_path().to_string(),
            }
        })?;

        let component = registration.data::<ReflectComponent>().ok_or_else(|| {
            ItemSpawnError::UnregisteredComponent {
                type_path: type_info.type_path().to_string(),
            }
        })?;

        // do not overwrite
        if !component.contains(EntityRef::from(&entity)) {
            component.insert(&mut entity, reflect.as_partial_reflect(), registry);
        }
    }

    Ok(())
}

/// Errors that can occur when spawning a item.
#[derive(Error, Debug)]
pub enum ItemSpawnError {
    /// Item contains a proxy without a represented type.
    #[error("item contains dynamic type `{type_path}` without a represented type. consider changing this using `set_represented_type`.")]
    NoRepresentedType {
        /// The dynamic instance type.
        type_path: String,
    },
    /// Item contains an unregistered type which has a `TypePath`.
    #[error(
        "item contains the reflected type `{type_path}` but it was not found in the type registry. \
        consider registering the type using `app.register_type::<T>()``"
    )]
    UnregisteredButReflectedType {
        /// The unregistered type.
        type_path: String,
    },

    /// Item contains an unregistered component type.
    #[error("item contains the unregistered component `{type_path}`. consider adding `#[reflect(Component)]` to your type")]
    UnregisteredComponent {
        /// Type of the unregistered component.
        type_path: String,
    },

    /// Item with the given id does not exist.
    #[error("item does not exist")]
    NonExistentItem {
        /// Id of the non-existent item.
        id: AssetId<ItemAsset>,
    },
}
