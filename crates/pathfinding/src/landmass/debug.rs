use super::{
    Agent, AgentId, AgentOptions, Archipelago, CoordinateSystem, Island,
    nav_data::NodeRef,
    path::{Path, Waypoint},
};
use bevy::ecs::{entity::Entity, system::Query};
use thiserror::Error;

#[cfg(feature = "debug-avoidance")]
use bevy::math::Vec2;

/// The type of debug points.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PointType {
    /// The position of an agent.
    AgentPosition(AgentId),
    /// The target of an agent.
    TargetPosition(AgentId),
    /// The waypoint of an agent.
    Waypoint(AgentId),
}

/// The type of debug lines.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum LineType {
    /// An edge of a node that is the boundary of a nav mesh.
    BoundaryEdge,
    /// An edge of a node that is connected to another node.
    ConnectivityEdge,
    /// A link between two islands along their boundary edge.
    BoundaryLink,
    /// Part of an agent's current path. The corridor follows the path along
    /// nodes, not the actual path the agent will travel.
    AgentCorridor(AgentId),
    /// Line from an agent to its target.
    Target(AgentId),
    /// Line to the waypoint of an agent.
    Waypoint(AgentId),
}

/// The type of debug triangles.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TriangleType {
    /// Part of a node/polygon in a nav mesh.
    Node,
}

/// Trait to "draw" Archipelago state to. Users should implement this to
/// visualize the state of their Archipelago.
pub trait DebugDrawer<T: CoordinateSystem> {
    fn add_point(&mut self, point_type: PointType, point: T::Coord);
    fn add_line(&mut self, line_type: LineType, line: [T::Coord; 2]);
    fn add_triangle(&mut self, triangle_type: TriangleType, triangle: [T::Coord; 3]);

    fn add_target(&mut self, agent: AgentId, position: T::Coord, target: T::Coord) {
        self.add_line(LineType::Target(agent), [position, target]);
        self.add_point(PointType::TargetPosition(agent), target);
    }

    fn add_waypoint(&mut self, agent: AgentId, position: T::Coord, waypoint: T::Coord) {
        self.add_line(LineType::Waypoint(agent), [position, waypoint]);
        self.add_point(PointType::Waypoint(agent), waypoint);
    }
}

/// An error resulting from trying to debug draw an archipelago.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DebugDrawError {
    #[error("The navigation data of the archipelago has been mutated since the last update.")]
    NavDataDirty,
}

/// Draws all parts of `archipelago` to `debug_drawer`.
pub fn draw_archipelago_debug<T: CoordinateSystem>(
    agents: Query<(Entity, &Agent<T>)>,
    islands: Query<(Entity, &Island<T>)>,
    nav: &Archipelago<T>,
    drawer: &mut impl DebugDrawer<T>,
    options: &AgentOptions<T>,
) -> Result<(), DebugDrawError> {
    for (island_id, island) in islands {
        for (polygon_index, polygon) in island.mesh.polygons.iter().enumerate() {
            let center = island.transform.apply(polygon.center);
            for i in 0..polygon.vertices.len() {
                let j = (i + 1) % polygon.vertices.len();

                let [i, j] = [i, j]
                    .map(|index| polygon.vertices[index])
                    .map(|(index, _)| island.transform.apply(island.mesh.vertices[index as usize]));

                drawer.add_triangle(TriangleType::Node, [i, j, center].map(T::from_landmass));
            }

            for (edge_index, (_, connection)) in polygon.vertices.iter().enumerate() {
                let line_type = match connection.as_ref() {
                    // Ignore connections where the connected polygon has a greater
                    // index. This prevents drawing the same edge multiple
                    // times by picking one of the edges to draw.
                    Some(connection) if connection.polygon < polygon_index => continue,
                    Some(_) => LineType::ConnectivityEdge,
                    None => LineType::BoundaryEdge,
                };

                let i = edge_index;
                let j = (i + 1) % polygon.vertices.len();

                let line = [i, j]
                    .map(|index| polygon.vertices[index])
                    .map(|(index, _)| island.transform.apply(island.mesh.vertices[index as usize]));

                drawer.add_line(line_type, line.map(T::from_landmass));
            }

            let node_ref = NodeRef::new(island_id, polygon_index);
            if let Some(boundary_link_ids) = nav.node_to_boundary_link_ids.get(&node_ref) {
                for &boundary_link_id in boundary_link_ids {
                    let boundary_link = nav
                        .boundary_links
                        .get(boundary_link_id)
                        .expect("Boundary links are present.");
                    // Ignore links where the connected node has a greater node_ref. This
                    // prevents drawing the same link multiple times by picking one of the
                    // links to draw.
                    if node_ref > boundary_link.destination {
                        continue;
                    }

                    let line = boundary_link.portal.map(T::from_landmass);
                    drawer.add_line(LineType::BoundaryLink, line);
                }
            }
        }
    }

    for (agent_id, agent) in agents.iter() {
        let agent_id = AgentId(agent_id);
        drawer.add_point(PointType::AgentPosition(agent_id), agent.position);
        if let Some(target) = agent.current_target {
            drawer.add_target(agent_id, agent.position, target);
        }
        if let Some(path) = agent.current_path.as_ref() {
            draw_path(islands, path, agent_id, agent, nav, drawer, options);
        }
    }

    Ok(())
}

/// Draws `path` to `debug_drawer`. The path belongs to `agent` and both belong to `archipelago`.
fn draw_path<T: CoordinateSystem>(
    islands: Query<(Entity, &Island<T>)>,
    path: &Path,
    agent_id: AgentId,
    agent: &Agent<T>,
    nav: &Archipelago<T>,
    drawer: &mut impl DebugDrawer<T>,
    options: &AgentOptions<T>,
) {
    let target = agent
        .current_target
        .expect("The path is valid, so the target is valid.");

    let points = path
        .island_segments
        .iter()
        .flat_map(|segment| {
            segment.corridor.iter().map(|&polygon_index| {
                let (_, island) = islands.get(segment.island).unwrap();
                let center = island.mesh.polygons[polygon_index].center;
                island.transform.apply(center)
            })
        })
        .collect::<Vec<_>>();

    for pair in points.windows(2) {
        if let Ok(pair) = <[_; 2]>::try_from(pair) {
            let line_type = LineType::AgentCorridor(agent_id);
            drawer.add_line(line_type, pair.map(T::from_landmass));
        }
    }

    let agent_waypoint = {
        let agent_position = T::to_landmass(agent.position);
        let (agent_sample_point, agent_node_ref) =
            Archipelago::<T>::sample_point(islands, agent_position, options.point_sample_distance)
                .expect("Path exists, so sampling the agent should be fine.");
        let agent_corridor_index = path
            .find_index_of_node(agent_node_ref)
            .expect("Path exists, so the agent's node must be in the corridor.");
        Waypoint::new(agent_corridor_index, agent_sample_point)
    };

    let target_waypoint = {
        let point = T::to_landmass(target);
        let (target_sample_point, target_node_ref) =
            Archipelago::<T>::sample_point(islands, point, options.point_sample_distance)
                .expect("Path exists, so sampling the agent should be fine.");
        let target_corridor_index = path
            .find_index_of_node_rev(target_node_ref)
            .expect("Path exists, so the target's node must be in the corridor.");
        Waypoint::new(target_corridor_index, target_sample_point)
    };

    let waypoint =
        path.find_next_point_in_straight_path(islands, nav, agent_waypoint, target_waypoint);

    drawer.add_waypoint(agent_id, agent.position, T::from_landmass(waypoint.point));
}

#[cfg(feature = "debug-avoidance")]
/// A constraint in velocity-space for an agent's velocity for local collision
/// avoidance. The constraint restricts the velocity to lie on one side of a
/// line (aka., only a half-plane is considered valid).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintLine {
    /// A point on the line separating the valid and invalid velocities.
    pub point: Vec2,
    /// The normal of the line separating the valid and invalid velocities. The
    /// normal always points towards the valid velocities.
    pub normal: Vec2,
}

#[cfg(feature = "debug-avoidance")]
/// The kinds of constraint during avoidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstraintKind {
    /// The constraints from the original algorithm. This is either just all the
    /// constraints if the algorithm succeeded, or it is the constraints that
    /// failed, and there are also fallback constraints.
    Original,
    /// The constraints after the algorithm has fallen back to ensure a valid
    /// avoidance direction.
    Fallback,
}

#[cfg(feature = "debug-avoidance")]
/// A trait for reporting agent local collision avoidance constraints.
pub trait AvoidanceDrawer {
    /// Reports a single avoidance constraint.
    fn add_constraint(&mut self, agent: AgentId, constraint: ConstraintLine, kind: ConstraintKind);
}

#[cfg(feature = "debug-avoidance")]
impl ConstraintLine {
    fn from_dodgy(line: &dodgy_2d::debug::Line) -> Self {
        Self {
            point: Vec2::new(line.point.x, line.point.y),
            normal: Vec2::new(-line.direction.y, line.direction.x),
        }
    }
}

#[cfg(feature = "debug-avoidance")]
pub fn draw_avoidance_data<T: CoordinateSystem>(
    agents: Query<(Entity, &Agent<T>)>,
    drawer: &mut impl AvoidanceDrawer,
) {
    for (agent_id, agent) in agents {
        let Some(avoidance_data) = agent.avoidance_data.as_ref() else {
            continue;
        };

        let (original_constraints, fallback_constraints) = match avoidance_data {
            dodgy_2d::debug::DebugData::Satisfied { constraints } => {
                (constraints.as_slice(), [].as_slice())
            }
            dodgy_2d::debug::DebugData::Fallback {
                original_constraints,
                fallback_constraints,
            } => (
                original_constraints.as_slice(),
                fallback_constraints.as_slice(),
            ),
        };

        let agent = AgentId(agent_id);

        for constraint in original_constraints {
            let constraint = ConstraintLine::from_dodgy(constraint);
            drawer.add_constraint(agent, constraint, ConstraintKind::Original);
        }

        for constraint in fallback_constraints {
            let constraint = ConstraintLine::from_dodgy(constraint);
            drawer.add_constraint(agent, constraint, ConstraintKind::Fallback);
        }
    }
}
