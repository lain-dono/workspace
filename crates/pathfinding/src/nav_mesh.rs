use crate::coords::CoordinateSystem;
use crate::landmass::{NavMesh, NavMeshBuilder, NodeType};
use bevy::asset::{Asset, Handle};
use bevy::ecs::component::Component;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::platform::collections::HashMap;
use bevy::reflect::TypePath;
use std::sync::Arc;

/// An asset holding a `landmass` nav mesh.
#[derive(Asset, TypePath)]
pub struct NavMeshAsset<T: CoordinateSystem> {
    /// The nav mesh data.
    pub nav_mesh: Arc<NavMesh<T>>,
    /// A map from the type indices used by [`Self::nav_mesh`] to the
    /// [`NodeType`]s used in the [`crate::Archipelago`]. Type indices not
    /// present in this map are implicitly assigned the "default" node type,
    /// which always has a cost of 1.0.
    pub type_index_to_node_type: HashMap<u32, NodeType>,
}

/// A handle to a navigation mesh for an [`Island`].
#[derive(Component, Clone, Debug)]
pub struct NavMeshHandle<T: CoordinateSystem>(pub Handle<NavMeshAsset<T>>);

impl<T: CoordinateSystem> Default for NavMeshHandle<T> {
    fn default() -> Self {
        Self(Handle::default())
    }
}

/// A conversion error for Bevy meshes to `landmass` nav meshes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertMeshError {
    /// The Bevy mesh had an unsupported topology. Only
    /// [`PrimitiveTopology::TriangleList`] is supported.
    InvalidTopology,
    /// The Bevy mesh does not have a [`Mesh::ATTRIBUTE_POSITION`].
    MissingVertexPositions,
    /// The mesh has an unsupported type for its indices. This may either be
    /// because the mesh has **no** indices, or an unknown index format is used
    /// (currently u16 and u32 are both supported). Indices are necessary since
    /// they inform connectivity.
    WrongTypeForIndices,
}

impl<T: CoordinateSystem> NavMeshBuilder<T> {
    /// Converts a Bevy Mesh to a landmass `NavigationMesh`. This is done naively -
    /// each triangle forms a single polygon in the navigation mesh, which can cause
    /// strange paths to form (agents may take turns inside if wide open regions).
    /// This function is provided as a convenience, and a better method for
    /// generating navigation meshes should be used.
    pub fn from_bevy_mesh(
        mesh: &Mesh,
        from_mesh_vertex: impl Fn([f32; 3]) -> T::Coord,
    ) -> Result<Self, ConvertMeshError> {
        let PrimitiveTopology::TriangleList = mesh.primitive_topology() else {
            return Err(ConvertMeshError::InvalidTopology);
        };

        let Some(values) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            return Err(ConvertMeshError::MissingVertexPositions);
        };

        let vertices = match values {
            VertexAttributeValues::Float32x3(vertices) => {
                vertices.iter().copied().map(from_mesh_vertex).collect()
            }
            _ => panic!("Mesh POSITION must be Float32x3"),
        };

        let polygons = match mesh.indices() {
            Some(Indices::U16(indices)) => {
                assert!(indices.len() % 3 == 0);
                let mut polygons = Vec::with_capacity(indices.len() / 3);
                for i in (0..indices.len()).step_by(3) {
                    polygons.push(vec![
                        indices[i] as u32,
                        indices[i + 1] as u32,
                        indices[i + 2] as u32,
                    ]);
                }
                polygons
            }
            Some(Indices::U32(indices)) => {
                assert!(indices.len() % 3 == 0);
                let mut polygons = Vec::with_capacity(indices.len() / 3);
                for i in (0..indices.len()).step_by(3) {
                    polygons.push(vec![indices[i], indices[i + 1], indices[i + 2]]);
                }
                polygons
            }
            _ => return Err(ConvertMeshError::WrongTypeForIndices),
        };

        Ok(Self {
            vertices,
            polygon_type_indices: (0..polygons.len()).map(|_| 0).collect(),
            polygons,
        })
    }
}
