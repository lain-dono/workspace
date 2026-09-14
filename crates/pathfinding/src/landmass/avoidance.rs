use super::{
    Agent, AgentOptions, Archipelago, Character, CoordinateSystem, Island, IslandId,
    nav_data::NodeRef,
};
use crate::agent::AgentState;
use bevy::{
    ecs::{
        entity::Entity,
        system::{Query, Res},
    },
    math::{Vec2, Vec3, Vec3Swizzles},
    platform::collections::{HashMap, HashSet},
    time::Time,
};
use dodgy_2d::VisibilitySet;
use kdtree::{KdTree, distance::squared_euclidean};
use std::{borrow::Cow, collections::BinaryHeap};

/// Adjusts the velocity of `agents` to apply local avoidance. `delta_time` must
/// be positive.
pub(crate) fn agent_avoidance<T: CoordinateSystem + 'static>(
    time: Res<Time>,
    nav: Res<Archipelago<T>>,
    options: Res<AgentOptions<T>>,
    agents: Query<(Entity, &mut Agent<T>)>,
    characters: Query<(Entity, &Character<T>)>,
    islands: Query<(Entity, &Island<T>)>,
) {
    let mut delta_time = time.delta_secs();

    // Make sure that our delta time is positive.
    // Since the delta time is 0.0,
    // the desired velocity doesn't really matter.
    if delta_time <= 0.0 {
        delta_time = 1.0;
    }

    let mut agent_id_to_dodgy_agent = HashMap::new();
    let mut agent_max_radius = 0.0_f32;

    let mut agent_kdtree = KdTree::new(3);
    let mut character_kdtree = KdTree::new(3);

    for (agent_id, agent) in agents.iter() {
        let Some((agent_point, _)) = agent.sampled_position else {
            continue;
        };

        let dagent = dodgy_2d::Agent {
            position: to_dodgy_vec2(agent_point.xy()),
            velocity: to_dodgy_vec2(T::to_landmass(agent.current_velocity).xy()),
            radius: agent.radius,
            avoidance_responsibility: if agent.state == AgentState::ReachedTarget {
                options.reached_destination_avoidance_responsibility
            } else {
                1.0
            },
        };

        let point = [agent_point.x, agent_point.y, agent_point.z];

        agent_id_to_dodgy_agent.insert(agent_id, dagent);
        agent_kdtree.add(point, agent_id).unwrap();
        agent_max_radius = agent_max_radius.max(agent.radius);
    }

    for (_, character) in characters.iter() {
        let Some((character_point, _)) = character.sampled_position else {
            continue;
        };

        let dagent = dodgy_2d::Agent {
            position: to_dodgy_vec2(character_point.xy()),
            velocity: to_dodgy_vec2(T::to_landmass(character.velocity).xy()),
            radius: character.radius,
            // Characters are not responsible for any avoidance since landmass has
            // no control over them.
            avoidance_responsibility: 0.0,
        };

        let point = [character_point.x, character_point.y, character_point.z];
        character_kdtree.add(point, dagent).unwrap();
    }

    let neighbourhood = agent_max_radius + options.neighbourhood;
    let neighbourhood_sq = neighbourhood * neighbourhood;
    for (agent_id, mut agent) in agents {
        let Some(agent_node) = agent.sampled_position else {
            continue;
        };

        let agent_point = [agent_node.0.x, agent_node.0.y, agent_node.0.z];
        let nearby_agents = agent_kdtree
            .within(&agent_point, neighbourhood_sq, &squared_euclidean)
            .unwrap()
            .into_iter()
            .filter_map(|(distance_sq, neighbour_id)| {
                if *neighbour_id == agent_id {
                    return None;
                }

                let dodgy_agent = agent_id_to_dodgy_agent.get(neighbour_id).unwrap();
                let neighbourhood = options.neighbourhood + dodgy_agent.radius;
                let neighbourhood_sq = neighbourhood * neighbourhood;
                if distance_sq < neighbourhood_sq {
                    Some(Cow::Borrowed(dodgy_agent))
                } else {
                    None
                }
            });

        let nearby_characters = character_kdtree
            .within(&agent_point, neighbourhood_sq, &squared_euclidean)
            .unwrap()
            .into_iter()
            .filter_map(|(distance_sq, dodgy_agent)| {
                (distance_sq < neighbourhood_sq).then_some(Cow::Borrowed(dodgy_agent))
            });

        let neighbours = nearby_agents.chain(nearby_characters).collect::<Vec<_>>();

        let mut obstacles =
            nav_mesh_borders_to_dodgy_obstacles(agent_node, &nav, islands, options.neighbourhood);
        let obstacles = obstacles
            .drain(..)
            .map(std::borrow::Cow::Owned)
            .collect::<Vec<_>>();
        let preferred_velocity = to_dodgy_vec2(T::to_landmass(agent.desired_velocity).xy());
        let avoidance_options = dodgy_2d::AvoidanceOptions {
            // Always use an avoidance margin of zero since we assume the nav mesh
            // is the "valid" region.
            obstacle_margin: 0.0,
            time_horizon: options.avoidance_time_horizon,
            obstacle_time_horizon: options.obstacle_avoidance_time_horizon,
        };

        let dodgy_agent = agent_id_to_dodgy_agent.get(&agent_id).unwrap();
        #[cfg(not(feature = "debug-avoidance"))]
        let desired_move = dodgy_agent.compute_avoiding_velocity(
            &nearby_agents,
            &nearby_obstacles,
            preferred_velocity,
            agent.max_speed,
            delta_time,
            &avoidance_options,
        );
        #[cfg(feature = "debug-avoidance")]
        let desired_move = {
            let (desired_move, debug_data) = dodgy_agent.compute_avoiding_velocity_with_debug(
                &neighbours,
                &obstacles,
                preferred_velocity,
                agent.max_speed,
                delta_time,
                &avoidance_options,
            );
            agent.avoidance_data = agent.keep_avoidance_data.then_some(debug_data);
            desired_move
        };

        agent.desired_velocity = T::from_landmass(Vec3::new(desired_move.x, desired_move.y, 0.0));
    }
}

fn to_dodgy_vec2(Vec2 { x, y }: Vec2) -> dodgy_2d::Vec2 {
    dodgy_2d::Vec2 { x, y }
}

/// Computes the dodgy obstacles corresponding to the navigation mesh borders.
/// These obstacles are from the perspective of `agent_node` (to avoid problems
/// with obstacles above/below the agent). `distance_limit` is the distance from
/// the agent to include obstacles.
pub(crate) fn nav_mesh_borders_to_dodgy_obstacles<T: CoordinateSystem>(
    agent_node: (Vec3, NodeRef),
    nav_data: &Archipelago<T>,
    islands: Query<(Entity, &Island<T>)>,
    distance_limit: f32,
) -> Vec<dodgy_2d::Obstacle> {
    struct ExploreNode {
        node: NodeRef,
        score: f32,
    }
    impl PartialEq for ExploreNode {
        fn eq(&self, other: &Self) -> bool {
            self.score == other.score
        }
    }
    impl Eq for ExploreNode {}
    // Since we are comparing floats which are not Ord, it is more meaningful to
    // impl PartialOrd, then unwrap in Ord.
    #[allow(clippy::non_canonical_partial_ord_impl)]
    impl PartialOrd for ExploreNode {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            other.score.partial_cmp(&self.score)
        }
    }
    impl Ord for ExploreNode {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    fn vertex_index_to_dodgy_vec<T: CoordinateSystem>(
        island: &Island<T>,
        index: u32,
        relative_point: Vec2,
    ) -> dodgy_2d::Vec2 {
        to_dodgy_vec2(
            island
                .transform
                .apply(island.mesh.vertices[index as usize])
                .xy()
                - relative_point,
        )
    }

    let distance_limit = distance_limit * distance_limit;

    let mut visibility_set = VisibilitySet::new();
    let mut border_edges = HashMap::new();
    let mut new_vertices = Vec::new();

    let mut explored_nodes = HashSet::new();
    let mut next_nodes = BinaryHeap::new();
    next_nodes.push(ExploreNode {
        node: agent_node.1,
        score: 0.0,
    });

    let agent_point = agent_node.0.xy();

    while !next_nodes.is_empty() {
        let node = next_nodes.pop().unwrap().node;
        if !explored_nodes.insert(node) {
            // This node has already been explored. Skip to the next one.
            continue;
        }

        let (_, island) = islands.get(node.island).unwrap();

        let polygon = &island.mesh.polygons[node.polygon];
        let boundary_links = nav_data.node_to_boundary_link_ids.get(&node);
        let modified_node = nav_data.modified_nodes.get(&node);

        let mut remaining_edges: HashSet<usize> = (0..polygon.vertices.len()).collect();
        for (vertex_1, vertex_2, node_ref) in polygon
            .vertices
            .iter()
            .enumerate()
            .filter_map(|(edge_index, (_, conn))| conn.as_ref().map(|conn| (edge_index, conn)))
            .map(|(edge_index, connectivity)| {
                remaining_edges.remove(&edge_index);

                let [vertex_1, vertex_2] = polygon.edge_indices(edge_index);
                (
                    vertex_index_to_dodgy_vec(island, vertex_1, agent_point),
                    vertex_index_to_dodgy_vec(island, vertex_2, agent_point),
                    NodeRef::new(node.island, connectivity.polygon),
                )
            })
            .chain(boundary_links.iter().flat_map(|boundary_links| {
                boundary_links
                    .iter()
                    .map(|link_id| nav_data.boundary_links.get(*link_id).unwrap())
                    .map(|link| {
                        (
                            to_dodgy_vec2(link.portal[0].xy() - agent_point),
                            to_dodgy_vec2(link.portal[1].xy() - agent_point),
                            link.destination,
                        )
                    })
            }))
        {
            if !visibility_set.is_line_visible(vertex_1, vertex_2) {
                continue;
            }

            let node = ExploreNode {
                node: node_ref,
                score: vertex_1.length_squared().min(vertex_2.length_squared()),
            };

            if node.score < distance_limit {
                next_nodes.push(node);
            }
        }

        if let Some(modified_node) = modified_node {
            for &(left, right) in &modified_node.new_boundary {
                let [(left_point, left_index), (right_point, right_index)] =
                    [left, right].map(|index| {
                        if index >= island.mesh.vertices.len() as u32 {
                            let new_vertex = modified_node.new_vertices
                                [index as usize - island.mesh.vertices.len()];
                            let new_index = new_vertices.len() as u32;
                            new_vertices.push(to_dodgy_vec2(new_vertex));
                            (to_dodgy_vec2(new_vertex - agent_point), (None, new_index))
                        } else {
                            let vertex = vertex_index_to_dodgy_vec(island, index, agent_point);
                            (vertex, (Some(node.island), index))
                        }
                    });

                if let Some(line_index) = visibility_set.add_line(left_point, right_point) {
                    border_edges.insert(line_index, (left_index, right_index));
                }
            }
        } else {
            for border_edge in remaining_edges {
                let [border_vertex_1, border_vertex_2] = polygon.edge_indices(border_edge);
                let (vertex_1, vertex_2) = (
                    vertex_index_to_dodgy_vec(island, border_vertex_1, agent_point),
                    vertex_index_to_dodgy_vec(island, border_vertex_2, agent_point),
                );

                if let Some(line_index) = visibility_set.add_line(vertex_1, vertex_2) {
                    border_edges.insert(
                        line_index,
                        (
                            (Some(node.island), border_vertex_1),
                            (Some(node.island), border_vertex_2),
                        ),
                    );
                }
            }
        }
    }

    let mut finished_loops = Vec::new();
    let mut unfinished_loops = Vec::<Vec<(Option<IslandId>, u32)>>::new();
    for line_id in visibility_set.get_visible_line_ids() {
        let edge = border_edges.get(&line_id).unwrap();

        let mut left_loop = None;
        let mut right_loop = None;

        for (loop_index, edge_loop) in unfinished_loops.iter().enumerate() {
            if left_loop.is_none() && edge_loop[edge_loop.len() - 1] == edge.0 {
                left_loop = Some(loop_index);
                if right_loop.is_some() {
                    break;
                }
            }
            if right_loop.is_none() && edge_loop[0] == edge.1 {
                right_loop = Some(loop_index);
                if left_loop.is_some() {
                    break;
                }
            }
        }

        match (left_loop, right_loop) {
            (None, None) => unfinished_loops.push(vec![edge.0, edge.1]),
            (Some(left_loop), None) => unfinished_loops[left_loop].push(edge.1),
            (None, Some(right_loop)) => unfinished_loops[right_loop].insert(0, edge.0),
            (Some(left_loop_index), Some(right_loop_index)) => {
                if left_loop_index == right_loop_index {
                    finished_loops.push(unfinished_loops.remove(left_loop_index));
                } else {
                    let mut left_loop = unfinished_loops.swap_remove(left_loop_index);
                    let mut right_loop = unfinished_loops.swap_remove(
                        if right_loop_index == unfinished_loops.len() {
                            left_loop_index
                        } else {
                            right_loop_index
                        },
                    );
                    unfinished_loops
                        .push(left_loop.drain(..).chain(right_loop.drain(..)).collect());
                }
            }
        }
    }

    let island_and_vertex_index_to_dodgy_vec = |(island, index)| match island {
        Some(island) => {
            let (_, island) = islands.get(island).unwrap();
            vertex_index_to_dodgy_vec(island, index, Vec2::ZERO)
        }
        None => new_vertices[index as usize],
    };

    finished_loops
        .drain(..)
        .map(|looop| dodgy_2d::Obstacle::Closed {
            vertices: looop
                .iter()
                .rev()
                .copied()
                .map(island_and_vertex_index_to_dodgy_vec)
                .collect(),
        })
        .chain(unfinished_loops.drain(..).map(|looop| {
            dodgy_2d::Obstacle::Open {
                vertices: looop
                    .iter()
                    .rev()
                    .copied()
                    .map(island_and_vertex_index_to_dodgy_vec)
                    .collect(),
            }
        }))
        .collect()
}
