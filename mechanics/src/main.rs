#![allow(clippy::missing_safety_doc)]

use bevy::{
    app::ScheduleRunnerPlugin,
    asset::AssetIndex,
    ecs::{
        component::ComponentId,
        event::EventCursor,
        schedule::ScheduleLabel,
        system::{ParamBuilder, QueryParamBuilder},
        world::FilteredEntityRef,
    },
    prelude::*,
};
use std::time::Duration;

// define
// system
// struct
// script
// mixin
// patch
// apply

pub use vm;
pub mod compiler;
pub mod stat;

use self::stat::{InsertStat, Stat, StatAsset, StatIndex, StatParam, StatPlugin};
use bevy_def::def_maintain_system;

fn main() {
    init_logging();

    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(200))),
        AssetPlugin::default(),
    ));

    app.add_plugins(StatPlugin::default());

    app.add_systems(Startup, startup);
    app.add_systems(Update, (update, spawn));
    app.add_systems(
        PostUpdate,
        add_print_systems.after(def_maintain_system::<Stat>),
    );

    app.run();
}

fn init_logging() {
    use bevy::log::tracing_subscriber::prelude::*;
    use bevy::log::tracing_subscriber::{EnvFilter, Registry, fmt};

    let filter = "info,mechanics=trace";
    let env_layer = EnvFilter::builder().parse_lossy(filter);
    let fmt_layer = fmt::layer().without_time();
    let subscriber = Registry::default().with(fmt_layer).with(env_layer);

    bevy::log::tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn startup(mut stat_assets: ResMut<Assets<StatAsset>>) {
    info!("startup");

    std::mem::forget(stat_assets.add(StatAsset {
        defname: String::from("mental_break_threshold"),
        default: 0.35,
        minimal: 0.01,
        maximal: 0.50,
    }));
}

fn add_print_systems(world: &mut World, mut reader: Local<EventCursor<AssetEvent<StatAsset>>>) {
    world.resource_scope(|world, index: Mut<StatIndex>| {
        world.resource_scope(|world, events: Mut<Events<AssetEvent<StatAsset>>>| {
            for event in reader.read(&events) {
                if let &AssetEvent::Added { id, .. } = event {
                    world.add_schedule(Schedule::new(PrintSchedule));
                    world.schedule_scope(PrintSchedule, |world, schedule| {
                        for (&asset_id, &component_id) in index.asset_to_id() {
                            let system =
                                crate::compiler::print_system(world, asset_id, component_id);
                            schedule.add_systems(system);
                        }
                    });
                }
            }
        });
    });
}

fn print_sys(
    world: &mut World,
    asset_index: AssetIndex,
    component_id: ComponentId,
) -> impl System<In = (), Out = ()> {
    let query = QueryParamBuilder::new(move |builder| {
        builder.ref_id(component_id);
    });

    let system = (ParamBuilder, query).build_state(world);
    system.build_any_system(move |defs: StatParam, query: Query<FilteredEntityRef>| {
        let iter = query.into_iter();
        for entity in iter {
            let item = defs.filtered_entity_ref(&entity, asset_index).unwrap();
            println!(
                "{}: {} [{} .. {}]",
                item.asset.defname, item.value.current, item.asset.minimal, item.asset.maximal
            );
        }
    })
}

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct PrintSchedule;

#[derive(Component)]
struct SpawnMarker;

fn spawn(index: Res<StatIndex>, mut commands: Commands, spawned: Query<(), With<SpawnMarker>>) {
    if spawned.is_empty() {
        for (_, &component_id) in index.asset_to_id() {
            let mut entity = commands.spawn(SpawnMarker);
            entity.queue(InsertStat::new(component_id, Stat { current: 5.0 }));
        }
    }
}

fn update(mut commands: Commands) {
    info!("update");

    commands.run_schedule(PrintSchedule);
}
