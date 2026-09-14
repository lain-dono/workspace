use crate::coords::ThreeD;
use crate::debug::{self, DebugDrawer, LineType, PointType, TriangleType, draw_archipelago_debug};
use crate::landmass::{AgentOptions, Archipelago, FromAgentRadius};
use crate::nav_mesh::NavMeshHandle;
use crate::prelude::{Agent3d, LandmassPlugin3d, NavMesh3d, NavigationMesh3d};
use bevy::asset::{AssetPlugin, Assets};
use bevy::ecs::system::Res;
use bevy::math::Vec3;
use bevy::platform::collections::HashMap;
use bevy::transform::{TransformPlugin, components::Transform};
use bevy::{
    app::App,
    ecs::{
        entity::Entity,
        system::{Query, SystemState},
    },
};
use std::{cmp::Ordering, sync::Arc};

struct FakeDrawer {
    points: Vec<(PointType, Vec3)>,
    lines: Vec<(LineType, [Vec3; 2])>,
    triangles: Vec<(TriangleType, [Vec3; 3])>,
}
impl DebugDrawer<ThreeD> for FakeDrawer {
    fn add_point(&mut self, point_type: debug::PointType, point: Vec3) {
        self.points.push((point_type, point));
    }

    fn add_line(&mut self, line_type: debug::LineType, line: [Vec3; 2]) {
        self.lines.push((line_type, line));
    }

    fn add_triangle(&mut self, triangle_type: debug::TriangleType, triangle: [Vec3; 3]) {
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
fn draws_archipelago_debug() {
    let mut app = App::new();

    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::time::TimePlugin,
        bevy::app::ScheduleRunnerPlugin::default(),
    ))
    .add_plugins(TransformPlugin)
    .add_plugins(AssetPlugin::default())
    .add_plugins(LandmassPlugin3d::default());

    app.insert_resource(Archipelago::<ThreeD>::new(0.01));
    app.insert_resource(AgentOptions::<ThreeD>::from_agent_radius(0.5));

    // Update early to allow the time to not be 0.0.
    app.update();

    let nav_mesh = Arc::new(
        NavigationMesh3d {
            vertices: vec![
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(4.0, 0.0, 1.0),
                Vec3::new(4.0, 0.0, 4.0),
                Vec3::new(1.0, 0.0, 4.0),
            ],
            polygons: vec![vec![3, 2, 1, 0]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("is valid"),
    );

    let nav_mesh_handle = app
        .world_mut()
        .resource_mut::<Assets<NavMesh3d>>()
        .add(NavMesh3d {
            nav_mesh,
            type_index_to_node_type: HashMap::default(),
        });

    app.world_mut().spawn((
        Transform::from_translation(Vec3::new(1.0, 1.0, 1.0)),
        NavMeshHandle(nav_mesh_handle.clone()),
    ));

    let agent = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(3.0, 1.0, 3.0)),
            Agent3d::new(Vec3::ZERO, 0.5, 1.0, 1.0),
        ))
        .id();

    // Sync the islands with landmass.
    app.update();

    let mut fake_drawer = FakeDrawer::new();

    {
        let world = app.world_mut();

        let mut system_state: SystemState<(
            Res<Archipelago<ThreeD>>,
            Res<AgentOptions<ThreeD>>,
            Query<(Entity, &crate::landmass::Agent<ThreeD>)>,
            Query<(Entity, &crate::landmass::Island<ThreeD>)>,
        )> = SystemState::new(world);

        let (nav, options, agents, islands) = system_state.get_mut(world);
        draw_archipelago_debug(agents, islands, &nav, &mut fake_drawer, &options).unwrap();
    }

    fake_drawer.sort();

    assert_eq!(
        fake_drawer.points,
        [(PointType::AgentPosition(agent), Vec3::new(3.0, 1.0, 3.0))]
    );

    assert_eq!(
        fake_drawer.lines,
        [
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 1.0, 2.0), Vec3::new(2.0, 1.0, 5.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(2.0, 1.0, 5.0), Vec3::new(5.0, 1.0, 5.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(5.0, 1.0, 2.0), Vec3::new(2.0, 1.0, 2.0)]
            ),
            (
                LineType::BoundaryEdge,
                [Vec3::new(5.0, 1.0, 5.0), Vec3::new(5.0, 1.0, 2.0)]
            ),
        ]
    );

    assert_eq!(
        fake_drawer.triangles,
        [
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 1.0, 2.0),
                    Vec3::new(2.0, 1.0, 5.0),
                    Vec3::new(3.5, 1.0, 3.5),
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(2.0, 1.0, 5.0),
                    Vec3::new(5.0, 1.0, 5.0),
                    Vec3::new(3.5, 1.0, 3.5),
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(5.0, 1.0, 2.0),
                    Vec3::new(2.0, 1.0, 2.0),
                    Vec3::new(3.5, 1.0, 3.5),
                ]
            ),
            (
                TriangleType::Node,
                [
                    Vec3::new(5.0, 1.0, 5.0),
                    Vec3::new(5.0, 1.0, 2.0),
                    Vec3::new(3.5, 1.0, 3.5),
                ]
            ),
        ]
    );
}

#[cfg(feature = "debug-avoidance")]
#[googletest::gtest]
fn draws_avoidance_data_when_requested() {
    use crate::agent::KeepAvoidanceData;
    use crate::debug::{AvoidanceDrawer, ConstraintKind, ConstraintLine, draw_avoidance_data};
    use crate::landmass::NavMeshBuilder;
    use crate::prelude::AgentTarget3d;
    use bevy::{math::Vec2, prelude::Entity};
    use googletest::{matcher::MatcherResult, prelude::*};

    #[derive(Default)]
    struct FakeAvoidanceDrawer(HashMap<Entity, HashMap<ConstraintKind, Vec<ConstraintLine>>>);

    impl AvoidanceDrawer for FakeAvoidanceDrawer {
        fn add_constraint(
            &mut self,
            agent: Entity,
            constraint: ConstraintLine,
            kind: ConstraintKind,
        ) {
            self.0
                .entry(agent)
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

    let mut app = App::new();

    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::time::TimePlugin,
        bevy::app::ScheduleRunnerPlugin::default(),
    ))
    .add_plugins(TransformPlugin)
    .add_plugins(AssetPlugin::default())
    .add_plugins(LandmassPlugin3d::default());

    app.insert_resource(Archipelago::<ThreeD>::new(0.01));
    app.insert_resource(AgentOptions::<ThreeD> {
        neighbourhood: 100.0,
        avoidance_time_horizon: 100.0,
        obstacle_avoidance_time_horizon: 100.0,
        ..AgentOptions::from_agent_radius(0.5)
    });

    // Update early to allow the time to not be 0.0.
    app.update();

    let nav_mesh = Arc::new(
        NavMeshBuilder {
            vertices: vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(11.0, 1.0, 1.0),
                Vec3::new(11.0, 1.0, 11.0),
                Vec3::new(1.0, 1.0, 11.0),
            ],
            polygons: vec![vec![3, 2, 1, 0]],
            polygon_type_indices: vec![0],
        }
        .validate()
        .expect("The mesh is valid."),
    );

    let nav_mesh_handle = app
        .world_mut()
        .resource_mut::<Assets<NavMesh3d>>()
        .add(NavMesh3d {
            nav_mesh,
            type_index_to_node_type: HashMap::default(),
        });

    app.world_mut()
        .spawn((Transform::default(), NavMeshHandle(nav_mesh_handle.clone())));

    let agent_1 = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(6.0, 1.0, 2.0)),
            Agent3d::new(Vec3::ZERO, 0.5, 1.0, 1.0)
                // Use a velocity that allows both agents to agree on their "passing side".
                .with_velocity(Vec3::new(1.0, 0.0, 1.0)),
            AgentTarget3d::Point(Vec3::new(6.0, 1.0, 10.0)),
            KeepAvoidanceData,
        ))
        .id();

    app.world_mut().spawn((
        Transform::from_translation(Vec3::new(6.0, 1.0, 10.0)),
        Agent3d::new(Vec3::ZERO, 0.5, 1.0, 1.0),
        AgentTarget3d::Point(Vec3::new(6.0, 1.0, 2.0)),
    ));

    // We now have avoidance data for agent_1.
    app.update();

    let mut drawer = FakeAvoidanceDrawer::default();

    {
        let world = app.world_mut();
        let mut agents = world.query::<(Entity, &crate::landmass::Agent<ThreeD>)>();
        let agents = agents.query(world);
        draw_avoidance_data(agents, &mut drawer);
    }

    // I would ideally use three levels of unordered_elements_are here instead,
    // but due to https://github.com/rust-lang/rust/issues/134719
    expect_eq!(drawer.0.len(), 1);
    let agent_constraints = drawer.0.get(&agent_1).unwrap();
    expect_eq!(agent_constraints.len(), 1);
    let agent_constraints = agent_constraints.get(&ConstraintKind::Original).unwrap();

    // Only one of the agents was rendered.
    expect_that!(
        agent_constraints,
        unordered_elements_are!(
            // Lines for the edges of the nav mesh.
            equiv_line(ConstraintLine {
                normal: Vec2::new(-1.0, 0.0),
                point: Vec2::new(0.05, 0.0),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(0.0, 1.0),
                point: Vec2::new(0.0, -0.09),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(1.0, 0.0),
                point: Vec2::new(-0.05, 0.0),
            }),
            equiv_line(ConstraintLine {
                normal: Vec2::new(0.0, -1.0),
                point: Vec2::new(0.0, 0.01),
            }),
            // Line for the other agent.
            equiv_line(ConstraintLine {
                normal: Vec2::new(1.007_905, -8.0).normalize().perp(),
                point: Vec2::new(0.0, 0.0),
            }),
        )
    );
}
