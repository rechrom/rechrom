// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/navigation_api/navigation_api_method_tracker.h:25-79;
// .cc:18-109. Promise operations require the current stack invocation. The
// owner never retains ScriptEngine/JavaScriptHostRuntime or a reusable token.
use super::{
    navigate_event_dispatch_params::FrameLoadType,
    navigation_api::{NavigationHistoryEntry, SerializedScriptValue},
};
use crate::bindings::core::v8::{
    script_promise::*, script_promise_resolver::ScriptPromiseResolver,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
#[derive(Clone)]
pub struct NavigationResult {
    pub committed: ScriptPromise,
    pub finished: ScriptPromise,
}
pub struct NavigationApiMethodTrackerOwner {
    self_: Weak<Self>,
    serialized_state: RefCell<Option<Rc<dyn SerializedScriptValue>>>,
    info: Value,
    key: String,
    committed_to_entry: RefCell<Option<Rc<dyn NavigationHistoryEntry>>>,
    committed: Rc<ScriptPromiseResolver>,
    finished: Rc<ScriptPromiseResolver>,
    result: NavigationResult,
}
impl NavigationApiMethodTrackerOwner {
    pub fn new(
        invocation: &dyn ScriptInvocation,
        info: Option<Value>,
        key: String,
        state: Option<Rc<dyn SerializedScriptValue>>,
    ) -> Result<Rc<Self>, Value> {
        let committed = ScriptPromiseResolver::new(invocation)?;
        let finished = ScriptPromiseResolver::new(invocation)?;
        let result = NavigationResult {
            committed: committed.Promise(),
            finished: finished.Promise(),
        };
        invocation.MarkPromiseHandled(&result.finished);
        Ok(Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            serialized_state: RefCell::new(state),
            info: info.unwrap_or_else(|| invocation.Undefined()),
            key,
            committed_to_entry: RefCell::new(None),
            committed,
            finished,
            result,
        }))
    }
    pub fn GetInfo(&self) -> Value {
        self.info.clone()
    }
    pub fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.info.clone());
        self.committed.VisitScriptValues(visit);
        self.finished.VisitScriptValues(visit);
    }
    pub fn GetKey(&self) -> &str {
        &self.key
    }
    pub fn GetSerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.serialized_state.borrow().clone()
    }
    pub fn SetSerializedState(&self, state: Option<Rc<dyn SerializedScriptValue>>) {
        *self.serialized_state.borrow_mut() = state;
    }
    pub fn GetNavigationResult(&self) -> &NavigationResult {
        &self.result
    }
    pub fn NotifyAboutTheCommittedToEntry(
        &self,
        invocation: &dyn ScriptInvocation,
        entry: Rc<dyn NavigationHistoryEntry>,
        load: FrameLoadType,
    ) {
        assert!(self.committed_to_entry.borrow().is_none());
        *self.committed_to_entry.borrow_mut() = Some(entry.clone());
        if load != FrameLoadType::BackForward {
            let state = self.serialized_state.borrow_mut().take();
            entry.SetAndSaveState(state);
        }
        invocation.TraceNativeObject(self.self_.upgrade().expect("live method tracker"));
        self.committed
            .Resolve(invocation, invocation.HistoryEntryValue(Some(entry)));
    }
    pub fn ResolveFinishedPromise(&self, invocation: &dyn ScriptInvocation) {
        let entry = self.committed_to_entry.borrow().clone();
        self.finished
            .Resolve(invocation, invocation.HistoryEntryValue(entry));
    }
    pub fn RejectFinishedPromise(&self, invocation: &dyn ScriptInvocation, error: Value) {
        self.committed.Reject(invocation, error.clone());
        self.finished.Reject(invocation, error);
        self.serialized_state.borrow_mut().take();
    }
    pub fn CleanupForWillNeverSettle(&self) {
        assert!(self.committed_to_entry.borrow().is_none());
        self.committed.Detach();
        self.finished.Detach();
        self.serialized_state.borrow_mut().take();
    }
}

impl super::navigation_api::NavigationApiMethodTracker for NavigationApiMethodTrackerOwner {
    fn NavigationResult(&self) -> NavigationResult {
        self.result.clone()
    }
    fn SerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.GetSerializedState()
    }
    fn SetSerializedState(&self, state: Option<Rc<dyn SerializedScriptValue>>) {
        self.SetSerializedState(state);
    }
    fn Info(&self) -> Value {
        self.GetInfo()
    }
    fn Key(&self) -> String {
        self.GetKey().to_owned()
    }
    fn RejectFinishedPromise(&self, i: &dyn ScriptInvocation, error: Value) {
        self.RejectFinishedPromise(i, error);
    }
    fn ResolveFinishedPromise(&self, i: &dyn ScriptInvocation) {
        self.ResolveFinishedPromise(i);
    }
    fn NotifyAboutTheCommittedToEntry(
        &self,
        i: &dyn ScriptInvocation,
        entry: Rc<dyn NavigationHistoryEntry>,
        load: FrameLoadType,
    ) {
        self.NotifyAboutTheCommittedToEntry(i, entry, load);
    }
}

impl ScriptNativeObject for NavigationApiMethodTrackerOwner {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "InternalNavigationApiMethodTracker"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        self.VisitScriptValues(visit);
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        if let Some(entry) = self.committed_to_entry.borrow().clone() {
            visit(entry);
        }
    }
}
