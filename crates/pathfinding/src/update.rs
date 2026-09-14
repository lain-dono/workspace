use crate::agent::AgentState;
use crate::landmass::CoordinateSystem;
use crate::landmass::agent::RepathResult;
use crate::landmass::path::{PathIndex, Waypoint};
use crate::landmass::{Agent, AgentId, AgentOptions, Archipelago, Character, Island};
use bevy::prelude::*;

/// The result of path finding.
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathingResult {
    /// The agent that searched for the path.
    pub agent: AgentId,
    /// Whether the pathing succeeded or failed.
    pub success: bool,
    /// The number of "nodes" explored while finding the path. Note this may be
    /// zero if the start and end point are known to be disconnected.
    pub explored_nodes: u32,
}

pub fn navigation_update<T: CoordinateSystem>(
    mut nav: ResMut<Archipelago<T>>,
    islands: Query<(Entity, &'static Island<T>)>,
    dirty_islands: Query<Entity, Changed<Island<T>>>,
    deleted_islands: RemovedComponents<Island<T>>,
) {
    Archipelago::<T>::update(&mut nav, islands, dirty_islands, deleted_islands);
}

pub fn character_sampling<T: CoordinateSystem>(
    options: Res<AgentOptions<T>>,
    characters: Query<(Entity, &mut Character<T>)>,
    islands: Query<(Entity, &Island<T>)>,
) {
    let distance = options.point_sample_distance;
    for (_, mut character) in characters {
        let position = T::to_landmass(character.position);
        character.sampled_position = Archipelago::<T>::sample_point(islands, position, distance);
    }
}

pub fn agent_sampling<T: CoordinateSystem>(
    options: Res<AgentOptions<T>>,
    agents: Query<(Entity, &mut Agent<T>)>,
    islands: Query<(Entity, &Island<T>)>,
) {
    let distance = options.point_sample_distance;
    for (_, mut agent) in agents {
        let position = T::to_landmass(agent.position);
        agent.sampled_position = Archipelago::<T>::sample_point(islands, position, distance);
        agent.sampled_target = agent.current_target.and_then(|target| {
            let target = T::to_landmass(target);
            Archipelago::<T>::sample_point(islands, target, distance)
        });
    }
}

pub fn agent_repath<T: CoordinateSystem>(
    mut commands: Commands,
    mut nav: ResMut<Archipelago<T>>,
    agents: Query<(Entity, &mut Agent<T>)>,
    islands: Query<(Entity, &'static Island<T>)>,
) {
    for (entity, mut agent) in agents {
        let agent_node = agent.sampled_position.map(|(_, n)| n);
        let target_node = agent.sampled_target.map(|(_, n)| n);

        agent.path_follow = None;

        match agent.need_repath(
            agent_node,
            target_node,
            &nav.invalidated_links,
            &nav.invalidated_islands,
        ) {
            RepathResult::DoNothing => {}
            RepathResult::FollowPath(agent_node_in_corridor, target_node_in_corridor) => {
                agent.path_follow = Some((agent_node_in_corridor, target_node_in_corridor));
            }
            RepathResult::ClearPathNoTarget => {
                agent.state = AgentState::Idle;
                agent.current_path = None;
            }
            RepathResult::ClearPathBadAgent => {
                agent.state = AgentState::AgentNotOnNavMesh;
                agent.current_path = None;
            }
            RepathResult::ClearPathBadTarget => {
                agent.state = AgentState::TargetNotOnNavMesh;
                agent.current_path = None;
            }
            RepathResult::NeedsRepath => {
                let start = agent_node.unwrap();
                let end = target_node.unwrap();
                let path_result =
                    nav.find_path(islands, start, end, &agent.override_node_type_to_cost);

                commands.trigger(dbg!(PathingResult {
                    agent: AgentId(entity),
                    success: path_result.path.is_some(),
                    explored_nodes: path_result.stats.explored_nodes,
                }));

                if let Some(new_path) = path_result.path {
                    agent.path_follow = Some((PathIndex::new(0, 0), new_path.last_index()));
                    agent.current_path = Some(new_path);
                } else {
                    agent.state = AgentState::NoPath;
                    agent.current_path = None;
                }
            }
        }
    }
}

pub fn agent_movement<T: CoordinateSystem>(
    nav: Res<Archipelago<T>>,
    agents: Query<(Entity, &mut Agent<T>)>,
    islands: Query<(Entity, &Island<T>)>,
) {
    for (_, mut agent) in agents {
        let Some(path) = &agent.current_path else {
            agent.desired_velocity = T::from_landmass(Vec3::ZERO);
            continue;
        };

        let (agent_point, _) = agent
            .sampled_position
            .expect("Agent has a path, so should have a valid start node");
        let (target_point, _) = agent
            .sampled_target
            .expect("Agent has a path, so should have a valid target node");

        let (agent_node_index_in_corridor, target_node_index_in_corridor) = agent
            .path_follow
            .expect("Any agent with a path must have its follow path indices filled out.");

        let start = Waypoint::new(agent_node_index_in_corridor, agent_point);
        let end = Waypoint::new(target_node_index_in_corridor, target_point);
        let next_waypoint = path.find_next_point_in_straight_path(islands, &nav, start, end);

        let target_waypoint = Waypoint::new(target_node_index_in_corridor, target_point);
        if agent.has_reached_target(islands, path, &nav, next_waypoint, target_waypoint) {
            agent.desired_velocity = T::from_landmass(Vec3::ZERO);
            agent.state = AgentState::ReachedTarget;
        } else {
            let desired_delta = (next_waypoint.point - T::to_landmass(agent.position)).xy();
            let desired_move = desired_delta.normalize_or_zero() * agent.desired_speed;

            agent.desired_velocity = T::from_landmass(desired_move.extend(0.0));
            agent.state = AgentState::Moving;
        }
    }
}
