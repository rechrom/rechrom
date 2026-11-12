//! Native delegates use exactly QuickJS's existing Promise job queue. The
//! handle is weak and enqueuing never borrows/reenters the ScriptEngine realm.
use super::*;
pub(super) struct NativeMicrotasks {
    context: WeakContext,
    state: std::rc::Weak<RefCell<State>>,
}
impl NativeMicrotasks {
    pub(super) fn new(context: &Context, state: &Rc<RefCell<State>>) -> Rc<Self> {
        Rc::new(Self {
            context: context.Downgrade(),
            state: Rc::downgrade(state),
        })
    }
}
struct NativeTask {
    task: RefCell<Option<JavaScriptNativeMicrotask>>,
    state: std::rc::Weak<RefCell<State>>,
}
impl HostCallable for NativeTask {
    fn call(
        &self,
        context: &mut Context,
        _: &Value,
        _: &[Value],
        _: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        let task = self.task.borrow_mut().take();
        if let (Some(task), Some(state)) = (task, self.state.upgrade()) {
            let _roots = SynchronizeHostGraph(context, &state)?;
            task(Some(&mut HostRuntime {
                context,
                state: &state,
            }));
        }
        Ok(Value::Undefined)
    }
}
impl JavaScriptAgentMicrotasks for NativeMicrotasks {
    fn Enqueue(&self, task: JavaScriptNativeMicrotask) -> Result<(), NativeMicrotaskEnqueueError> {
        let mut context = self
            .context
            .Upgrade()
            .ok_or(NativeMicrotaskEnqueueError::RealmDestroyed)?;
        if self.state.strong_count() == 0 {
            return Err(NativeMicrotaskEnqueueError::RealmDestroyed);
        }
        context
            .enqueue_native_job(Rc::new(NativeTask {
                task: RefCell::new(Some(task)),
                state: self.state.clone(),
            }))
            .map_err(|_| NativeMicrotaskEnqueueError::AllocationFailed)
    }
}
