use bevy::{
    app::ScheduleRunnerPlugin,
    asset::AssetIndex,
    ecs::{
        component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
        query::QueryIter,
        schedule::ScheduleLabel,
        system::{ParamBuilder, QueryParamBuilder, SystemParam},
        world::FilteredEntityRef,
    },
    platform::collections::HashMap,
    prelude::*,
    ptr::OwningPtr,
};
use std::{alloc::Layout, time::Duration};

// define
// system
// struct
// script
// mixin
// patch
// apply

fn notes() {
    struct StatComponent {
        current: f32,
    }

    struct StatAsset {
        default: f32,
        minimum: f32,
        maximum: f32,
    }

    struct StatScript {
        current: f32,

        default: f32,
        minimum: f32,
        maximum: f32,
    }
}

mod def;
mod lang;
pub use vm;

use self::def::DefIndex;

fn main() {
    init_logging();

    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(200))),
        AssetPlugin::default(),
    ));

    app.init_resource::<DefIndex<StatAsset>>();
    app.init_asset::<StatAsset>();

    app.add_systems(Startup, (startup, add_print_systems, spawn).chain());

    app.add_systems(Update, update);

    app.run();
}

fn init_logging() {
    use bevy::log::tracing_subscriber::prelude::*;
    use bevy::log::tracing_subscriber::{fmt, EnvFilter, Registry};

    let filter = "info,mechanics=trace";
    let env_layer = EnvFilter::builder().parse_lossy(filter);
    let fmt_layer = fmt::layer().without_time();
    let subscriber = Registry::default().with(fmt_layer).with(env_layer);

    bevy::log::tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn startup(world: &mut World) {
    info!("startup");

    world.resource_scope(|world, mut assets: Mut<Assets<StatAsset>>| {
        world.resource_scope(|world, mut index: Mut<DefIndex<StatAsset>>| {
            let name = "mental_break_threshold";
            let asset = StatAsset {
                name: String::from(name),
                default: 0.35,
                minimal: 0.01,
                maximal: 0.50,
            };

            index.register(world, &mut assets, asset, name);
        });
    });
}

fn add_print_systems(world: &mut World) {
    world.resource_scope(|world, index: Mut<DefIndex<StatAsset>>| {
        world.add_schedule(Schedule::new(PrintSchedule));
        world.schedule_scope(PrintSchedule, |world, schedule| {
            for (&asset_id, &component_id) in &index.asset_to_id {
                schedule.add_systems(print_system(world, asset_id, component_id));
            }
        });
    });
}

fn print_system(
    world: &mut World,
    asset_id: AssetId<StatAsset>,
    component_id: ComponentId,
) -> impl System<In = (), Out = ()> {
    let AssetId::Index { index, .. } = asset_id else {
        unreachable!()
    };

    fn stat_id(index: bevy::asset::AssetIndex) -> AssetId<StatAsset> {
        let marker = std::marker::PhantomData;
        AssetId::Index { index, marker }
    }

    // fn access_def_value<T, A>(
    //     mut ctx: lang::Context,
    //     name: &dyn PartialReflect,
    // ) -> lang::ScriptResult {
    //     let name = name.try_downcast_ref::<String>().unwrap();
    //     let value = ctx.local_scope(name, |ptr| unsafe {
    //         (*ptr.as_ptr().cast::<StatRef>()).value
    //     });
    //     ctx.push(value.unwrap());
    //     Ok(())
    // }

    // fn access_ref_asset<T, A>(
    //     mut ctx: lang::Context,
    //     name: &dyn PartialReflect,
    // ) -> lang::ScriptResult {
    //     let name = name.try_downcast_ref::<String>().unwrap();
    //     let value = ctx.local_scope(name, |ptr| unsafe {
    //         (*ptr.as_ptr().cast::<StatRef>()).value
    //     });
    //     ctx.push(value.unwrap());
    //     Ok(())
    // }

    fn op_jump(mut ctx: lang::Context, addr: &dyn PartialReflect) -> lang::ScriptResult {
        let addr = *addr.try_downcast_ref::<usize>().unwrap();
        ctx.jump(addr);
        Ok(true)
    }

    fn branch_if_true(mut ctx: lang::Context, addr: &dyn PartialReflect) -> lang::ScriptResult {
        let cond = ctx.pop()?;
        let cond = *cond.try_downcast_ref::<bool>().unwrap();
        let addr = *addr.try_downcast_ref::<usize>().unwrap();
        if cond {
            ctx.jump(addr);
        }
        Ok(cond)
    }

    fn branch_if_false(mut ctx: lang::Context, addr: &dyn PartialReflect) -> lang::ScriptResult {
        let cond = ctx.pop()?;
        let cond = *cond.try_downcast_ref::<bool>().unwrap();
        let addr = *addr.try_downcast_ref::<usize>().unwrap();
        if !cond {
            ctx.jump(addr);
        }
        Ok(!cond)
    }

    fn op_dbg(mut ctx: lang::Context, _: &dyn PartialReflect) -> lang::ScriptResult {
        dbg!(ctx.peek()?);
        Ok(false)
    }

    fn op_load(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult {
        let name = name.try_downcast_ref::<String>().unwrap();
        // let value = unsafe { ctx.local_ref(name).unwrap() };
        // let value = unsafe { ctx.local_ref(name).unwrap() };
        ctx.push(value);
        Ok(false)
    }

    fn op_load_usize(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult {
        let name = name.try_downcast_ref::<String>().unwrap();
        let value = unsafe { ctx.local_mut::<usize>(name).unwrap() };
        ctx.push(value);
        Ok(false)
    }

    fn op_store_usize(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult {
        let src = ctx.pop()?;
        let src = src.try_downcast_ref::<usize>().unwrap();
        let name = name.try_downcast_ref::<String>().unwrap();
        unsafe { *ctx.local_mut::<usize>(name).unwrap() = *src };
        Ok(false)
    }

    fn op_increment_usize(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult {
        let name = name.try_downcast_ref::<String>().unwrap();
        unsafe { *ctx.local_mut::<usize>(name).unwrap() += 1 }
        Ok(false)
    }

    fn query_next<D>(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult
    where
        D: bevy::ecs::query::QueryData + 'static,
    {
        let (iter, state) = name.try_downcast_ref::<(String, String)>().unwrap();
        let iter: &mut QueryIter<'static, 'static, D, ()> = unsafe { ctx.local_mut(iter) }.unwrap();

        if let Some(next) = iter.next() {
            ctx.local().set_or_insert(state.as_str(), next);
            ctx.push_alloc(true);
        } else {
            ctx.push_alloc(false);
        }

        Ok(false)
    }

    fn own_drop(mut ctx: lang::Context, name: &dyn PartialReflect) -> lang::ScriptResult {
        let name = name.try_downcast_ref::<String>().unwrap();
        ctx.local().drop(name);
        Ok(false)
    }

    fn read_stat(mut ctx: lang::Context, args: &dyn PartialReflect) -> lang::ScriptResult {
        let local = ctx.local();
        let args = args.try_downcast_ref::<(String, String, String, AssetIndex)>();
        let &(ref defs, ref entity, ref variable, id) = args.unwrap();
        let defs = local.get_ref(defs).unwrap();
        let entity = local.get_ref(entity).unwrap();
        unsafe {
            let defs = defs.deref::<def::DefParam<StatAsset, StatComponent>>();
            let entity = entity.deref::<FilteredEntityRef>();
            let (component, asset) = defs.filtered_ref(entity, id).unwrap();
            let value = (component as *const _, asset as *const _);
            local.set_or_insert(variable, value);
        }

        Ok(false)
    }

    let query = QueryParamBuilder::new(move |builder| {
        builder.ref_id(component_id);
    });

    let mut registration = lang::Registration::new();

    // registration.register("stat_value", access_stat_ref_value);
    // registration.register("stat_asset", access_stat_ref_asset);
    registration.register("dbg", op_dbg);
    registration.register("print", lang::debug::op_debug);

    registration.register("branch", op_jump);
    registration.register("branch_if_true", branch_if_true);
    registration.register("branch_if_false", branch_if_false);

    registration.register("load", op_load);

    registration.register("load_usize", op_load_usize);
    registration.register("store_usize", op_store_usize);
    registration.register("increment_usize", op_increment_usize);

    registration.register("query_next", query_next::<FilteredEntityRef<'static>>);
    registration.register("drop", own_drop);

    registration.register("read_stat", read_stat);

    let script = {
        impl lang::ScriptBuilder<'_> {
            fn emit_push(&mut self, data: impl PartialReflect) -> usize {
                self.add("push", data)
            }

            fn emit_eq(&mut self) -> usize {
                self.add("eq", ())
            }

            fn emit_access(&mut self, access: lang::ScriptAccess) -> usize {
                self.add("access", access)
            }

            fn emit_add(&mut self) -> usize {
                self.add("add", ())
            }

            fn emit_drop(&mut self, name: impl Into<String>) -> usize {
                self.add("drop", name.into())
            }

            fn emit_print(&mut self, format: impl Into<String>) -> usize {
                self.add("print", format.into())
            }

            fn emit_nop(&mut self) -> usize {
                self.add("nop", ())
            }

            fn emit_dbg(&mut self) -> usize {
                self.add("dbg", ())
            }

            fn emit_query_next(
                &mut self,
                iter: impl Into<String>,
                item: impl Into<String>,
            ) -> usize {
                self.add("query_next", (iter.into(), item.into()))
            }

            fn emit_branch(&mut self, addr: usize) -> usize {
                self.add("branch", addr)
            }

            fn emit_branch_if_false(&mut self, addr: usize) -> usize {
                self.add("branch_if_false", addr)
            }

            fn emit_branch_if_true(&mut self, addr: usize) -> usize {
                self.add("branch_if_true", addr)
            }

            fn emit_read_stat(
                &mut self,
                defs: impl Into<String>,
                entity: impl Into<String>,
                variable: impl Into<String>,
                index: AssetIndex,
            ) {
                self.add(
                    "read_stat",
                    (defs.into(), entity.into(), variable.into(), index),
                );
            }
        }

        impl lang::ScriptBuilder<'_> {
            fn set_branch(&mut self, label: usize, addr: usize) {
                self.set(label, "branch", addr)
            }

            fn set_branch_if_false(&mut self, label: usize, addr: usize) {
                self.set(label, "branch_if_false", addr);
            }

            fn set_branch_if_true(&mut self, label: usize, addr: usize) {
                self.set(label, "branch_if_true", addr);
            }
        }

        let mut script = lang::ScriptBuilder::new(&registration);

        if false {
            let loop_start = script.add("load_usize", String::from("counter"));
            script.emit_push(3usize);
            script.emit_eq();
            let exit_branch = script.emit_nop();

            // body
            script.emit_print("---");

            // update
            script.add("increment_usize", String::from("counter"));
            let loop_end = script.emit_branch(loop_start);

            script.set(exit_branch, "branch_if_true", loop_end + 1);
        }

        {
            let loop_start = script.emit_query_next("iter", "entity");
            let exit_branch = script.emit_nop();

            {
                #[derive(Reflect)]
                struct Something {
                    value: i32,
                }

                script.emit_read_stat("defs", "entity", "stat", index);
                script.emit_print("{stat.name}: {stat.current} [{stat.minimal} .. {stat.maximal}]");

                script.emit_push(Something { value: 5 });
                script.emit_access(lang::ScriptAccess::field("value"));
                script.emit_push(7i32);
                script.emit_add();
                script.emit_print("last: 7 + 5 = {[0]}");
            }

            script.emit_drop("entity");
            let loop_end = script.emit_branch(loop_start);
            script.set_branch_if_false(exit_branch, loop_end + 1);
        }

        script.build()
    };

    let system = (ParamBuilder, query).build_state(world);
    system.build_any_system(move |defs: StatParam, query: Query<FilteredEntityRef>| {
        let mut iter = query.into_iter();

        let mut local = lang::Local::new();
        local.add_ref("defs", &defs);
        local.add_mut("iter", &mut iter);

        let mut machine = lang::Machine::new(&registration, &script);
        machine.run(&mut local).unwrap();
    })
}

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct PrintSchedule;

fn spawn(world: &mut World) {
    world.resource_scope(|world, index: Mut<DefIndex<StatAsset>>| {
        for &component_id in index.asset_to_id.values() {
            let value = 5.0_f32;
            let mut entity = world.spawn_empty();
            insert_stat(&mut entity, component_id, value);
            assert!(entity.get_by_id(component_id).is_ok());
        }
    });
}

fn update(mut commands: Commands) {
    info!("update");

    commands.run_schedule(PrintSchedule);
}

fn insert_stat(entity: &mut EntityWorldMut, component_id: ComponentId, value: f32) {
    OwningPtr::make(value, |component| unsafe {
        entity.insert_by_id(component_id, component);
    });
}

#[derive(Reflect, Debug)]
pub struct StatComponent {
    current: f32,
}

#[derive(Asset, Reflect, Debug)]
pub struct StatAsset {
    name: String,
    default: f32,
    minimal: f32,
    maximal: f32,
}

pub type StatParam<'w> = self::def::DefParam<'w, StatAsset, StatComponent>;
