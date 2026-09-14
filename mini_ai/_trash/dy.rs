use bevy::ecs::system::SystemId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::reflect::func::{ArgList, DynamicFunctionMut};

pub fn run() {
    run_mut();

    // let mut app = App::new();

    // app.init_resource::<Callbacks>()
    //     .add_callback("hello", || println!("hello"));

    fn increment(In(increment_by): In<u8>, mut counter: Local<u8>) -> u8 {
        *counter += increment_by;
        *counter
    }

    let mut world = World::default();
    let counter_one = world.register_system(increment);
    assert_eq!(world.run_system_with(counter_one, 1).unwrap(), 1);

    {
        world.init_resource::<Callbacks>();
        world.init_resource::<Runtime>();

        add_callback(&mut world, "hello", |mut counter: Local<u8>| -> u8 {
            let increment_by = 1;
            *counter += increment_by;
            println!("hello {}", *counter);
            *counter
        });

        world.resource_scope(|world, mut callbacks: Mut<'_, Callbacks>| {
            for callback in callbacks.all.values_mut() {
                callback.system.initialize(world);
            }
        });

        run_callback(&mut world, "hello", vec![]);
        run_callback(&mut world, "hello", vec![]);
        run_callback(&mut world, "hello", vec![]);
    }
}

impl<'a, T: Clone> IntoRuntimeValue<'a, T> for T {
    fn into_runtime_value(value: T, _engine: &'a RawEngine) -> RuntimeValue {
        // DummyValue(Dynamic::from(value))
        RuntimeValue
    }
}

#[derive(Resource, Default)]
pub struct Callbacks {
    all: HashMap<String, CallbackSystem>,
}

fn run_callback(
    world: &mut World,
    name: impl AsRef<str>,
    input: Vec<RuntimeValue>,
) -> RuntimeValue {
    world.resource_scope(|world, mut callbacks: Mut<'_, Callbacks>| {
        let sys = callbacks.all.get_mut(name.as_ref()).unwrap();
        sys.system.run(input, world)
    })
}

pub fn add_callback<In: SystemInput, Out, Marker>(
    world: &mut World,
    name: impl Into<String>,
    fun: impl IntoCallbackSystem<In, Out, Marker>,
) {
    let system = fun.into_callback_system(world);
    let mut callbacks = world.resource_mut::<Callbacks>();
    callbacks.all.insert(name.into(), system);
}

// #[derive(Component)]
// struct Triggered;

// fn run_scripts(world: &mut World) {
//     //
// }

#[derive(Component)]
struct Script {
    instructions: Vec<Instruction>,
}

enum Instruction {
    CallCallback { name: String, args: Vec<()> },
}

/*
trait AddCallback {
    fn add_callback(&mut self, name: impl Into<String>, fun: impl Fn());
}

impl AddCallback for App {
    fn add_callback(&mut self, name: impl Into<String>, fun: impl Fn()) {
        let world = self.world_mut();
        let name = name.into();
        // let system = fun.into_callback_system(world);
        let system = ();

        let mut callbacks = world.resource_mut::<Callbacks>();
        callbacks.push(Callback::new(name, system));
    }
}

#[derive(Resource, Default)]
pub struct Callbacks(pub Vec<Callback>);

impl std::ops::Deref for Callbacks {
    type Target = Vec<Callback>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Callbacks {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Callback {
    name: String,
    system: (),
}

impl Callback {
    fn new(name: impl Into<String>, system: ()) -> Self {
        let name = name.into();
        Self { name, system }
    }
}
*/

fn run_mut() {
    let mut list: Vec<i32> = vec![1, 2, 3];

    // `replace` is a closure that captures a mutable reference to `list`
    let mut replace = |index: usize, value: i32| -> i32 {
        let old_value = list[index];
        list[index] = value;
        old_value
    };

    // Since this closure mutably borrows data, we can't convert it into a regular `DynamicFunction`,
    // as doing so would result in a compile-time error:
    // let mut func: DynamicFunction = replace.into_function();
    // Instead, we convert it into a `DynamicFunctionMut` using `IntoFunctionMut::into_function_mut`:
    let mut func: DynamicFunctionMut = replace.into_function_mut();

    // Dynamically call it:
    let args = ArgList::default().with_owned(1_usize).with_owned(-2_i32);
    let value = func.call(args).unwrap().unwrap_owned();

    // Check the result:
    assert_eq!(value.try_take::<i32>().unwrap(), 2);

    // Note that `func` still has a reference to `list`,
    // so we need to drop it before we can access `list` again.
    // Alternatively, we could have invoked `func` with
    // `DynamicFunctionMut::call_once` to immediately consume it.
    drop(func);

    assert_eq!(list, vec![1, -2, 3]);
}

#[derive(Resource, Default)]
pub struct Runtime {
    engine: RawEngine,
}

impl Runtime {
    /// Provides immutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine<T>(&self, f: impl FnOnce(&RawEngine) -> T) -> T {
        f(&self.engine)
    }

    /// Provides mutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut RawEngine) -> T) -> T {
        f(&mut self.engine)
    }
}

#[derive(Default)]
pub struct RawEngine;

pub struct RuntimeValue;

/// A system that can be used to call a script function.
pub struct CallbackSystem {
    pub system: Box<dyn System<In = In<Vec<RuntimeValue>>, Out = RuntimeValue>>,
    pub arg_types: Vec<std::any::TypeId>,
}

/// Allows converting to a wrapper type that the library uses internally for data
pub trait IntoRuntimeValue<'a, V> {
    fn into_runtime_value(value: V, engine: &'a RawEngine) -> RuntimeValue;
}

/// Allows converting from a wrapper type that the library uses internally for data to underlying concrete type.
pub trait FromRuntimeValue<'a> {
    fn from_runtime_value(value: RuntimeValue, engine: &'a RawEngine) -> Self;
}

/// Trait that alllows to convert a script callback function into a Bevy [`System`].
pub trait IntoCallbackSystem<In: SystemInput, Out, Marker>: IntoSystem<In, Out, Marker> {
    /// Convert this function into a [CallbackSystem].
    #[must_use]
    fn into_callback_system(self, world: &mut World) -> CallbackSystem;
}

impl<Out, FN, Marker> IntoCallbackSystem<(), Out, Marker> for FN
where
    FN: IntoSystem<(), Out, Marker>,
    Out: for<'a> IntoRuntimeValue<'a, Out>,
{
    fn into_callback_system(self, world: &mut World) -> CallbackSystem {
        let mut inner = IntoSystem::into_system(self);

        inner.initialize(world);

        let system_fn = move |_args: In<Vec<RuntimeValue>>, world: &mut World| {
            let result = inner.run((), world);
            let mut runtime = world.get_resource_mut::<Runtime>().expect("No runtime");
            runtime.with_engine_mut(move |engine| Out::into_runtime_value(result, engine))
        };

        CallbackSystem {
            arg_types: vec![],
            system: Box::new(IntoSystem::into_system(system_fn)),
        }
    }
}
