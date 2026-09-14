use super::{
    Archipelago, CoordinateSystem, Island, IslandId,
    nav_data::{BoundaryLinkId, NodeRef},
};
use bevy::{
    ecs::{entity::Entity, system::Query},
    math::{Vec3, Vec3Swizzles},
    platform::collections::HashSet,
};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Waypoint {
    pub index: PathIndex,
    pub point: Vec3,
}

impl Waypoint {
    pub const fn new(index: PathIndex, point: Vec3) -> Self {
        Self { index, point }
    }
}

/// A path computed on the navigation data.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Path {
    /// The segments of this path on islands. These are joined together with
    /// [`Path::boundary_link_segments`]. Note even if an island is only used to
    /// take another boundary link, it must be included with the relevant node.
    pub(crate) island_segments: Vec<IslandSegment>,
    /// The boundary links that connect the island segments. Must have exactly
    /// one less element than [`Path::island_segments`].
    pub(crate) boundary_link_segments: Vec<BoundaryLinkSegment>,
}

/// Part of a path entirely along a single island.
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct IslandSegment {
    /// The island that the nodes belong to.
    pub(crate) island: IslandId,
    /// The nodes belonging to the path as their polygon index.
    /// Must have at least one element.
    pub(crate) corridor: Vec<usize>,
    /// The "portals" used between each node in [`IslandSegment::corridor`].
    /// The portals are the edges that the agent must cross along its path.
    /// Must have exactly one less element than [`IslandSegment::corridor`].
    pub(crate) portal_edge_index: Vec<usize>,
}

/// Part of a path taking a `BoundaryLink`.
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct BoundaryLinkSegment {
    /// The node that the boundary link starts from.
    pub(crate) starting_node: NodeRef,
    /// The link to be used.
    pub(crate) boundary_link: BoundaryLinkId,
}

impl IslandSegment {
    /// Determines the endpoints of the portal at `portal_index` in `nav_data`.
    fn portal_endpoints<T: CoordinateSystem>(
        &self,
        islands: Query<(Entity, &Island<T>)>,
        portal: usize,
    ) -> [Vec3; 2] {
        let polygon_index = self.corridor[portal];
        let edge = self.portal_edge_index[portal];

        let (_, island) = islands
            .get(self.island)
            .expect("only called if path is still valid");

        island.mesh.polygons[polygon_index]
            .edge_indices(edge)
            .map(|index| island.transform.apply(island.mesh.vertices[index as usize]))
    }
}

impl BoundaryLinkSegment {
    /// Gets the endpoints of the portal for this boundary link in `nav_data`.
    fn portal_endpoints<T: CoordinateSystem>(&self, nav_data: &Archipelago<T>) -> [Vec3; 2] {
        nav_data
            .boundary_links
            .get(self.boundary_link)
            .expect("only called if path is still valid")
            .portal
    }
}

/// An index in a path.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct PathIndex {
    /// The index of the segment this belongs to.
    pub(crate) segment: usize,
    /// The index of the portal in the corridor.
    /// The last index in a corrider either corresponds to the boundary link
    /// to the next segment or the target node.
    pub(crate) portal: usize,
}

impl PathIndex {
    /// Creates a `PathIndex` starting at the `island_segment_index` at the `corridor_index`.
    pub(crate) fn new(segment: usize, portal: usize) -> Self {
        Self { segment, portal }
    }

    fn next(&self, path: &Path) -> Self {
        let new_portal_index = self.portal + 1;
        match new_portal_index.cmp(&path.island_segments[self.segment].portal_edge_index.len()) {
            Ordering::Less | Ordering::Equal => Self {
                segment: self.segment,
                portal: new_portal_index,
            },
            Ordering::Greater => {
                if self.segment + 1 < path.island_segments.len() {
                    Self {
                        segment: self.segment + 1,
                        portal: 0,
                    }
                } else {
                    // Only the last segment can go past the end of the path to allow
                    // being inclusive over the end index if the end index is
                    // after the last portal.
                    Self {
                        segment: self.segment,
                        portal: new_portal_index,
                    }
                }
            }
        }
    }
}

impl Path {
    /// Determines the endpoints of the portal at `segment_index` at
    /// `portal_index` in `nav_data`.
    fn portal_endpoints<T: CoordinateSystem>(
        &self,
        path_index: PathIndex,
        nav_data: &Archipelago<T>,
        islands: Query<(Entity, &Island<T>)>,
    ) -> [Vec3; 2] {
        let len = self.island_segments[path_index.segment]
            .portal_edge_index
            .len();

        if path_index.portal == len {
            self.boundary_link_segments[path_index.segment].portal_endpoints(nav_data)
        } else {
            self.island_segments[path_index.segment].portal_endpoints(islands, path_index.portal)
        }
    }

    /// Determines the next point along `self` that the agent can walk straight
    /// towards, starting at the node `start_index` at `start_point` and ending at
    /// the node `end_index` at `end_point`. `start_index` and `end_index` are
    /// indices into `self`. Returns the index of the node in the path where the
    /// next point is, and that next point. Note this can be called repeatedly by
    /// passing in the returned tuple as the `start_index` and `start_point` to
    /// generate the full straight path.
    pub(crate) fn find_next_point_in_straight_path<T: CoordinateSystem>(
        &self,
        islands: Query<(Entity, &Island<T>)>,
        nav: &Archipelago<T>,
        start: Waypoint,
        end: Waypoint,
    ) -> Waypoint {
        fn triangle_area_xy(a: Vec3, b: Vec3, c: Vec3) -> f32 {
            (b.xy() - a.xy()).perp_dot(c.xy() - a.xy())
        }

        let apex = start.point;
        let (mut left_index, mut right_index) = (start.index, start.index);

        let [mut current_left, mut current_right] = if start.index == end.index {
            [end.point, end.point]
        } else {
            self.portal_endpoints(start.index, nav, islands)
        };

        let mut portal_index = start.index.next(self);
        while portal_index <= end.index {
            let [portal_left, portal_right] = if portal_index == end.index {
                [end.point, end.point]
            } else {
                self.portal_endpoints(portal_index, nav, islands)
            };

            if triangle_area_xy(apex, current_right, portal_right) <= 0.0 {
                if triangle_area_xy(apex, current_left, portal_right) >= 0.0 {
                    right_index = portal_index;
                    current_right = portal_right;
                } else {
                    return Waypoint::new(left_index, current_left);
                }
            }

            if triangle_area_xy(apex, current_left, portal_left) >= 0.0 {
                if triangle_area_xy(apex, current_right, portal_left) <= 0.0 {
                    left_index = portal_index;
                    current_left = portal_left;
                } else {
                    return Waypoint::new(right_index, current_right);
                }
            }

            portal_index = portal_index.next(self);
        }

        end
    }

    pub(crate) fn last_index(&self) -> PathIndex {
        let segment = self.island_segments.len() - 1;
        let portal = self.island_segments[segment].portal_edge_index.len();
        PathIndex { segment, portal }
    }

    /// Determines if a path is valid. A path may be invalid if an island it
    /// travelled across was invalidared, or a boundary link it used was
    /// invalidated.
    pub(crate) fn is_valid(
        &self,
        invalidated_boundary_links: &HashSet<BoundaryLinkId>,
        invalidated_islands: &HashSet<IslandId>,
    ) -> bool {
        for island_segment in &self.island_segments {
            if invalidated_islands.contains(&island_segment.island) {
                return false;
            }
        }
        for boundary_link_segment in &self.boundary_link_segments {
            if invalidated_boundary_links.contains(&boundary_link_segment.boundary_link) {
                return false;
            }
        }

        true
    }

    /// Finds the index of `node` in the path.
    pub(crate) fn find_index_of_node(&self, node: NodeRef) -> Option<PathIndex> {
        for (segment, island_segment) in self.island_segments.iter().enumerate() {
            if node.island != island_segment.island {
                continue;
            }
            for (portal, &corridor_node) in island_segment.corridor.iter().enumerate() {
                if node.polygon == corridor_node {
                    return Some(PathIndex { segment, portal });
                }
            }
        }
        None
    }

    /// Finds the index of `node` in the path, iterating backwards. This is
    /// slightly more efficient than [`Path::find_index_of_node`] for the target
    /// node, since most of the time the target node will be near the end of the
    /// path.
    pub(crate) fn find_index_of_node_rev(&self, node: NodeRef) -> Option<PathIndex> {
        for (segment, island_segment) in self.island_segments.iter().enumerate().rev() {
            if node.island != island_segment.island {
                continue;
            }
            for (portal, &corridor_node) in island_segment.corridor.iter().enumerate().rev() {
                if node.polygon == corridor_node {
                    return Some(PathIndex { segment, portal });
                }
            }
        }
        None
    }
}
