use super::{
    Archipelago, CoordinateSystem, Island, IslandId, NodeType,
    nav_data::{BoundaryLinkId, NodeRef},
    path::{Path, PathIndex, Waypoint},
};
use crate::agent::{AgentState, ReachedCondition};
use bevy::{
    ecs::{component::Component, entity::Entity, system::Query},
    math::Vec3,
    platform::collections::{HashMap, HashSet},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct AgentId(pub Entity);

/// The determination of what to do in regards to an agent's path.
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum RepathResult {
    /// Do nothing.
    DoNothing,
    /// The existing path should be followed. Stores the index in a path for the
    /// first portal and the target point.
    FollowPath(PathIndex, PathIndex),
    /// Clear the path and don't repath, since there is no longer a target.
    ClearPathNoTarget,
    /// Clear the path and don't repath, since the agent is not on a valid node.
    ClearPathBadAgent,
    /// Clear the path and don't repath, since the target is not on a valid node.
    ClearPathBadTarget,
    /// Recompute the path.
    NeedsRepath,
}

/// An agent in an archipelago.
#[derive(Component)]
pub struct Agent<T: CoordinateSystem> {
    /// The current position of the agent.
    pub position: T::Coord,
    /// The current velocity of the agent.
    pub current_velocity: T::Coord,
    /// The desired velocity of the agent to move towards its goal.
    pub(crate) desired_velocity: T::Coord,
    /// The radius of the agent.
    pub radius: f32,

    /// The speed the agent prefers to move at. This should often be set lower
    /// than the [`Self::max_speed`] to allow the agent to "speed up" in order to
    /// get out of another agent's way.
    pub desired_speed: f32,

    /// The maximum speed that the agent can move at.
    pub max_speed: f32,

    /// The current target to move towards.
    ///
    /// Modifying this every update is fine.
    /// Paths will be reused for target points near each other if possible.
    /// However, swapping between two distant targets every update can be
    /// detrimental to be performance.
    pub current_target: Option<T::Coord>,

    /// The condition to test for reaching the target.
    pub reached_condition: ReachedCondition,

    #[cfg(feature = "debug-avoidance")]
    /// If true, avoidance debug data will be stored during update iterations.
    /// This can later be used for visualization.
    pub keep_avoidance_data: bool,
    #[cfg(feature = "debug-avoidance")]
    /// The avoidance data from the most recent update iteration. Only populated
    /// if [`Self::keep_avoidance_data`] is true.
    pub(crate) avoidance_data: Option<dodgy_2d::debug::DebugData>,

    /// Overrides for the "default" costs of each [`NodeType`].
    pub(crate) override_node_type_to_cost: HashMap<NodeType, f32>,
    /// The current path of the agent. None if a path is unavailable or a new
    /// path has not been computed yet (i.e., no path).
    pub(crate) current_path: Option<Path>,
    /// The state of the agent.
    pub(crate) state: AgentState,

    pub(crate) sampled_position: Option<(Vec3, NodeRef)>,
    pub(crate) sampled_target: Option<(Vec3, NodeRef)>,

    pub(crate) path_follow: Option<(PathIndex, PathIndex)>,
}

impl<T: CoordinateSystem> Agent<T> {
    /// Creates a new agent.
    pub fn new(position: T::Coord, radius: f32, desired_speed: f32, max_speed: f32) -> Self {
        Self {
            position,
            current_velocity: T::Coord::default(),
            desired_velocity: T::Coord::default(),
            radius,
            desired_speed,
            max_speed,
            current_target: None,
            reached_condition: ReachedCondition::Distance(None),

            #[cfg(feature = "debug-avoidance")]
            keep_avoidance_data: false,
            #[cfg(feature = "debug-avoidance")]
            avoidance_data: None,

            override_node_type_to_cost: HashMap::new(),
            current_path: None,
            state: AgentState::Idle,

            sampled_position: None,
            sampled_target: None,
            path_follow: None,
        }
    }

    #[must_use]
    pub fn with_reached_condition(self, reached_condition: ReachedCondition) -> Self {
        Self {
            reached_condition,
            ..self
        }
    }

    #[must_use]
    pub fn with_velocity(self, current_velocity: T::Coord) -> Self {
        Self {
            current_velocity,
            ..self
        }
    }

    #[must_use]
    pub fn with_target(self, target: T::Coord) -> Self {
        Self {
            current_target: Some(target),
            ..self
        }
    }

    /// Sets the node type cost for this agent to `cost`.
    ///
    /// Returns true if the cost is > 0.0, false otherwise.
    pub fn override_node_type_cost(&mut self, node_type: NodeType, cost: f32) -> bool {
        if cost > 0.0 {
            self.override_node_type_to_cost.insert(node_type, cost);
            true
        } else {
            false
        }
    }

    /// Removes the override cost for `node_type`.
    ///
    /// Returns true if `node_type` was overridden, false otherwise.
    pub fn remove_overridden_node_type_cost(&mut self, node_type: NodeType) -> bool {
        self.override_node_type_to_cost.remove(&node_type).is_some()
    }

    /// Returns the currently overriden node type costs.
    pub fn node_type_cost_overrides(&self) -> impl Iterator<Item = (NodeType, f32)> + '_ {
        self.override_node_type_to_cost
            .iter()
            .map(|(&node, &cost)| (node, cost))
    }

    /// Returns the desired velocity.
    ///
    /// This will only be updated if `update` was called on the associated [`crate::Archipelago`].
    pub fn desired_velocity(&self) -> T::Coord {
        self.desired_velocity
    }

    /// Returns the state of the agent.
    ///
    /// This will only be updated if `update` was called on the associated [`crate::Archipelago`].
    pub fn state(&self) -> AgentState {
        self.state
    }

    /// Determines if this agent has reached its target.
    ///
    /// `next_waypoint` and `target_waypoint` are formatted as an index into the `path`
    /// and the point of the waypoint.
    /// `next_waypoint` is the next waypoint on the way to the target.
    /// `target_waypoint` is the final waypoint that corresponds to the target.
    pub(crate) fn has_reached_target(
        &self,
        islands: Query<(Entity, &Island<T>)>,
        path: &Path,
        nav: &Archipelago<T>,
        next: Waypoint,
        target: Waypoint,
    ) -> bool {
        let position = T::to_landmass(self.position);
        match self.reached_condition {
            ReachedCondition::Distance(distance) => {
                let distance = distance.unwrap_or(self.radius);
                position.distance_squared(target.point) < distance * distance
            }
            ReachedCondition::VisibleAtDistance(distance) => {
                let distance = distance.unwrap_or(self.radius);
                next.index == target.index
                    && position.distance_squared(next.point) < distance * distance
            }
            ReachedCondition::StraightPathDistance(distance) => {
                let distance = distance.unwrap_or(self.radius);

                // Check Euclidean distance first so we don't do the expensive path
                // following if the agent is not even close.
                if position.distance_squared(target.point) > distance * distance {
                    return false;
                }

                // If the next waypoint is the target point, then we've already
                // computed the straight line distance and it is below the limit.
                if next.index == target.index {
                    return true;
                }

                let mut straight_line_distance = position.distance(next.point);
                let mut current = next;

                while current.index != target.index && straight_line_distance < distance {
                    let next = path.find_next_point_in_straight_path(islands, nav, current, target);
                    straight_line_distance += current.point.distance(next.point);
                    current = next;
                }

                straight_line_distance < distance
            }
        }
    }

    pub(crate) fn need_repath(
        &self,
        agent: Option<NodeRef>,
        target: Option<NodeRef>,
        invalidated_boundary_links: &HashSet<BoundaryLinkId>,
        invalidated_islands: &HashSet<IslandId>,
    ) -> RepathResult {
        if self.current_target.is_none() {
            return if self.current_path.is_some() {
                RepathResult::ClearPathNoTarget
            } else {
                RepathResult::DoNothing
            };
        }

        let Some(agent) = agent else {
            return RepathResult::ClearPathBadAgent;
        };
        let Some(target) = target else {
            return RepathResult::ClearPathBadTarget;
        };
        let Some(current_path) = &self.current_path else {
            return RepathResult::NeedsRepath;
        };
        if !current_path.is_valid(invalidated_boundary_links, invalidated_islands) {
            return RepathResult::NeedsRepath;
        }
        let Some(agent_in_path) = current_path.find_index_of_node(agent) else {
            return RepathResult::NeedsRepath;
        };
        let Some(target_in_path) = current_path.find_index_of_node_rev(target) else {
            return RepathResult::NeedsRepath;
        };
        if agent_in_path > target_in_path {
            return RepathResult::NeedsRepath;
        }

        RepathResult::FollowPath(agent_in_path, target_in_path)
    }
}
