//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! navigation_api.cc:61-102,452-713. Original promise/tracker ownership and
//! synchronous Frame::Navigate call order, including aborted-before-dispatch.
use super::super::{
    navigate_event::{NavigationHistoryBehavior, NavigationNavigateOptions},
    navigation_api_method_tracker::{NavigationApiMethodTrackerOwner, NavigationResult},
};
use super::*;
use crate::bindings::core::v8::script_promise::{NavigationScriptInvocation, Value};
use crate::core::loader::frame_load_request::{ClientNavigationReason, FrameLoadRequest};
#[derive(Default)]
pub struct NavigationOptions {
    pub info: Option<Value>,
}
#[derive(Default)]
pub struct NavigationReloadOptions {
    pub info: Option<Value>,
    pub state: Option<Value>,
}
#[derive(Clone, Debug)]
pub struct NavigationTraverseRequest {
    pub key: String,
    pub has_transient_user_activation: bool,
    pub actual_navigation_start: interaction_input::base::time::time::TimeTicks,
    pub task_state_id: Option<u64>,
}
fn early_error(i: &dyn ScriptInvocation, error: Value) -> Result<NavigationResult, Value> {
    let committed = i.NewPromiseCapability()?;
    let finished = i.NewPromiseCapability()?;
    i.Call(&committed.reject, &i.Undefined(), &[error.clone()])?;
    i.Call(&finished.reject, &i.Undefined(), &[error])?;
    Ok(NavigationResult {
        committed: committed.promise,
        finished: finished.promise,
    })
}
fn error(
    i: &dyn ScriptInvocation,
    name: &'static str,
    message: &str,
) -> Result<NavigationResult, Value> {
    early_error(i, i.DOMException(name, message))
}
fn success(
    i: &dyn ScriptInvocation,
    entry: Rc<dyn NavigationHistoryEntry>,
) -> Result<NavigationResult, Value> {
    let value = i.HistoryEntryValue(Some(entry));
    Ok(NavigationResult {
        committed: i.PromiseResolve(value.clone())?,
        finished: i.PromiseResolve(value)?,
    })
}
impl NavigationApi {
    pub fn CanGoBack(&self) -> bool {
        !self.HasEntriesAndEventsDisabled()
            && self.state.borrow().current_index.is_some_and(|n| n > 0)
    }
    pub fn CanGoForward(&self) -> bool {
        if self.HasEntriesAndEventsDisabled() {
            return false;
        }
        let s = self.state.borrow();
        s.current_index.is_some_and(|n| n + 1 < s.entries.len())
    }
    pub fn Transition(&self) -> Option<Rc<dyn NavigationTransition>> {
        self.state.borrow().transition.clone()
    }
    pub fn UpdateCurrentEntry(&self, i: &dyn ScriptInvocation, state: Value) -> Result<(), Value> {
        let entry = self.CurrentEntry().ok_or_else(|| {
            i.DOMException(
                "InvalidStateError",
                "updateCurrent() cannot be called when navigation.current is null.",
            )
        })?;
        let serialized = i.SerializeForStorage(&state)?;
        entry.SetAndSaveState(Some(serialized));
        self.environment
            .DispatchCurrentEntryChange(i, Some(entry), None);
        Ok(())
    }
    fn SharedNavigationChecks(&self, i: &dyn ScriptInvocation, method: &str) -> Result<(), Value> {
        if !self.environment.HasFrame() {
            return Err(i.DOMException(
                "InvalidStateError",
                &format!("{method} cannot be called when the Window is detached."),
            ));
        }
        if self.environment.PageDismissalEventBeingDispatched() {
            return Err(i.DOMException(
                "InvalidStateError",
                &format!("{method} cannot be called during unload or beforeunload."),
            ));
        }
        Ok(())
    }
    pub fn Navigate(
        &self,
        i: &dyn ScriptInvocation,
        url: &str,
        options: NavigationNavigateOptions,
    ) -> Result<NavigationResult, Value> {
        let Some(url) = self.environment.CompleteURL(url) else {
            return error(i, "SyntaxError", "Invalid URL ''.");
        };
        if url::Url::parse(&url).is_ok_and(|u| u.scheme() == "javascript") {
            self.environment.CountNavigationNavigateJavaScriptURL();
            return error(
                i,
                "NotSupportedError",
                "The javascript: protocol is not supported",
            );
        }
        let state = match options.state {
            Some(state) => match i.SerializeForStorage(&state) {
                Ok(v) => Some(v),
                Err(e) => return early_error(i, e),
            },
            None => None,
        };
        let request = self.environment.CreateFrameLoadRequest(i, &url);
        request
            .client_navigation_reason
            .set(ClientNavigationReason::FrameNavigation);
        let push = options.history == NavigationHistoryBehavior::Push;
        if push {
            if self.environment.IsOnInitialEmptyDocument() {
                return error(i,"NotSupportedError","A \"push\" navigation was explicitly requested, but only a \"replace\" navigation is possible while on the initial about:blank document.");
            }
            if self.environment.ShouldMaintainTrivialSessionHistory() {
                return error(i,"NotSupportedError","A \"push\" navigation was explicitly requested, but only a \"replace\" navigation is possible when navigating in a trivial session history context, which maintains only one session history entry.");
            }
        }
        let load = if options.history == NavigationHistoryBehavior::Replace {
            FrameLoadType::ReplaceCurrentItem
        } else {
            FrameLoadType::Standard
        };
        self.PerformNonTraverseNavigation(
            i,
            {
                request.force_history_push.set(push);
                request
            },
            load,
            state,
            options.info,
        )
    }
    pub fn Reload(
        &self,
        i: &dyn ScriptInvocation,
        options: NavigationReloadOptions,
    ) -> Result<NavigationResult, Value> {
        let state = match options.state {
            Some(state) => match i.SerializeForStorage(&state) {
                Ok(v) => Some(v),
                Err(e) => return early_error(i, e),
            },
            None => self.CurrentEntry().and_then(|e| e.SerializedState()),
        };
        let request = self
            .environment
            .CreateFrameLoadRequest(i, &self.environment.CurrentURL());
        request
            .client_navigation_reason
            .set(ClientNavigationReason::FrameNavigation);
        self.PerformNonTraverseNavigation(i, request, FrameLoadType::Reload, state, options.info)
    }
    fn PerformNonTraverseNavigation(
        &self,
        i: &dyn ScriptInvocation,
        request: Rc<FrameLoadRequest>,
        load: FrameLoadType,
        state: Option<Rc<dyn SerializedScriptValue>>,
        info: Option<Value>,
    ) -> Result<NavigationResult, Value> {
        if let Err(e) = self.SharedNavigationChecks(
            i,
            if load == FrameLoadType::Reload {
                "reload()"
            } else {
                "navigate()"
            },
        ) {
            return early_error(i, e);
        }
        let tracker = NavigationApiMethodTrackerOwner::new(i, info, String::new(), state)?;
        if self.HasEntriesAndEventsDisabled() {
            tracker.CleanupForWillNeverSettle();
        } else {
            self.state.borrow_mut().upcoming_non_traverse = Some(tracker.clone());
            i.TraceNativeObject(self.Retain());
        }
        self.environment.Navigate(i, request, load);
        let still_upcoming = self
            .state
            .borrow()
            .upcoming_non_traverse
            .as_ref()
            .is_some_and(|t| t.ScriptIdentity() == tracker.ScriptIdentity());
        if still_upcoming {
            self.state.borrow_mut().upcoming_non_traverse = None;
            i.TraceNativeObject(self.Retain());
            return error(i, "AbortError", "Navigation was aborted");
        }
        Ok(tracker.GetNavigationResult().clone())
    }
    pub fn TraverseTo(
        &self,
        i: &dyn ScriptInvocation,
        key: &str,
        options: NavigationOptions,
    ) -> Result<NavigationResult, Value> {
        let actual_navigation_start = self.environment.NavigationStartTime();
        if let Err(e) = self.SharedNavigationChecks(i, "traverseTo()/back()/forward()") {
            return early_error(i, e);
        }
        if !self.state.borrow().keys.contains_key(key) {
            return error(i, "InvalidStateError", "Invalid key");
        }
        let current = self
            .CurrentEntry()
            .expect("valid traversal has current entry");
        if key == current.Key() {
            return success(i, current);
        }
        if let Some(tracker) = self.state.borrow().upcoming_traverse.get(key) {
            return Ok(tracker.NavigationResult());
        }
        let tracker = NavigationApiMethodTrackerOwner::new(i, options.info, key.into(), None)?;
        self.state
            .borrow_mut()
            .upcoming_traverse
            .insert(key.into(), tracker.clone());
        i.TraceNativeObject(self.Retain());
        let task_state_id = if i.WorldId() == 0 && self.environment.IsOutermostMainFrame() {
            self.environment.AsyncSameDocumentNavigationStarted(i)
        } else {
            None
        };
        self.environment
            .NavigateToNavigationApiKey(NavigationTraverseRequest {
                key: key.into(),
                has_transient_user_activation: self.environment.HasTransientUserActivation(),
                actual_navigation_start,
                task_state_id,
            });
        Ok(tracker.GetNavigationResult().clone())
    }
    pub fn Back(
        &self,
        i: &dyn ScriptInvocation,
        options: NavigationOptions,
    ) -> Result<NavigationResult, Value> {
        if !self.CanGoBack() {
            return error(i, "InvalidStateError", "Cannot go back");
        }
        let key = {
            let s = self.state.borrow();
            s.entries[s.current_index.unwrap() - 1].Key()
        };
        self.TraverseTo(i, &key, options)
    }
    pub fn Forward(
        &self,
        i: &dyn ScriptInvocation,
        options: NavigationOptions,
    ) -> Result<NavigationResult, Value> {
        if !self.CanGoForward() {
            return error(i, "InvalidStateError", "Cannot go forward");
        }
        let key = {
            let s = self.state.borrow();
            s.entries[s.current_index.unwrap() + 1].Key()
        };
        self.TraverseTo(i, &key, options)
    }
}
