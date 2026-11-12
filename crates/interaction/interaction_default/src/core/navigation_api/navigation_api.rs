// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/navigation_api/navigation_api.h:97-194;
// .cc:105-109,120-209,216-317,438-451,715-1003,1005-1110.
// Dispatch/cancel/abort owns its original mutable state. Native Window retains
// this Rc. Script/event/promise/history collaborators are REQUIRED source owners;
// this slice never manufactures a successful dispatch when one is absent.
use super::{
    navigate_event::{NavigateEventEnvironment, NavigateEventOwner},
    navigate_event_dispatch_params::*,
};
#[path = "navigation_api_methods.rs"]
mod methods;
use crate::bindings::core::v8::script_promise::{ScriptInvocation, ScriptNativeObject};
use crate::core::html::anchor_element_utils::{
    AnchorNavigationApi, DownloadNavigateEventParams, NavigateDispatchResult,
};
pub use interaction_dispatch::bindings::script_invocation::{ScriptValue, SerializedScriptValue};
use interaction_dispatch::core::dom::events::event::EventTargetRef;
pub use methods::{NavigationOptions, NavigationReloadOptions, NavigationTraverseRequest};
pub use navigation_history::HistoryItemOwner as HistoryItem;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};
pub trait NavigationHistoryEntry: ScriptNativeObject {
    fn Url(&self) -> Option<String>;
    fn SameDocument(&self) -> bool;
    fn Index(&self) -> i32;
    fn SerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>>;
    fn SetAndSaveState(&self, state: Option<Rc<dyn SerializedScriptValue>>);
    fn Key(&self) -> String;
    fn Id(&self) -> String;
    fn CloneForWindow(&self, window: u64) -> Rc<dyn NavigationHistoryEntry>;
}
pub trait NavigationApiMethodTracker: ScriptNativeObject {
    fn NavigationResult(&self) -> super::navigation_api_method_tracker::NavigationResult;
    fn SerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>>;
    fn SetSerializedState(&self, state: Option<Rc<dyn SerializedScriptValue>>);
    fn Info(&self) -> Rc<dyn ScriptValue>;
    fn Key(&self) -> String;
    fn RejectFinishedPromise(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>);
    fn ResolveFinishedPromise(&self, invocation: &dyn ScriptInvocation);
    fn NotifyAboutTheCommittedToEntry(
        &self,
        invocation: &dyn ScriptInvocation,
        entry: Rc<dyn NavigationHistoryEntry>,
        load: FrameLoadType,
    );
}
pub trait NavigationTransition: ScriptNativeObject {
    fn NavigationType(&self) -> NavigationType;
    fn From(&self) -> Option<Rc<dyn NavigationHistoryEntry>>;
    fn Destination(&self) -> Rc<super::navigation_destination::NavigationDestination>;
    fn Committed(
        &self,
        invocation: &dyn ScriptInvocation,
    ) -> Result<crate::bindings::core::v8::script_promise::ScriptPromise, Rc<dyn ScriptValue>>;
    fn Finished(
        &self,
        invocation: &dyn ScriptInvocation,
    ) -> Result<crate::bindings::core::v8::script_promise::ScriptPromise, Rc<dyn ScriptValue>>;
    fn RejectFinishedPromise(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>);
    fn ResolveFinishedPromise(&self, invocation: &dyn ScriptInvocation);
    fn ResolveCommittedPromise(&self, invocation: &dyn ScriptInvocation);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationType {
    Push,
    Replace,
    Reload,
    Traverse,
}
fn ToNavigationType(load: FrameLoadType) -> NavigationType {
    match load {
        FrameLoadType::Standard => NavigationType::Push,
        FrameLoadType::ReplaceCurrentItem => NavigationType::Replace,
        FrameLoadType::Reload | FrameLoadType::ReloadBypassingCache => NavigationType::Reload,
        FrameLoadType::BackForward | FrameLoadType::Restore => NavigationType::Traverse,
    }
}
#[derive(Clone, Copy, Debug)]
pub enum NavigationDirection {
    Back,
    Forward,
    Reload,
}
/// This is NavigateEvent's source constructor input, not a script-free event
/// snapshot. All identity-bearing fields reference their actual source owners.
pub struct NavigateEventInit {
    pub navigation_type: NavigationType,
    pub destination: Rc<NavigateEventDispatchParams>,
    pub destination_state: Option<Rc<dyn SerializedScriptValue>>,
    pub destination_entry: Option<Rc<dyn NavigationHistoryEntry>>,
    pub cancelable: bool,
    pub can_intercept: bool,
    pub hash_change: bool,
    pub user_initiated: bool,
    pub form_data: Option<Rc<dyn ScriptValue>>,
    pub info: Option<Rc<dyn ScriptValue>>,
    pub source_element: Option<EventTargetRef>,
    pub download_request: Option<String>,
    pub has_ua_visual_transition: bool,
}
/// Required native NavigateEvent (navigate_event.cc:236-582): the event owner
/// retains AbortController, handlers, deferred-commit pipe and promise lists.
/// Its Abort calls this same NavigationApi::ClearOngoingEventIf/DidAbort.
pub trait NavigateEvent: ScriptNativeObject {
    fn EventRef(&self) -> interaction_dispatch::core::dom::events::event::EventRef;
    fn DefaultPrevented(&self) -> bool;
    fn SignalAborted(&self) -> bool;
    fn HasNavigationActions(&self) -> bool;
    fn Abort(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>);
    fn MaybeCommitImmediately(&self, invocation: &dyn ScriptInvocation);
    fn MaybeDeferCrossDocumentCommit(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
    );
    fn FinalizeNavigationActionPromisesList(&self, invocation: &dyn ScriptInvocation);
    fn React(&self, invocation: &dyn ScriptInvocation);
}
pub trait NavigationRestoreCallback {
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Rc<dyn ScriptValue>));
    fn Invoke(&self, invocation: &dyn ScriptInvocation, api: &NavigationApi);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelNavigationReason {
    NewNavigation,
    Other,
    Dropped,
    NavigateEvent,
}
#[derive(Clone, Copy, Debug)]
pub enum TraverseCancelledReason {
    NotFound,
    SandboxViolation,
    AbortedBeforeCommit,
}
/// Each operation belongs to the named native Script/Window/History/DOM owner.
/// None runs the navigation selection or returns a synthesized dispatch result.
pub trait NavigationApiEnvironment {
    fn CompleteURL(&self, url: &str) -> Option<String>;
    fn PageDismissalEventBeingDispatched(&self) -> bool;
    fn IsOnInitialEmptyDocument(&self) -> bool;
    fn ShouldMaintainTrivialSessionHistory(&self) -> bool;
    fn CreateFrameLoadRequest(
        &self,
        invocation: &dyn ScriptInvocation,
        url: &str,
    ) -> Rc<crate::core::loader::frame_load_request::FrameLoadRequest>;
    fn Navigate(
        &self,
        invocation: &dyn ScriptInvocation,
        request: Rc<crate::core::loader::frame_load_request::FrameLoadRequest>,
        load: FrameLoadType,
    );
    fn NavigationStartTime(&self) -> interaction_input::base::time::time::TimeTicks;
    fn IsOutermostMainFrame(&self) -> bool;
    fn HasTransientUserActivation(&self) -> bool;
    fn AsyncSameDocumentNavigationStarted(&self, invocation: &dyn ScriptInvocation) -> Option<u64>;
    fn NavigateToNavigationApiKey(&self, request: NavigationTraverseRequest);
    fn CountNavigationNavigateJavaScriptURL(&self);
    fn EventTarget(&self)
        -> Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>;
    fn NavigateEventHandlerPresenceChanged(&self, present: bool);
    fn WindowIdentity(&self) -> u64;
    fn HasWindow(&self) -> bool;
    fn HasFrame(&self) -> bool;
    fn ContextDestroyed(&self) -> bool;
    fn HasLoadedNonInitialEmptyDocument(&self) -> bool;
    fn OriginIsOpaque(&self) -> bool;
    fn ShouldAllowBindingAccess(&self, element: &EventTargetRef) -> bool;
    fn IsMainFrame(&self) -> bool;
    fn IsHistoryUserActivationActive(&self) -> bool;
    fn ConsumeHistoryUserActivation(&self);
    fn DispatchCurrentEntryChange(
        &self,
        invocation: &dyn ScriptInvocation,
        from: Option<Rc<dyn NavigationHistoryEntry>>,
        kind: Option<NavigationType>,
    );
    fn DispatchDispose(
        &self,
        invocation: &dyn ScriptInvocation,
        entry: &Rc<dyn NavigationHistoryEntry>,
    );
    fn MarkNavigationStateCommittedIfPresent(&self);
    fn CurrentURL(&self) -> String;
    fn SameOriginWithDestination(&self, url: &str) -> bool;
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool;
    fn EqualIgnoringFragment(&self, url: &str, current: &str) -> bool;
    fn CurrentHistoryItem(&self) -> Rc<HistoryItem>;
    fn SameOriginWithWindow(&self, window: u64) -> bool;
    fn IsPrerendering(&self) -> bool;
    fn MakeEntryFromItem(&self, item: &Rc<HistoryItem>) -> Rc<dyn NavigationHistoryEntry>;
    /// Returns None for non-form, non-POST or absent source. FormData::Create may
    /// dispatch synchronously; the caller checks navigation/window/frame again.
    fn CreatePostFormData(
        &self,
        invocation: &dyn ScriptInvocation,
        source: &EventTargetRef,
    ) -> Option<Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>>;
    /// Native Document/Frame services used by NavigateEvent. Event construction,
    /// NavigationApi callbacks and dispatch stay in this package, so assembly
    /// cannot replace the event or synthesize a successful dispatch.
    fn NavigateEventEnvironment(&self) -> Rc<dyn NavigateEventEnvironment>;
    fn BeginNavigationEventTiming(
        &self,
        event: &Rc<dyn NavigateEvent>,
    ) -> (
        Box<dyn std::any::Any>,
        Option<PerformanceTimelineEntryIdInfo>,
    );
    fn NavigationSourcePseudoClassEnabled(&self) -> bool;
    fn StartNavigationState(
        &self,
        url: &str,
        source: Option<EventTargetRef>,
        direction: Option<NavigationDirection>,
    );
    fn FinishNavigationStateIfPresent(&self);
    fn SetPendingNavigationDestinationURL(&self, url: &str);
    fn CreateTransition(
        &self,
        invocation: &dyn ScriptInvocation,
        kind: NavigationType,
        current: Option<Rc<dyn NavigationHistoryEntry>>,
        event: &Rc<dyn NavigateEvent>,
    ) -> Rc<dyn NavigationTransition>;
    fn SkeletonLoaderNavigateIfPresent(&self, url: &str);
    fn SkeletonLoaderCancelIfPresent(&self);
    fn StartNavigationPreviewIfNeeded(&self);
    fn AbortNavigationPreview(&self);
    fn ResetSameDocumentNavigationTasksIfTrackerPresent(&self);
    fn FrameIsAttached(&self) -> bool;

    fn DispatchNavigateError(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>);
    fn DispatchNavigateSuccess(&self, invocation: &dyn ScriptInvocation);
    fn NavigateEventDeferCrossDocumentCommitEnabled(&self) -> bool;
}
struct State {
    entries: Vec<Rc<dyn NavigationHistoryEntry>>,
    keys: HashMap<String, usize>,
    current_index: Option<usize>,
    ongoing_tracker: Option<Rc<dyn NavigationApiMethodTracker>>,
    upcoming_non_traverse: Option<Rc<dyn NavigationApiMethodTracker>>,
    upcoming_traverse: HashMap<String, Rc<dyn NavigationApiMethodTracker>>,
    ongoing_event: Option<Rc<dyn NavigateEvent>>,
    transition: Option<Rc<dyn NavigationTransition>>,
    restore_callbacks: Vec<Rc<dyn NavigationRestoreCallback>>,
}
pub struct NavigationApi {
    self_: Weak<Self>,
    environment: Rc<dyn NavigationApiEnvironment>,
    activation: Rc<super::navigation_activation::NavigationActivation>,
    state: RefCell<State>,
    has_dropped_navigation: Cell<bool>,
    navigate_event_handler_count: Cell<u32>,
}
impl NavigationApi {
    pub fn CreateUntrustedNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        event_type: &str,
        init: super::navigate_event::NavigateEventConstructorInit,
    ) -> Rc<NavigateEventOwner> {
        NavigateEventOwner::CreateUntrusted(
            self.environment.NavigateEventEnvironment(),
            self.self_.clone(),
            invocation,
            event_type,
            init,
        )
    }
    pub fn new(environment: Rc<dyn NavigationApiEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            environment,
            activation: super::navigation_activation::NavigationActivation::new(),
            state: RefCell::new(State {
                entries: Vec::new(),
                keys: HashMap::new(),
                current_index: None,
                ongoing_tracker: None,
                upcoming_non_traverse: None,
                upcoming_traverse: HashMap::new(),
                ongoing_event: None,
                transition: None,
                restore_callbacks: Vec::new(),
            }),
            has_dropped_navigation: Cell::new(false),
            navigate_event_handler_count: Cell::new(0),
        })
    }
    fn Retain(&self) -> Rc<Self> {
        self.self_.upgrade().expect("live NavigationApi")
    }
    fn HasEntriesAndEventsDisabled(&self) -> bool {
        !self.environment.HasFrame()
            || self.environment.ContextDestroyed()
            || !self.environment.HasLoadedNonInitialEmptyDocument()
            || self.environment.OriginIsOpaque()
    }
    pub fn Activation(&self) -> Option<Rc<super::navigation_activation::NavigationActivation>> {
        (!self.HasEntriesAndEventsDisabled()).then(|| self.activation.clone())
    }
    pub fn CurrentEntry(&self) -> Option<Rc<dyn NavigationHistoryEntry>> {
        if self.HasEntriesAndEventsDisabled() {
            return None;
        }
        let state = self.state.borrow();
        state.current_index.map(|i| state.entries[i].clone())
    }
    pub fn Entries(&self) -> Vec<Rc<dyn NavigationHistoryEntry>> {
        if self.HasEntriesAndEventsDisabled() {
            Vec::new()
        } else {
            self.state.borrow().entries.clone()
        }
    }
    fn PopulateKeySet(state: &mut State) {
        assert!(state.keys.is_empty());
        for (i, entry) in state.entries.iter().enumerate() {
            state.keys.entry(entry.Key()).or_insert(i);
        }
    }
    pub fn GetExistingEntryFor(
        &self,
        key: &str,
        id: &str,
    ) -> Option<Rc<dyn NavigationHistoryEntry>> {
        let s = self.state.borrow();
        s.keys
            .get(key)
            .and_then(|i| (s.entries[*i].Id() == id).then(|| s.entries[*i].clone()))
    }
    pub fn InitializeForNewWindow(
        &self,
        invocation: &dyn ScriptInvocation,
        current: Rc<HistoryItem>,
        load: FrameLoadType,
        regular_commit: bool,
        previous: Option<&NavigationApi>,
        back: &[Rc<HistoryItem>],
        forward: &[Rc<HistoryItem>],
        previous_item: Option<Rc<HistoryItem>>,
    ) {
        assert!(self.state.borrow().entries.is_empty());
        if self.HasEntriesAndEventsDisabled() {
            return;
        }
        if !regular_commit
            || ((current.URL() == "about:blank" || current.URL() == "about:srcdoc")
                && load != FrameLoadType::BackForward)
        {
            if let Some(previous) = previous {
                if !previous.state.borrow().entries.is_empty()
                    && self
                        .environment
                        .SameOriginWithWindow(previous.environment.WindowIdentity())
                {
                    let (entries, index) = {
                        let s = previous.state.borrow();
                        (
                            s.entries
                                .iter()
                                .map(|e| e.CloneForWindow(self.environment.WindowIdentity()))
                                .collect(),
                            s.current_index,
                        )
                    };
                    {
                        let mut s = self.state.borrow_mut();
                        s.entries = entries;
                        s.current_index = index;
                        Self::PopulateKeySet(&mut s);
                    }
                    // UpdateForNavigation is a required History/NavigationApi commit stage. It
                    // cannot be replaced by an initialization-only vector rewrite.
                    self.UpdateForNavigation(invocation, &current, load);
                    return;
                }
            }
        }
        let entries = back
            .iter()
            .chain(std::iter::once(&current))
            .chain(forward.iter())
            .map(|item| self.environment.MakeEntryFromItem(item))
            .collect();
        {
            let mut s = self.state.borrow_mut();
            s.entries = entries;
            s.current_index = Some(back.len());
            Self::PopulateKeySet(&mut s);
        }
        let previous = previous_item.map(|item| {
            self.GetExistingEntryFor(&item.NavigationApiKey(), &item.NavigationApiId())
                .unwrap_or_else(|| self.environment.MakeEntryFromItem(&item))
        });
        self.activation.Update(
            self.CurrentEntry(),
            previous,
            if self.environment.IsPrerendering() {
                NavigationType::Push
            } else {
                ToNavigationType(load)
            },
        );
    }
    // NavigationApi.cc:216-317. Commit publishes native entry identity before
    // events; the actual JS microtask scope preserves promise/event order.
    pub fn UpdateForNavigation(
        &self,
        invocation: &dyn ScriptInvocation,
        item: &Rc<HistoryItem>,
        load: FrameLoadType,
    ) {
        if self.HasEntriesAndEventsDisabled() {
            return;
        }
        let old_current = self.CurrentEntry();
        let mut disposed = Vec::new();
        if load.IsTraversal() {
            let mut s = self.state.borrow_mut();
            let index = *s
                .keys
                .get(&item.NavigationApiKey())
                .expect("traverse entry exists");
            s.current_index = Some(index);
        } else if load == FrameLoadType::Standard {
            let mut s = self.state.borrow_mut();
            let index = s.current_index.map_or(0, |i| i + 1);
            s.current_index = Some(index);
            disposed = s.entries.iter().skip(index).cloned().collect();
            for entry in &disposed {
                s.keys.remove(&entry.Key());
            }
            s.entries.truncate(index);
        } else if load == FrameLoadType::ReplaceCurrentItem {
            let s = self.state.borrow();
            disposed.push(s.entries[s.current_index.expect("replace has current entry")].clone());
        }
        if matches!(
            load,
            FrameLoadType::Standard | FrameLoadType::ReplaceCurrentItem
        ) {
            let entry = self.environment.MakeEntryFromItem(item);
            let key = entry.Key();
            let mut s = self.state.borrow_mut();
            let index = s.current_index.unwrap();
            if load == FrameLoadType::Standard {
                assert_eq!(s.entries.len(), index);
                s.entries.push(entry);
            } else {
                s.entries[index] = entry;
            }
            s.keys.entry(key).or_insert(index);
        }
        let tracker = self.state.borrow().ongoing_tracker.clone();
        if let Some(tracker) = tracker {
            tracker.NotifyAboutTheCommittedToEntry(
                invocation,
                self.CurrentEntry().expect("committed entry"),
                load,
            );
        }
        let transition = self.state.borrow().transition.clone();
        if let Some(transition) = transition {
            transition.ResolveCommittedPromise(invocation);
        }
        let event = self.state.borrow().ongoing_event.clone();
        invocation.TraceNativeObject(self.Retain());
        invocation.WithRunMicrotasksScope(&mut || {
            self.environment.DispatchCurrentEntryChange(
                invocation,
                old_current.clone(),
                Some(ToNavigationType(load)),
            );
            if let Some(event) = &event {
                event.FinalizeNavigationActionPromisesList(invocation);
            }
            for entry in &disposed {
                self.environment.DispatchDispose(invocation, entry);
            }
            if self.state.borrow().transition.is_some() {
                self.environment.MarkNavigationStateCommittedIfPresent();
            }
            if let Some(event) = &event {
                if self.environment.HasFrame() {
                    event.React(invocation);
                }
            }
        });
    }
    pub fn SetUpcomingNonTraverseTracker(&self, tracker: Rc<dyn NavigationApiMethodTracker>) {
        self.state.borrow_mut().upcoming_non_traverse = Some(tracker);
    }
    pub fn SetUpcomingTraverseTracker(
        &self,
        key: String,
        tracker: Rc<dyn NavigationApiMethodTracker>,
    ) {
        self.state
            .borrow_mut()
            .upcoming_traverse
            .insert(key, tracker);
    }
    fn PromoteUpcomingNavigationToOngoing(&self, key: Option<&str>) {
        let mut s = self.state.borrow_mut();
        assert!(s.ongoing_tracker.is_none());
        s.ongoing_tracker = if let Some(key) = key {
            assert!(s.upcoming_non_traverse.is_none());
            s.upcoming_traverse.remove(key)
        } else {
            s.upcoming_non_traverse.take()
        };
    }
    pub fn Dispatch(
        &self,
        invocation: &dyn ScriptInvocation,
        params: Rc<NavigateEventDispatchParams>,
    ) -> NavigateDispatchResult {
        let e = self.environment.as_ref();
        assert!(e.HasWindow());
        if self.HasEntriesAndEventsDisabled() {
            let s = self.state.borrow();
            assert!(
                s.ongoing_tracker.is_none()
                    && s.upcoming_non_traverse.is_none()
                    && s.upcoming_traverse.is_empty()
            );
            return NavigateDispatchResult::Continue;
        }
        let source = params.source_element.borrow().clone();
        if source.is_some_and(|s| !e.ShouldAllowBindingAccess(&s)) {
            params.source_element.borrow_mut().take();
        }
        while self.state.borrow().ongoing_event.is_some() {
            self.AbortOngoingNavigation(invocation);
        }
        assert!(self.state.borrow().ongoing_tracker.is_none());
        if !e.HasWindow() {
            return NavigateDispatchResult::Abort;
        }
        let key = params
            .destination_item
            .as_ref()
            .map(|item| item.NavigationApiKey());
        if params.frame_load_type.IsTraversal()
            && params.event_type == NavigateEventType::Fragment
            && !self
                .state
                .borrow()
                .keys
                .contains_key(key.as_deref().unwrap_or(""))
        {
            self.TraverseCancelled(
                invocation,
                key.as_deref().unwrap_or(""),
                TraverseCancelledReason::AbortedBeforeCommit,
            );
            return NavigateDispatchResult::Abort;
        }
        self.PromoteUpcomingNavigationToOngoing(key.as_deref());
        let navigation_type = ToNavigationType(params.frame_load_type);
        let tracker = self.state.borrow().ongoing_tracker.clone();
        let destination_state = if let Some(item) = &params.destination_item {
            item.NavigationApiState()
        } else if let Some(tracker) = &tracker {
            tracker.SerializedState()
        } else if navigation_type == NavigationType::Reload {
            e.CurrentHistoryItem().NavigationApiState()
        } else {
            None
        };
        let destination_entry = if params.frame_load_type.IsTraversal() {
            let s = self.state.borrow();
            key.as_ref()
                .and_then(|key| s.keys.get(key))
                .map(|i| s.entries[*i].clone())
        } else {
            None
        };
        let traversal_cancel = params.frame_load_type.IsTraversal()
            && params.event_type != NavigateEventType::CrossDocument
            && e.IsMainFrame()
            && (!params.is_browser_initiated || e.IsHistoryUserActivationActive());
        let current_url = e.CurrentURL();
        let can_intercept = e.CanChangeToURLForHistoryApi(&params.URL())
            && (params.event_type != NavigateEventType::CrossDocument
                || !params.frame_load_type.IsTraversal());
        let hash_change = params.event_type == NavigateEventType::Fragment
            && params.URL() != current_url
            && e.EqualIgnoringFragment(&params.URL(), &current_url);
        let source = params.source_element.borrow().clone();
        let form_data = source
            .as_ref()
            .and_then(|source| e.CreatePostFormData(invocation, source));
        if form_data.is_some()
            && (self.state.borrow().ongoing_event.is_some() || !e.HasWindow() || !e.HasFrame())
        {
            return NavigateDispatchResult::Abort;
        }
        let info = self
            .state
            .borrow()
            .ongoing_tracker
            .clone()
            .map(|t| t.Info());
        let init = NavigateEventInit {
            navigation_type,
            destination: params.clone(),
            destination_state,
            destination_entry: destination_entry.clone(),
            cancelable: !params.frame_load_type.IsTraversal() || traversal_cancel,
            can_intercept,
            hash_change,
            user_initiated: params.involvement != UserNavigationInvolvement::None,
            form_data: form_data.map(|data| invocation.NativeObjectValue(data)),
            info,
            source_element: source,
            download_request: params.download_filename.clone(),
            has_ua_visual_transition: params.has_ua_visual_transition,
        };
        let event: Rc<dyn NavigateEvent> = NavigateEventOwner::new(
            e.NavigateEventEnvironment(),
            self.self_.clone(),
            invocation,
            init,
            params.clone(),
            true,
        );
        {
            let mut s = self.state.borrow_mut();
            assert!(s.ongoing_event.is_none());
            s.ongoing_event = Some(event.clone());
        }
        invocation.TraceNativeObject(self.Retain());
        if e.NavigationSourcePseudoClassEnabled() {
            let direction = if params.frame_load_type == FrameLoadType::BackForward {
                destination_entry.as_ref().map(|dest| {
                    if self.GetIndexFor(dest)
                        < self
                            .CurrentEntry()
                            .as_ref()
                            .map_or(-1, |entry| self.GetIndexFor(entry))
                    {
                        NavigationDirection::Back
                    } else {
                        NavigationDirection::Forward
                    }
                })
            } else if params.frame_load_type.IsReload() {
                Some(NavigationDirection::Reload)
            } else {
                None
            };
            e.StartNavigationState(
                &params.URL(),
                params.source_element.borrow().clone(),
                direction,
            );
        }
        self.has_dropped_navigation.set(false);
        {
            let (_timing, interaction_id) = e.BeginNavigationEventTiming(&event);
            if let Some(id) = interaction_id {
                params.interaction_id.set(Some(id));
            }
            let target = e.EventTarget();
            target.DispatchEventInternalWithTarget(
                &event.EventRef(),
                target.AsEventTargetRef(),
                Some(invocation.EventListenerInvoker()),
            );
        }
        if event.DefaultPrevented() {
            if params.frame_load_type.IsTraversal() && e.HasFrame() {
                e.ConsumeHistoryUserActivation();
            }
            if !event.SignalAborted() {
                self.AbortOngoingNavigation(invocation);
            }
            return NavigateDispatchResult::Abort;
        }
        if params.event_type == NavigateEventType::CrossDocument
            && !event.HasNavigationActions()
            && e.SameOriginWithDestination(&params.URL())
        {
            e.SetPendingNavigationDestinationURL(&params.URL());
        }
        if event.HasNavigationActions() {
            let transition =
                e.CreateTransition(invocation, navigation_type, self.CurrentEntry(), &event);
            self.state.borrow_mut().transition = Some(transition);
            invocation.TraceNativeObject(self.Retain());
            event.MaybeCommitImmediately(invocation);
        } else if params.event_type == NavigateEventType::CrossDocument {
            e.SkeletonLoaderNavigateIfPresent(&params.URL());
            e.StartNavigationPreviewIfNeeded();
            event.MaybeDeferCrossDocumentCommit(invocation, &params);
        }
        if event.HasNavigationActions() {
            NavigateDispatchResult::Intercept
        } else {
            NavigateDispatchResult::Continue
        }
    }
    pub fn InformAboutCanceledNavigation(
        &self,
        invocation: &dyn ScriptInvocation,
        reason: CancelNavigationReason,
    ) {
        let e = self.environment.as_ref();
        if reason != CancelNavigationReason::NavigateEvent {
            e.ResetSameDocumentNavigationTasksIfTrackerPresent();
        }
        e.SkeletonLoaderCancelIfPresent();
        e.AbortNavigationPreview();
        if reason == CancelNavigationReason::Dropped {
            self.has_dropped_navigation.set(true);
            return;
        }
        if self.HasEntriesAndEventsDisabled() {
            return;
        }
        if self.state.borrow().ongoing_event.is_some() {
            self.AbortOngoingNavigation(invocation);
        }
        if !self.state.borrow().upcoming_traverse.is_empty() && e.HasFrame() && !e.FrameIsAttached()
        {
            let trackers: Vec<_> = self
                .state
                .borrow()
                .upcoming_traverse
                .values()
                .cloned()
                .collect();
            for tracker in trackers {
                self.TraverseCancelled(
                    invocation,
                    &tracker.Key(),
                    TraverseCancelledReason::AbortedBeforeCommit,
                );
            }
            assert!(self.state.borrow().upcoming_traverse.is_empty());
        }
    }
    pub fn TraverseCancelled(
        &self,
        invocation: &dyn ScriptInvocation,
        key: &str,
        reason: TraverseCancelledReason,
    ) {
        let tracker = self.state.borrow().upcoming_traverse.get(key).cloned();
        let Some(tracker) = tracker else {
            return;
        };
        let (name,message): (&'static str, String)=match reason{TraverseCancelledReason::NotFound=>("InvalidStateError","Invalid key".into()),TraverseCancelledReason::SandboxViolation=>("SecurityError",format!("Navigating to key {key} would require a navigation that violates this frame's sandbox policy")),TraverseCancelledReason::AbortedBeforeCommit=>("AbortError","Navigation was aborted".into())};
        tracker.RejectFinishedPromise(invocation, invocation.DOMException(name, &message));
        self.state.borrow_mut().upcoming_traverse.remove(key);
        invocation.TraceNativeObject(self.Retain());
    }
    pub fn AbortOngoingNavigation(&self, invocation: &dyn ScriptInvocation) {
        let event = self
            .state
            .borrow()
            .ongoing_event
            .clone()
            .expect("ongoing NavigateEvent");
        let error = invocation.DOMException("AbortError", "Navigation was aborted");
        event.Abort(invocation, error);
    }
    /// Called by NavigateEvent::Abort after abort listeners have reentered. Only
    /// clear the event which was current before those listeners ran.
    pub fn ClearOngoingEventIf(&self, invocation: &dyn ScriptInvocation, identity: Option<usize>) {
        {
            let mut state = self.state.borrow_mut();
            if state
                .ongoing_event
                .as_ref()
                .map(|event| event.ScriptIdentity())
                == identity
            {
                state.ongoing_event = None;
            }
        }
        invocation.TraceNativeObject(self.Retain());
    }
    pub fn OngoingEventIdentity(&self) -> Option<usize> {
        self.state
            .borrow()
            .ongoing_event
            .as_ref()
            .map(|event| event.ScriptIdentity())
    }
    /// The outgoing FrameLoader request carries the tracker's actual immutable
    /// storage payload to the destination DocumentLoader across host sequences.
    pub fn OngoingNavigationApiState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.state
            .borrow()
            .ongoing_tracker
            .clone()
            .and_then(|tracker| tracker.SerializedState())
    }
    pub fn HasOngoingMethodTracker(&self) -> bool {
        self.state.borrow().ongoing_tracker.is_some()
    }
    pub fn HasNonDroppedOngoingNavigation(&self) -> bool {
        self.state
            .borrow()
            .ongoing_event
            .as_ref()
            .is_some_and(|e| e.HasNavigationActions())
            && !self.has_dropped_navigation.get()
    }
    // Same keys_to_indices_ lookup as GetIndexFor, for the concrete entry's
    // source Index() callback without allocating another Rc or index table.
    pub fn GetIndexForIdentity(&self, key: &str, identity: usize) -> i32 {
        let s = self.state.borrow();
        s.keys
            .get(key)
            .filter(|i| s.entries[**i].ScriptIdentity() == identity)
            .map_or(-1, |i| *i as i32)
    }
    pub fn SetOngoingTrackerSerializedState(&self, state: Rc<dyn SerializedScriptValue>) {
        let tracker = self.state.borrow().ongoing_tracker.clone();
        if let Some(tracker) = tracker {
            tracker.SetSerializedState(Some(state));
        }
    }
    pub fn GetIndexFor(&self, entry: &Rc<dyn NavigationHistoryEntry>) -> i32 {
        let s = self.state.borrow();
        s.keys
            .get(&entry.Key())
            .filter(|i| Rc::ptr_eq(&s.entries[**i], entry))
            .map_or(-1, |i| *i as i32)
    }
    pub fn AddRestoreCallback(
        &self,
        invocation: &dyn ScriptInvocation,
        callback: Rc<dyn NavigationRestoreCallback>,
    ) {
        self.state.borrow_mut().restore_callbacks.push(callback);
        invocation.TraceNativeObject(self.Retain());
    }
    pub fn FlushRestoreCallbacks(&self, invocation: &dyn ScriptInvocation) {
        let callbacks = std::mem::take(&mut self.state.borrow_mut().restore_callbacks);
        assert!(
            callbacks.is_empty()
                || self
                    .environment
                    .NavigateEventDeferCrossDocumentCommitEnabled()
        );
        let mut callback_values = Vec::new();
        for callback in &callbacks {
            callback.VisitScriptValues(&mut |value| callback_values.push(value));
        }
        let _callback_roots = invocation.KeepAlive(&callback_values);
        for callback in callbacks {
            callback.Invoke(invocation, self);
        }
        invocation.TraceNativeObject(self.Retain());
    }
    pub fn DidAbort(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>) {
        let tracker = self.state.borrow().ongoing_tracker.clone();
        if let Some(tracker) = tracker {
            tracker.RejectFinishedPromise(invocation, error.clone());
            self.state.borrow_mut().ongoing_tracker = None;
        }
        if self.environment.NavigationSourcePseudoClassEnabled() {
            self.environment.FinishNavigationStateIfPresent();
        }
        invocation.TraceNativeObject(self.Retain());
        self.environment
            .DispatchNavigateError(invocation, error.clone());
        self.FlushRestoreCallbacks(invocation);
        let transition = self.state.borrow().transition.clone();
        if let Some(transition) = transition {
            transition.RejectFinishedPromise(invocation, error);
            self.state.borrow_mut().transition = None;
        }
        invocation.TraceNativeObject(self.Retain());
    }
    pub fn DidFinishOngoingNavigation(&self, invocation: &dyn ScriptInvocation) {
        let tracker = self.state.borrow().ongoing_tracker.clone();
        if let Some(tracker) = tracker {
            tracker.ResolveFinishedPromise(invocation);
            self.state.borrow_mut().ongoing_tracker = None;
        }
        if self.environment.NavigationSourcePseudoClassEnabled() {
            self.environment.FinishNavigationStateIfPresent();
        }
        invocation.TraceNativeObject(self.Retain());
        self.environment.DispatchNavigateSuccess(invocation);
        let transition = self.state.borrow().transition.clone();
        if let Some(transition) = transition {
            transition.ResolveFinishedPromise(invocation);
            self.state.borrow_mut().transition = None;
        }
        invocation.TraceNativeObject(self.Retain());
    }
}
impl AnchorNavigationApi for NavigationApi {
    fn DispatchNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        p: DownloadNavigateEventParams,
    ) -> NavigateDispatchResult {
        let mut params = NavigateEventDispatchParams::new(
            p.url,
            NavigateEventType::CrossDocument,
            FrameLoadType::Standard,
        );
        if p.activation {
            params.involvement = UserNavigationInvolvement::Activation;
        }
        params.download_filename = Some(p.download_filename);
        *params.source_element.borrow_mut() = Some(p.source_element);
        self.Dispatch(invocation, Rc::new(params))
    }
    fn InformAboutCanceledNavigationDropped(&self, invocation: &dyn ScriptInvocation) {
        self.InformAboutCanceledNavigation(invocation, CancelNavigationReason::Dropped);
    }
}

impl ScriptNativeObject for NavigationApi {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "Navigation"
    }
    fn EventTarget(
        &self,
    ) -> Option<Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>> {
        Some(self.environment.EventTarget())
    }
    fn DidAddEventListener(&self, event_type: &str) {
        if event_type != "navigate" || !self.environment.HasFrame() {
            return;
        }
        let count = self
            .navigate_event_handler_count
            .get()
            .checked_add(1)
            .expect("navigate handler count");
        self.navigate_event_handler_count.set(count);
        if count == 1 {
            self.environment.NavigateEventHandlerPresenceChanged(true);
        }
    }
    fn DidRemoveEventListener(&self, event_type: &str) {
        if event_type != "navigate" || !self.environment.HasFrame() {
            return;
        }
        let count = self
            .navigate_event_handler_count
            .get()
            .checked_sub(1)
            .expect("registered navigate handler");
        self.navigate_event_handler_count.set(count);
        if count == 0 {
            self.environment.NavigateEventHandlerPresenceChanged(false);
        }
    }

    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Rc<dyn ScriptValue>)) {
        for callback in &self.state.borrow().restore_callbacks {
            callback.VisitScriptValues(visit);
        }
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        visit(self.activation.clone());
        let objects = {
            let s = self.state.borrow();
            let mut objects: Vec<Rc<dyn ScriptNativeObject>> = s
                .entries
                .iter()
                .cloned()
                .map(|e| e as Rc<dyn ScriptNativeObject>)
                .collect();
            if let Some(t) = &s.ongoing_tracker {
                objects.push(t.clone());
            }
            if let Some(t) = &s.upcoming_non_traverse {
                objects.push(t.clone());
            }
            for t in s.upcoming_traverse.values() {
                objects.push(t.clone());
            }
            if let Some(e) = &s.ongoing_event {
                objects.push(e.clone());
            }
            if let Some(t) = &s.transition {
                objects.push(t.clone());
            }
            objects
        };
        for object in objects {
            visit(object);
        }
    }
}
