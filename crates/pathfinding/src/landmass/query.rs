use super::{
    CoordinateSystem, Island, IslandId, NodeType,
    nav_data::{Archipelago, NodeRef},
    path::PathIndex,
    pathfinding::PathResult,
};
use crate::landmass::path::Waypoint;
use bevy::{
    ecs::{entity::Entity, system::Query},
    platform::collections::HashMap,
};
use thiserror::Error;

/// A point on the navigation meshes.
pub struct SampledPoint<T: CoordinateSystem> {
    /// The point on the navigation meshes.
    point: T::Coord,
    /// The node that the point is on.
    node_ref: NodeRef,
    /// The node type for `node_ref`.
    node_type: Option<NodeType>,
}

// Manual Clone impl for `SampledPoint` to avoid the Clone bound on T.
impl<T: CoordinateSystem> Clone for SampledPoint<T> {
    fn clone(&self) -> Self {
        Self {
            point: self.point,
            node_ref: self.node_ref,
            node_type: self.node_type,
        }
    }
}

impl<T: CoordinateSystem> SampledPoint<T> {
    /// Gets the point on the navigation meshes.
    pub fn point(&self) -> T::Coord {
        self.point
    }

    /// Gets the island the sampled point is on.
    pub fn island(&self) -> IslandId {
        self.node_ref.island
    }

    /// Gets the node type the sampled point is on.
    ///
    /// Returns None if the node type is the default node type.
    pub fn node_type(&self) -> Option<NodeType> {
        self.node_type
    }
}

/// An error from finding a path between two sampled points.
#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum FindPathError {
    #[error("The node type {0:?} had a cost of {1}, which is non-positive.")]
    NonPositiveNodeTypeCost(NodeType, f32),
    #[error("No path was found between the start and end points.")]
    NoPathFound,
}

/// An error while sampling a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum SamplePointError {
    #[error("The sample point is too far from any island.")]
    OutOfRange,
    #[error("The navigation data of the archipelago has been mutated since the last update.")]
    NavDataDirty,
}

impl<T: CoordinateSystem> Archipelago<T> {
    /// Finds the nearest point on the navigation meshes to (and within
    /// `distance_to_node` of) `point`.
    pub(crate) fn query_sample_point(
        islands: Query<(Entity, &Island<T>)>,
        point: T::Coord,
        point_sample_distance: T::SampleDistance,
    ) -> Result<SampledPoint<T>, SamplePointError> {
        let Some((point, node_ref)) =
            Archipelago::<T>::sample_point(islands, T::to_landmass(point), point_sample_distance)
        else {
            return Err(SamplePointError::OutOfRange);
        };

        let (_, island) = islands.get(node_ref.island).unwrap();
        let type_index = island.mesh.polygons[node_ref.polygon].type_index;

        Ok(SampledPoint {
            point: T::from_landmass(point),
            node_ref,
            node_type: island.type_index_to_node_type.get(&type_index).copied(),
        })
    }

    /// Finds a path from `start_point` and `end_point` along the navigation meshes.
    /// Only [`SampledPoint`]s from this archipelago are supported.
    /// This should only be used for querying (e.g., finding the walking distance to an object),
    /// not for controlling movement. For controlling movement, use agents.
    pub(crate) fn query_find_path(
        &mut self,
        islands: Query<(Entity, &'static Island<T>)>,
        start: &SampledPoint<T>,
        end: &SampledPoint<T>,
        override_node_type_costs: &HashMap<NodeType, f32>,
    ) -> Result<Vec<T::Coord>, FindPathError> {
        // This assert can actually be triggered. This can happen if a user samples
        // points from one archipelago, but finds a path in a **different**
        // archipelago. This seems almost malicious though, so I don't think we should
        // handle it at all. I'd rather the "wins" we get from avoiding
        // double-sampling (in cases where the user samples a point to check for
        // validity and then finds a path).

        for (&node_type, &cost) in override_node_type_costs {
            if cost <= 0.0 {
                return Err(FindPathError::NonPositiveNodeTypeCost(node_type, cost));
            }
        }

        let PathResult {
            path: Some(path), ..
        } = self.find_path(
            islands,
            start.node_ref,
            end.node_ref,
            override_node_type_costs,
        )
        else {
            return Err(FindPathError::NoPathFound);
        };

        let mut current = Waypoint::new(PathIndex::new(0, 0), T::to_landmass(start.point));

        let last = Waypoint::new(path.last_index(), T::to_landmass(end.point));

        let mut path_points = vec![start.point];
        while current.index != last.index {
            current = path.find_next_point_in_straight_path(islands, self, current, last);
            path_points.push(T::from_landmass(current.point));
        }

        Ok(path_points)
    }
}
