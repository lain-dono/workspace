use super::ArchipelagoWorld;
use crate::bvh::Transform;
use crate::landmass::{
    Agent, AgentOptions, FromAgentRadius, Island, NavMeshBuilder,
    coords::XYZ,
    debug::{DebugDrawError, DebugDrawer, LineType, PointType, TriangleType},
};
use bevy::{math::Vec3, platform::collections::HashMap};
use std::{cmp::Ordering, sync::Arc};

struct FakeDrawer {
    points: Vec<(PointType, Vec3)>,
    lines: Vec<(LineType, [Vec3; 2])>,
    triangles: Vec<(TriangleType, [Vec3; 3])>,
}
impl DebugDrawer<XYZ> for FakeDrawer {
    fn add_point(&mut self, point_type: PointType, point: Vec3) {
        self.points.push((point_type, point));
    }

    fn add_line(&mut self, line_type: LineType, line: [Vec3; 2]) {
        self.lines.push((line_type, line));
    }

    fn add_triangle(&mut self, triangle_type: TriangleType, triangle: [Vec3; 3]) {
        self.triangles.push((triangle_type, triangle));
    }
}

impl FakeDrawer {
    fn new() -> Self {
        Self {
            points: vec![],
            lines: vec![],
            triangles: vec![],
        }
    }

    fn sort(&mut self) {
        fn lex_order_points(a: Vec3, b: Vec3) -> Ordering {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then(a.y.partial_cmp(&b.y).unwrap())
                .then(a.z.partial_cmp(&b.z).unwrap())
        }
        self.points
            .sort_by(|a, b| a.0.cmp(&b.0).then(lex_order_points(a.1, b.1)));
        self.lines.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(lex_order_points(a.1[0], b.1[0]))
                .then(lex_order_points(a.1[1], b.1[1]))
        });
        self.triangles.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(lex_order_points(a.1[0], b.1[0]))
                .then(lex_order_points(a.1[1], b.1[1]))
                .then(lex_order_points(a.1[2], b.1[2]))
        });
    }
}

#[test]
fn draws_island_meshes_and_agents() {
    const TRANSLATION: Vec3 = Vec3::ONE;

    let nav_mesh = NavMeshBuilder {
        vertices: vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(4.0, 1.0, 0.0),
            Vec3::new(4.0, 2.0, 0.0),
            Vec3::new(2.0, 3.0, 0.0),
            Vec3::new(1.0, 3.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 5.0, 0.0),
            Vec3::new(2.0, 5.0, 0.0),
            Vec3::new(2.0, 4.0, 0.0),
            Vec3::new(3.0, 5.0, 1.0),
            Vec3::new(3.0, 4.0, 1.0),
            Vec3::new(3.0, 4.0, -2.0),
            Vec3::new(3.0, 3.0, -2.0),
        ],
        polygons: vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![5, 4, 10, 9, 8],
            vec![9, 10, 12, 11],
            vec![10, 4, 14, 13],
        ],
        polygon_type_indices: vec![0, 0, 0, 0],
    }
    .validate()
    .expect("Mesh is valid.");

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    world.add_island(Island::new(
        Transform::new(TRANSLATION, 0.0),
        Arc::new(nav_mesh),
    ));

    let agent_id = world.add_agent(Agent::new(
        Vec3::new(3.9, 1.5, 0.0) + TRANSLATION,
        1.0,
        1.0,
        1.0,
    ));
    world.agent_mut(agent_id).current_target = Some(Vec3::new(1.5, 4.5, 0.0) + TRANSLATION);

    // Update so everything is in sync.
    world.update(&options, 1.0);

    let mut fake_drawer = FakeDrawer::new();

    world
        .draw_debug(&mut fake_drawer, &options)
        .expect("the archipelago can be debug-drawed");

    fake_drawer.sort();

    assert_eq!(
        fake_drawer.points,
        [
            (PointType::AgentPosition(agent_id), Vec3::new(3.9, 1.5, 0.0)),
            (
                PointType::TargetPosition(agent_id),
                Vec3::new(1.5, 4.5, 0.0)
            ),
            (PointType::Waypoint(agent_id), Vec3::new(2.0, 3.0, 0.0)),
        ]
        .iter()
        .copied()
        .map(|(t, p)| (t, p + TRANSLATION))
        .collect::<Vec<_>>()
    );
    assert_eq!(
        fake_drawer.lines,
        [
            (
                LineType::BoundaryEdge,
                [Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 1.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(1.0, 3.0, 0.0), Vec3::new(0.0, 2.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(1.0, 5.0, 0.0), Vec3::new(1.0, 3.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 0.0, 0.0), Vec3::new(4.0, 1.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 3.0, 0.0), Vec3::new(3.0, 3.0, -2.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 4.0, 0.0), Vec3::new(3.0, 4.0, 1.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 5.0, 0.0), Vec3::new(1.0, 5.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(3.0, 3.0, -2.0), Vec3::new(3.0, 4.0, -2.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(3.0, 4.0, -2.0), Vec3::new(2.0, 4.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(3.0, 4.0, 1.0), Vec3::new(3.0, 5.0, 1.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(3.0, 5.0, 1.0), Vec3::new(2.0, 5.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(4.0, 1.0, 0.0), Vec3::new(4.0, 2.0, 0.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(4.0, 2.0, 0.0), Vec3::new(2.0, 3.0, 0.0)]
            ),
            //
            (
                LineType::ConnectivityEdge,
                [Vec3::new(2.0, 3.0, 0.0), Vec3::new(1.0, 3.0, 0.0)]
            ),
            (
                LineType::ConnectivityEdge,
                [Vec3::new(2.0, 3.0, 0.0), Vec3::new(2.0, 4.0, 0.0)]
            ),
            (
                LineType::ConnectivityEdge,
                [Vec3::new(2.0, 4.0, 0.0), Vec3::new(2.0, 5.0, 0.0)]
            ),
            //
            (
                LineType::AgentCorridor(agent_id),
                [Vec3::new(1.75, 1.5, 0.0), Vec3::new(1.6, 4.0, 0.0)]
            ),
            //
            (
                LineType::Target(agent_id),
                [Vec3::new(3.9, 1.5, 0.0), Vec3::new(1.5, 4.5, 0.0)]
            ),
            //
            (
                LineType::Waypoint(agent_id),
                [Vec3::new(3.9, 1.5, 0.0), Vec3::new(2.0, 3.0, 0.0)]
            ),
        ]
        .iter()
        .copied()
        .map(|(t, [p0, p1])| (t, [p0 + TRANSLATION, p1 + TRANSLATION]))
        .collect::<Vec<_>>()
    );
    assert_eq!(
        fake_drawer.triangles,
        [
            (
                TriangleType::Node,
                [
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(2.0, 0.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(1.0, 3.0, 0.0),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(1.0, 3.0, 0.0),
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(1.6, 4.0, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(1.0, 5.0, 0.0),
                    Vec3::new(1.0, 3.0, 0.0),
                    Vec3::new(1.6, 4.0, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 0.0, 0.0),
                    Vec3::new(4.0, 1.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(1.0, 3.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(1.6, 4.0, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(3.0, 3.0, -2.0),
                    Vec3::new(2.5, 3.5, -1.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(2.5, 3.5, -1.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(2.0, 5.0, 0.0),
                    Vec3::new(1.6, 4.0, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(3.0, 4.0, 1.0),
                    Vec3::new(2.5, 4.5, 0.5)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 5.0, 0.0),
                    Vec3::new(1.0, 5.0, 0.0),
                    Vec3::new(1.6, 4., 0.00)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 5.0, 0.0),
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(2.5, 4.5, 0.5)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(3.0, 3.0, -2.0),
                    Vec3::new(3.0, 4.0, -2.0),
                    Vec3::new(2.5, 3.5, -1.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(3.0, 4.0, -2.0),
                    Vec3::new(2.0, 4.0, 0.0),
                    Vec3::new(2.5, 3.5, -1.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(3.0, 4.0, 1.0),
                    Vec3::new(3.0, 5.0, 1.0),
                    Vec3::new(2.5, 4.5, 0.5)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(3.0, 5.0, 1.0),
                    Vec3::new(2.0, 5.0, 0.0),
                    Vec3::new(2.5, 4.5, 0.5)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(4.0, 1.0, 0.0),
                    Vec3::new(4.0, 2.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(4.0, 2.0, 0.0),
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(1.75, 1.5, 0.0)
                ]
            ),
        ]
        .iter()
        .copied()
        .map(|(t, [p0, p1, p2])| (t, [p0 + TRANSLATION, p1 + TRANSLATION, p2 + TRANSLATION]))
        .collect::<Vec<_>>()
    );
}

#[test]
fn draws_boundary_links() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(2.0, 1.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(1.0, 2.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("The mesh is valid."),
    );

    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    world.add_island(Island::new(Transform::default(), nav_mesh.clone()));
    world.add_island(Island::new(
        Transform::new(Vec3::new(1.0, 0.0, 0.0), 0.0),
        nav_mesh.clone(),
    ));

    // Update so everything is in sync.
    world.update(&options, 1.0);

    let mut fake_drawer = FakeDrawer::new();
    world
        .draw_debug(&mut fake_drawer, &options)
        .expect("the archipelago can be debug-drawed");
    fake_drawer.sort();

    let lines = fake_drawer
        .lines
        .iter()
        .filter(|(line_type, _)| *line_type == LineType::BoundaryLink)
        .map(|(_, edge)| *edge)
        .collect::<Vec<_>>();

    assert_eq!(
        lines,
        [[Vec3::new(2.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 1.0)]]
    );
}

#[test]
#[ignore]
fn fails_to_draw_dirty_archipelago() {
    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(2.0, 1.0, 1.0),
                Vec3::new(2.0, 2.0, 1.0),
                Vec3::new(1.0, 2.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("The mesh is valid."),
    );

    let mut fake_drawer = FakeDrawer::new();

    // A brand new archipelago is considered clean.
    let options = AgentOptions::from_agent_radius(0.5);
    let mut world = ArchipelagoWorld::<XYZ>::new();

    assert_eq!(world.draw_debug(&mut fake_drawer, &options), Ok(()));

    // Creating an island marks the nav data as dirty.
    let island_id = world.add_island(Island::new(Transform::default(), nav_mesh.clone()));
    assert_eq!(
        world.draw_debug(&mut fake_drawer, &options),
        Err(DebugDrawError::NavDataDirty)
    );

    world.update(&options, 1.0);

    // Nav data is clean again.
    assert_eq!(world.draw_debug(&mut fake_drawer, &options), Ok(()));

    // Setting a nav mesh marks the nav data as dirty.
    world
        .island_mut(island_id)
        .unwrap()
        .set_nav_mesh(nav_mesh.clone());

    assert_eq!(
        world.draw_debug(&mut fake_drawer, &options),
        Err(DebugDrawError::NavDataDirty)
    );
}

#[cfg(feature = "debug-avoidance")]
#[googletest::gtest]
fn draws_avoidance_data_when_requested() {
    use crate::landmass::{
        AgentId,
        debug::{AvoidanceDrawer, ConstraintKind, ConstraintLine, draw_avoidance_data},
    };
    use bevy::{ecs::entity::Entity, math::Vec2};
    use googletest::{matcher::MatcherResult, prelude::*};

    #[derive(Default)]
    struct FakeAvoidanceDrawer(HashMap<Entity, HashMap<ConstraintKind, Vec<ConstraintLine>>>);

    impl AvoidanceDrawer for FakeAvoidanceDrawer {
        fn add_constraint(
            &mut self,
            agent: AgentId,
            constraint: ConstraintLine,
            kind: ConstraintKind,
        ) {
            self.0
                .entry(agent.0)
                .or_default()
                .entry(kind)
                .or_default()
                .push(constraint);
        }
    }

    #[derive(MatcherBase)]
    struct EquivLineMatcher(ConstraintLine);

    impl Matcher<&ConstraintLine> for EquivLineMatcher {
        fn matches(&self, actual: &ConstraintLine) -> MatcherResult {
            if self.0.normal.angle_to(actual.normal).abs() >= 1e-3 {
                // The lines don't point in the same direction.
                return MatcherResult::NoMatch;
            }
            if (self.0.point - actual.point).dot(actual.normal).abs() >= 1e-3 {
                // The expected line point is not on the actual line.
                return MatcherResult::NoMatch;
            }
            MatcherResult::Match
        }

        fn describe(&self, matcher_result: MatcherResult) -> googletest::description::Description {
            match matcher_result {
                MatcherResult::Match => format!("is equivalent to the line {:?}", self.0).into(),
                MatcherResult::NoMatch => {
                    format!("isn't equivalent to the line {:?}", self.0).into()
                }
            }
        }
    }

    fn equiv_line<'a>(line: ConstraintLine) -> impl Matcher<&'a ConstraintLine> {
        EquivLineMatcher(line)
    }

    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(11.0, 1.0, 1.0),
                Vec3::new(11.0, 11.0, 1.0),
                Vec3::new(1.0, 11.0, 1.0),
            ],
            polygons: vec![vec![0, 1, 2, 3]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("The mesh is valid."),
    );

    let options = AgentOptions {
        neighbourhood: 100.0,
        avoidance_time_horizon: 100.0,
        obstacle_avoidance_time_horizon: 100.0,
        ..AgentOptions::from_agent_radius(0.5)
    };

    let mut world = ArchipelagoWorld::<XYZ>::new();
    world.add_island(Island::new(Transform::default(), nav_mesh.clone()));

    let agent_1 = world.add_agent({
        let mut agent = Agent::new(Vec3::new(6.0, 2.0, 1.0), 0.5, 1.0, 1.0)
            // Use a velocity that allows both agents to agree on their "passing side".
            .with_velocity(Vec3::new(1.0, 1.0, 0.0))
            .with_target(Vec3::new(6.0, 10.0, 1.0));
        // This agent we want to see the avoidance data for.
        agent.keep_avoidance_data = true;
        agent
    });

    world.add_agent(
        Agent::new(Vec3::new(6.0, 10.0, 1.0), 0.5, 1.0, 1.0).with_target(Vec3::new(6.0, 2.0, 1.0)),
    );

    // We now have avoidance data for agent_1.
    world.update(&options, 1.0);

    let mut drawer = FakeAvoidanceDrawer::default();
    draw_avoidance_data::<XYZ>(world.agents(), &mut drawer);

    // Only one of the agents was rendered.

    assert_eq!(drawer.0.len(), 1);
    let agent_1 = drawer.0.get(&agent_1.0).unwrap();
    assert_eq!(agent_1.len(), 1);
    let original = agent_1.get(&ConstraintKind::Original).unwrap();

    expect_that!(
        original,
        unordered_elements_are!(
            // Lines for the edges of the nav mesh.
            equiv_line(ConstraintLine {
                normal: Vec2::new(-1.0, 0.0),
                point: Vec2::new(0.05, 0.0),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(0.0, 1.0),
                point: Vec2::new(0.0, -0.01),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(1.0, 0.0),
                point: Vec2::new(-0.05, 0.0),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(0.0, -1.0),
                point: Vec2::new(0.0, 0.09),
            }),
            // Line for the other agent.
            equiv_line(ConstraintLine {
                normal: -Vec2::new(1.007_905, 8.0).normalize().perp(),
                point: Vec2::new(0.0, 0.0),
            }),
        )
    );
}
