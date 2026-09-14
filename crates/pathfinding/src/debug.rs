use crate::coords::{CoordinateSystem, ThreeD, TwoD};
use crate::landmass::{AgentOptions, Archipelago};
use crate::{LandmassSystemSet, landmass};
use bevy::app::{Plugin, Update};
use bevy::color::Color;
use bevy::ecs::{
    entity::Entity,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Query, Res},
};
use bevy::gizmos::{
    AppGizmoBuilder,
    config::{GizmoConfig, GizmoConfigGroup},
    gizmos::Gizmos,
};
use bevy::math::{Isometry3d, Quat};
use bevy::reflect::Reflect;
use bevy::time::Time;
use bevy::transform::components::Transform;
use std::marker::PhantomData;

#[cfg(feature = "debug-avoidance")]
use bevy::math::Vec2;

pub use landmass::debug::DebugDrawError;

#[cfg(feature = "debug-avoidance")]
pub use landmass::debug::ConstraintKind;

/// The type of debug points.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PointType {
    /// The position of an agent.
    AgentPosition(Entity),
    /// The target of an agent.
    TargetPosition(Entity),
    /// The waypoint of an agent.
    Waypoint(Entity),
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
    AgentCorridor(Entity),
    /// Line from an agent to its target.
    Target(Entity),
    /// Line to the waypoint of an agent.
    Waypoint(Entity),
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
}

/// Draws all parts of `archipelago` to `debug_drawer`. This is a lower level
/// API to allow custom debug drawing. For a pre-made implementation, use
/// [`LandmassDebugPlugin`].
pub fn draw_archipelago_debug<T: CoordinateSystem>(
    agents: Query<(Entity, &landmass::Agent<T>)>,
    islands: Query<(Entity, &'static landmass::Island<T>)>,
    nav: &Archipelago<T>,
    debug_drawer: &mut impl DebugDrawer<T>,
    options: &AgentOptions<T>,
) -> Result<(), DebugDrawError> {
    struct DebugDrawerAdapter<'a, T: CoordinateSystem, D: DebugDrawer<T>> {
        drawer: &'a mut D,
        marker: PhantomData<T>,
    }

    impl<T: CoordinateSystem, D: DebugDrawer<T>> landmass::debug::DebugDrawer<T>
        for DebugDrawerAdapter<'_, T, D>
    {
        fn add_point(&mut self, point_type: landmass::debug::PointType, point: T::Coord) {
            let point_type = match point_type {
                landmass::debug::PointType::AgentPosition(agent_id) => {
                    PointType::AgentPosition(agent_id.0)
                }
                landmass::debug::PointType::TargetPosition(agent_id) => {
                    PointType::TargetPosition(agent_id.0)
                }
                landmass::debug::PointType::Waypoint(agent_id) => PointType::Waypoint(agent_id.0),
            };
            self.drawer.add_point(point_type, point);
        }

        fn add_line(&mut self, line_type: landmass::debug::LineType, line: [T::Coord; 2]) {
            let line_type = match line_type {
                landmass::debug::LineType::BoundaryEdge => LineType::BoundaryEdge,
                landmass::debug::LineType::ConnectivityEdge => LineType::ConnectivityEdge,
                landmass::debug::LineType::BoundaryLink => LineType::BoundaryLink,
                landmass::debug::LineType::AgentCorridor(agent_id) => {
                    LineType::AgentCorridor(agent_id.0)
                }
                landmass::debug::LineType::Target(agent_id) => LineType::Target(agent_id.0),
                landmass::debug::LineType::Waypoint(agent_id) => LineType::Waypoint(agent_id.0),
            };
            self.drawer.add_line(line_type, line);
        }

        fn add_triangle(
            &mut self,
            triangle_type: landmass::debug::TriangleType,
            triangle: [T::Coord; 3],
        ) {
            let triangle_type = match triangle_type {
                landmass::debug::TriangleType::Node => TriangleType::Node,
            };
            self.drawer.add_triangle(triangle_type, triangle);
        }
    }

    let mut drawer = DebugDrawerAdapter::<T, _> {
        drawer: debug_drawer,
        marker: PhantomData,
    };

    landmass::debug::draw_archipelago_debug(agents, islands, nav, &mut drawer, options)
}

#[cfg(feature = "debug-avoidance")]
/// A constraint in velocity-space for an agent's velocity for local collision
/// avoidance. The constraint restricts the velocity to lie on one side of a
/// line (aka., only a half-plane is considered valid). This is equivalent to
/// [`landmass::debug::ConstraintLine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintLine {
    /// A point on the line separating the valid and invalid velocities.
    pub point: Vec2,
    /// The normal of the line separating the valid and invalid velocities. The
    /// normal always points towards the valid velocities.
    pub normal: Vec2,
}

#[cfg(feature = "debug-avoidance")]
/// A trait for reporting agent local collision avoidance constraints.
pub trait AvoidanceDrawer {
    /// Reports a single avoidance constraint.
    fn add_constraint(&mut self, agent: Entity, constraint: ConstraintLine, kind: ConstraintKind);
}

#[cfg(feature = "debug-avoidance")]
impl ConstraintLine {
    fn from_landmass(line: &landmass::debug::ConstraintLine) -> Self {
        Self {
            point: Vec2::new(line.point.x, line.point.y),
            normal: Vec2::new(line.normal.x, line.normal.y),
        }
    }
}

/// Draws the avoidance data for any agent marked with TODO
#[cfg(feature = "debug-avoidance")]
pub fn draw_avoidance_data<T: CoordinateSystem>(
    agents: Query<(Entity, &landmass::Agent<T>)>,
    drawer: &mut impl AvoidanceDrawer,
) {
    struct AvoidanceDrawerAdapter<'a, T: CoordinateSystem, D: AvoidanceDrawer> {
        drawer: &'a mut D,
        marker: PhantomData<T>,
    }

    impl<T: CoordinateSystem, D: AvoidanceDrawer> landmass::debug::AvoidanceDrawer
        for AvoidanceDrawerAdapter<'_, T, D>
    {
        fn add_constraint(
            &mut self,
            agent: landmass::AgentId,
            constraint: landmass::debug::ConstraintLine,
            kind: landmass::debug::ConstraintKind,
        ) {
            self.drawer
                .add_constraint(agent.0, ConstraintLine::from_landmass(&constraint), kind);
        }
    }

    landmass::debug::draw_avoidance_data::<T>(
        agents,
        &mut AvoidanceDrawerAdapter::<T, _> {
            drawer,
            marker: PhantomData,
        },
    );
}

/// A plugin to draw landmass debug data with Bevy gizmos.
pub struct LandmassDebugPlugin<T: CoordinateSystem> {
    /// Whether to begin drawing on startup.
    pub draw_on_start: bool,
    // Marker for the coordinate systems.
    pub marker: PhantomData<T>,
}

pub type Landmass2dDebugPlugin = LandmassDebugPlugin<TwoD>;
pub type Landmass3dDebugPlugin = LandmassDebugPlugin<ThreeD>;

impl<T: CoordinateSystem> Default for LandmassDebugPlugin<T> {
    fn default() -> Self {
        Self {
            draw_on_start: true,
            marker: PhantomData,
        }
    }
}

impl<T: CoordinateSystem> Plugin for LandmassDebugPlugin<T> {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(EnableLandmassDebug(self.draw_on_start))
            .add_systems(
                Update,
                draw_archipelagos_default::<T>
                    .in_set(LandmassSystemSet::Post)
                    .run_if(|enable: Res<EnableLandmassDebug>| enable.0),
            )
            .insert_gizmo_config(
                LandmassGizmoConfigGroup,
                GizmoConfig {
                    depth_bias: -1.0,
                    ..Default::default()
                },
            );
    }
}

/// A resource controlling whether debug data is drawn.
#[derive(Resource, Default)]
pub struct EnableLandmassDebug(pub bool);

impl std::ops::Deref for EnableLandmassDebug {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for EnableLandmassDebug {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A config group for landmass debug gizmos.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LandmassGizmoConfigGroup;

/// A gizmo debug drawer.
struct GizmoDrawer<'w, 's, 'a, T: CoordinateSystem>(
    &'a mut Gizmos<'w, 's, LandmassGizmoConfigGroup>,
    PhantomData<T>,
);

impl<T: CoordinateSystem> DebugDrawer<T> for GizmoDrawer<'_, '_, '_, T> {
    fn add_point(&mut self, point_type: PointType, point: T::Coord) {
        self.0.sphere(
            Isometry3d::new(T::to_world_position(point), Quat::IDENTITY),
            0.2,
            match point_type {
                PointType::AgentPosition(_) => Color::srgba(0.0, 1.0, 0.0, 0.6),
                PointType::TargetPosition(_) => Color::srgba(1.0, 1.0, 0.0, 0.6),
                PointType::Waypoint(_) => Color::srgba(0.6, 0.6, 0.6, 0.6),
            },
        );
    }

    fn add_line(&mut self, line_type: LineType, line: [T::Coord; 2]) {
        if line_type == LineType::BoundaryLink {
            let line = [T::to_world_position(line[0]), T::to_world_position(line[1])];
            self.0.cuboid(
                Transform::default()
                    .looking_to(line[1] - line[0], bevy::math::Vec3::new(0.0, 1.0, 0.0))
                    .with_translation((line[0] + line[1]) * 0.5)
                    .with_scale(bevy::math::Vec3::new(0.01, 0.01, line[0].distance(line[1]))),
                Color::srgba(0.0, 1.0, 0.0, 0.6),
            );
            return;
        }
        self.0.line(
            T::to_world_position(line[0]),
            T::to_world_position(line[1]),
            match line_type {
                LineType::BoundaryEdge => Color::srgba(0.0, 0.0, 1.0, 0.6),
                LineType::ConnectivityEdge => Color::srgba(0.5, 0.5, 1.0, 0.6),
                LineType::BoundaryLink => unreachable!(),
                LineType::AgentCorridor(_) => Color::srgba(0.6, 0.0, 0.6, 0.6),
                LineType::Target(_) => Color::srgba(1.0, 1.0, 0.0, 0.6),
                LineType::Waypoint(_) => Color::srgba(0.6, 0.6, 0.6, 0.6),
            },
        );
    }

    fn add_triangle(&mut self, _triangle_type: TriangleType, _triangle: [T::Coord; 3]) {
        // Bevy doesn't have a way to draw triangles :'(
    }
}

/// A system for drawing debug data.
fn draw_archipelagos_default<T: CoordinateSystem>(
    time: Res<Time>,
    nav: Res<Archipelago<T>>,
    options: Res<AgentOptions<T>>,
    mut gizmos: Gizmos<'_, '_, LandmassGizmoConfigGroup>,

    agents: Query<(Entity, &landmass::Agent<T>)>,
    islands: Query<(Entity, &'static landmass::Island<T>)>,
) {
    if time.delta_secs() == 0.0 {
        return;
    }

    let mut drawer = GizmoDrawer(&mut gizmos, PhantomData::<T>);
    draw_archipelago_debug(agents, islands, &nav, &mut drawer, &options)
        .expect("the archipelago can be debug-drawn");
}
