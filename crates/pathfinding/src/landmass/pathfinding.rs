use super::{
    Archipelago, CoordinateSystem, Island, NodeType,
    astar::{self, AStarProblem, PathStats},
    nav_data::{BoundaryLinkId, NodeRef},
    path::{BoundaryLinkSegment, IslandSegment, Path},
};
use bevy::{
    ecs::{entity::Entity, system::Query},
    math::Vec3,
    platform::collections::{HashMap, HashSet},
};
use std::borrow::Cow;

/// A concrete A* problem specifically for [`crate::Archipelago`]s.
struct ArchipelagoPathProblem<'a, 'w, 's, T: CoordinateSystem> {
    /// The navigation data to search.
    nav_data: &'a Archipelago<T>,
    islands: Query<'w, 's, (Entity, &'static Island<T>)>,

    /// The node the agent is starting from.
    start_node: NodeRef,
    /// The node the target is in.
    end_node: NodeRef,
    /// The center of the `end_node`. This is just a cached point for easy access.
    end_point: Vec3,
    /// The cheapest node type cost in [`Self::nav_data`]. This is cached once
    /// since it is constant for the whole problem.
    cheapest_node_type_cost: f32,
    /// Replacement costs for the `nav_data.node_type_to_cost`.
    override_node_type_to_cost: &'a HashMap<NodeType, f32>,
}

/// An action taken in the path.
#[derive(Clone, Copy)]
enum PathStep {
    /// Take the node connection at the specified edge index in the current node.
    NodeConnection(usize),
    /// Take the boundary link with the specified ID in the current node.
    BoundaryLink(BoundaryLinkId),
}

impl<T: CoordinateSystem> ArchipelagoPathProblem<'_, '_, '_, T> {
    /// Determines the cost of the node type corresponding to `type_index` in
    /// `island`.
    fn type_index_to_cost(&self, island: &Island<T>, type_index: u32) -> f32 {
        self.node_type_to_cost(island.type_index_to_node_type.get(&type_index).copied())
    }

    /// Returns the cost associated with `node_type`. Returns 1.0 if the `node_type`
    /// is unset.
    fn node_type_to_cost(&self, node_type: Option<NodeType>) -> f32 {
        let Some(node_type) = node_type else {
            return 1.0;
        };
        self.override_node_type_to_cost
            .get(&node_type)
            .copied()
            .unwrap_or_else(|| {
                self.nav_data
                    .node_type_cost(node_type)
                    .expect("The node type is valid in the NavigationData.")
            })
    }
}

impl<T: CoordinateSystem> AStarProblem for ArchipelagoPathProblem<'_, '_, '_, T> {
    type ActionType = PathStep;

    type StateType = NodeRef;

    fn initial_state(&self) -> Self::StateType {
        self.start_node
    }

    fn successors(&self, state: &Self::StateType) -> Vec<(f32, Self::ActionType, Self::StateType)> {
        let (_, island) = self.islands.get(state.island).unwrap();
        let polygon = &island.mesh.polygons[state.polygon];
        let boundary_links = self
            .nav_data
            .node_to_boundary_link_ids
            .get(state)
            .map_or(Cow::Owned(HashSet::new()), Cow::Borrowed);

        let current_cost = self.type_index_to_cost(island, polygon.type_index);

        polygon
            .vertices
            .iter()
            .enumerate()
            .filter_map(|(edge_index, (_, conn))| conn.as_ref().map(|conn| (edge_index, conn)))
            .filter_map(|(edge_index, conn)| {
                let type_index = island.mesh.polygons[conn.polygon].type_index;
                let target_cost = self.type_index_to_cost(island, type_index);
                if target_cost.is_finite() {
                    let [current_travel, target_travel] = conn.travel_distances;
                    let cost = current_travel * current_cost + target_travel * target_cost;
                    let node = NodeRef::new(state.island, conn.polygon);
                    Some((cost, PathStep::NodeConnection(edge_index), node))
                } else {
                    None
                }
            })
            .chain(boundary_links.iter().filter_map(|&link_id| {
                let link = self.nav_data.boundary_links.get(link_id).unwrap();
                let dst_cost = self.node_type_to_cost(link.destination_node_type);
                if dst_cost.is_finite() {
                    let (current_travel, dst_travel) = link.travel_distances;
                    let cost = current_travel * current_cost + dst_travel * dst_cost;
                    Some((cost, PathStep::BoundaryLink(link_id), link.destination))
                } else {
                    None
                }
            }))
            .collect()
    }

    fn heuristic(&self, state: &Self::StateType) -> f32 {
        let (_, island) = self.islands.get(state.island).unwrap();
        island
            .transform
            .apply(island.mesh.polygons[state.polygon].center)
            .distance(self.end_point)
            * self.cheapest_node_type_cost
    }

    fn is_goal_state(&self, state: &Self::StateType) -> bool {
        *state == self.end_node
    }
}

/// The results of pathfinding.
#[derive(Debug)]
pub(crate) struct PathResult {
    /// Statistics about the pathfinding process.
    pub(crate) stats: PathStats,
    /// The path if one was found.
    pub(crate) path: Option<Path>,
}

impl<T: CoordinateSystem> Archipelago<T> {
    /// Finds a path in `nav_data` from `start_node` to `end_node`. Node costs are
    /// overriden with `override_node_type_to_cost`. Returns an `Err` if no path was
    /// found.
    pub(crate) fn find_path(
        &mut self,
        islands: Query<(Entity, &'static Island<T>)>,
        start_node: NodeRef,
        end_node: NodeRef,
        override_node_type_to_cost: &HashMap<NodeType, f32>,
    ) -> PathResult {
        if !self.are_nodes_connected(islands, start_node, end_node) {
            let stats = PathStats { explored_nodes: 0 };
            return PathResult { stats, path: None };
        }

        let path_problem = ArchipelagoPathProblem {
            nav_data: self,
            islands,
            start_node,
            end_node,
            end_point: {
                let (_, island) = islands.get(end_node.island).unwrap();
                let point = island.mesh.polygons[end_node.polygon].center;
                island.transform.apply(point)
            },
            cheapest_node_type_cost: self
                .node_types()
                .map(|(key, cost)| {
                    // Replace any node types with their overriden value,
                    // but only if it was overriden.
                    override_node_type_to_cost
                        .get(&key)
                        .map_or(cost, |&cost| cost)
                })
                .filter(|cost| cost.is_finite())
                .chain(std::iter::once(1.0))
                .min_by(f32::total_cmp)
                .unwrap(),
            override_node_type_to_cost,
        };

        let path_result = astar::find_path(&path_problem);
        let Some(astar_path) = path_result.path else {
            let stats = path_result.stats;
            return PathResult { stats, path: None };
        };

        let mut output_path = Path {
            island_segments: vec![],
            boundary_link_segments: vec![],
        };

        output_path.island_segments.push(IslandSegment {
            island: start_node.island,
            corridor: vec![start_node.polygon],
            portal_edge_index: vec![],
        });

        for path_step in astar_path {
            let last_segment = output_path.island_segments.last_mut().unwrap();

            let previous_node = *last_segment.corridor.last().unwrap();

            match path_step {
                PathStep::NodeConnection(edge_index) => {
                    let nav_mesh = &islands.get(last_segment.island).unwrap().1.mesh;
                    let connectivity = nav_mesh.polygons[previous_node].vertices[edge_index]
                        .1
                        .unwrap();
                    last_segment.corridor.push(connectivity.polygon);
                    last_segment.portal_edge_index.push(edge_index);
                }
                PathStep::BoundaryLink(boundary_link) => {
                    let link = BoundaryLinkSegment {
                        starting_node: NodeRef::new(last_segment.island, previous_node),
                        boundary_link,
                    };

                    output_path.boundary_link_segments.push(link);

                    let boundary_link = self.boundary_links.get(boundary_link).unwrap();
                    output_path.island_segments.push(IslandSegment {
                        island: boundary_link.destination.island,
                        corridor: vec![boundary_link.destination.polygon],
                        portal_edge_index: vec![],
                    });
                }
            }
        }

        PathResult {
            stats: path_result.stats,
            path: Some(output_path),
        }
    }
}
