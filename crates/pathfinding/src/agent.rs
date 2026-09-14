use crate::coords::{CoordinateSystem, ThreeD, TwoD};
use crate::landmass::{self, Agent, NodeType};
use bevy::ecs::{
    change_detection::DetectChanges, component::Component, entity::Entity, query::With,
    system::Query, world::Ref,
};
use bevy::platform::collections::HashMap;
use bevy::transform::{components::Transform, helper::TransformHelper};
use std::ops::Deref;

pub type Agent2d = Agent<TwoD>;
pub type Agent3d = Agent<ThreeD>;

/// The state of an agent.
///
/// This does not control an agent's state and is just used to report the
/// agent's state.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AgentState {
    /// The agent is idle, due to not having a target. Note this does not mean
    /// that they are motionless. An agent will still avoid nearby agents.
    #[default]
    Idle,
    /// The agent has reached their target. The agent may resume moving if the
    /// target moves or otherwise changes.
    ReachedTarget,
    /// The agent has a path and is moving towards their target.
    Moving,
    /// The agent is not on a nav mesh.
    AgentNotOnNavMesh,
    /// The target is not on a nav mesh.
    TargetNotOnNavMesh,
    /// The agent has a target but cannot find a path to it.
    NoPath,
}

/// The condition to consider the agent as having reached its target.
/// When this condition is satisfied, the agent will stop moving.
#[derive(Clone, Copy, Debug)]
pub enum ReachedCondition {
    /// The target is reached if it is within the provided (Euclidean) distance
    /// of the agent. Useful if the target is surrounded by small obstacles
    /// which don't need to be navigated around (e.g. the agent just needs to
    /// be close enough to shoot at the target, which is surrounded by cover).
    /// Alternatively, if the distance is low, this can simply mean "when the
    /// agent is really close to the target".
    Distance(Option<f32>),
    /// The target is reached if it is "visible" (there is a straight line from
    /// the agent to the target), and the target is within the provided
    /// (Euclidean) distance of the agent. Useful if the agent should be able
    /// to see the target (e.g. a companion character should remain visible to
    /// the player, but should ideally not stand too close).
    VisibleAtDistance(Option<f32>),
    /// The target is reached if the "straight line" path from the agent to the
    /// target is less than the provided distance. "Straight line" path means if
    /// the agent's path goes around a corner, the distance will be computed
    /// going around the corner. This can be more computationally expensive, as
    /// the straight line path must be computed every update. Useful for agents
    /// that care about the actual walking distance to the target.
    StraightPathDistance(Option<f32>),
}

impl Default for ReachedCondition {
    fn default() -> Self {
        Self::Distance(None)
    }
}

#[derive(Component, Default, Debug)]
pub struct AgentNodeTypeCostOverrides(HashMap<NodeType, f32>);

impl Deref for AgentNodeTypeCostOverrides {
    type Target = HashMap<NodeType, f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AgentNodeTypeCostOverrides {
    /// Sets the node type cost for this agent to `cost`. Returns false if the
    /// cost is <= 0.0. Otherwise returns true.
    pub fn set_node_type_cost(&mut self, node_type: NodeType, cost: f32) -> bool {
        if cost <= 0.0 {
            return false;
        }
        self.0.insert(node_type, cost);
        true
    }

    /// Removes the override cost for `node_type`. Returns true if `node_type` was
    /// overridden, false otherwise.
    pub fn remove_override(&mut self, node_type: NodeType) -> bool {
        self.0.remove(&node_type).is_some()
    }
}

/// The current target of the entity. Note this can be set by either reinserting
/// the component, or dereferencing:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use pathfinding::prelude::AgentTarget3d;
/// fn clear_targets(mut targets: Query<&mut AgentTarget3d>) {
///   for mut target in targets.iter_mut() {
///     *target = AgentTarget3d::None;
///   }
/// }
/// ```
#[derive(Component)]
pub enum AgentTarget<T: CoordinateSystem> {
    None,
    Point(T::Coord),
    Entity(Entity),
}

impl<T: CoordinateSystem> Default for AgentTarget<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: CoordinateSystem<Coord: std::fmt::Debug>> std::fmt::Debug for AgentTarget<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Point(arg0) => f.debug_tuple("Point").field(arg0).finish(),
            Self::Entity(arg0) => f.debug_tuple("Entity").field(arg0).finish(),
        }
    }
}

impl<T: CoordinateSystem<Coord: PartialEq>> PartialEq for AgentTarget<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Point(l0), Self::Point(r0)) => l0 == r0,
            (Self::Entity(l0), Self::Entity(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<T: CoordinateSystem<Coord: Eq>> Eq for AgentTarget<T> {}

impl<T: CoordinateSystem> AgentTarget<T> {
    /// Converts an agent target to a concrete world position.
    fn to_point(&self, transform_helper: &TransformHelper) -> Option<T::Coord> {
        match *self {
            Self::Point(point) => Some(point),
            Self::Entity(entity) => transform_helper
                .compute_global_transform(entity)
                .ok()
                .map(|transform| T::from_bevy_position(transform.translation())),
            Self::None => None,
        }
    }
}

#[cfg(feature = "debug-avoidance")]
/// If inserted on an agent, it will record avoidance data that can later be
/// visualized with [`crate::debug::draw_avoidance_data`].
#[derive(Component, Clone, Copy, Debug)]
pub struct KeepAvoidanceData;

#[cfg(feature = "debug-avoidance")]
type HasKeepAvoidanceData = bevy::ecs::query::Has<KeepAvoidanceData>;
#[cfg(not(feature = "debug-avoidance"))]
type HasKeepAvoidanceData = ();

/// Ensures the "input state" (position, velocity, etc) of every Bevy agent
/// matches its `landmass` counterpart.
#[allow(clippy::type_complexity)]
pub(crate) fn sync_agent<T: CoordinateSystem>(
    agents: Query<
        (
            Entity,
            &mut landmass::Agent<T>,
            Option<&AgentTarget<T>>,
            Option<Ref<AgentNodeTypeCostOverrides>>,
            HasKeepAvoidanceData,
        ),
        With<Transform>,
    >,
    transform_helper: TransformHelper,
) {
    for (agent_entity, mut dst, target, cost_overrides, keep_avoidance_data) in agents {
        let Ok(transform) = transform_helper.compute_global_transform(agent_entity) else {
            continue;
        };

        dst.position = T::from_bevy_position(transform.translation());
        dst.current_target = target.and_then(|target| target.to_point(&transform_helper));

        if let Some(node_type_cost_overrides) = cost_overrides {
            if !node_type_cost_overrides.is_changed() {
                continue;
            }

            for (node_type, _) in dst.node_type_cost_overrides().collect::<Vec<_>>() {
                if node_type_cost_overrides.0.contains_key(&node_type) {
                    continue;
                }
                dst.remove_overridden_node_type_cost(node_type);
            }

            for (&node_type, &cost) in &node_type_cost_overrides.0 {
                assert!(dst.override_node_type_cost(node_type, cost));
            }
        } else {
            for (node_type, _) in dst.node_type_cost_overrides().collect::<Vec<_>>() {
                dst.remove_overridden_node_type_cost(node_type);
            }
        }

        #[cfg(feature = "debug-avoidance")]
        {
            dst.keep_avoidance_data = keep_avoidance_data;
        }

        #[cfg(not(feature = "debug-avoidance"))]
        #[expect(clippy::let_unit_value)]
        let _ = keep_avoidance_data;
    }
}
