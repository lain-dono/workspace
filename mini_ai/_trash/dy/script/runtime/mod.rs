pub mod assets;
pub mod callback;
pub mod promise;
pub mod systems;

use self::{
    assets::{GetExtensions, ScriptLoader},
    callback::{Callback, IntoCallbackSystem},
    promise::Promise,
    systems::{init_callbacks, log_errors, process_calls, process_new_scripts, reload_scripts},
};
use bevy::prelude::*;
use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel};
use std::{
    any::TypeId,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use thiserror::Error;

pub mod prelude {
    pub use super::{BuildScriptingRuntime as _, Runtime as _, Script};
}

/// A component that represents a script.
#[derive(Component)]
pub struct Script<A: Asset> {
    pub script: Handle<A>,
}

impl<A: Asset> Script<A> {
    /// Create a new script component from a handle to a [Script] obtained using [AssetServer].
    pub fn new(script: Handle<A>) -> Self {
        Self { script }
    }
}

/// An error that can occur when internal [ScriptingPlugin] systems are being executed
#[derive(Error, Debug)]
pub enum ScriptingError {
    #[error("script runtime error: {0}")]
    RuntimeError(Box<dyn std::error::Error>),
    #[error("script compilation error: {0}")]
    CompileError(Box<dyn std::error::Error>),
    #[error("no runtime resource present")]
    NoRuntimeResource,
    #[error("no settings resource present")]
    NoSettingsResource,
}

/// Trait that represents a scripting runtime/engine. In practice it is
/// implemented for a scripint language interpreter and the implementor provides
/// function implementations for calling and registering functions within the interpreter.
pub trait Runtime: Resource + Default {
    type Schedule: ScheduleLabel + Debug + Clone + Eq + Hash + Default;
    type ScriptAsset: Asset + From<String> + GetExtensions;
    type ScriptData: Component;
    type CallContext: Send + Clone;
    type Value: Send + Clone;
    type RawEngine;

    /// Provides mutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T;

    /// Provides immutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T;

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: Entity,
    ) -> Result<Self::ScriptData, ScriptingError>;

    /// Registers a new function within the scripting engine. Provided callback
    /// function will be called when the function with provided name gets called
    /// in script.
    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<TypeId>,
        f: impl Fn(
            Self::CallContext,
            Vec<Self::Value>,
        ) -> Result<Promise<Self::CallContext, Self::Value>, ScriptingError>
        + Send
        + Sync
        + 'static,
    ) -> Result<(), ScriptingError>;

    /// Calls a function by name defined within the runtime in the context of the
    /// entity that has been paassed. Can return a dynamically typed value
    /// that got returned from the function within a script.
    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: Entity,
        args: impl for<'a> FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, ScriptingError>;

    /// Calls a function by value defined within the runtime in the context of the
    /// entity that has been paassed. Can return a dynamically typed value
    /// that got returned from the function within a script.
    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, ScriptingError>;
}

pub trait FuncArgs<'a, V, R: Runtime> {
    fn parse(self, engine: &'a R::RawEngine) -> Vec<V>;
}

/// An extension trait for [App] that allows to setup a scripting runtime `R`.
pub trait BuildScriptingRuntime {
    /// Returns a "runtime" type than can be used to setup scripting runtime(
    /// add scripting functions etc.).
    fn add_scripting<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self;

    /// Returns a "runtime" type that can be used to add additional scripting functions from plugins etc.
    fn add_scripting_api<R: Runtime>(
        &mut self,
        f: impl Fn(ScriptingRuntimeBuilder<R>),
    ) -> &mut Self;

    fn add_function<R: Runtime, In: SystemInput, Out, Marker>(
        &mut self,
        name: impl Into<String>,
        fun: impl IntoCallbackSystem<R, In, Out, Marker>,
    ) -> &mut Self;
}

pub struct ScriptingRuntimeBuilder<'a, R: Runtime> {
    world: &'a mut World,
    marker: PhantomData<R>,
}

impl<'a, R: Runtime> ScriptingRuntimeBuilder<'a, R> {
    fn new(world: &'a mut World) -> Self {
        Self {
            world,
            marker: PhantomData,
        }
    }

    /// Registers a function for calling from within a script.
    /// Provided function needs to be a valid bevy system and its
    /// arguments and return value need to be convertible to runtime
    /// value types.
    pub fn add_function<In: SystemInput, Out, Marker>(
        self,
        name: impl Into<String>,
        fun: impl IntoCallbackSystem<R, In, Out, Marker>,
    ) -> Self {
        let system = fun.into_callback_system(self.world);

        let mut callbacks = self.world.resource_mut::<Callbacks<R>>();
        callbacks.uninitialized_callbacks.push(Callback {
            name: name.into(),
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });

        self
    }
}

impl BuildScriptingRuntime for App {
    /// Adds a scripting runtime. Registers required bevy systems that take
    /// care of processing and running the scripts.
    fn add_scripting<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self {
        self.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Update, R::Schedule::default());

        self.register_asset_loader(ScriptLoader::<R::ScriptAsset>::default())
            .init_schedule(R::Schedule::default())
            .init_asset::<R::ScriptAsset>()
            .init_resource::<Callbacks<R>>()
            .insert_resource(R::default())
            .add_systems(
                R::Schedule::default(),
                (
                    reload_scripts::<R>,
                    (
                        init_callbacks::<R>.pipe(log_errors),
                        process_new_scripts::<R>.pipe(log_errors),
                        process_calls::<R>.pipe(log_errors),
                    )
                        .chain(),
                ),
            );

        f(ScriptingRuntimeBuilder::<R>::new(self.world_mut()));
        self
    }

    /// Adds a way to add additional accesspoints to the scripting runtime. For example from plugins to add
    /// for example additional lua functions to the runtime.
    ///
    /// Be careful with calling this though, make sure that the `add_scripting` call is already called before calling this function.
    fn add_scripting_api<R: Runtime>(
        &mut self,
        f: impl Fn(ScriptingRuntimeBuilder<R>),
    ) -> &mut Self {
        f(ScriptingRuntimeBuilder::<R>::new(self.world_mut()));
        self
    }

    fn add_function<R: Runtime, In: SystemInput, Out, Marker>(
        &mut self,
        name: impl Into<String>,
        fun: impl IntoCallbackSystem<R, In, Out, Marker>,
    ) -> &mut Self {
        ScriptingRuntimeBuilder::<R>::new(self.world_mut()).add_function(name, fun);
        self
    }
}

/// A resource that stores all the callbacks that were registered using [AddScriptFunctionAppExt::add_function].
#[derive(Resource)]
struct Callbacks<R: Runtime> {
    uninitialized_callbacks: Vec<Callback<R>>,
    callbacks: Mutex<Vec<Callback<R>>>,
}

impl<R: Runtime> Default for Callbacks<R> {
    fn default() -> Self {
        Self {
            uninitialized_callbacks: Default::default(),
            callbacks: Default::default(),
        }
    }
}
