use super::{
    CoordinateSystem, coords::PointSampleDistance, geometry::edge_intersection,
    nav_mesh::MeshEdgeRef,
};
use crate::bvh::{BoundingBox, BoundingBoxHierarchy};
use crate::disjoint_set::DisjointSet;
use crate::island::{Island, IslandId};
use bevy::{
    ecs::{
        entity::Entity, lifecycle::RemovedComponents, query::Changed, resource::Resource,
        system::Query,
    },
    math::{Vec2, Vec3, Vec3Swizzles},
    platform::collections::{HashMap, HashSet},
};
use geo::{BooleanOps, Coord, LineString, LinesIter, MultiPolygon, Polygon};
use kdtree::{KdTree, distance::squared_euclidean};
use slotmap::{HopSlotMap, SlotMap, new_key_type};
use std::mem::swap;
use thiserror::Error;

/// The navigation data of a whole [`crate::Archipelago`].
/// This only includes "static" features.
#[derive(Resource)]
pub struct Archipelago<T: CoordinateSystem> {
    market: std::marker::PhantomData<T>,

    /// The "default" cost of each node type. This also defines the node types
    /// (excluding the `None` type which has an implicit cost of 0.0).
    node_type_to_cost: HopSlotMap<NodeType, f32>,
    /// Maps a "region id" (consisting of the `IslandId` and the region in that
    /// island's nav mesh) to its "region number" (the number used in
    /// [`Self::region_connections`]).
    region_id_to_number: HashMap<(IslandId, u32), u32>,
    /// Connectedness of regions based on their "region number" in [`Self::region_id_to_number`].
    region_connections: DisjointSet,

    /// The links to other islands by [`NodeRef`]
    pub(crate) boundary_links: SlotMap<BoundaryLinkId, BoundaryLink>,
    /// The links that can be taken from a particular node ref.
    pub(crate) node_to_boundary_link_ids: HashMap<NodeRef, HashSet<BoundaryLinkId>>,
    /// The nodes that have been modified.
    pub(crate) modified_nodes: HashMap<NodeRef, ModifiedNode>,

    pub(crate) invalidated_links: HashSet<BoundaryLinkId>,
    pub(crate) invalidated_islands: HashSet<IslandId>,

    pub(crate) edge_link_distance: f32,
}

new_key_type! {
    /// A unique type of node.
    pub struct NodeType;
}

/// A reference to a node in the navigation data.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct NodeRef {
    /// The island of the node.
    pub island: IslandId,
    /// The index of the node in the island.
    pub polygon: usize,
}

impl NodeRef {
    pub fn new(island: IslandId, polygon: usize) -> Self {
        Self { island, polygon }
    }
}

new_key_type! {
    /// The ID of a boundary link.
    pub(crate) struct BoundaryLinkId;
}

/// A single link between two nodes on the boundary of an island.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct BoundaryLink {
    /// The node that taking this link leads to.
    pub destination: NodeRef,
    /// The node type of the destination node. This is storedfor convenience
    /// since it is stable once the boundary link is created.
    /// [`None`] if the destination node is the "default" node type.
    pub destination_node_type: Option<NodeType>,
    /// The portal that this link occupies on the boundary of the source node.
    /// This is essentially the intersection of the linked islands' linkable edges.
    pub portal: [Vec3; 2],
    /// The distances of travelling across this link. The first is the distance
    /// travelled across the starting node, and the second is the
    /// distance travelled across the destination node.
    /// These must be multiplied by the actual node costs.
    pub travel_distances: (f32, f32),
}

/// A node that has been modified (e.g., by being connected with a boundary link
/// to another island).
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct ModifiedNode {
    /// The new (2D) edges that make up the boundary of this node. These are
    /// indices in the nav mesh this corresponds to. Indices larger than the nav
    /// mesh vertices refer to [`ModifiedNode::new_vertices`]. Note the boundary
    /// winds in the same direction as nav mesh polygons (CCW).
    pub new_boundary: Vec<(u32, u32)>,
    /// The "new" vertices (in world space) that are needed for the modified
    /// node. These are not vertices in the original nav mesh and should only
    /// be used by a single boundary edge.
    pub new_vertices: Vec<Vec2>,
}

impl<T: CoordinateSystem> Archipelago<T> {
    /// Creates new navigation data.
    #[must_use]
    pub fn new(edge_link_distance: f32) -> Self {
        Self {
            market: std::marker::PhantomData,
            node_type_to_cost: HopSlotMap::with_key(),
            region_id_to_number: HashMap::new(),
            region_connections: DisjointSet::new(),
            boundary_links: SlotMap::with_key(),
            node_to_boundary_link_ids: HashMap::new(),
            modified_nodes: HashMap::new(),

            invalidated_links: HashSet::new(),
            invalidated_islands: HashSet::new(),
            edge_link_distance,
        }
    }

    /// Creates a new node type with the specified `cost`.
    ///
    /// The cost is a multiplier on the distance travelled along this node
    /// (essentially the cost per meter).
    /// Agents will prefer to travel along low-cost terrain.
    /// This node type is distinct from all other node types.
    pub fn add_node_type(&mut self, cost: f32) -> Result<NodeType, NewNodeTypeError> {
        if cost <= 0.0 {
            return Err(NewNodeTypeError::NonPositiveCost(cost));
        }
        Ok(self.node_type_to_cost.insert(cost))
    }

    /// Sets the cost of `node_type` to `cost`.
    ///
    /// See [`NavigationData::add_node_type`] for the meaning of cost.
    pub fn set_node_type_cost(
        &mut self,
        node_type: NodeType,
        cost: f32,
    ) -> Result<(), SetNodeTypeCostError> {
        if cost <= 0.0 {
            return Err(SetNodeTypeCostError::NonPositiveCost(cost));
        }
        if let Some(node_type_cost) = self.node_type_to_cost.get_mut(node_type) {
            *node_type_cost = cost;
            Ok(())
        } else {
            Err(SetNodeTypeCostError::NodeTypeDoesNotExist(node_type))
        }
    }

    /// Gets the cost of `node_type`. Returns [`None`] if `node_type` is not in this nav data.
    #[must_use]
    pub fn node_type_cost(&self, node_type: NodeType) -> Option<f32> {
        self.node_type_to_cost.get(node_type).copied()
    }

    /// Removes the node type from the navigation data. Returns false if any
    /// islands still use this node type (so the node type cannot be removed).
    /// Otherwise, returns true.
    pub fn remove_node_type(
        &mut self,
        islands: Query<(Entity, &Island<T>)>,
        node_type: NodeType,
    ) -> bool {
        if !self.node_type_to_cost.contains_key(node_type) {
            return false;
        }

        for (_, island) in islands {
            for &type_index in &island.mesh.used_type_indices {
                let Some(&island_node_type) = island.type_index_to_node_type.get(&type_index)
                else {
                    continue;
                };

                if island_node_type == node_type {
                    return false;
                }
            }
        }

        self.node_type_to_cost.remove(node_type);
        true
    }

    /// Gets the current node types and their costs.
    pub fn node_types(&self) -> impl Iterator<Item = (NodeType, f32)> + '_ {
        self.node_type_to_cost
            .iter()
            .map(|(node_type, &cost)| (node_type, cost))
    }

    /// Finds the node nearest to (and within `distance_to_node` of) `point`.
    /// Returns the point on the nav data nearest to `point` and the reference to
    /// the corresponding node.
    pub fn sample_point(
        islands: Query<(Entity, &Island<T>)>,
        point: Vec3,
        distance: T::SampleDistance,
    ) -> Option<(Vec3, NodeRef)> {
        // Note we flip the distance_above and distance_below since we are
        // expanding the nav mesh bounding boxes. From the query point's
        // perspective, we need to sample up by distance_above, which is the
        // same as expanding the bounding box down by distance_above.

        let delta_min = Vec3::new(
            -distance.horizontal_distance(),
            -distance.horizontal_distance(),
            -distance.distance_above(),
        );
        let delta_max = Vec3::new(
            distance.horizontal_distance(),
            distance.horizontal_distance(),
            distance.distance_below(),
        );

        let mut best_point = None;
        for (island_id, island) in islands {
            let relative_point = island.transform.apply_inverse(point);
            let bounds = island.mesh.bounds.add_to_corners(delta_min, delta_max);

            if !bounds.contains_point(relative_point) {
                continue;
            }

            let Some((point, node)) = island.mesh.sample_point(relative_point, distance) else {
                continue;
            };

            let distance = relative_point.distance_squared(point);
            if best_point.is_some_and(|(best_distance, _)| distance >= best_distance) {
                continue;
            }

            let point = island.transform.apply(point);
            let node = NodeRef::new(island_id, node);

            best_point = Some((distance, (point, node)));
        }
        best_point.map(|(_, b)| b)
    }

    pub(crate) fn update(
        nav: &mut Self,
        islands: Query<(Entity, &Island<T>)>,
        dirty_islands: Query<Entity, Changed<Island<T>>>,
        deleted_islands: RemovedComponents<Island<T>>,
    ) {
        let modified_node_refs_to_update =
            nav.update_islands(islands, dirty_islands, deleted_islands);

        for node_ref in modified_node_refs_to_update {
            nav.update_modified_node(islands, node_ref);
        }

        if !nav.invalidated_islands.is_empty() {
            nav.update_regions(islands);
        }
    }

    fn update_islands(
        &mut self,
        islands: Query<(Entity, &Island<T>)>,
        dirty_islands: Query<Entity, Changed<Island<T>>>,
        mut deleted_islands: RemovedComponents<Island<T>>,
    ) -> HashSet<NodeRef> {
        self.invalidated_links.clear();
        self.invalidated_islands.clear();

        let dirty_islands: HashSet<IslandId> = dirty_islands.iter().collect();
        let deleted_islands: HashSet<IslandId> = deleted_islands.read().collect();

        self.invalidated_islands
            .extend(deleted_islands.union(&dirty_islands).copied());

        let mut modified_node_refs_to_update = HashSet::new();
        if !deleted_islands.is_empty() || !dirty_islands.is_empty() {
            self.node_to_boundary_link_ids.retain(|node_ref, links| {
                if self.invalidated_islands.contains(&node_ref.island) {
                    for &link in links.iter() {
                        self.boundary_links.remove(link);
                    }
                    modified_node_refs_to_update.insert(*node_ref);
                    self.invalidated_links.extend(links.iter().copied());
                    return false;
                }

                let links_before = links.len();

                links.retain(|&id| {
                    let link = self.boundary_links.get(id).unwrap();
                    if self.invalidated_islands.contains(&link.destination.island) {
                        self.boundary_links.remove(id);
                        self.invalidated_links.insert(id);
                        false
                    } else {
                        true
                    }
                });

                if links_before != links.len() {
                    // If a node has a different set of boundary links,
                    // we need to recompute that node.
                    modified_node_refs_to_update.insert(*node_ref);
                }

                !links.is_empty()
            });
        }

        if dirty_islands.is_empty() {
            // No new or changed islands, so no need to check for new links.
            return modified_node_refs_to_update;
        }

        let mut island_bounds = islands
            .iter()
            .filter(|(_, island)| !island.transformed_bounds.is_empty())
            .map(|(island_id, island)| (island.transformed_bounds, island_id))
            .collect::<Vec<_>>();

        // There are no islands with nav data, so no islands to link and prevents a panic.
        if island_bounds.is_empty() {
            return modified_node_refs_to_update;
        }
        let island_bbh = BoundingBoxHierarchy::new(&mut island_bounds);

        for &dirty_island_id in &dirty_islands {
            let (_, dirty_island) = islands.get(dirty_island_id).unwrap();
            // Check that all the island's node types are valid.
            for &type_index in &dirty_island.mesh.used_type_indices {
                if let Some(&node_type) = dirty_island.type_index_to_node_type.get(&type_index) {
                    assert!(
                        self.node_type_to_cost.contains_key(node_type),
                        "Island {dirty_island_id:?} uses node type {node_type:?} which is not in this navigation data."
                    );
                }
            }

            if dirty_island.mesh.boundary_edges.is_empty() {
                continue;
            }
            let query = dirty_island
                .transformed_bounds
                .expand_by_size(Vec3::ONE * self.edge_link_distance);
            let candidate_islands = island_bbh.query(query);
            // If the only candidate island is the dirty island itself, skip the rest.
            if candidate_islands.len() <= 1 {
                continue;
            }
            let dirty_island_edge_bbh = island_edges_bbh(dirty_island);
            for candidate_island_id in candidate_islands {
                if candidate_island_id == dirty_island_id {
                    continue;
                }

                // `link_edges_between_islands` links forwards and backwards. This means
                // for a pair of dirty islands, we must only process them once, so only
                // execute `link_edges_between_islands` for one ordering of the island
                // ids and not the other.
                if dirty_islands.contains(&candidate_island_id)
                    && candidate_island_id < dirty_island_id
                {
                    continue;
                }

                let (_, candidate_island) = islands.get(candidate_island_id).unwrap();
                link_edges_between_islands(
                    (dirty_island_id, dirty_island),
                    (candidate_island_id, candidate_island),
                    &dirty_island_edge_bbh,
                    self.edge_link_distance,
                    &mut self.boundary_links,
                    &mut self.node_to_boundary_link_ids,
                    &mut modified_node_refs_to_update,
                );
            }
        }

        modified_node_refs_to_update
    }

    fn update_modified_node(&mut self, islands: Query<(Entity, &Island<T>)>, node_ref: NodeRef) {
        fn vec2_to_coord(v: Vec2) -> Coord<f32> {
            Coord::from((v.x, v.y))
        }

        fn coord_to_vec2(c: Coord<f32>) -> Vec2 {
            Vec2::new(c.x, c.y)
        }

        fn push_vertex<T: CoordinateSystem>(
            vertex: u32,
            island: &Island<T>,
            line_string: &mut Vec<Coord<f32>>,
        ) {
            let vertex = island
                .transform
                .apply(island.mesh.vertices[vertex as usize]);
            line_string.push(vec2_to_coord(vertex.xy()));
        }

        fn boundary_link_to_clip_polygon(
            link: &BoundaryLink,
            edge_link_distance: f32,
        ) -> MultiPolygon<f32> {
            let flat_portal = link.portal.map(Vec3Swizzles::xy);
            let portal_forward =
                (flat_portal[1] - flat_portal[0]).normalize().perp() * edge_link_distance;
            MultiPolygon::new(vec![Polygon::new(
                LineString(vec![
                    vec2_to_coord(flat_portal[0] + portal_forward),
                    vec2_to_coord(flat_portal[0] - portal_forward),
                    vec2_to_coord(flat_portal[1] - portal_forward),
                    vec2_to_coord(flat_portal[1] + portal_forward),
                ]),
                vec![],
            )])
        }

        // Any node from an island that doesn't exist (deleted), or one without nav
        // data (the nav mesh was removed), should be removed.
        let Ok((_, island)) = islands.get(node_ref.island) else {
            self.modified_nodes.remove(&node_ref);
            return;
        };
        // Any nodes without boundary links don't need to be modified.
        let Some(boundary_links) = self.node_to_boundary_link_ids.get(&node_ref) else {
            self.modified_nodes.remove(&node_ref);
            return;
        };

        let polygon = &island.mesh.polygons[node_ref.polygon];

        let mut multi_line_string = vec![];
        let mut current_line_string = vec![];

        for &(vertex, ref connectivity) in &polygon.vertices {
            if connectivity.is_some() {
                if !current_line_string.is_empty() {
                    let mut line_string = vec![];
                    swap(&mut current_line_string, &mut line_string);

                    // Add the right vertex of the previous edge (the current vertex).
                    push_vertex(vertex, island, &mut line_string);

                    multi_line_string.push(LineString(line_string));
                }
                continue;
            }

            // Add the left vertex. The right vertex will be added when we finish the
            // line string.
            push_vertex(vertex, island, &mut current_line_string);
        }

        if !current_line_string.is_empty() {
            // The last "closing" edge must not be connected, so add the first vertex
            // to finish that edge and push it into the `multi_line_string`.
            push_vertex(polygon.vertices[0].0, island, &mut current_line_string);

            multi_line_string.push(LineString(current_line_string));
        }

        let boundary_edges = geo::MultiLineString(multi_line_string);

        let mut clip_polygons = boundary_links
            .iter()
            .map(|&link_id| self.boundary_links.get(link_id).unwrap())
            .map(|link| boundary_link_to_clip_polygon(link, self.edge_link_distance));

        let mut link_clip = clip_polygons.next().unwrap();
        for clip_polygon in clip_polygons {
            link_clip = link_clip.union(&clip_polygon);
        }

        let clipped_boundary_edges = link_clip.clip(&boundary_edges, true);

        let mut original_vertices = KdTree::new(2);
        for &(index, _) in &polygon.vertices {
            let point = island.transform.apply(island.mesh.vertices[index as usize]);
            let point = [point.x, point.y];

            original_vertices
                .add(point, index)
                .expect("Vertex is valid");
        }

        let mut modified_node = self.modified_nodes.entry(node_ref).insert(ModifiedNode {
            new_boundary: Vec::new(),
            new_vertices: Vec::new(),
        });
        let modified_node = modified_node.get_mut();

        for line_string in clipped_boundary_edges.iter() {
            for edge in line_string.lines_iter() {
                let start_index = original_vertices
                    .nearest(&[edge.start.x, edge.start.y], 1, &squared_euclidean)
                    .unwrap()
                    .first()
                    .filter(|&(distance, _)| *distance < 0.01)
                    .map(|&(_, &index)| index);

                let end_index = original_vertices
                    .nearest(&[edge.end.x, edge.end.y], 1, &squared_euclidean)
                    .unwrap()
                    .first()
                    .filter(|&(distance, _)| *distance < 0.01)
                    .map(|&(_, &index)| index);

                if let (Some(start_index), Some(end_index)) = (start_index, end_index) {
                    // We don't want degenerate edges, so ignore edges with equal indices.
                    if start_index == end_index {
                        continue;
                    }
                }

                let mut start_index = start_index.unwrap_or_else(|| {
                    modified_node.new_vertices.push(coord_to_vec2(edge.start));
                    (island.mesh.vertices.len() + modified_node.new_vertices.len() - 1) as u32
                });

                let mut end_index = end_index.unwrap_or_else(|| {
                    modified_node.new_vertices.push(coord_to_vec2(edge.end));
                    (island.mesh.vertices.len() + modified_node.new_vertices.len() - 1) as u32
                });

                let polygon_center = island.transform.apply(polygon.center).xy();

                // Ensure the winding order of the modified node boundary matches the
                // polygon edges.
                if (coord_to_vec2(edge.start) - polygon_center)
                    .perp_dot(coord_to_vec2(edge.end) - polygon_center)
                    < 0.0
                {
                    swap(&mut start_index, &mut end_index);
                }

                modified_node.new_boundary.push((start_index, end_index));
            }
        }
    }

    fn node_to_region_id(
        islands: Query<(Entity, &Island<T>)>,
        node_ref: NodeRef,
    ) -> (IslandId, u32) {
        let id = islands.get(node_ref.island).unwrap().1.mesh.polygons[node_ref.polygon].region;
        (node_ref.island, id)
    }

    /// Determines whether `node_1` and `node_2` can be connected by some path.
    pub(crate) fn are_nodes_connected(
        &mut self,
        islands: Query<(Entity, &Island<T>)>,
        node_1: NodeRef,
        node_2: NodeRef,
    ) -> bool {
        let region_id_1 = Self::node_to_region_id(islands, node_1);
        let region_id_2 = Self::node_to_region_id(islands, node_2);
        if region_id_1 == region_id_2 {
            // The regions are the same, so they are definitely connected. Skip all
            // the rest of the work.
            return true;
        }

        let Some(&region_number_1) = self.region_id_to_number.get(&region_id_1) else {
            // If the requested region is not in the `region_id_to_number` map, it is
            // not connected to any other region by a boundary link. Therefore, the
            // regions are unconnected.
            return false;
        };

        let Some(&region_number_2) = self.region_id_to_number.get(&region_id_2) else {
            // Same reasoning as above.
            return false;
        };

        self.region_connections
            .is_joined(region_number_1, region_number_2)
    }

    fn update_regions(&mut self, islands: Query<(Entity, &Island<T>)>) {
        self.region_id_to_number.clear();
        let region_connections = &mut self.region_connections;
        region_connections.clear();

        for (node_ref, link) in
            self.node_to_boundary_link_ids
                .iter()
                .flat_map(|(&node_ref, links)| {
                    links
                        .iter()
                        .map(|&link| self.boundary_links.get(link).unwrap())
                        .map(move |link| (node_ref, link))
                })
        {
            let start_region = Self::node_to_region_id(islands, node_ref);
            let end_region = Self::node_to_region_id(islands, link.destination);

            let start_region = *self
                .region_id_to_number
                .entry(start_region)
                .or_insert_with(|| region_connections.add_singleton());

            let end_region = *self
                .region_id_to_number
                .entry(end_region)
                .or_insert_with(|| region_connections.add_singleton());

            region_connections.join(start_region, end_region);
        }
    }
}

/// An error for creating a new node type.
#[derive(Clone, Copy, PartialEq, Error, Debug)]
pub enum NewNodeTypeError {
    #[error("The provided cost {0} is non-positive. Node costs must be positive.")]
    NonPositiveCost(f32),
}

/// An error for settings the cost of an existing node type.
#[derive(Clone, Copy, PartialEq, Error, Debug)]
pub enum SetNodeTypeCostError {
    #[error("The provided cost {0} is non-positive. Node costs must be positive.")]
    NonPositiveCost(f32),
    #[error("The node type {0:?} does not exist.")]
    NodeTypeDoesNotExist(NodeType),
}

fn edge_ref_to_world_edge<T: CoordinateSystem>(edge: MeshEdgeRef, island: &Island<T>) -> [Vec3; 2] {
    let edges = island.mesh.edge_points(edge);
    edges.map(|point| island.transform.apply(point))
}

pub(crate) fn island_edges_bbh<T: CoordinateSystem>(
    island: &Island<T>,
) -> BoundingBoxHierarchy<MeshEdgeRef> {
    let edges = island.mesh.boundary_edges.iter();
    let values = edges.map(|&edge| {
        let bbox = BoundingBox::from_point_cloud(edge_ref_to_world_edge(edge, island));
        (bbox, edge)
    });
    BoundingBoxHierarchy::new(&mut values.collect::<Vec<_>>())
}

pub(crate) fn link_edges_between_islands<T: CoordinateSystem>(
    (island_id_1, island_1): (IslandId, &Island<T>),
    (island_id_2, island_2): (IslandId, &Island<T>),
    island_1_edge_bbh: &BoundingBoxHierarchy<MeshEdgeRef>,
    edge_link_distance: f32,
    boundary_links: &mut SlotMap<BoundaryLinkId, BoundaryLink>,
    node_to_boundary_link_ids: &mut HashMap<NodeRef, HashSet<BoundaryLinkId>>,
    modified_node_refs_to_update: &mut HashSet<NodeRef>,
) {
    let edge_link_distance_squared = edge_link_distance * edge_link_distance;

    for &edge2_ref in &island_2.mesh.boundary_edges {
        let edge2 = edge_ref_to_world_edge(edge2_ref, island_2);
        let edge2_bbox =
            BoundingBox::from_point_cloud(edge2).expand_by_size(Vec3::ONE * edge_link_distance);

        for edge1_ref in island_1_edge_bbh.query(edge2_bbox) {
            let edge1 = edge_ref_to_world_edge(edge1_ref, island_1);
            if let Some(portal) = edge_intersection(edge1, edge2, edge_link_distance_squared) {
                if portal[0].distance_squared(portal[1]) < edge_link_distance_squared {
                    continue;
                }

                let node_1 = NodeRef::new(island_id_1, edge1_ref.poly);
                let node_2 = NodeRef::new(island_id_2, edge2_ref.poly);

                let polygon_1 = &island_1.mesh.polygons[edge1_ref.poly];
                let polygon_2 = &island_2.mesh.polygons[edge2_ref.poly];

                let polygon_center_1 = island_1.transform.apply(polygon_1.center);
                let polygon_center_2 = island_2.transform.apply(polygon_2.center);
                let portal_center = (portal[0] + portal[1]) / 2.0;

                let travel_distances = (
                    polygon_center_1.distance(portal_center),
                    polygon_center_2.distance(portal_center),
                );

                let node_type_1 = island_1
                    .type_index_to_node_type
                    .get(&polygon_1.type_index)
                    .copied();
                let node_type_2 = island_2
                    .type_index_to_node_type
                    .get(&polygon_2.type_index)
                    .copied();

                let id_1 = boundary_links.insert(BoundaryLink {
                    destination: node_2,
                    destination_node_type: node_type_2,
                    portal,
                    travel_distances,
                });
                let id_2 = boundary_links.insert(BoundaryLink {
                    destination: node_1,
                    destination_node_type: node_type_1,
                    portal: [portal[1], portal[0]],
                    travel_distances: (travel_distances.1, travel_distances.0),
                });

                node_to_boundary_link_ids
                    .entry(node_1)
                    .or_default()
                    .insert(id_1);
                node_to_boundary_link_ids
                    .entry(node_2)
                    .or_default()
                    .insert(id_2);

                modified_node_refs_to_update.insert(node_1);
                modified_node_refs_to_update.insert(node_2);
            }
        }
    }
}
