//! Chromium core/frame/history.cc:87-197,246-372. Window-scoped History
//! owner; the frame lends its current loader and routes browser traversals.
use crate::core::{
    html::anchor_element_utils::NavigateDispatchResult,
    navigation_api::navigate_event_dispatch_params::{
        FrameLoadType, NavigateEventDispatchParams, NavigateEventType,
    },
};
use interaction_dispatch::bindings::script_invocation::{
    ScriptInvocation, ScriptNativeObject, Value,
};
use navigation_history::{HistoryItemOwner, ScrollRestorationType, SerializedScriptValue};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait HistoryEnvironment {
    fn IsFullyActive(&self) -> bool;
    fn CurrentHistoryItem(&self) -> Option<Rc<HistoryItemOwner>>;
    fn BackForwardLength(&self) -> usize;
    fn Go(&self, invocation: &dyn ScriptInvocation, delta: i32);
    fn NotifyCurrentHistoryItemChanged(&self);
    fn URL(&self) -> String;
    fn CompleteURL(&self, url: &str) -> Option<String>;
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool;
    fn DispatchNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        params: Rc<NavigateEventDispatchParams>,
    ) -> NavigateDispatchResult;
    fn RunURLAndHistoryUpdateSteps(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
    );
}
struct CachedState {
    serialized: Option<Rc<dyn SerializedScriptValue>>,
    value: Value,
}
pub struct History {
    environment: Rc<dyn HistoryEnvironment>,
    // V8 private History state cache is per wrapper/world. Retain the exact
    // serialization identity, even when a replace reuses its HistoryItem.
    states: RefCell<HashMap<u64, CachedState>>,
}
impl History {
    pub fn new(environment: Rc<dyn HistoryEnvironment>) -> Rc<Self> {
        Rc::new(Self {
            environment,
            states: Default::default(),
        })
    }
    fn CheckActive(&self, invocation: &dyn ScriptInvocation) -> Result<(), Value> {
        if self.environment.IsFullyActive() {
            Ok(())
        } else {
            Err(invocation.DOMException(
                "SecurityError",
                "May not use a History object associated with a Document that is not fully active",
            ))
        }
    }
    fn StateInternal(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        if !self.environment.IsFullyActive() {
            return None;
        }
        self.environment
            .CurrentHistoryItem()
            .and_then(|item| item.StateObject())
    }
    pub fn IsSameAsCurrentState(&self, state: &Option<Rc<dyn SerializedScriptValue>>) -> bool {
        SameState(&self.StateInternal(), state)
    }
    pub fn StateHelper(&self, invocation: &dyn ScriptInvocation) -> Result<Value, Value> {
        let current = self.StateInternal();
        let world = invocation.WorldIdentity();
        if let Some(cached) = self.states.borrow().get(&world) {
            if SameState(&cached.serialized, &current) {
                return Ok(cached.value.clone());
            }
        }
        self.CheckActive(invocation)?;
        let value = current.as_ref().map_or_else(
            || Ok(invocation.Null()),
            |state| invocation.Deserialize(state),
        )?;
        self.states.borrow_mut().insert(
            world,
            CachedState {
                serialized: current,
                value: value.clone(),
            },
        );
        Ok(value)
    }
    pub fn Length(&self, invocation: &dyn ScriptInvocation) -> Result<usize, Value> {
        self.CheckActive(invocation)?;
        Ok(self.environment.BackForwardLength())
    }
    pub fn ScrollRestoration(
        &self,
        invocation: &dyn ScriptInvocation,
    ) -> Result<ScrollRestorationType, Value> {
        self.CheckActive(invocation)?;
        Ok(self
            .environment
            .CurrentHistoryItem()
            .map_or(ScrollRestorationType::Auto, |item| {
                item.ScrollRestorationType()
            }))
    }
    pub fn SetScrollRestoration(
        &self,
        invocation: &dyn ScriptInvocation,
        value: ScrollRestorationType,
    ) -> Result<(), Value> {
        self.CheckActive(invocation)?;
        let item = self.environment.CurrentHistoryItem().ok_or_else(|| {
            invocation.DOMException(
                "SecurityError",
                "May not use a History object associated with a Document that is not fully active",
            )
        })?;
        if item.ScrollRestorationType() != value {
            item.SetScrollRestorationType(value);
            self.environment.NotifyCurrentHistoryItemChanged();
        }
        Ok(())
    }
    pub fn Go(&self, invocation: &dyn ScriptInvocation, delta: i32) -> Result<(), Value> {
        self.CheckActive(invocation)?;
        self.environment.Go(invocation, delta);
        Ok(())
    }
    pub fn StateObjectAdded(
        &self,
        invocation: &dyn ScriptInvocation,
        value: &Value,
        url: Option<&str>,
        replace: bool,
    ) -> Result<(), Value> {
        // Serialization precedes active-document and URL checks in Chromium.
        let state = invocation.SerializeForStorage(value)?;
        self.CheckActive(invocation)?;
        let url = match url.filter(|url| !url.is_empty()) {
            Some(url) => self.environment.CompleteURL(url),
            None => Some(self.environment.URL()),
        }
        .filter(|url| self.environment.CanChangeToURLForHistoryApi(url))
        .ok_or_else(|| {
            invocation.DOMException(
                "SecurityError",
                "A history state object cannot be created with this URL",
            )
        })?;
        let load = if replace {
            FrameLoadType::ReplaceCurrentItem
        } else {
            FrameLoadType::Standard
        };
        let mut params = NavigateEventDispatchParams::new(url, NavigateEventType::HistoryApi, load);
        params.state_object = Some(state);
        let params = Rc::new(params);
        if self
            .environment
            .DispatchNavigateEvent(invocation, params.clone())
            == NavigateDispatchResult::Continue
        {
            self.environment
                .RunURLAndHistoryUpdateSteps(invocation, &params, load);
        }
        Ok(())
    }
}
fn SameState(
    a: &Option<Rc<dyn SerializedScriptValue>>,
    b: &Option<Rc<dyn SerializedScriptValue>>,
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}
impl ScriptNativeObject for History {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "History"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        for state in self.states.borrow().values() {
            visit(state.value.clone());
        }
    }
}
