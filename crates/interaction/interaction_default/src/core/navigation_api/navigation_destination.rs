// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/navigation_api/navigation_destination.h:25-66.
use super::{
    navigate_event_dispatch_params::{NavigateEventDispatchParams, NavigateEventType},
    navigation_api::{NavigationHistoryEntry, SerializedScriptValue},
};
use crate::bindings::core::v8::script_promise::{ScriptInvocation, Value};
use std::{cell::RefCell, rc::Rc};
pub struct NavigationDestination {
    pub params: Rc<NavigateEventDispatchParams>,
    state: RefCell<Option<Rc<dyn SerializedScriptValue>>>,
    entry: Option<Rc<dyn NavigationHistoryEntry>>,
}
impl NavigationDestination {
    pub fn new(
        params: Rc<NavigateEventDispatchParams>,
        state: Option<Rc<dyn SerializedScriptValue>>,
        entry: Option<Rc<dyn NavigationHistoryEntry>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            params,
            state: RefCell::new(state),
            entry,
        })
    }
    pub fn Key(&self) -> String {
        self.entry.as_ref().map_or_else(String::new, |e| e.Key())
    }
    pub fn Id(&self) -> String {
        self.entry.as_ref().map_or_else(String::new, |e| e.Id())
    }
    pub fn URL(&self) -> String {
        self.params.URL()
    }
    pub fn Index(&self) -> i32 {
        self.entry.as_ref().map_or(-1, |e| e.Index())
    }
    pub fn SameDocument(&self) -> bool {
        self.params.event_type != NavigateEventType::CrossDocument
    }
    pub fn GetState(&self, invocation: &dyn ScriptInvocation) -> Result<Option<Value>, Value> {
        let state = self.state.borrow().clone();
        state.map(|s| invocation.Deserialize(&s)).transpose()
    }
    pub fn SetSerializedState(&self, state: Option<Rc<dyn SerializedScriptValue>>) {
        *self.state.borrow_mut() = state;
    }
}

impl crate::bindings::core::v8::script_promise::ScriptNativeObject for NavigationDestination {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "NavigationDestination"
    }
    fn VisitNativeObjects(
        &self,
        visit: &mut dyn FnMut(
            Rc<dyn crate::bindings::core::v8::script_promise::ScriptNativeObject>,
        ),
    ) {
        if let Some(entry) = &self.entry {
            visit(entry.clone());
        }
    }
}
