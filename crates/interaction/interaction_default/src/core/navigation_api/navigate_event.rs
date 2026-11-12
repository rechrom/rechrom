// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/navigation_api/navigate_event.h:49-211; .cc:76-720.
// Script calls borrow the active invocation; the event owns its handlers,
// interception state, abort controller, promises and deferred-commit listener.
use super::{
    navigate_event_dispatch_params::*,
    navigation_api::{NavigateEventInit, NavigationApi, NavigationRestoreCallback, NavigationType},
    navigation_destination::NavigationDestination,
};
use crate::{
    bindings::core::v8::{promise_all, script_promise::*},
    core::dom::{
        abort_controller::AbortController,
        abort_signal::{AbortSignal, AbortSignalContext},
    },
};
use interaction_dispatch::core::dom::events::event::{
    Event, EventClock, EventInit, EventInterface, EventRef,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterceptState {
    None,
    Intercepted,
    Committed,
    Scrolled,
    Finished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationFocusReset {
    AfterTransition,
    Manual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationScrollBehavior {
    AfterTransition,
    Manual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationHistoryBehavior {
    Auto,
    Push,
    Replace,
}
pub struct NavigationInterceptOptions {
    pub handler: Option<Rc<dyn NavigationHandler>>,
    pub precommit_handler: Option<Rc<dyn NavigationHandler>>,
    pub focus_reset: Option<NavigationFocusReset>,
    pub scroll: Option<NavigationScrollBehavior>,
}
/// The binding retains callback identity and its captured task-attribution
/// state. Invocation projects this same event and its native controller.
pub trait NavigationHandler {
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value));
    fn SetTaskState(&self, state: Option<Rc<dyn std::any::Any>>);
    fn Invoke(
        &self,
        invocation: &dyn ScriptInvocation,
        event: &Rc<NavigateEventOwner>,
        controller: Option<Rc<NavigationController>>,
    ) -> Option<ScriptPromise>;
}
// Source navigation_{precommit,defer_page_swap}_controller.h:24-55.
// Each list invocation shares this exact controller allocation.
pub enum NavigationController {
    Precommit(Rc<NavigateEventOwner>),
    DeferPageSwap(Rc<NavigateEventOwner>),
}
impl NavigationController {
    pub fn Redirect(
        &self,
        invocation: &dyn ScriptInvocation,
        url: &str,
        options: NavigationNavigateOptions,
    ) -> Result<(), Value> {
        match self {
            Self::Precommit(event) => event.Redirect(invocation, url, options),
            Self::DeferPageSwap(_) => unreachable!("IDL controller class has no redirect"),
        }
    }
    pub fn AddHandler(
        &self,
        invocation: &dyn ScriptInvocation,
        handler: Rc<dyn NavigationHandler>,
    ) -> Result<(), Value> {
        match self {
            Self::Precommit(event) => event.AddHandlerDuringPrecommit(invocation, handler),
            Self::DeferPageSwap(_) => unreachable!("IDL controller class has no addHandler"),
        }
    }
    pub fn AddRestoreCallback(
        &self,
        invocation: &dyn ScriptInvocation,
        callback: Rc<dyn NavigationRestoreCallback>,
    ) -> Result<(), Value> {
        match self {
            Self::DeferPageSwap(event) => {
                event.AddDeferPageSwapRestoreCallback(invocation, callback)
            }
            Self::Precommit(_) => unreachable!("IDL controller class has no addRestoreCallback"),
        }
    }
}
pub struct NavigationNavigateOptions {
    pub history: NavigationHistoryBehavior,
    pub state: Option<Value>,
    pub info: Option<Value>,
}
pub trait NavigateEventTask {
    fn Cancel(&self);
}
/// Source-internal calls on the named Document, FrameLoader and Navigation
/// owners. These are required mechanisms, never host URL callbacks. Assembly
/// must construct their real owners before admitting the native event.
pub trait NavigateEventEnvironment {
    fn HasWindow(&self) -> bool;
    fn Clock(&self) -> &dyn EventClock;
    fn AbortContext(&self) -> Rc<dyn AbortSignalContext>;
    fn CurrentURL(&self) -> String;
    fn OriginString(&self) -> String;
    fn CompleteURL(&self, url: &str) -> Option<String>;
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool;
    fn ShouldMaintainTrivialSessionHistory(&self) -> bool;
    fn CaptureCurrentTaskState(&self) -> Option<Rc<dyn std::any::Any>>;
    fn EnterTaskAttributionScope(&self, override_current: bool) -> Box<dyn std::any::Any>;
    fn ConsoleWarning(&self, message: String);
    fn AddFocusedElementChangeObserver(&self, event: Weak<NavigateEventOwner>);
    fn RemoveFocusedElementChangeObserver(&self, event: usize);
    fn AutofocusDelegate(&self) -> Option<u64>;
    fn FocusAutofocusDelegate(
        &self,
        invocation: &dyn ScriptInvocation,
        element: u64,
        user_gesture: bool,
    );
    fn ClearFocusedElement(&self, invocation: &dyn ScriptInvocation);
    fn ClearSequentialFocusNavigationStartingPoint(&self);
    fn PostDelayedLoadStart(
        &self,
        event: Weak<NavigateEventOwner>,
        delay: Duration,
    ) -> Rc<dyn NavigateEventTask>;
    fn ProgressStarted(&self);
    fn ProgressCompleted(&self);
    fn FrameIsLoading(&self) -> bool;
    fn DidFailAsyncSameDocumentCommit(&self);
    fn WillNavigateEventCommitNow(&self, interaction: Option<PerformanceTimelineEntryIdInfo>);
    fn RunURLAndHistoryUpdateSteps(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
        fire_popstate: bool,
    );
    fn DidNavigationApiIntercept(&self);
    fn FrameIsAttached(&self) -> bool;
    fn DidFinishNavigation(&self, success: bool);
    fn AXHandleLoadStart(&self);
    fn AXHandleLoadComplete(&self);
    fn StartLoadingForAsyncNavigationApiCommit(&self);
    fn NavigateEventDeferCrossDocumentCommitEnabled(&self) -> bool;
    fn BindResumeDeferredCommitListener(&self) -> Rc<dyn NavigationResumeDeferredCommitListener>;
    fn TryCreateFragmentAnchor(&self, url: &str) -> bool;
    fn SetProgrammaticScrollOffsetToOrigin(&self);
    fn ProcessScrollForSameDocumentNavigation(
        &self,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
        instant: bool,
    );
}
struct NavigateEventBase(Event);
impl EventInterface for NavigateEventBase {
    fn BaseEvent(&self) -> &Event {
        &self.0
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.0
    }
    fn InterfaceName(&self) -> &str {
        "NavigateEvent"
    }
}
/// A browser dispatch lends the original native Element owner. Web IDL
/// construction retains the caller's exact Element wrapper, including its
/// originating Document namespace, without resolving IDs in another Document.
pub enum NavigateEventSourceElement {
    Native(interaction_dispatch::core::dom::events::event::EventTargetRef),
    Script(Value),
}
pub struct NavigateEventData {
    pub can_intercept: bool,
    pub user_initiated: bool,
    pub hash_change: bool,
    pub form_data: Option<Value>,
    pub download_request: Option<String>,
    pub has_ua_visual_transition: bool,
    pub source_element: Option<NavigateEventSourceElement>,
}
/// Source Web IDL constructor input. Synthetic events retain the caller's
/// destination and AbortSignal; only browser events have an AbortController.
pub struct NavigateEventConstructorInit {
    pub event_init: EventInit,
    pub navigation_type: NavigationType,
    pub destination: Rc<NavigationDestination>,
    pub signal: Rc<AbortSignal>,
    pub data: NavigateEventData,
    pub info: Option<Value>,
}
pub struct NavigateEventOwner {
    self_: Weak<Self>,
    environment: Rc<dyn NavigateEventEnvironment>,
    navigation: Weak<NavigationApi>,
    event: EventRef,
    pub destination: Rc<NavigationDestination>,
    controller: Option<Rc<AbortController>>,
    signal: Rc<AbortSignal>,
    pub data: NavigateEventData,
    pub params: Rc<NavigateEventDispatchParams>,
    navigation_type: Cell<NavigationType>,
    info: RefCell<Value>,
    intercept_state: Cell<InterceptState>,
    focus_reset: Cell<Option<NavigationFocusReset>>,
    scroll_behavior: Cell<Option<NavigationScrollBehavior>>,
    precommit_handlers: RefCell<Vec<Rc<dyn NavigationHandler>>>,
    handlers: RefCell<Vec<Rc<dyn NavigationHandler>>>,
    action_promises: RefCell<Vec<ScriptPromise>>,
    deferred_handlers: RefCell<Vec<Rc<dyn NavigationHandler>>>,
    did_change_focus: Cell<bool>,
    delayed_load_start: RefCell<Option<Rc<dyn NavigateEventTask>>>,
    resume_deferred_commit: RefCell<Option<Rc<dyn NavigationResumeDeferredCommitListener>>>,
}
impl NavigateEventOwner {
    pub fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.Info());
        if let Some(value) = &self.data.form_data {
            visit(value.clone());
        }
        if let Some(NavigateEventSourceElement::Script(element)) = &self.data.source_element {
            visit(element.clone());
        }
        for list in [
            &self.precommit_handlers,
            &self.handlers,
            &self.deferred_handlers,
        ] {
            for handler in list.borrow().iter() {
                handler.VisitScriptValues(visit);
            }
        }
        for promise in self.action_promises.borrow().iter() {
            visit(promise.0.clone());
        }
    }
    pub fn new(
        environment: Rc<dyn NavigateEventEnvironment>,
        navigation: Weak<NavigationApi>,
        invocation: &dyn ScriptInvocation,
        mut init: NavigateEventInit,
        params: Rc<NavigateEventDispatchParams>,
        trusted: bool,
    ) -> Rc<Self> {
        let controller = AbortController::new(environment.AbortContext());
        let destination = NavigationDestination::new(
            params.clone(),
            init.destination_state.take(),
            init.destination_entry.take(),
        );
        let mut event = Event::CreateWithInitializer(
            "navigate",
            &EventInit {
                cancelable: init.cancelable,
                ..EventInit::default()
            },
            environment.Clock(),
        );
        event.SetTrusted(trusted);
        let navigation_type = init.navigation_type;
        let info = init.info.take().unwrap_or_else(|| invocation.Undefined());
        let data = NavigateEventData {
            can_intercept: init.can_intercept,
            user_initiated: init.user_initiated,
            hash_change: init.hash_change,
            form_data: init.form_data.take(),
            download_request: init.download_request.take(),
            has_ua_visual_transition: init.has_ua_visual_transition,
            source_element: init
                .source_element
                .take()
                .map(NavigateEventSourceElement::Native),
        };
        let signal = controller.Signal().clone();
        Self::Construct(
            environment,
            navigation,
            event,
            navigation_type,
            destination,
            Some(controller),
            signal,
            data,
            params,
            info,
        )
    }
    pub fn CreateUntrusted(
        environment: Rc<dyn NavigateEventEnvironment>,
        navigation: Weak<NavigationApi>,
        invocation: &dyn ScriptInvocation,
        event_type: &str,
        init: NavigateEventConstructorInit,
    ) -> Rc<Self> {
        let event = Event::CreateWithInitializer(event_type, &init.event_init, environment.Clock());
        let params = init.destination.params.clone();
        Self::Construct(
            environment,
            navigation,
            event,
            init.navigation_type,
            init.destination,
            None,
            init.signal,
            init.data,
            params,
            init.info.unwrap_or_else(|| invocation.Undefined()),
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn Construct(
        environment: Rc<dyn NavigateEventEnvironment>,
        navigation: Weak<NavigationApi>,
        event: Event,
        navigation_type: NavigationType,
        destination: Rc<NavigationDestination>,
        controller: Option<Rc<AbortController>>,
        signal: Rc<AbortSignal>,
        data: NavigateEventData,
        params: Rc<NavigateEventDispatchParams>,
        info: Value,
    ) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            environment,
            navigation,
            event: EventRef::new(NavigateEventBase(event)),
            destination,
            controller,
            signal,
            data,
            params,
            navigation_type: Cell::new(navigation_type),
            info: RefCell::new(info),
            intercept_state: Cell::new(InterceptState::None),
            focus_reset: Cell::new(None),
            scroll_behavior: Cell::new(None),
            precommit_handlers: RefCell::new(Vec::new()),
            handlers: RefCell::new(Vec::new()),
            action_promises: RefCell::new(Vec::new()),
            deferred_handlers: RefCell::new(Vec::new()),
            did_change_focus: Cell::new(false),
            delayed_load_start: RefCell::new(None),
            resume_deferred_commit: RefCell::new(None),
        })
    }
    // LocalDOMWindow owns navigation_. The weak link mirrors the event's
    // ExecutionContextClient lifetime and avoids Navigation -> Event -> Window
    // ownership cycles. SharedChecks verifies the live Window before script use.
    fn Navigation(&self) -> Rc<NavigationApi> {
        self.navigation
            .upgrade()
            .expect("live Window Navigation owner")
    }
    pub fn Identity(&self) -> usize {
        self as *const Self as usize
    }
    /// Event projection retains this exact source subclass allocation. The
    /// undecorated base handle held above creates no owner -> view cycle.
    pub fn EventRef(&self) -> EventRef {
        self.event.WithClassOwner(self.Retain())
    }
    fn Retain(&self) -> Rc<Self> {
        self.self_.upgrade().expect("live NavigateEvent")
    }
    pub fn NavigationType(&self) -> NavigationType {
        self.navigation_type.get()
    }
    pub fn Info(&self) -> Value {
        self.info.borrow().clone()
    }
    pub fn State(&self) -> InterceptState {
        self.intercept_state.get()
    }
    pub fn DefaultPrevented(&self) -> bool {
        self.event.borrow().BaseEvent().defaultPrevented()
    }
    pub fn Signal(&self) -> Rc<AbortSignal> {
        self.signal.clone()
    }
    pub fn SignalAborted(&self) -> bool {
        self.signal.Aborted()
    }
    pub fn HasNavigationActions(&self) -> bool {
        self.intercept_state.get() != InterceptState::None
    }
    fn SharedChecks(&self, i: &dyn ScriptInvocation, name: &str) -> Result<(), Value> {
        if !self.environment.HasWindow() {
            return Err(i.DOMException(
                "InvalidStateError",
                &format!("{name}() may not be called in a detached window."),
            ));
        }
        if !self.event.borrow().BaseEvent().isTrusted() {
            return Err(i.DOMException(
                "SecurityError",
                &format!("{name}() may only be called on a trusted event."),
            ));
        }
        if self.DefaultPrevented() {
            return Err(i.DOMException(
                "InvalidStateError",
                &format!("{name}() may not be called if the event has been canceled."),
            ));
        }
        Ok(())
    }
    fn CheckInterceptible(&self, i: &dyn ScriptInvocation, deferred: bool) -> Result<(), Value> {
        if self.data.can_intercept {
            return Ok(());
        }
        let verb = if deferred {
            "deferred"
        } else {
            "intercepted by"
        };
        Err(i.DOMException("SecurityError", &format!("A navigation with URL '{}' cannot be {verb} in a window with origin '{}' and URL '{}'.", self.params.URL(), self.environment.OriginString(), self.environment.CurrentURL())))
    }
    // navigate_event.cc:135-202.
    pub fn Intercept(
        &self,
        i: &dyn ScriptInvocation,
        options: NavigationInterceptOptions,
    ) -> Result<(), Value> {
        self.SharedChecks(i, "intercept")?;
        self.CheckInterceptible(i, false)?;
        if !self.event.borrow().BaseEvent().IsBeingDispatched() {
            return Err(i.DOMException(
                "InvalidStateError",
                "intercept() may only be called while the navigate event is being dispatched.",
            ));
        }
        if let Some(handler) = options.precommit_handler {
            if !self.event.borrow().BaseEvent().cancelable() {
                return Err(i.DOMException("InvalidStateError", "intercept() may only be called with a precommitHandler when thenavigate event is cancelable."));
            }
            handler.SetTaskState(self.environment.CaptureCurrentTaskState());
            self.precommit_handlers.borrow_mut().push(handler);
        }
        if !self.HasNavigationActions() {
            self.environment
                .AddFocusedElementChangeObserver(self.self_.clone());
        }
        if let Some(value) = options.focus_reset {
            if let Some(old) = self.focus_reset.get().filter(|old| *old != value) {
                self.environment.ConsoleWarning(format!("The \"{}\" value for intercept()'s focusReset option will override the previously-passed value of \"{}\".", FocusName(value), FocusName(old)));
            }
            self.focus_reset.set(Some(value));
        }
        if let Some(value) = options.scroll {
            if let Some(old) = self.scroll_behavior.get().filter(|old| *old != value) {
                self.environment.ConsoleWarning(format!("The \"{}\" value for intercept()'s scroll option will override the previously-passed value of \"{}\".", ScrollName(value), ScrollName(old)));
            }
            self.scroll_behavior.set(Some(value));
        }
        assert!(matches!(
            self.State(),
            InterceptState::None | InterceptState::Intercepted
        ));
        self.intercept_state.set(InterceptState::Intercepted);
        if let Some(handler) = options.handler {
            handler.SetTaskState(self.environment.CaptureCurrentTaskState());
            self.handlers.borrow_mut().push(handler);
        }
        i.TraceNativeObject(self.Retain());
        Ok(())
    }
    // navigate_event.cc:204-269.
    pub fn DeferPageSwap(
        &self,
        i: &dyn ScriptInvocation,
        handler: Rc<dyn NavigationHandler>,
    ) -> Result<(), Value> {
        self.SharedChecks(i, "defer")?;
        self.CheckInterceptible(i, true)?;
        if !self.event.borrow().BaseEvent().IsBeingDispatched() {
            return Err(i.DOMException(
                "InvalidStateError",
                "deferPageSwap() may only be called while the navigate event is being dispatched.",
            ));
        }
        if self.destination.SameDocument() {
            return Err(i.DOMException(
                "InvalidStateError",
                "deferPageSwap() may only be called for cross-document navigations.",
            ));
        }
        self.deferred_handlers.borrow_mut().push(handler);
        i.TraceNativeObject(self.Retain());
        Ok(())
    }
    pub fn MaybeDeferCrossDocumentCommit(&self, i: &dyn ScriptInvocation) -> Result<(), Value> {
        if self.deferred_handlers.borrow().is_empty() {
            return Ok(());
        }
        assert!(self
            .environment
            .NavigateEventDeferCrossDocumentCommitEnabled());
        let listener = self.environment.BindResumeDeferredCommitListener();
        *self.params.resume_deferred_commit_listener.borrow_mut() = Some(listener.clone());
        *self.resume_deferred_commit.borrow_mut() = Some(listener);
        let handlers = std::mem::take(&mut *self.deferred_handlers.borrow_mut());
        let _handler_roots = Self::KeepHandlersAlive(i, &handlers);
        i.TraceNativeObject(self.Retain());
        let event = self.Retain();
        let controller = Rc::new(NavigationController::DeferPageSwap(event.clone()));
        let promises: Vec<_> = handlers
            .into_iter()
            .filter_map(|h| h.Invoke(i, &event, Some(controller.clone())))
            .collect();
        let fulfilled = event.clone();
        promise_all::WaitForAll(
            i,
            vec![self.Retain()],
            &promises,
            false,
            Box::new(move |_, _| fulfilled.ResumeDeferredCommit()),
            Box::new(move |_, _| event.ResumeDeferredCommit()),
        )
    }
    // navigate_event.cc:271-347. Publish state only after storage serialization.
    pub fn Redirect(
        &self,
        i: &dyn ScriptInvocation,
        url: &str,
        options: NavigationNavigateOptions,
    ) -> Result<(), Value> {
        assert_ne!(self.State(), InterceptState::None);
        self.SharedChecks(i, "redirect")?;
        if self.State() > InterceptState::Intercepted {
            return Err(i.DOMException("InvalidStateError", "navigation has already committed."));
        }
        if !matches!(
            self.NavigationType(),
            NavigationType::Push | NavigationType::Replace
        ) {
            return Err(i.DOMException(
                "InvalidStateError",
                "redirect() may only be used on push and replace navigations.",
            ));
        }
        let completed = self
            .environment
            .CompleteURL(url)
            .ok_or_else(|| i.DOMException("SyntaxError", &format!("Invalid URL '{url}'.")))?;
        if !self.environment.CanChangeToURLForHistoryApi(&completed) {
            return Err(i.DOMException(
                "SecurityError",
                &format!(
                    "Cannot redirect to '{}' in a document with origin '{}' and URL '{}'.",
                    completed,
                    self.environment.OriginString(),
                    self.environment.CurrentURL()
                ),
            ));
        }
        if options.history == NavigationHistoryBehavior::Push
            && self.environment.ShouldMaintainTrivialSessionHistory()
        {
            return Err(i.DOMException("NotSupportedError", "redirect() may not override the history behavior when navigating in a trivial session history context"));
        }
        if let Some(state) = options.state {
            let serialized = i.SerializeForStorage(&state)?;
            self.destination
                .SetSerializedState(Some(serialized.clone()));
            self.Navigation()
                .SetOngoingTrackerSerializedState(serialized);
        }
        self.params.SetURL(completed);
        match options.history {
            NavigationHistoryBehavior::Push => self.navigation_type.set(NavigationType::Push),
            NavigationHistoryBehavior::Replace => self.navigation_type.set(NavigationType::Replace),
            NavigationHistoryBehavior::Auto => {}
        }
        if let Some(info) = options.info {
            *self.info.borrow_mut() = info;
        }
        i.TraceNativeObject(self.Retain());
        Ok(())
    }
    pub fn AddHandlerDuringPrecommit(
        &self,
        i: &dyn ScriptInvocation,
        handler: Rc<dyn NavigationHandler>,
    ) -> Result<(), Value> {
        self.SharedChecks(i, "addHandler")?;
        if self.State() > InterceptState::Intercepted {
            return Err(i.DOMException("InvalidStateError", "navigation has already committed."));
        }
        self.handlers.borrow_mut().push(handler);
        i.TraceNativeObject(self.Retain());
        Ok(())
    }
    pub fn AddDeferPageSwapRestoreCallback(
        &self,
        i: &dyn ScriptInvocation,
        callback: Rc<dyn NavigationRestoreCallback>,
    ) -> Result<(), Value> {
        self.SharedChecks(i, "addRestoreCallback")?;
        if self.State() >= InterceptState::Intercepted {
            return Err(i.DOMException(
                "InvalidStateError",
                "navigation has already committed or intercepted.",
            ));
        }
        self.Navigation().AddRestoreCallback(i, callback);
        Ok(())
    }
    // navigate_event.cc:377-469.
    pub fn MaybeCommitImmediately(&self, i: &dyn ScriptInvocation) -> Result<(), Value> {
        *self.delayed_load_start.borrow_mut() = Some(
            self.environment
                .PostDelayedLoadStart(self.self_.clone(), Duration::from_millis(50)),
        );
        if self.precommit_handlers.borrow().is_empty() {
            self.CommitNow(i);
            return Ok(());
        }
        self.environment.ProgressStarted();
        let handlers = std::mem::take(&mut *self.precommit_handlers.borrow_mut());
        let _handler_roots = Self::KeepHandlersAlive(i, &handlers);
        i.TraceNativeObject(self.Retain());
        let event = self.Retain();
        let controller = Rc::new(NavigationController::Precommit(event.clone()));
        let promises: Vec<_> = handlers
            .into_iter()
            .filter_map(|h| {
                let _scope = self.environment.EnterTaskAttributionScope(
                    self.params.involvement != UserNavigationInvolvement::None,
                );
                h.Invoke(i, &event, Some(controller.clone()))
            })
            .collect();
        let fulfilled = event.clone();
        promise_all::WaitForAll(
            i,
            vec![self.Retain()],
            &promises,
            true,
            Box::new(move |scope, _| fulfilled.CommitNow(scope)),
            Box::new(move |scope, error| event.ReactDone(scope, false, Some(error))),
        )
    }
    fn CommitNow(&self, i: &dyn ScriptInvocation) {
        assert_eq!(self.State(), InterceptState::Intercepted);
        assert!(self.params.destination_item.is_none() || self.params.state_object.is_none());
        if self.SignalAborted() {
            return;
        }
        self.environment
            .WillNavigateEventCommitNow(self.params.interaction_id.get());
        self.intercept_state.set(InterceptState::Committed);
        let fire_popstate = self.params.event_type == NavigateEventType::Fragment
            && (!self.Navigation().HasOngoingMethodTracker()
                || self.NavigationType() == NavigationType::Traverse);
        self.environment.RunURLAndHistoryUpdateSteps(
            i,
            &self.params,
            ToLoadType(self.NavigationType()),
            fire_popstate,
        );
        if self.environment.HasWindow() {
            self.environment.DidNavigationApiIntercept();
        }
    }
    // navigate_event.cc:472-584.
    pub fn React(&self, i: &dyn ScriptInvocation) -> Result<(), Value> {
        assert!(self.handlers.borrow().is_empty());
        let promises = self.action_promises.borrow().clone();
        let event = self.Retain();
        let fulfilled = event.clone();
        promise_all::WaitForAll(
            i,
            vec![self.Retain()],
            &promises,
            true,
            Box::new(move |scope, _| fulfilled.ReactDone(scope, true, None)),
            Box::new(move |scope, error| event.ReactDone(scope, false, Some(error))),
        )?;
        if self.HasNavigationActions() && self.environment.HasWindow() {
            self.environment.AXHandleLoadStart();
        }
        Ok(())
    }
    pub fn ResumeDeferredCommit(&self) {
        let listener = self.resume_deferred_commit.borrow().clone();
        if let Some(listener) = listener {
            assert!(self
                .environment
                .NavigateEventDeferCrossDocumentCommitEnabled());
            listener.ResumeDeferredCommit();
        }
    }
    fn KeepHandlersAlive(
        i: &dyn ScriptInvocation,
        handlers: &[Rc<dyn NavigationHandler>],
    ) -> Box<dyn std::any::Any> {
        let mut values = Vec::new();
        for handler in handlers {
            handler.VisitScriptValues(&mut |value| values.push(value));
        }
        i.KeepAlive(&values)
    }
    fn CancelDelayedLoadStart(&self) {
        let task = self.delayed_load_start.borrow_mut().take();
        if let Some(task) = task {
            task.Cancel();
        }
    }
    fn ReactDone(&self, i: &dyn ScriptInvocation, fulfilled: bool, error: Option<Value>) {
        assert_ne!(self.State(), InterceptState::Finished);
        if !self.environment.HasWindow() {
            return;
        }
        self.CancelDelayedLoadStart();
        if self.SignalAborted() {
            return;
        }
        assert_eq!(
            self.Navigation().OngoingEventIdentity(),
            Some(self.Identity())
        );
        self.Navigation()
            .ClearOngoingEventIf(i, Some(self.Identity()));
        if self.State() == InterceptState::Intercepted {
            assert!(!fulfilled);
            self.environment.DidFailAsyncSameDocumentCommit();
        }
        if self.State() >= InterceptState::Committed {
            self.PotentiallyResetTheFocus(i);
            if fulfilled {
                self.PotentiallyProcessScrollBehavior();
            }
            self.intercept_state.set(InterceptState::Finished);
        }
        if fulfilled {
            self.Navigation().DidFinishOngoingNavigation(i);
        } else {
            self.Abort(i, error.expect("rejected navigation reason"));
        }
        if self.HasNavigationActions() {
            if self.environment.FrameIsAttached() {
                self.environment.DidFinishNavigation(fulfilled);
            }
            self.environment.AXHandleLoadComplete();
        }
    }
    pub fn Abort(&self, i: &dyn ScriptInvocation, error: Value) {
        if self.event.borrow().BaseEvent().IsBeingDispatched() {
            self.event.borrow_mut().BaseEventMut().preventDefault();
        }
        let navigation = self.Navigation();
        let previous = navigation.OngoingEventIdentity();
        self.controller
            .as_ref()
            .expect("browser NavigateEvent controller")
            .Abort(i, error.clone());
        navigation.ClearOngoingEventIf(i, previous);
        self.CancelDelayedLoadStart();
        if !self.DefaultPrevented() {
            match self.State() {
                InterceptState::Intercepted => {
                    if self.environment.FrameIsLoading() {
                        self.environment.ProgressCompleted();
                    }
                    self.environment.DidFailAsyncSameDocumentCommit();
                }
                InterceptState::Committed => self.environment.ProgressCompleted(),
                _ => {}
            }
        }
        navigation.DidAbort(i, error);
    }
    pub fn DelayedLoadStartTimerFired(&self) {
        if self.environment.HasWindow() {
            self.environment.StartLoadingForAsyncNavigationApiCommit();
        }
    }
    pub fn FinalizeNavigationActionPromisesList(&self, i: &dyn ScriptInvocation) {
        let handlers = std::mem::take(&mut *self.handlers.borrow_mut());
        let _handler_roots = Self::KeepHandlersAlive(i, &handlers);
        i.TraceNativeObject(self.Retain());
        let event = self.Retain();
        for handler in handlers {
            let _scope = self.environment.EnterTaskAttributionScope(
                self.params.involvement != UserNavigationInvolvement::None,
            );
            if let Some(promise) = handler.Invoke(i, &event, None) {
                self.action_promises.borrow_mut().push(promise);
                i.TraceNativeObject(self.Retain());
            }
        }
    }
    // navigate_event.cc:607-720.
    fn PotentiallyResetTheFocus(&self, i: &dyn ScriptInvocation) {
        assert!(matches!(
            self.State(),
            InterceptState::Committed | InterceptState::Scrolled
        ));
        self.environment
            .RemoveFocusedElementChangeObserver(self.Identity());
        if self.did_change_focus.get()
            || self.focus_reset.get() == Some(NavigationFocusReset::Manual)
        {
            return;
        }
        if let Some(delegate) = self.environment.AutofocusDelegate() {
            self.environment
                .FocusAutofocusDelegate(i, delegate, self.data.user_initiated);
        } else {
            self.environment.ClearFocusedElement(i);
            self.environment
                .ClearSequentialFocusNavigationStartingPoint();
        }
    }
    pub fn DidChangeFocus(&self) {
        assert!(self.HasNavigationActions());
        self.did_change_focus.set(true);
    }
    pub fn Scroll(&self, i: &dyn ScriptInvocation) -> Result<(), Value> {
        self.SharedChecks(i, "scroll")?;
        let error = match self.State() {
            InterceptState::Finished => {
                Some("scroll() may not be called after transition completes")
            }
            InterceptState::Scrolled => Some("scroll() already called"),
            InterceptState::None => Some("intercept() must be called before scroll()"),
            InterceptState::Intercepted => Some("scroll() may not be called before commit."),
            InterceptState::Committed => None,
        };
        if let Some(error) = error {
            return Err(i.DOMException("InvalidStateError", error));
        }
        self.ProcessScrollBehavior();
        Ok(())
    }
    fn PotentiallyProcessScrollBehavior(&self) {
        assert!(matches!(
            self.State(),
            InterceptState::Committed | InterceptState::Scrolled
        ));
        if self.State() == InterceptState::Scrolled
            || self.scroll_behavior.get() == Some(NavigationScrollBehavior::Manual)
        {
            return;
        }
        self.ProcessScrollBehavior();
    }
    fn ProcessScrollBehavior(&self) {
        assert_eq!(self.State(), InterceptState::Committed);
        self.intercept_state.set(InterceptState::Scrolled);
        if !self.environment.TryCreateFragmentAnchor(&self.params.URL()) {
            self.environment.SetProgrammaticScrollOffsetToOrigin();
        }
        self.environment.ProcessScrollForSameDocumentNavigation(
            &self.params,
            ToLoadType(self.NavigationType()),
            self.data.has_ua_visual_transition,
        );
    }
}
impl Drop for NavigateEventOwner {
    fn drop(&mut self) {
        if let Some(task) = self.delayed_load_start.get_mut().take() {
            task.Cancel();
        }
    }
}
fn ToLoadType(kind: NavigationType) -> FrameLoadType {
    match kind {
        NavigationType::Push => FrameLoadType::Standard,
        NavigationType::Replace => FrameLoadType::ReplaceCurrentItem,
        NavigationType::Reload => FrameLoadType::Reload,
        NavigationType::Traverse => FrameLoadType::BackForward,
    }
}
fn FocusName(value: NavigationFocusReset) -> &'static str {
    match value {
        NavigationFocusReset::AfterTransition => "after-transition",
        NavigationFocusReset::Manual => "manual",
    }
}
fn ScrollName(value: NavigationScrollBehavior) -> &'static str {
    match value {
        NavigationScrollBehavior::AfterTransition => "after-transition",
        NavigationScrollBehavior::Manual => "manual",
    }
}

impl super::navigation_api::NavigateEvent for NavigateEventOwner {
    fn EventRef(&self) -> EventRef {
        self.EventRef()
    }
    fn DefaultPrevented(&self) -> bool {
        self.DefaultPrevented()
    }
    fn SignalAborted(&self) -> bool {
        self.SignalAborted()
    }
    fn HasNavigationActions(&self) -> bool {
        self.HasNavigationActions()
    }
    fn Abort(&self, i: &dyn ScriptInvocation, error: Value) {
        self.Abort(i, error);
    }
    fn MaybeCommitImmediately(&self, i: &dyn ScriptInvocation) {
        if let Err(error) = self.MaybeCommitImmediately(i) {
            self.Abort(i, error);
        }
    }
    fn MaybeDeferCrossDocumentCommit(
        &self,
        i: &dyn ScriptInvocation,
        _: &Rc<NavigateEventDispatchParams>,
    ) {
        if let Err(error) = self.MaybeDeferCrossDocumentCommit(i) {
            self.Abort(i, error);
        }
    }
    fn FinalizeNavigationActionPromisesList(&self, i: &dyn ScriptInvocation) {
        self.FinalizeNavigationActionPromisesList(i);
    }
    fn React(&self, i: &dyn ScriptInvocation) {
        if let Err(error) = self.React(i) {
            self.Abort(i, error);
        }
    }
}

impl ScriptNativeObject for NavigateEventOwner {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "NavigateEvent"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        self.VisitScriptValues(visit);
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        visit(self.destination.clone());
        if let Some(controller) = &self.controller {
            visit(controller.clone());
        }
        visit(self.signal.clone());
    }
}
impl ScriptNativeObject for NavigationController {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        match self {
            Self::Precommit(_) => "NavigationPrecommitController",
            Self::DeferPageSwap(_) => "NavigationDeferPageSwapController",
        }
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        match self {
            Self::Precommit(event) | Self::DeferPageSwap(event) => visit(event.clone()),
        }
    }
}
