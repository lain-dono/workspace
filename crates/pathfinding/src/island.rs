use crate::bvh::BoundingBox;
use crate::coords::{ThreeD, TwoD};
use crate::landmass::{CoordinateSystem, NavMesh, NodeType};
use crate::nav_mesh::{NavMeshAsset, NavMeshHandle};
use bevy::asset::Assets;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Changed, With, Without};
use bevy::ecs::system::{Commands, Query, Res};
use bevy::platform::collections::HashMap;
use bevy::transform::components::Transform;
use bevy::transform::helper::TransformHelper;
use std::sync::Arc;

pub type Island2d = Island<TwoD>;
pub type Island3d = Island<ThreeD>;

pub type IslandId = Entity;

/// An Island in an Archipelago. Each island holds a navigation mesh.
#[derive(Component)]
pub struct Island<T: CoordinateSystem> {
    /// The transform from the Island's frame to the Archipelago's frame.
    pub(crate) transform: crate::bvh::Transform<T>,
    /// The navigation mesh for the island.
    pub(crate) mesh: Arc<NavMesh<T>>,
    /// A map from the type indices used by [`Self::nav_mesh`] to the
    /// [`NodeType`]s used in the [`crate::Archipelago`].
    pub(crate) type_index_to_node_type: HashMap<u32, NodeType>,
    /// The bounds of `nav_mesh` after being transformed by `transform`.
    pub(crate) transformed_bounds: BoundingBox,
}

impl<T: CoordinateSystem> Island<T> {
    /// Creates a new island. For details on the `type_index_to_node_type`
    /// argument, see [`Self::set_type_index_to_node_type`].
    pub fn new(transform: crate::bvh::Transform<T>, nav_mesh: Arc<NavMesh<T>>) -> Self {
        Self {
            transformed_bounds: nav_mesh.bounds.transform(&transform),
            transform,
            mesh: nav_mesh,
            type_index_to_node_type: HashMap::default(),
        }
    }

    #[must_use]
    pub fn with_node_types(self, type_index_to_node_type: HashMap<u32, NodeType>) -> Self {
        Self {
            type_index_to_node_type,
            ..self
        }
    }

    /// Gets the current transform of the island.
    pub fn transform(&self) -> crate::bvh::Transform<T> {
        self.transform
    }

    /// Sets the current transform of the island.
    pub fn set_transform(&mut self, transform: crate::bvh::Transform<T>) {
        self.transform = transform;
        self.transformed_bounds = self.mesh.bounds.transform(&self.transform);
    }

    /// Gets the current navigation mesh used by the island.
    pub fn nav_mesh(&self) -> Arc<NavMesh<T>> {
        self.mesh.clone()
    }

    /// Sets the navigation mesh of the island.
    pub fn set_nav_mesh(&mut self, nav_mesh: Arc<NavMesh<T>>) {
        self.mesh = nav_mesh;
        self.transformed_bounds = self.mesh.bounds.transform(&self.transform);
    }

    /// Gets the current `type_index_to_node_type` used by the island.
    pub fn type_index_to_node_type(&self) -> &HashMap<u32, NodeType> {
        &self.type_index_to_node_type
    }

    /// Sets the "translation" from the type indices used in the navigation mesh
    /// into [`NodeType`]s from the [`crate::landmass::Archipelago`]. Type indices without a
    /// corresponding node type will be treated as the "default" node type, which
    /// has a cost of 1.0. See [`crate::Archipelago::add_node_type`] for
    /// details on cost. [`NodeType`]s not present in the corresponding
    /// [`crate::Archipelago`] will cause a panic, so do not mix [`NodeType`]s
    /// across [`crate::Archipelago`]s.
    pub fn set_type_index_to_node_type(&mut self, type_index_to_node_type: HashMap<u32, NodeType>) {
        self.type_index_to_node_type = type_index_to_node_type;
    }
}

/// Ensures that the island transform and nav mesh are up to date.
pub(crate) fn sync_islands<T: crate::coords::CoordinateSystem>(
    handles: Query<(Entity, &NavMeshHandle<T>), (With<Transform>, Without<Island<T>>)>,
    islands: Query<(Entity, &mut Island<T>), Changed<Transform>>,
    transform_helper: TransformHelper,
    nav_meshes: Res<Assets<NavMeshAsset<T>>>,
    mut commands: Commands,
) {
    for (entity, mesh) in handles {
        let Some(mesh) = nav_meshes.get(&mesh.0) else {
            continue;
        };

        let Ok(transform) = transform_helper.compute_global_transform(entity) else {
            continue;
        };

        let transform = transform.compute_transform();
        let transform = crate::bvh::Transform::new(
            T::from_bevy_position(transform.translation),
            T::from_bevy_rotation(transform.rotation),
        );

        let bundle = Island::<T>::new(transform, mesh.nav_mesh.clone())
            .with_node_types(mesh.type_index_to_node_type.clone());

        commands.entity(entity).insert(bundle);
    }

    for (entity, mut island) in islands {
        let Ok(island_transform) = transform_helper.compute_global_transform(entity) else {
            continue;
        };

        let transform = island_transform.compute_transform();
        let transform = crate::bvh::Transform::new(
            T::from_bevy_position(transform.translation),
            T::from_bevy_rotation(transform.rotation),
        );

        if island.transform() != transform {
            island.set_transform(transform);
        }
    }
}
