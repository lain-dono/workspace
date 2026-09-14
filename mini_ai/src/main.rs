#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use crate::label::EntityLabel;
use crate::utils::SpawnEntity;
use ai::prelude::*;
use bevy::{ecs::component::Components, log::LogPlugin, prelude::*};

pub mod label;
pub mod lmass;
pub mod text2d;
pub mod utils;

pub mod time;

pub mod ass;
pub mod find;
pub mod nonmax;
pub mod script;
pub mod ui;

fn main() {
    init_logging();

    // let speed = std::time::Duration::from_secs_f64(1.0 / 5.0);
    // let speed = std::time::Duration::from_secs_f64(1.0);
    App::new()
        // .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(speed)))
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(ai::plugin)
        .add_plugins(crate::ass::plugin)
        .add_plugins(crate::ui::plugin)
        .add_plugins(crate::time::plugin)
        .add_plugins(crate::lmass::plugin)
        .add_systems(Startup, setup)
        .add_plugins(|app: &mut App| {
            app.add_action(Idle::init)
                .add_action_default::<Urinate>()
                .add_systems(FixedUpdate, (Idle::action, Urinate::action))
                .add_systems(FixedUpdate, (Idle::decay_0, Idle::decay_1))
                .add_systems(FixedUpdate, (XStep::step, YStep::step))
                .add_step(XStep::init)
                .add_step(YStep::init);
        })
        .add_plugins(crate::text2d::plugin)
        .run();
}

fn init_logging() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{EnvFilter, Registry, fmt};

    let filter = "error,mini_ai=trace,wgpu=error,naga=warn";
    let env_layer = EnvFilter::builder().parse_lossy(filter);
    let fmt_layer = fmt::layer().without_time();
    let subscriber = Registry::default().with(fmt_layer).with(env_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

// pub const OWNER_ATTENUATION: [f32; 5] = [0.0, 0.0, 0.10, 0.30, 0.60]; // custom, none, low, medium, high
// pub const GUEST_ATTENUATION: [f32; 5] = [0.0, 0.0, 0.01, 0.02, 0.03]; // custom, none, low, medium, high

// bitflags::bitflags! {
//     pub struct InteractionFlags: u64 {
//         const F0 = 0b00000001;
//     }
// }

pub const BLADDER: usize = 0;
pub const HUNGER: usize = 1;
pub const ENERGY: usize = 2;
pub const FUN: usize = 3;
pub const SOCIAL: usize = 4;
pub const HYGIENE: usize = 5;

pub static MOTIVE_NAMES: [&str; 6] = [
    "🚽 bladder",
    "🍴 hunger",
    "💤 energy",
    "🎮 fun",
    "💬 social",
    "🚿 hygiene",
];

pub fn setup(
    mut commands: Commands,
    components: &Components,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    use bevy::color::palettes::tailwind;

    // Light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Msaa::Off,
    ));

    let toilet = commands
        .spawn((
            Name::new("Toilet"),
            Transform::from_xyz(2.0, 0.0, 0.0),
            MeshMaterial3d(materials.add(StandardMaterial::from(Color::from(tailwind::BLUE_400)))),
            Mesh3d(meshes.add(Cylinder::new(0.5, 2.00))),
        ))
        .with_interaction((
            Name::new("urinate"),
            ai::Action::new::<Urinate>(components),
            ai::Interaction::single(0.0, ai::Ad::new(BLADDER, 0.0, 50.0)),
        ))
        .id();

    let food = commands
        .spawn((
            Name::new("Food"),
            Transform::from_xyz(-2.0, 0.0, 0.0),
            MeshMaterial3d(materials.add(StandardMaterial::from(Color::from(tailwind::RED_600)))),
            Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        ))
        .with_interaction((
            Name::new("eat step-script"),
            ai::Action::new::<ai::Step>(components),
            ai::Interaction::single(0.0, ai::Ad::new(HUNGER, 0.0, 50.0)),
            ai::Script::new(vec![
                components.component_id::<XStep>().unwrap(),
                components.component_id::<YStep>().unwrap(),
            ]),
        ))
        .id();

    let actor = commands.spawn_entity((
        Name::new("Actor"),
        ai::Actor {
            motives: smallvec::smallvec![
                ai::Motive::with_keys(40.0, 1.0, [-12, -60, -97, -120, -128, -128]),
                ai::Motive::with_keys(40.0, 1.0, [0, -51, -91, -117, -128, -128]),
                ai::Motive::with_keys(40.0, 0.9, [-12, -60, -97, -120, -128, -128]),
                ai::Motive::with_keys(40.0, 0.8, [-12, -60, -97, -120, -128, -128]),
                ai::Motive::with_keys(40.0, 0.7, [-12, -60, -97, -120, -128, -128]),
                ai::Motive::with_keys(40.0, 0.6, [-12, -60, -97, -120, -128, -128]),
            ],
            modificators: vec![0.0; 16],
            threshold: 0.1,
            default: ai::Action::new::<Idle>(components),
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::BLACK))),
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.000))),
    ));

    commands.trigger(ai::RequestAction {
        caller: actor,
        target: None,
    });

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let color = tailwind::CYAN_700;
    commands.spawn(EntityLabel::bundle(toilet, color, font.clone(), "╴Toilet╶"));
    commands.spawn(EntityLabel::bundle(food, color, font.clone(), "╴Food╶"));
    commands.spawn(EntityLabel::bundle(actor, color, font.clone(), "╴Actor╶"));

    let plane = false;
    if plane {
        // Chessboard Plane
        let black = materials.add(Color::BLACK);
        let white = materials.add(Color::WHITE);
        let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

        for x in -3 * 5..=3 * 5 {
            for z in -3 * 5..=3 * 5 {
                commands.spawn((
                    Mesh3d(plane_mesh.clone()),
                    MeshMaterial3d(if (x + z) % 2 == 0 { &black } else { &white }.clone()),
                    Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
                ));
            }
        }
    }
}

#[derive(Component, Default)]
struct Idle;

impl Idle {
    pub fn init(trigger: On<ai::ActionInit, Self>) -> (Entity, Self) {
        (trigger.caller, Self)
    }

    pub fn action(mut action: ai::ActionParam<Self>, query: Query<(Entity, &Self)>) {
        for (caller, _) in query {
            // trace!("idle...");
            action.request_action(caller, None);
        }
    }

    fn decay_0(time: Res<Time>, query: Query<&mut ai::Actor, Without<Urinate>>) {
        // fn decay_0(time: Res<Time>, query: Query<&mut ai::Actor>) {
        for mut actor in query {
            actor.motives[0].tick(&time, -10.0, -100.0);
        }
    }

    fn decay_1(time: Res<Time>, query: Query<&mut ai::Actor, Without<ai::Step>>) {
        // fn decay_1(time: Res<Time>, query: Query<&mut ai::Actor>) {
        for mut actor in query {
            actor.motives[1].tick(&time, -10.0, -100.0);
        }
    }
}

#[derive(Component)]
pub struct Urinate {
    speed: f32,
    until: f32,
}

impl Default for Urinate {
    fn default() -> Self {
        Self {
            speed: 40.0,
            until: 80.0,
        }
    }
}

impl Urinate {
    pub fn action(
        mut action: ai::ActionParam<Self>,
        time: Res<Time>,
        query: Query<(Entity, &mut ai::Actor, &Self)>,
    ) {
        for (caller, mut actor, &Urinate { speed, until }) in query {
            if actor.motives[BLADDER].tick(&time, speed, until) {
                action.stop(caller);
            }
        }
    }
}

#[derive(Component)]
pub struct XStep(Timer);

impl XStep {
    #[must_use]
    pub fn init(trigger: On<ai::StepInit, Self>) -> (Entity, Self) {
        let timer = Timer::from_seconds(2.0, TimerMode::Once);
        (trigger.caller, Self(timer))
    }

    pub fn step(mut step: ai::StepParam<Self>, time: Res<Time>, query: Query<(Entity, &mut Self)>) {
        for (caller, mut this) in query {
            this.0.tick(time.delta());
            if this.0.is_finished() {
                step.stop(caller);
            }
        }
    }
}

#[derive(Component, Default)]
pub struct YStep;

impl YStep {
    #[must_use]
    pub fn init(trigger: On<ai::StepInit, Self>) -> (Entity, Self) {
        (trigger.caller, Self)
    }

    pub fn step(
        mut step: ai::StepParam<Self>,
        time: Res<Time>,
        query: Query<(Entity, &mut ai::Actor, &Self)>,
    ) {
        for (caller, mut actor, _) in query {
            if actor.motives[HUNGER].tick(&time, 40.0, 100.0) {
                step.stop(caller);
            }
        }
    }
}
