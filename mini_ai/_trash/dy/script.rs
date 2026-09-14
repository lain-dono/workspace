mod arena;
mod handle;
mod ir;
mod non_max_u32;
mod range;
mod statement;
mod unique_arena;

pub mod runtime;

pub use self::arena::Arena;
pub use self::handle::{BadHandle, Handle, Index};
pub use self::ir::{Expression, Function, Module};
pub use self::range::{BadRangeError, Range};
pub use self::unique_arena::UniqueArena;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Span {}

/// Insertion-order-preserving hash set (`IndexSet<K>`), but with the same
/// hasher as `FastHashSet<K>` (faster but not resilient to DoS attacks).
pub type FastIndexSet<K> =
    indexmap::IndexSet<K, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

/// Insertion-order-preserving hash map (`IndexMap<K, V>`), but with the same
/// hasher as `FastHashMap<K, V>` (faster but not resilient to DoS attacks).
pub type FastIndexMap<K, V> =
    indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

mod dummy {
    const ENTITY_VAR_NAME: &str = "entity";

    use super::runtime::{
        FuncArgs, Runtime, ScriptingError,
        assets::GetExtensions,
        callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
        promise::Promise,
    };
    use bevy::{
        asset::Asset,
        ecs::{component::Component, entity::Entity, resource::Resource, schedule::ScheduleLabel},
        reflect::TypePath,
    };
    use serde::Deserialize;
    use std::fmt::Debug;

    #[derive(Default)]
    pub struct DummyEngine;

    #[derive(Asset, Debug, Deserialize, TypePath)]
    pub struct DummyScript(pub String);

    impl GetExtensions for DummyScript {
        fn extensions() -> &'static [&'static str] {
            &["rhai"]
        }
    }

    impl From<String> for DummyScript {
        fn from(value: String) -> Self {
            Self(value)
        }
    }

    #[derive(Clone)]
    pub struct DummyCallContext;

    #[derive(Resource)]
    pub struct DummyRuntime {
        engine: DummyEngine,
    }

    #[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
    pub struct DummySchedule;

    /// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
    /// and [rhai::AST] which is a cached AST representation of the script.
    #[derive(Component)]
    pub struct DummyScriptData {
        // pub scope: rhai::Scope<'static>,
        // pub(crate) ast: rhai::AST,
    }

    #[derive(Debug, Clone)]
    pub struct DummyValue;

    impl From<usize> for DummyValue {
        fn from(value: usize) -> Self {
            Self
        }
    }

    impl Runtime for DummyRuntime {
        type Schedule = DummySchedule;
        type ScriptAsset = DummyScript;
        type ScriptData = DummyScriptData;
        #[allow(deprecated)]
        type CallContext = DummyCallContext;
        type Value = DummyValue;
        type RawEngine = DummyEngine;

        fn eval(
            &self,
            script: &Self::ScriptAsset,
            entity: Entity,
        ) -> Result<Self::ScriptData, ScriptingError> {
            /*
            let mut scope = Scope::new();
            scope.push(ENTITY_VAR_NAME, entity);

            let engine = &self.engine;

            let ast = engine
                .compile_with_scope(&scope, script.0.as_str())
                .map_err(|e| ScriptingError::CompileError(Box::new(e)))?;

            engine
                .run_ast_with_scope(&mut scope, &ast)
                .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;

            scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();

            Ok(Self::ScriptData { ast, scope })
            */
            Ok(Self::ScriptData {})
        }

        fn register_fn(
            &mut self,
            name: String,
            arg_types: Vec<std::any::TypeId>,
            f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<Promise<Self::CallContext, Self::Value>, ScriptingError>
            + Send
            + Sync
            + 'static,
        ) -> Result<(), ScriptingError> {
            // self.engine
            //     .register_raw_fn(name, arg_types, move |context, args| {
            //         let args = args.iter_mut().map(|arg| DummyValue(arg.clone())).collect();
            //         Ok(f(context.store_data(), args).unwrap())
            //     });

            Ok(todo!())
        }

        fn call_fn(
            &self,
            name: &str,
            script_data: &mut Self::ScriptData,
            entity: Entity,
            args: impl for<'a> FuncArgs<'a, Self::Value, Self>,
        ) -> Result<Self::Value, ScriptingError> {
            /*
            let ast = script_data.ast.clone();
            let scope = &mut script_data.scope;
            scope.push(ENTITY_VAR_NAME, entity);
            let options = CallFnOptions::new().eval_ast(false);
            let args = args
                .parse(&self.engine)
                .into_iter()
                .map(|a| a.0)
                .collect::<Vec<Dynamic>>();
            let result = self
                .engine
                .call_fn_with_options::<Dynamic>(options, scope, &ast, name, args);
            scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
            match result {
                Ok(val) => Ok(Self::Value(val)),
                Err(e) => Err(ScriptingError::RuntimeError(Box::new(e))),
            }
            */

            Ok(DummyValue)
        }

        fn call_fn_from_value(
            &self,
            value: &Self::Value,
            context: &Self::CallContext,
            args: Vec<Self::Value>,
        ) -> Result<Self::Value, ScriptingError> {
            /*
            let f = value.0.clone_cast::<FnPtr>();

            #[allow(deprecated)]
            let ctx = &context.create_context(&self.engine);

            let result = if args.len() == 1 && args.first().unwrap().0.is_unit() {
                f.call_raw(ctx, None, [])
                    .map_err(|e| ScriptingError::RuntimeError(e))?
            } else {
                let args = args.into_iter().map(|a| a.0).collect::<Vec<Dynamic>>();
                f.call_raw(ctx, None, args)
                    .map_err(|e| ScriptingError::RuntimeError(e))?
            };

            Ok(Self::Value(result))
            */

            Ok(DummyValue)
        }

        fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
            f(&mut self.engine)
        }

        fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
            f(&self.engine)
        }
    }

    impl Default for DummyRuntime {
        fn default() -> Self {
            let mut engine = DummyEngine::default();

            /*
            engine
                .register_type_with_name::<Entity>("Entity")
                .register_get("index", |entity: &mut Entity| entity.index());
            #[allow(deprecated)]
            engine
                .register_type_with_name::<Promise<rhai::NativeCallContextStore, RhaiValue>>(
                    "Promise",
                )
                .register_fn(
                    "then",
                    |promise: &mut Promise<rhai::NativeCallContextStore, RhaiValue>,
                     callback: rhai::Dynamic| {
                        Promise::then(promise, RhaiValue(callback));
                    },
                );

            engine
                .register_type_with_name::<Vec3>("Vec3")
                .register_fn("new_vec3", |x: f64, y: f64, z: f64| {
                    Vec3::new(x as f32, y as f32, z as f32)
                })
                .register_get("x", |vec: &mut Vec3| vec.x as f64)
                .register_get("y", |vec: &mut Vec3| vec.y as f64)
                .register_get("z", |vec: &mut Vec3| vec.z as f64);
            #[allow(deprecated)]
            engine.on_def_var(|_, info, _| Ok(info.name != "entity"));
            */

            Self { engine }
        }
    }

    impl<'a, T: Clone> IntoRuntimeValueWithEngine<'a, T, DummyRuntime> for T {
        fn into_runtime_value(value: T, _engine: &'a DummyEngine) -> DummyValue {
            // DummyValue(Dynamic::from(value))
            DummyValue
        }
    }

    impl FuncArgs<'_, DummyValue, DummyRuntime> for () {
        fn parse(self, _engnie: &DummyEngine) -> Vec<DummyValue> {
            Vec::new()
        }
    }
    impl<T: Clone + Send + Sync + 'static> FuncArgs<'_, DummyValue, DummyRuntime> for Vec<T> {
        fn parse(self, _engine: &DummyEngine) -> Vec<DummyValue> {
            // self.into_iter()
            //     // .map(|v| DummyValue(Dynamic::from(v)))
            //     .map(|v| DummyValue)
            //     .collect()
            todo!()
        }
    }

    impl<T: Clone + 'static> FromRuntimeValueWithEngine<'_, DummyRuntime> for T {
        fn from_runtime_value(value: DummyValue, _engine: &DummyEngine) -> Self {
            //value.0.clone_cast()
            todo!()
        }
    }

    pub mod prelude {
        pub use super::{DummyRuntime, DummyScript, DummyScriptData};
    }

    macro_rules! impl_tuple {
        ($($idx:tt $t:tt),+) => {
            impl<$($t: Clone,)+> FuncArgs<'_, DummyValue, DummyRuntime>
                for ($($t,)+)
            {
                fn parse(self, _engine: &DummyEngine) -> Vec<DummyValue> {
                    vec![ $(
                        // DummyValue(Dynamic::from(self.$idx)),
                        DummyValue::from($idx),
                    )+ ]
                }
            }
        };
    }

    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
    impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);
    impl_tuple!(0 A, 1 B, 2 C, 3 D);
    impl_tuple!(0 A, 1 B, 2 C);
    impl_tuple!(0 A, 1 B);
    impl_tuple!(0 A);
}
