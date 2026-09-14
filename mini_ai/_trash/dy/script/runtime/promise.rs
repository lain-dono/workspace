use super::{Runtime, ScriptingError};
use std::sync::{Arc, Mutex};

/// A struct that represents a function that will get called when the Promise is resolved.
pub(crate) struct PromiseCallback<C: Send, V: Send> {
    callback: V,
    following_promise: Arc<Mutex<PromiseInner<C, V>>>,
}

/// Internal representation of a Promise.
pub(crate) struct PromiseInner<C: Send, V: Send> {
    pub(crate) callbacks: Vec<PromiseCallback<C, V>>,
    pub(crate) context: C,
}

/// A struct that represents a Promise.
#[derive(Clone)]
pub struct Promise<C: Send, V: Send> {
    pub(crate) inner: Arc<Mutex<PromiseInner<C, V>>>,
}

impl<C: Send, V: Send + Clone> PromiseInner<C, V> {
    /// Resolve the Promise. This will call all the callbacks that were added to the Promise.
    fn resolve<R: Runtime<Value = V, CallContext = C>>(
        &mut self,
        runtime: &mut R,
        value: R::Value,
    ) -> Result<(), ScriptingError> {
        for callback in &self.callbacks {
            let value = runtime.call_fn_from_value(
                &callback.callback,
                &self.context,
                vec![value.clone()],
            )?;

            callback
                .following_promise
                .lock()
                .expect("Failed to lock promise mutex")
                .resolve(runtime, value)?;
        }
        Ok(())
    }
}

impl<C: Clone + Send + 'static, V: Send + Clone> Promise<C, V> {
    /// Acquire [Mutex] for writing the promise and resolve it. Call will be forwarded to [PromiseInner::resolve].
    pub(crate) fn resolve<R: Runtime<Value = V, CallContext = C>>(
        &mut self,
        runtime: &mut R,
        value: R::Value,
    ) -> Result<(), ScriptingError> {
        if let Ok(mut inner) = self.inner.lock() {
            inner.resolve(runtime, value)?;
        }
        Ok(())
    }

    /// Register a callback that will be called when the [Promise] is resolved.
    #[cfg(any(feature = "rhai", feature = "lua"))]
    pub(crate) fn then(&mut self, callback: V) -> Self {
        let mut inner = self
            .inner
            .lock()
            .expect("Failed to lock inner promise mutex");

        let following_inner = Arc::new(Mutex::new(PromiseInner {
            callbacks: vec![],
            context: inner.context.clone(),
        }));

        inner.callbacks.push(PromiseCallback {
            following_promise: following_inner.clone(),
            callback,
        });

        Self {
            inner: following_inner,
        }
    }
}
