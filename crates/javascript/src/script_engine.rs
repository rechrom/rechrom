//! Document JavaScript execution lifecycle.
//!
//! `ScriptEngine` owns one runtime and realm. It deliberately knows only the
//! neutral host-binding interface; Web APIs, document mutation, resources and
//! Page scheduling remain outside this crate.

use crate::javascript_runtime::{
    HostValue, JavaScriptException, JavaScriptFunction, JavaScriptHostBindings,
    JavaScriptModuleCompilationWake, JavaScriptRealm, JavaScriptResult, JavaScriptRuntime,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptSource {
    pub source: String,
    pub source_name: String,
}

/// Result of one complete JavaScript task boundary.
///
/// The direct exception stays on `result`; exceptions produced by Promise
/// reactions and other microtasks are returned separately.
pub struct ScriptExecution {
    pub result: JavaScriptResult,
    pub pending_exceptions: Vec<JavaScriptException>,
}

pub struct ScriptEngine {
    runtime: Box<dyn JavaScriptRuntime>,
    realm: JavaScriptRealm,
}

/// Thread-policy-neutral execution port. The owner decides whether this is a
/// direct call or a message bridge; loading code never sees the concrete JS
/// runtime implementation.
pub trait ScriptExecutor {
    fn SupportsModules(&self) -> bool;
    fn SetModuleCompilationWake(&self, wake: Option<JavaScriptModuleCompilationWake>);
    fn WithRuntimeAndRealm(
        &self,
        callback: &mut dyn FnMut(&mut dyn JavaScriptRuntime, &JavaScriptRealm),
    );
}

pub type ScriptExecutorHandle = Rc<dyn ScriptExecutor>;

impl ScriptExecutor for RefCell<ScriptEngine> {
    fn SupportsModules(&self) -> bool {
        self.borrow().SupportsModules()
    }

    fn SetModuleCompilationWake(&self, wake: Option<JavaScriptModuleCompilationWake>) {
        self.borrow_mut().SetModuleCompilationWake(wake);
    }

    fn WithRuntimeAndRealm(
        &self,
        callback: &mut dyn FnMut(&mut dyn JavaScriptRuntime, &JavaScriptRealm),
    ) {
        let mut engine = self.borrow_mut();
        let (runtime, realm) = engine.RuntimeAndRealm();
        callback(runtime, realm);
    }
}

impl ScriptEngine {
    pub fn new<H>(mut runtime: Box<dyn JavaScriptRuntime>, global: Rc<RefCell<H>>) -> Self
    where
        H: JavaScriptHostBindings + 'static,
    {
        let realm = runtime.CreateRealm(global);
        Self { runtime, realm }
    }

    pub fn Realm(&self) -> &JavaScriptRealm {
        &self.realm
    }

    /// Temporary adapter for the existing parser/module scheduler. New Page
    /// code should prefer the complete-task methods on this type.
    pub fn Runtime(&self) -> &dyn JavaScriptRuntime {
        &*self.runtime
    }

    /// Temporary adapter for the existing parser/module scheduler.
    pub fn RuntimeMut(&mut self) -> &mut dyn JavaScriptRuntime {
        &mut *self.runtime
    }

    /// Temporary split adapter for lifecycle code that already accepts the
    /// runtime and realm as separate parameters.
    pub fn RuntimeAndRealm(&mut self) -> (&mut dyn JavaScriptRuntime, &JavaScriptRealm) {
        (&mut *self.runtime, &self.realm)
    }

    pub fn SupportsModules(&self) -> bool {
        self.runtime.SupportsModules()
    }

    pub fn SetModuleCompilationWake(&mut self, wake: Option<JavaScriptModuleCompilationWake>) {
        self.runtime.SetModuleCompilationWake(wake);
    }

    /// Trusted realm initialization. Bootstrap evaluation is not a web task,
    /// so it does not run a microtask checkpoint between source fragments.
    pub fn EvaluateBootstrap(&mut self, sources: &[ScriptSource]) -> Vec<JavaScriptException> {
        sources
            .iter()
            .filter_map(|source| {
                self.runtime
                    .EvaluateBootstrap(&self.realm, &source.source, &source.source_name)
                    .exception
            })
            .collect()
    }

    pub fn Evaluate(&mut self, source: &str, source_name: &str) -> ScriptExecution {
        let result = self.EvaluateRaw(source, source_name);
        let pending_exceptions = self.Checkpoint();
        ScriptExecution {
            result,
            pending_exceptions,
        }
    }

    /// Embedder-internal evaluation inside a larger task. The caller must
    /// finish that task with `Checkpoint`.
    pub fn EvaluateRaw(&mut self, source: &str, source_name: &str) -> JavaScriptResult {
        self.runtime.Evaluate(&self.realm, source, source_name)
    }

    pub fn Call(
        &mut self,
        function: &JavaScriptFunction,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> ScriptExecution {
        let result = self
            .runtime
            .Call(&self.realm, function, receiver, arguments);
        let pending_exceptions = self.Checkpoint();
        ScriptExecution {
            result,
            pending_exceptions,
        }
    }

    /// Complete the microtask checkpoint at the end of a host-dispatched task.
    pub fn Checkpoint(&mut self) -> Vec<JavaScriptException> {
        self.runtime.PerformMicrotaskCheckpoint();
        self.runtime.TakePendingExceptions(&self.realm)
    }
}
