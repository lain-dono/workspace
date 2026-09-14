use super::{
    Callback, Callbacks, Runtime, Script, ScriptingError,
    callback::FunctionCallEvent,
    promise::{Promise, PromiseInner},
};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Reloads scripts when they are modified.
pub(crate) fn reload_scripts<R: Runtime>(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<R::ScriptAsset>>,
    mut scripts: Query<(Entity, &mut Script<R::ScriptAsset>)>,
) {
    for ev in ev_asset.read() {
        if let &AssetEvent::Modified { id } = ev {
            for (entity, script) in &mut scripts {
                if script.script.id() == id {
                    commands.entity(entity).remove::<R::ScriptData>();
                }
            }
        }
    }
}

/// Processes new scripts. Evaluates them and stores the script data in the entity.
#[allow(clippy::type_complexity)]
pub(crate) fn process_new_scripts<R: Runtime>(
    mut commands: Commands,
    mut added_scripted_entities: Query<
        (Entity, &mut Script<R::ScriptAsset>),
        Without<R::ScriptData>,
    >,
    scripting_runtime: ResMut<R>,
    scripts: Res<Assets<R::ScriptAsset>>,
    asset_server: Res<AssetServer>,
) -> Result<(), ScriptingError> {
    for (entity, script_component) in &mut added_scripted_entities {
        trace!("evaulating a new script");

        if let Some(script) = scripts.get(&script_component.script) {
            match scripting_runtime.eval(script, entity) {
                Ok(script_data) => {
                    commands.entity(entity).insert(script_data);
                }
                Err(err) => {
                    let path = asset_server
                        .get_path(&script_component.script)
                        .unwrap_or_default();
                    error!("error running script {path} {err:?}");
                }
            }
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
pub(crate) fn init_callbacks<R: Runtime>(world: &mut World) -> Result<(), ScriptingError> {
    let mut callbacks = world
        .get_resource_mut::<Callbacks<R>>()
        .ok_or(ScriptingError::NoSettingsResource)?
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback<R>>>();

    for callback in &mut callbacks {
        if let Ok(mut system) = callback.system.lock() {
            system.system.initialize(world);

            let mut scripting_runtime = world
                .get_resource_mut::<R>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            let name = callback.name.clone();
            let calls = callback.calls.clone();

            trace!("init_callbacks: registering callback: '{name}'");

            let result = scripting_runtime.register_fn(
                name,
                system.arg_types.clone(),
                move |context, params| {
                    let new_promise = Promise {
                        inner: Arc::new(Mutex::new(PromiseInner {
                            callbacks: vec![],
                            context,
                        })),
                    };

                    let promise = new_promise.clone();
                    calls
                        .lock()
                        .expect("Failed to lock callback calls mutex")
                        .push(FunctionCallEvent { promise, params });

                    Ok(new_promise)
                },
            );

            if let Err(err) = result {
                error!("error registering function: {err:?}");
            }
        }
    }

    world
        .get_resource_mut::<Callbacks<R>>()
        .ok_or(ScriptingError::NoSettingsResource)?
        .callbacks
        .lock()
        .expect("Failed to lock callbacks mutex")
        .append(&mut callbacks.clone());

    Ok(())
}

/// Processes calls. Calls the user-defined callback systems
pub(crate) fn process_calls<R: Runtime>(world: &mut World) -> Result<(), ScriptingError> {
    let callbacks = world
        .get_resource::<Callbacks<R>>()
        .ok_or(ScriptingError::NoSettingsResource)?
        .callbacks
        .lock()
        .expect("Failed to lock callbacks mutex")
        .clone();

    for Callback {
        name,
        system,
        calls,
    } in callbacks
    {
        let calls = calls
            .lock()
            .expect("Failed to lock callback calls mutex")
            .drain(..)
            .collect::<Vec<FunctionCallEvent<R::CallContext, R::Value>>>();

        for mut call in calls {
            trace!("process_calls: calling '{name}'");

            let mut system = system.lock().expect("Failed to lock callback system mutex");

            let value = system.call(&call, world);
            let mut runtime = world
                .get_resource_mut::<R>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            if let Err(err) = call.promise.resolve(runtime.as_mut(), value) {
                error!("error resolving call: {name} {err:?}");
            }
        }
    }

    Ok(())
}

/// Error logging system
pub fn log_errors<E: std::fmt::Display>(In(res): In<Result<(), E>>) {
    if let Err(error) = res {
        error!("{error}");
    }
}
