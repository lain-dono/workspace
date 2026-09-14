pub(crate) mod agent;
pub(crate) mod astar;
pub(crate) mod avoidance;
pub(crate) mod coords;
pub(crate) mod geometry;
pub(crate) mod nav_data;
pub(crate) mod nav_mesh;
pub(crate) mod path;
pub(crate) mod pathfinding;
pub(crate) mod query;

use bevy::ecs::resource::Resource;

pub mod debug;

pub use self::agent::{Agent, AgentId};
pub use self::coords::{
    CoordinateSystem, FromAgentRadius, PointSampleDistance, PointSampleDistance3d, XY, XYZ,
};
pub use self::nav_data::{Archipelago, NewNodeTypeError, NodeType, SetNodeTypeCostError};
pub use self::nav_mesh::{NavMeshBuilder, NavMesh, ValidationError};
pub use self::query::{FindPathError, SamplePointError, SampledPoint};
pub use crate::character::{Character, CharacterId};
pub use crate::island::{Island, IslandId};

/// Options that apply to all agents
#[derive(Resource, Clone, Copy)]
pub struct AgentOptions<T: CoordinateSystem> {
    /// The options for sampling agent and target points.
    pub point_sample_distance: T::SampleDistance,
    /// The distance that an agent will consider avoiding another agent.
    pub neighbourhood: f32,
    // The time into the future that collisions with other agents should be
    /// avoided.
    pub avoidance_time_horizon: f32,
    /// The time into the future that collisions with obstacles should be
    /// avoided.
    pub obstacle_avoidance_time_horizon: f32,
    /// The avoidance responsibility to use when an agent has reached its target.
    /// A value of 1.0 is the default avoidance responsibility. A value of 0.0
    /// would mean no avoidance responsibility, but a value of 0.0 is invalid and
    /// may panic. This should be a value between 0.0 and 1.0.
    pub reached_destination_avoidance_responsibility: f32,
}

impl<T: CoordinateSystem<SampleDistance: FromAgentRadius>> FromAgentRadius for AgentOptions<T> {
    fn from_agent_radius(radius: f32) -> Self {
        Self {
            point_sample_distance: T::SampleDistance::from_agent_radius(radius),
            neighbourhood: 10.0 * radius,
            avoidance_time_horizon: 1.0,
            obstacle_avoidance_time_horizon: 0.5,
            reached_destination_avoidance_responsibility: 0.1,
        }
    }
}
