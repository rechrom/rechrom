//! LocalDOMWindow/FrameLoader services for the translated Navigation API.
//! Selection, event/interception state and promises stay in interaction;
//! URL/history commit stages stay in DocumentLoader. This file only wires the
//! original Window, Document, task queue and host navigation request owners.
use super::*;
use interaction::{
    default::{
        bindings::core::v8::script_promise::{ScriptInvocation, ScriptNativeObject},
        core::{
            dom::abort_signal::AbortSignalContext,
            loader::{
                frame_load_request::{
                    FrameLoadRequest, FrameLoadRequestEnvironment, NavigationInitiator,
                },
                resource_request::{ReferrerPolicy, RequestorOrigin, ResourceRequest},
            },
            navigation_api::{
                navigate_event::{NavigateEventEnvironment, NavigateEventOwner, NavigateEventTask},
                navigate_event_dispatch_params::*,
                navigation_api::*,
                navigation_current_entry_change_event::NavigationCurrentEntryChangeEvent,
                navigation_history_entry::{
                    NavigationHistoryEntryOwner, NavigationHistoryEntryWindow,
                },
                navigation_transition::NavigationTransitionOwner,
            },
            timing::{event_timing::NavigationEventTiming, window_performance::WindowPerformance},
        },
    },
    dispatch::core::dom::events::{
        event::{Event as NativeEvent, EventClock, EventInterface, EventRef, EventTargetRef},
        event_target::EventTarget,
    },
    input::base::time::time::TimeTicks,
};
use std::{any::Any, cell::OnceCell, time::Duration};

pub(super) mod anchor_navigation;

/// HistoryItem::GenerateResourceRequest replay fields shared by native History,
/// Navigation traversal, and the browser's Back/Reload producer. The caller
/// retains navigation provenance and destination/entry-array metadata.
pub fn RestoreHistoryRequest(
    request: &mut URLRequest,
    item: &navigation_history::HistoryItemSnapshot,
) {
    request.referrer = item.referrer.clone();
    request.referrer_policy = Some(LoaderReferrerPolicy(item.referrer_policy));
    request.encoded_body = item.form_data.clone();
    request.body.clear();
    request.method = if item.form_data.is_some() {
        "POST"
    } else {
        "GET"
    }
    .into();
    request
        .headers
        .retain(|(name, _)| !name.eq_ignore_ascii_case("content-type"));
    if item.form_data.is_some() {
        if let Some(content_type) = &item.form_content_type {
            request
                .headers
                .push(("Content-Type".into(), content_type.clone()));
        }
    }
}
pub(super) fn SnapshotHistoryItem(
    item: &HistoryItem,
    origin: &dom::persistent_document::SecurityOrigin,
) -> Arc<navigation_history::HistoryItemSnapshot> {
    let mut snapshot = item.Snapshot();
    Arc::get_mut(&mut snapshot)
        .expect("new history snapshot")
        .committed_origin = Some(RequestorOriginFor(origin));
    snapshot
}

impl PageState {
    pub(super) fn SnapshotCurrentHistoryItem(
        &self,
        item: &HistoryItem,
    ) -> Arc<navigation_history::HistoryItemSnapshot> {
        if let Some(domain) = self.local_frames.get() {
            return domain.SnapshotHistoryItem(item, domain.root);
        }
        SnapshotHistoryItem(
            item,
            self.document
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin(),
        )
    }
    pub(super) fn SaveHistoryState(&self) {
        let Some(item) = self.DocumentLoaderNavigation().CurrentHistoryItem() else {
            return;
        };
        let snapshot = self.SnapshotCurrentHistoryItem(&item);
        self.client.borrow_mut().DidUpdateHistoryState(snapshot);
    }
    pub(super) fn PublishChildHistory(&self, load: FrameLoadType) {
        let Some(item) = self.DocumentLoaderNavigation().CurrentHistoryItem() else {
            return;
        };
        let snapshot = self.SnapshotCurrentHistoryItem(&item);
        self.client
            .borrow_mut()
            .DidCommitChildHistory(snapshot, load);
        if load == FrameLoadType::Standard {
            if let Some(domain) = self.local_frames.get() {
                domain.session_entry_id.set(None);
                let mut state = domain.session_history_state.get();
                state.IncreaseHistoryListFromNavigation();
                domain.SetMirrorHistoryState(state);
            }
        }
    }
}

pub(super) struct NavigationWindowEnvironment {
    self_: Weak<Self>,
    page: RefCell<Weak<PageState>>,
    frame: Weak<super::local_frame::LocalFrame>,
    document: RefCell<Weak<DocumentEngine>>,
    bindings: RefCell<Weak<RefCell<DOMJavaScriptBindings>>>,
    window: Weak<RefCell<WindowJavaScriptBindings>>,
    interaction: RefCell<Weak<crate::interaction_services::PageInteraction>>,
    performance: RefCell<Weak<WindowPerformance>>,
    loader: RefCell<Option<Rc<document_loader::DocumentLoaderNavigationState>>>,
    navigation: OnceCell<Weak<NavigationApi>>,
    history_length: Cell<usize>,
    history_index: Cell<usize>,
    previous_window: RefCell<Weak<Self>>,
    successor_window: RefCell<Weak<Self>>,
    previous_api: RefCell<Option<Rc<NavigationApi>>>,
    task_tracker: RefCell<Option<Rc<foundation::task_attribution::TaskAttributionTracker>>>,
    target: Rc<EventTarget>,
    clock: Rc<dyn EventClock>,
    initialized: Cell<bool>,
    loaded: Cell<bool>,
    focused_observers: RefCell<Vec<Weak<NavigateEventOwner>>>,
}
impl NavigationWindowEnvironment {
    pub(super) fn SetSessionHistoryState(&self, state: navigation_history::SessionHistoryState) {
        self.history_index.set(state.offset.max(0) as usize);
        self.history_length.set(state.length.max(0) as usize);
    }
    /// Browser traversal supplies the original frame item, including entries
    /// not exposed through this realm's Navigation API entry array.
    pub(super) fn TraverseToHistoryItem(
        &self,
        snapshot: Arc<navigation_history::HistoryItemSnapshot>,
    ) -> bool {
        if !self.Live() {
            return false;
        }
        let Some(window) = self.window.upgrade() else {
            return false;
        };
        let environment = self.Retain();
        window.borrow_mut().EnqueueTaskWithRuntime(Box::new(move |runtime, realm| {
            let _ = runtime.WithHostRuntime(realm, &mut |host| {
                let Some(bindings) = environment.bindings.borrow().upgrade() else { return; };
                DOMJavaScriptBindings::RunNavigationInvocation(&bindings, host, &mut |invocation| {
                    let item = HistoryItem::Restore(&snapshot);
                    let mut params = NavigateEventDispatchParams::new(item.URL(), NavigateEventType::Fragment, FrameLoadType::BackForward);
                    params.destination_item = Some(item);
                    params.is_synchronously_committed_same_document = false;
                    params.is_browser_initiated = true;
                    let params = Rc::new(params);
                    if environment.Api().Dispatch(invocation, params.clone()) == interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Continue {
                        environment.CommitSameDocument(invocation, &params, FrameLoadType::BackForward, false, true);
                        NavigateEventEnvironment::ProcessScrollForSameDocumentNavigation(environment.as_ref(), &params, FrameLoadType::BackForward, false);
                    }
                });
            });
        }), None);
        true
    }
    pub(super) fn TraverseToHistoryEntry(&self, key: &str) -> bool {
        if !self.Live() || !self.Api().Entries().iter().any(|entry| entry.Key() == key) {
            return false;
        }
        NavigationApiEnvironment::NavigateToNavigationApiKey(
            self,
            NavigationTraverseRequest {
                key: key.to_owned(),
                has_transient_user_activation: self.Active(),
                actual_navigation_start: self.clock.Now(),
                task_state_id: None,
            },
        );
        true
    }
    pub(super) fn Install(
        page: Weak<PageState>,
        frame: Weak<super::local_frame::LocalFrame>,
        document: &Rc<DocumentEngine>,
        web_api: &WebApiEngine,
        interaction: &Rc<crate::interaction_services::PageInteraction>,
        performance: Option<Rc<WindowPerformance>>,
    ) -> Rc<Self> {
        let clock = page
            .upgrade()
            .expect("Window belongs to its Page domain")
            .InteractionEventClock();
        let target = web_api.DOMBindings().borrow().CreateNavigationEventTarget();
        let owner = Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            page: RefCell::new(page),
            frame,
            document: RefCell::new(Rc::downgrade(document)),
            bindings: RefCell::new(Rc::downgrade(web_api.DOMBindings())),
            window: Rc::downgrade(web_api.WindowBindings()),
            interaction: RefCell::new(Rc::downgrade(interaction)),
            performance: RefCell::new(performance.as_ref().map_or_else(Weak::new, Rc::downgrade)),
            loader: RefCell::new(None),
            task_tracker: RefCell::new(None),
            navigation: OnceCell::new(),
            history_length: Cell::new(1),
            history_index: Cell::new(0),
            previous_window: Default::default(),
            successor_window: Default::default(),
            previous_api: Default::default(),
            target,
            clock,
            initialized: Cell::new(false),
            loaded: Cell::new(false),
            focused_observers: RefCell::new(Vec::new()),
        });
        if let Some(domain) = owner
            .page
            .borrow()
            .upgrade()
            .and_then(|page| page.local_frames.get().cloned())
        {
            owner.SetSessionHistoryState(domain.session_history_state.get());
            if owner.IsRoot() {
                *domain.root_navigation.borrow_mut() = Rc::downgrade(&owner);
            }
        }
        let task_window = Rc::downgrade(web_api.WindowBindings());
        interaction.SetFormStateTaskRunner(Rc::new(move |task| {
            if let Some(window) = task_window.upgrade() {
                window
                    .borrow_mut()
                    .EnqueueDelayedNativeTask(Duration::ZERO, task);
            }
        }));
        let api = NavigationApi::new(owner.clone());
        assert!(owner.navigation.set(Rc::downgrade(&api)).is_ok());
        web_api.DOMBindings().borrow_mut().InstallNavigationApi(api);
        web_api.DOMBindings().borrow_mut().InstallHistory(
            interaction::default::core::frame::history::History::new(owner.clone()),
        );
        if let Some(domain) = owner
            .page
            .borrow()
            .upgrade()
            .and_then(|p| p.local_frames.get().cloned())
        {
            if let Some(host) = domain.AnchorNavigationHost(
                owner
                    .frame
                    .upgrade()
                    .map_or(domain.root, |f| f.state.Identity()),
            ) {
                interaction.Engine().SetAnchorNavigationHost(Some(host));
            }
        }
        document
            .borrow()
            .GetDocument()
            .InstallWindowNavigationApiOwner(owner.clone());
        owner
    }
    /// Non-regular DocumentLoader commits retain the previous Window only
    /// through source Navigation initialization. Entry clones receive the new
    /// native Window context; no Page/session-history vector is copied here.
    pub(super) fn SetPreviousWindow(&self, previous: &Rc<Self>) {
        self.history_length.set(previous.history_length.get());
        self.history_index.set(previous.history_index.get());
        *self.previous_window.borrow_mut() = Rc::downgrade(previous);
        *previous.successor_window.borrow_mut() = Rc::downgrade(&self.Retain());
        *self.previous_api.borrow_mut() = Some(previous.Api());
    }
    pub(super) fn AbortForWindowReuse(
        &self,
        runtime: &mut dyn javascript::javascript_runtime::JavaScriptHostRuntime,
    ) {
        let Some(bindings) = self.bindings.borrow().upgrade() else {
            return;
        };
        let api = self.Api();
        DOMJavaScriptBindings::RunNavigationInvocation(&bindings, runtime, &mut |invocation| {
            if api.OngoingEventIdentity().is_some() {
                api.AbortOngoingNavigation(invocation);
            }
        });
    }
    pub(super) fn Rebind(
        &self,
        page: &Rc<PageState>,
        document: &Rc<DocumentEngine>,
        web_api: &WebApiEngine,
        interaction: &Rc<crate::interaction_services::PageInteraction>,
        performance: Option<Rc<WindowPerformance>>,
    ) {
        assert!(
            self.window
                .upgrade()
                .is_some_and(|window| Rc::ptr_eq(&window, web_api.WindowBindings())),
            "ClearForReuse retains the Window"
        );
        if let Some(previous) = self.document.borrow().upgrade() {
            previous
                .borrow()
                .GetDocument()
                .ClearDocumentWindowNavigationApiAlias();
        }
        *self.page.borrow_mut() = Rc::downgrade(page);
        *self.document.borrow_mut() = Rc::downgrade(document);
        *self.bindings.borrow_mut() = Rc::downgrade(web_api.DOMBindings());
        *self.interaction.borrow_mut() = Rc::downgrade(interaction);
        *self.performance.borrow_mut() = performance.as_ref().map_or_else(Weak::new, Rc::downgrade);
        self.loader.borrow_mut().take();
        self.loaded.set(false);
        self.focused_observers.borrow_mut().clear();
        if let Some(domain) = page.local_frames.get() {
            if let Some(host) = domain.AnchorNavigationHost(
                self.frame
                    .upgrade()
                    .map_or(domain.root, |f| f.state.Identity()),
            ) {
                interaction.Engine().SetAnchorNavigationHost(Some(host));
            }
        }
        let window = self.window.clone();
        interaction.SetFormStateTaskRunner(Rc::new(move |task| {
            if let Some(window) = window.upgrade() {
                window
                    .borrow_mut()
                    .EnqueueDelayedNativeTask(Duration::ZERO, task);
            }
        }));
        document
            .borrow()
            .GetDocument()
            .InstallWindowNavigationApiOwner(self.Retain());
    }
    pub(super) fn InitializeCommitted(
        &self,
        loader: Rc<document_loader::DocumentLoaderNavigationState>,
        runtime: &mut dyn javascript::javascript_runtime::JavaScriptHostRuntime,
    ) {
        *self.loader.borrow_mut() = Some(loader.clone());
        let Some(item) = loader.CurrentHistoryItem() else {
            return;
        };
        self.loaded.set(true);
        let was_initialized = self.initialized.replace(true);
        let Some(bindings) = self.bindings.borrow().upgrade() else {
            return;
        };
        let performance = self
            .frame
            .upgrade()
            .and_then(|frame| frame.WindowPerformanceTiming())
            .or_else(|| {
                self.page
                    .borrow()
                    .upgrade()
                    .map(|page| page.window_performance_timing.clone())
            })
            .and_then(|timing| timing.ExistingEventTimingSource());
        if let Some(performance) = performance {
            *self.performance.borrow_mut() = Rc::downgrade(&performance);
            bindings
                .borrow()
                .BindNavigationTimeStampContext(performance);
        }
        let api = self.Api();
        let previous_api = self.previous_api.borrow_mut().take();
        let metadata = loader.CrossDocumentNavigationMetadata();
        if let Some(metadata) = metadata.as_ref().filter(|_| !was_initialized) {
            self.history_index.set(metadata.back_entries.len());
            self.history_length
                .set(metadata.back_entries.len() + 1 + metadata.forward_entries.len());
        } else if metadata.is_none()
            && !was_initialized
            && loader.LoadType() == Some(FrameLoadType::Standard)
            && previous_api
                .as_ref()
                .is_some_and(|previous| previous.CurrentEntry().is_some())
        {
            // In-process embedders can keep the Window's existing browser
            // length without transporting another copy of entry owners.
            let index = self.history_index.get() + 1;
            self.history_index.set(index);
            self.history_length.set(index + 1);
        }
        // Content's Navigation API entry exposure stops at the first
        // cross-origin session entry on either side of the current item.
        let back = metadata
            .as_ref()
            .map(|metadata| {
                let mut entries =
                    self.RestoreHistoryEntries(metadata.back_entries.iter().rev(), &item);
                entries.reverse();
                entries
            })
            .unwrap_or_default();
        let forward = metadata
            .as_ref()
            .map(|metadata| self.RestoreHistoryEntries(metadata.forward_entries.iter(), &item))
            .unwrap_or_default();
        let previous_item = self
            .previous_window
            .borrow()
            .upgrade()
            .and_then(|previous| {
                previous
                    .loader
                    .borrow()
                    .as_ref()
                    .and_then(|loader| loader.CurrentHistoryItem())
            });
        DOMJavaScriptBindings::RunNavigationInvocation(&bindings, runtime, &mut |invocation| {
            if was_initialized && api.CurrentEntry().is_some() {
                self.UpdateHistoryPosition(
                    &item,
                    loader
                        .LoadType()
                        .unwrap_or(FrameLoadType::ReplaceCurrentItem),
                );
                api.UpdateForNavigation(
                    invocation,
                    &item,
                    loader
                        .LoadType()
                        .unwrap_or(FrameLoadType::ReplaceCurrentItem),
                );
                return;
            }
            api.InitializeForNewWindow(
                invocation,
                item.clone(),
                loader.LoadType().unwrap_or(FrameLoadType::Standard),
                previous_api.is_none(),
                previous_api.as_deref(),
                &back,
                &forward,
                previous_item.clone(),
            );
        });
        self.PublishHistory(&item, loader.LoadType().unwrap_or(FrameLoadType::Standard));
    }
    fn PublishHistory(&self, item: &Rc<HistoryItem>, load: FrameLoadType) {
        if self.IsRoot() {
            if let Some(page) = self.page.borrow().upgrade() {
                if let Some(domain) = page.local_frames.get() {
                    domain.SetMirrorHistoryState(navigation_history::SessionHistoryState {
                        offset: self.history_index.get() as i32,
                        length: self.history_length.get() as i32,
                    });
                }
                let snapshot = page.SnapshotCurrentHistoryItem(item);
                page.client.borrow_mut().DidCommitHistory(snapshot, load);
            }
        } else if let Some(page) = self.page.borrow().upgrade() {
            if let Some(frame) = self.frame.upgrade() {
                frame.navigation.borrow_mut().pending_history_restore = None;
            }
            page.PublishChildHistory(load);
        }
    }
    fn RestoreHistoryEntries<'a>(
        &self,
        snapshots: impl Iterator<Item = &'a Arc<navigation_history::HistoryItemSnapshot>>,
        current: &HistoryItem,
    ) -> Vec<Rc<HistoryItem>> {
        let current_origin = RequestorOriginFor(
            self.Document()
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin(),
        );
        let mut previous_sequence = current.ItemSequenceNumber();
        let mut result = Vec::new();
        for snapshot in snapshots {
            if !snapshot.committed_origin.as_ref().map_or_else(
                || self.SameOrigin(&snapshot.url),
                |origin| *origin == current_origin,
            ) {
                break;
            }
            if snapshot.item_sequence_number == previous_sequence {
                continue;
            }
            result.push(HistoryItem::Restore(snapshot));
            previous_sequence = snapshot.item_sequence_number;
        }
        result
    }
    fn UpdateHistoryPosition(&self, item: &Rc<HistoryItem>, load: FrameLoadType) {
        if load == FrameLoadType::Standard {
            let index = self.history_index.get() + 1;
            self.history_index.set(index);
            self.history_length.set(index + 1);
        } else if load.IsTraversal() {
            let api = self.Api();
            if let (Some(current), Some(destination)) = (
                api.CurrentEntry(),
                api.Entries()
                    .into_iter()
                    .find(|entry| entry.Key() == item.NavigationApiKey()),
            ) {
                let index = self.history_index.get() as i64
                    + i64::from(destination.Index() - current.Index());
                self.history_index.set(index.max(0) as usize);
            }
        }
    }
    fn Retain(&self) -> Rc<Self> {
        self.self_
            .upgrade()
            .expect("live Window navigation environment")
    }
    fn Api(&self) -> Rc<NavigationApi> {
        self.navigation
            .get()
            .and_then(Weak::upgrade)
            .expect("Window owns Navigation")
    }
    fn Document(&self) -> Rc<DocumentEngine> {
        self.document
            .borrow()
            .upgrade()
            .expect("live Window Document")
    }
    fn Loader(&self) -> Rc<document_loader::DocumentLoaderNavigationState> {
        self.loader.borrow().clone().unwrap_or_else(|| {
            self.page
                .borrow()
                .upgrade()
                .expect("live Page")
                .DocumentLoaderNavigation()
        })
    }
    fn Live(&self) -> bool {
        self.document
            .borrow()
            .upgrade()
            .is_some_and(|document| document.borrow().GetDocument().IsActive())
            && self.window.upgrade().is_some()
    }
    fn URL(&self) -> String {
        self.Document()
            .borrow()
            .GetDocument()
            .DocumentURL()
            .to_owned()
    }
    fn Complete(&self, url: &str) -> Option<String> {
        self.Document()
            .borrow()
            .GetDocument()
            .BaseURL()
            .CompleteURL(url)
            .map(|url| url.into())
    }
    fn CanChange(&self, url: &str) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .CanChangeToURLForHistoryApi(url)
    }
    fn Active(&self) -> bool {
        self.interaction
            .borrow()
            .upgrade()
            .is_some_and(|interaction| interaction.Engine().HasTransientUserActivation())
    }
    fn IsRoot(&self) -> bool {
        self.frame.upgrade().is_none()
    }
    fn SameOrigin(&self, url: &str) -> bool {
        let document = self.Document();
        let document = document.borrow();
        let origin = document.GetDocument().SecurityContext().Origin();
        let destination = dom::persistent_document::SecurityOrigin::CreateForNavigation(
            url,
            None,
            Rc::new(origin.SchemeRegistryForRequests()),
        );
        origin.IsSameOriginWith(&destination)
    }
    fn Dispatch(
        &self,
        invocation: &dyn ScriptInvocation,
        target: &Rc<EventTarget>,
        event: EventRef,
    ) {
        target.DispatchEventInternalWithTarget(
            &event,
            target.AsEventTargetRef(),
            Some(invocation.EventListenerInvoker()),
        );
    }
    fn SimpleEvent(&self, name: &str) -> EventRef {
        struct Plain(NativeEvent);
        impl EventInterface for Plain {
            fn BaseEvent(&self) -> &NativeEvent {
                &self.0
            }
            fn BaseEventMut(&mut self) -> &mut NativeEvent {
                &mut self.0
            }
        }
        EventRef::new(Plain(NativeEvent::CreateWithType(
            name,
            self.clock.as_ref(),
        )))
    }
    fn CommitSameDocument(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
        intercepted: bool,
        fire_popstate: bool,
    ) {
        use document_loader::{
            SameDocumentCommitStage as Stage, SameDocumentNavigationFacts,
            SameDocumentNavigationParams, SameDocumentNavigationType,
        };
        let document = self.Document();
        let loader = self.Loader();
        // Freeze the outgoing entry before the loader changes its item. The
        // new entry retains the same live FormController owner below.
        if let Some(page) = self.page.borrow().upgrade() {
            page.SaveHistoryState();
        }
        let facts = {
            let owner = document.borrow();
            let doc = owner.GetDocument();
            SameDocumentNavigationFacts {
                document_url: doc.DocumentURL().to_owned(),
                is_on_initial_empty_document: !self.loaded.get(),
                was_loading: NavigateEventEnvironment::FrameIsLoading(self),
                has_transient_user_activation: self.Active(),
                document_origin: doc.SecurityContext().Origin().clone(),
            }
        };
        let mut commit = loader.RunURLAndHistoryUpdateSteps(
            SameDocumentNavigationParams {
                new_url: params.URL(),
                history_item: params.destination_item.clone(),
                navigation_type: if intercepted {
                    SameDocumentNavigationType::NavigationApiIntercept
                } else if params.event_type == NavigateEventType::HistoryApi {
                    SameDocumentNavigationType::HistoryApi
                } else {
                    SameDocumentNavigationType::Fragment
                },
                state: params
                    .destination_item
                    .as_ref()
                    .and_then(|item| item.StateObject())
                    .or_else(|| params.state_object.clone()),
                load_type: load,
                fire_popstate,
                initiator_origin: params.initiator_origin.clone(),
                is_browser_initiated: params.is_browser_initiated,
                is_synchronously_committed: params.is_synchronously_committed_same_document,
                has_ua_visual_transition: params.has_ua_visual_transition,
                should_skip_screenshot: params.should_skip_screenshot.get().unwrap_or(false),
                involvement: params.involvement,
                interaction_id: params
                    .interaction_id
                    .get()
                    .map(|id| [id.web_exposed_id, id.non_web_exposed_id]),
            },
            facts,
        );
        loop {
            match commit.Next(self.Live()) {
                Stage::EnqueueHashchange {
                    old_url,
                    new_url,
                    involvement,
                } => {
                    if let Some(page) = self.page.borrow().upgrade().filter(|_| self.IsRoot()) {
                        if let Some(owner) = page.mouse_autoscroll.borrow().upgrade() {
                            use interaction::scroll::core::input::scroll_manager::ScrollAutoscrollController;
                            owner.StopAutoscroll();
                        }
                    }
                    self.EnqueueHashchange(old_url, new_url, involvement);
                }
                Stage::UpdateDocument { url } => {
                    let form_state = document.borrow().GetDocument().FormElementsState();
                    commit.ApplyDocumentUpdate(document.borrow_mut().GetDocumentMut(), form_state);
                    if let Some(window) = self.window.upgrade() {
                        window.borrow_mut().SetURL(url.clone());
                    }
                    if let Some(frame) = self.frame.upgrade() {
                        frame.MarkNavigationDirty();
                    } else if let Some(page) = self.page.borrow().upgrade() {
                        if let Some(resources) = page.resources.borrow().as_ref() {
                            resources.SetDocumentURL(url);
                        }
                    }
                }
                Stage::UpdateNavigationApi { item, load_type } => {
                    if load_type.IsTraversal() && params.is_browser_initiated {
                        if let Some(domain) = self
                            .page
                            .borrow()
                            .upgrade()
                            .and_then(|page| page.local_frames.get().cloned())
                        {
                            self.SetSessionHistoryState(domain.session_history_state.get());
                        }
                    } else {
                        self.UpdateHistoryPosition(&item, load_type);
                    }
                    self.Api().UpdateForNavigation(invocation, &item, load_type);
                    self.PublishHistory(&item, load_type);
                }
                Stage::DispatchPopstate {
                    state,
                    has_ua_visual_transition,
                    involvement,
                } => {
                    let event =
                        interaction::dispatch::core::events::pop_state_event::PopStateEvent::Create(
                            state,
                            has_ua_visual_transition,
                            involvement,
                            self.clock.as_ref(),
                        );
                    let user_initiated = event.IsUserInitiated();
                    self.DispatchNavigationWindowEvent(
                        invocation,
                        EventRef::new(event),
                        user_initiated,
                    );
                }
                Stage::SoftNavigationCommitted {
                    old_url,
                    new_url,
                    load_type,
                    metrics_token,
                    interaction_id,
                } => {
                    if let Some(performance) = self.performance.borrow().upgrade() {
                        if let Some(heuristics) = performance.SoftNavigationHeuristics() {
                            heuristics.SameDocumentNavigationCommitted(
                                &old_url,
                                &new_url,
                                load_type,
                                metrics_token,
                                interaction_id.map(|id| PerformanceTimelineEntryIdInfo {
                                    web_exposed_id: id[0],
                                    non_web_exposed_id: id[1],
                                }),
                            );
                        }
                    }
                }
                Stage::DidCommitProvisionalLoad { .. } => {
                    if let Some(page) = self.page.borrow().upgrade() {
                        page.client.borrow_mut().DidCommit(&self.URL());
                    }
                }
                Stage::ProgressStarted => NavigateEventEnvironment::ProgressStarted(self),
                Stage::ProgressCompleted => NavigateEventEnvironment::ProgressCompleted(self),
                Stage::DidFinishSameDocumentNavigation(_) => {}
                Stage::Done => break,
            }
        }
    }
    fn DispatchNavigationWindowEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        event: EventRef,
        user_initiated: bool,
    ) {
        let Some(bindings) = self.bindings.borrow().upgrade() else {
            return;
        };
        let target = bindings.borrow().NavigationWindowEventTarget();
        let _timing = self.performance.borrow().upgrade().map(|performance| {
            NavigationEventTiming::new(performance, event.clone(), user_initiated)
        });
        self.Dispatch(invocation, &target, event);
    }
    // LocalDOMWindow.cc:1056-1063; EventTarget.cc:1163-1168. The native
    // Window task retains the event and lends its executing realm at delivery.
    fn EnqueueHashchange(
        &self,
        old_url: String,
        new_url: String,
        involvement: UserNavigationInvolvement,
    ) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let event = interaction::dispatch::core::events::hash_change_event::HashChangeEvent::Create(
            old_url,
            new_url,
            involvement,
            self.clock.as_ref(),
        );
        let user_initiated = event.IsUserInitiated();
        let event = EventRef::new(event);
        let environment = self.Retain();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                let _ = runtime.WithHostRuntime(realm, &mut |runtime| {
                    if !environment.Live() {
                        return;
                    }
                    let Some(bindings) = environment.bindings.borrow().upgrade() else {
                        return;
                    };
                    DOMJavaScriptBindings::RunNavigationInvocation(
                        &bindings,
                        runtime,
                        &mut |invocation| {
                            environment.DispatchNavigationWindowEvent(
                                invocation,
                                event.clone(),
                                user_initiated,
                            );
                        },
                    );
                });
            }),
            None,
        );
    }
    fn RequestCrossDocument(
        &self,
        request: Rc<FrameLoadRequest>,
        load: FrameLoadType,
        history_item: Option<Rc<HistoryItem>>,
    ) {
        let host = anchor_navigation::HostMetadata(&request, None);
        let (url, referrer, headers, method, encoded_body) = request.resource.Read(|resource| {
            (
                resource.url.clone(),
                resource.referrer.clone(),
                resource.headers.clone(),
                resource.method.clone(),
                resource.body.clone(),
            )
        });
        let referrer_policy = Some(history_item.as_ref().map_or_else(
            || {
                request
                    .resource
                    .Read(|resource| LoaderReferrerPolicy(resource.referrer_policy))
            },
            |item| LoaderReferrerPolicy(item.ReferrerPolicy()),
        ));
        let requestor_origin = history_item
            .as_ref()
            .and_then(|item| item.RequestorOrigin())
            .or_else(|| Some(request.initiator.origin.clone()));
        let mut request = URLRequest {
            url,
            referrer,
            headers,
            method,
            encoded_body,
            referrer_policy,
            navigation: Some(navigation_history::NavigationRequestMetadata {
                load_type: load,
                navigation_type: navigation_history::WebNavigationType::Determine(
                    load,
                    history_item
                        .as_ref()
                        .is_some_and(|item| item.FormData().is_some()),
                    false,
                ),
                is_browser_initiated: false,
                had_transient_user_activation: self.Active(),
                requestor_origin,
                propagated_text_fragment_token: None,
                internal_scroll_to_text_fragment: None,
                history_item: history_item.as_ref().map(|item| item.Snapshot()),
                back_entries: Arc::from([]),
                forward_entries: Arc::from([]),
                navigation_api_state: self
                    .Api()
                    .OngoingNavigationApiState()
                    .map(|state| state.StorageBytes()),
            }),
            ..Default::default()
        };
        if let Some(item) = &history_item {
            RestoreHistoryRequest(&mut request, &item.Snapshot());
        }
        if let Some(frame) = self.frame.upgrade() {
            frame.RequestScriptNavigationRequest(request, load);
        } else if let Some(page) = self.page.borrow().upgrade() {
            page.client
                .borrow_mut()
                .DidRequestNavigation(&NavigationRequest {
                    request,
                    target: "_self".into(),
                    replace_history: load != FrameLoadType::Standard,
                    host: Some(host),
                });
        }
    }
}
impl interaction::default::core::frame::history::HistoryEnvironment
    for NavigationWindowEnvironment
{
    fn IsFullyActive(&self) -> bool {
        self.Live()
    }
    fn CurrentHistoryItem(&self) -> Option<Rc<HistoryItem>> {
        self.Loader().CurrentHistoryItem()
    }
    fn BackForwardLength(&self) -> usize {
        self.history_length.get()
    }
    fn NotifyCurrentHistoryItemChanged(&self) {
        NavigationHistoryEntryWindow::NotifyCurrentHistoryItemChanged(self);
    }
    fn Go(&self, invocation: &dyn ScriptInvocation, delta: i32) {
        if delta == 0 {
            let request =
                NavigationApiEnvironment::CreateFrameLoadRequest(self, invocation, &self.URL());
            NavigationApiEnvironment::Navigate(self, invocation, request, FrameLoadType::Reload);
        } else {
            if let Some(page) = self.page.borrow().upgrade() {
                page.client.borrow_mut().DidRequestHistoryNavigation(delta);
            }
        }
    }
    fn URL(&self) -> String {
        NavigationWindowEnvironment::URL(self)
    }
    fn CompleteURL(&self, url: &str) -> Option<String> {
        self.Complete(url)
    }
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool {
        self.CanChange(url)
    }
    fn DispatchNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        params: Rc<NavigateEventDispatchParams>,
    ) -> interaction::default::core::html::anchor_element_utils::NavigateDispatchResult {
        self.Api().Dispatch(invocation, params)
    }
    fn RunURLAndHistoryUpdateSteps(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
    ) {
        self.CommitSameDocument(invocation, params, load, false, false);
    }
}
impl Page {
    pub fn TraverseToHistoryEntry(&self, key: &str) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.TraverseToHistoryEntry(key))
    }
}
impl dom::persistent_document::WindowNavigationApiOwner for NavigationWindowEnvironment {
    fn FocusedElementChanged(&self) {
        let observers = self
            .focused_observers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for observer in observers {
            observer.DidChangeFocus();
        }
    }
    fn WindowIdentity(&self) -> u64 {
        self.window.as_ptr() as usize as u64
    }
}
impl NavigationHistoryEntryWindow for NavigationWindowEnvironment {
    fn HasWindow(&self) -> bool {
        self.Live()
    }
    fn NavigationApi(&self) -> Rc<NavigationApi> {
        self.Api()
    }
    fn CurrentHistoryItem(&self) -> Rc<HistoryItem> {
        self.Loader()
            .CurrentHistoryItem()
            .expect("committed DocumentLoader history item")
    }
    fn NotifyCurrentHistoryItemChanged(&self) {
        if let Some(item) = self.Loader().CurrentHistoryItem() {
            self.PublishHistory(&item, FrameLoadType::ReplaceCurrentItem);
        }
    }
    fn EntryWindow(&self, identity: u64) -> Rc<dyn NavigationHistoryEntryWindow> {
        if identity == NavigationApiEnvironment::WindowIdentity(self) {
            return self.Retain();
        }
        let successor = self
            .successor_window
            .borrow()
            .upgrade()
            .expect("DocumentLoader supplied the destination Window for source entry cloning");
        assert_eq!(
            identity,
            NavigationApiEnvironment::WindowIdentity(successor.as_ref())
        );
        successor
    }
    fn CreateEntryEventTarget(&self) -> Rc<EventTarget> {
        self.bindings
            .borrow()
            .upgrade()
            .expect("live entry bindings")
            .borrow()
            .CreateNavigationEventTarget()
    }
}
impl NavigationApiEnvironment for NavigationWindowEnvironment {
    fn CompleteURL(&self, url: &str) -> Option<String> {
        self.Complete(url)
    }
    fn PageDismissalEventBeingDispatched(&self) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .PageDismissalEventBeingDispatched()
            != dom::document_parser_lifecycle::PageDismissalType::NoDismissal
    }
    fn IsOnInitialEmptyDocument(&self) -> bool {
        !self.loaded.get()
    }
    fn ShouldMaintainTrivialSessionHistory(&self) -> bool {
        false
    }
    fn CreateFrameLoadRequest(
        &self,
        invocation: &dyn ScriptInvocation,
        url: &str,
    ) -> Rc<FrameLoadRequest> {
        *self.task_tracker.borrow_mut() = invocation.TaskAttributionTracker();
        let document = self.Document();
        let owner = document.borrow();
        let doc = owner.GetDocument();
        FrameLoadRequest::new(
            &ResourceRequest::new(url),
            self.Retain(),
            NavigationInitiator {
                origin: RequestorOriginFor(doc.SecurityContext().Origin()),
                state_token: self.Loader().InitiatorStateToken(),
                document_token: Some(doc.Token()),
                world_id: invocation.WorldId(),
                source_location: invocation.CaptureStackSourceLocation(),
                script_tool_invocation_id: invocation.ScriptToolInvocationId(),
            },
        )
    }
    fn Navigate(
        &self,
        invocation: &dyn ScriptInvocation,
        request: Rc<FrameLoadRequest>,
        load: FrameLoadType,
    ) {
        if !self.Live() {
            return;
        }
        let url = request.resource.Read(|resource| resource.url.clone());
        let same_document = interaction::default::core::loader::frame_navigation_lifecycle::ShouldPerformFragmentNavigation(
            &request.resource.Read(|resource| resource.method.clone()), load, &url, &self.URL(), false,
            self.Document().borrow().GetDocument().IsFrameSet(),
        );
        let mut params = NavigateEventDispatchParams::new(
            url,
            if same_document {
                NavigateEventType::Fragment
            } else {
                NavigateEventType::CrossDocument
            },
            load,
        );
        params.initiator_origin = Some(request.InitiatorSecurityOriginOwner());
        *params.source_element.borrow_mut() = request.source_element.borrow().clone();
        let params = Rc::new(params);
        match self.Api().Dispatch(invocation, params.clone()) {
            interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Abort => {},
            interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Intercept => {},
            interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Continue => {
                if same_document { self.CommitSameDocument(invocation, &params, load, false, true); NavigateEventEnvironment::ProcessScrollForSameDocumentNavigation(self, &params, load, false); }
                else { self.RequestCrossDocument(request, load, load.IsReload().then(|| NavigationHistoryEntryWindow::CurrentHistoryItem(self))); }
            }
        }
    }
    fn NavigationStartTime(&self) -> TimeTicks {
        self.clock.Now()
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.IsRoot()
    }
    fn HasTransientUserActivation(&self) -> bool {
        self.Active()
    }
    fn AsyncSameDocumentNavigationStarted(&self, invocation: &dyn ScriptInvocation) -> Option<u64> {
        let tracker = invocation.TaskAttributionTracker()?;
        let id = tracker.AsyncSameDocumentNavigationStarted().map(u64::from);
        *self.task_tracker.borrow_mut() = Some(tracker);
        id
    }
    fn NavigateToNavigationApiKey(&self, request: NavigationTraverseRequest) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let environment = self.Retain();
        window.borrow_mut().EnqueueTaskWithRuntime(Box::new(move |runtime, realm| {
            let _ = runtime.WithHostRuntime(realm, &mut |host_runtime| {
            let Some(bindings) = environment.bindings.borrow().upgrade() else { return; };
            DOMJavaScriptBindings::RunNavigationInvocation(&bindings, host_runtime, &mut |invocation| {
                let destination = environment.Api().Entries().into_iter().find(|entry| entry.Key() == request.key);
                let Some(destination) = destination else { environment.Api().TraverseCancelled(invocation, &request.key, TraverseCancelledReason::NotFound); return; };
                let item = destination.AsAny().downcast_ref::<NavigationHistoryEntryOwner>().expect("native history entry").HistoryItem();
                let same_document = destination.SameDocument();
                let _task_scope = if same_document {
                    request.task_state_id.and_then(|id| environment.task_tracker.borrow().clone().and_then(|tracker| {
                        let continuation = tracker.CommitSameDocumentNavigation(id.try_into().expect("TaskAttributionId is u32"));
                        tracker.SetCurrentTaskStateIfTopLevel(continuation, invocation.IsInContext(), foundation::task_attribution::TaskScopeType::PopState)
                    }))
                } else { None };
                let mut params = NavigateEventDispatchParams::new(item.URL(), if same_document { NavigateEventType::Fragment } else { NavigateEventType::CrossDocument }, FrameLoadType::BackForward);
                params.destination_item = Some(item.clone()); params.is_synchronously_committed_same_document = false;
                let params = Rc::new(params);
                match environment.Api().Dispatch(invocation, params.clone()) {
                    interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Continue if same_document => environment.CommitSameDocument(invocation, &params, FrameLoadType::BackForward, false, true),
                    interaction::default::core::html::anchor_element_utils::NavigateDispatchResult::Continue => {
                        let navigation = NavigationApiEnvironment::CreateFrameLoadRequest(environment.as_ref(), invocation, &item.URL());
                        environment.RequestCrossDocument(navigation, FrameLoadType::BackForward, Some(item.clone()));
                    }
                    _ => {},
                }
            });
            });
        }), None);
    }
    fn CountNavigationNavigateJavaScriptURL(&self) {}
    fn EventTarget(&self) -> Rc<EventTarget> {
        self.target.clone()
    }
    fn NavigateEventHandlerPresenceChanged(&self, _present: bool) {}
    fn WindowIdentity(&self) -> u64 {
        self.window.as_ptr() as usize as u64
    }
    fn HasWindow(&self) -> bool {
        self.Live()
    }
    fn HasFrame(&self) -> bool {
        self.Live()
    }
    fn ContextDestroyed(&self) -> bool {
        !self.Live()
    }
    fn HasLoadedNonInitialEmptyDocument(&self) -> bool {
        self.loaded.get()
    }
    fn OriginIsOpaque(&self) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .IsOpaque()
    }
    fn ShouldAllowBindingAccess(&self, element: &EventTargetRef) -> bool {
        let Some(origin) = element.SecurityOriginOwner() else {
            return false;
        };
        foundation::security::CanAccessSecurityOrigin(
            self.Document()
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin()
                .as_ref(),
            origin.as_ref(),
        )
    }
    fn IsMainFrame(&self) -> bool {
        self.IsRoot()
    }
    fn IsHistoryUserActivationActive(&self) -> bool {
        self.interaction
            .borrow()
            .upgrade()
            .is_some_and(|interaction| interaction.Engine().IsHistoryUserActivationActive())
    }
    fn ConsumeHistoryUserActivation(&self) {
        if let Some(interaction) = self.interaction.borrow().upgrade() {
            interaction.Engine().ConsumeHistoryUserActivation();
        }
    }
    fn DispatchCurrentEntryChange(
        &self,
        invocation: &dyn ScriptInvocation,
        from: Option<Rc<dyn NavigationHistoryEntry>>,
        kind: Option<NavigationType>,
    ) {
        self.Dispatch(
            invocation,
            &self.target,
            EventRef::new(NavigationCurrentEntryChangeEvent {
                event: NativeEvent::CreateWithType("currententrychange", self.clock.as_ref()),
                from,
                navigation_type: kind,
            }),
        );
    }
    fn DispatchDispose(
        &self,
        invocation: &dyn ScriptInvocation,
        entry: &Rc<dyn NavigationHistoryEntry>,
    ) {
        if let Some(target) = entry.EventTarget() {
            self.Dispatch(invocation, &target, self.SimpleEvent("dispose"));
        }
    }
    fn MarkNavigationStateCommittedIfPresent(&self) {
        if let Some(state) = self
            .Document()
            .borrow()
            .GetDocument()
            .NavigationStateIfExists()
        {
            state.SetCommitted();
        }
    }
    fn CurrentURL(&self) -> String {
        self.URL()
    }
    fn SameOriginWithDestination(&self, url: &str) -> bool {
        self.SameOrigin(url)
    }
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool {
        self.CanChange(url)
    }
    fn EqualIgnoringFragment(&self, url: &str, current: &str) -> bool {
        EqualIgnoringFragment(url, current)
    }
    fn CurrentHistoryItem(&self) -> Rc<HistoryItem> {
        NavigationHistoryEntryWindow::CurrentHistoryItem(self)
    }
    fn SameOriginWithWindow(&self, window: u64) -> bool {
        if window == NavigationApiEnvironment::WindowIdentity(self) {
            return true;
        }
        let Some(previous) = self.previous_window.borrow().upgrade().filter(|previous| {
            NavigationApiEnvironment::WindowIdentity(previous.as_ref()) == window
        }) else {
            return false;
        };
        let (Some(current), Some(previous)) = (
            self.document.borrow().upgrade(),
            previous.document.borrow().upgrade(),
        ) else {
            return false;
        };
        let current = current.borrow();
        let previous = previous.borrow();
        current
            .GetDocument()
            .SecurityContext()
            .Origin()
            .IsSameOriginWith(previous.GetDocument().SecurityContext().Origin())
    }
    fn IsPrerendering(&self) -> bool {
        false
    }
    fn MakeEntryFromItem(&self, item: &Rc<HistoryItem>) -> Rc<dyn NavigationHistoryEntry> {
        NavigationHistoryEntryOwner::new(self.Retain(), item)
    }
    fn CreatePostFormData(
        &self,
        invocation: &dyn ScriptInvocation,
        source: &EventTargetRef,
    ) -> Option<Rc<interaction::dispatch::core::fileapi::form_data_object::FormDataObject>> {
        let document = self.Document();
        let node = source.NodeId()?;
        {
            let owner = document.borrow();
            let doc = owner.GetDocument();
            let source = doc.FindNodeById(node)?;
            if !doc.IsHTMLFormElement(source)
                || !doc
                    .Node(source)
                    .FindAttribute("method")
                    .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("post"))
            {
                return None;
            }
        }
        let interaction = self.interaction.borrow().upgrade()?;
        let listener_interaction = interaction.clone();
        let listeners = Rc::new(
            move |event: &interaction::event::EventListenerInvocation<'_>,
                  source_document: &interaction::ownership::InteractionDocument,
                  _: &interaction::ownership::InteractionDOMMutationEmitter| {
                let Some(bindings) = listener_interaction.BindingsForDocument(source_document)
                else {
                    return Default::default();
                };
                DOMJavaScriptBindings::DispatchNavigationDOMListeners(&bindings, event, invocation)
            },
        );
        let result = interaction
            .Engine()
            .WithScopedListeners(listeners)
            .ConstructFormEntryList(&document.Handle(), node, "UTF-8")
            .unwrap_or_else(|error| panic!("native FormData construction admission: {error:?}"));
        result
    }
    fn NavigateEventEnvironment(&self) -> Rc<dyn NavigateEventEnvironment> {
        self.Retain()
    }
    fn BeginNavigationEventTiming(
        &self,
        event: &Rc<dyn NavigateEvent>,
    ) -> (Box<dyn Any>, Option<PerformanceTimelineEntryIdInfo>) {
        let timing = if let Some(performance) = self.performance.borrow().upgrade() {
            let native = event
                .AsAny()
                .downcast_ref::<NavigateEventOwner>()
                .expect("native NavigateEvent");
            NavigationEventTiming::new(performance, event.EventRef(), native.data.user_initiated)
        } else {
            NavigationEventTiming::ForAbsentFrame()
        };
        let id = timing.GetInteractionIdInfo();
        (Box::new(timing), id)
    }
    fn NavigationSourcePseudoClassEnabled(&self) -> bool {
        true
    }
    fn StartNavigationState(
        &self,
        url: &str,
        source: Option<EventTargetRef>,
        direction: Option<NavigationDirection>,
    ) {
        self.Document().borrow().GetDocument().StartNavigationState(
            url.to_owned(),
            None,
            direction.map(|direction| match direction {
                NavigationDirection::Back => dom::persistent_document::NavigationTraverseType::Back,
                NavigationDirection::Forward => {
                    dom::persistent_document::NavigationTraverseType::Forward
                }
                NavigationDirection::Reload => {
                    dom::persistent_document::NavigationTraverseType::Reload
                }
            }),
        );
    }
    fn FinishNavigationStateIfPresent(&self) {
        self.Document()
            .borrow()
            .GetDocument()
            .FinishNavigationState();
    }
    fn SetPendingNavigationDestinationURL(&self, url: &str) {
        if let Some(performance) = self.performance.borrow().upgrade() {
            performance.SetPendingNavigationDestinationURL(url);
        }
    }
    fn CreateTransition(
        &self,
        _: &dyn ScriptInvocation,
        kind: NavigationType,
        current: Option<Rc<dyn NavigationHistoryEntry>>,
        event: &Rc<dyn NavigateEvent>,
    ) -> Rc<dyn NavigationTransition> {
        let event = event
            .AsAny()
            .downcast_ref::<NavigateEventOwner>()
            .expect("native NavigateEvent");
        NavigationTransitionOwner::new(kind, current, event.destination.clone())
    }
    fn SkeletonLoaderNavigateIfPresent(&self, url: &str) {
        if let Some(loader) = self
            .Document()
            .borrow()
            .GetDocument()
            .SkeletonLoaderIfExists()
        {
            loader.NavigateTo(url);
        }
    }
    fn SkeletonLoaderCancelIfPresent(&self) {
        if let Some(loader) = self
            .Document()
            .borrow()
            .GetDocument()
            .SkeletonLoaderIfExists()
        {
            loader.CancelNavigation();
        }
    }
    fn StartNavigationPreviewIfNeeded(&self) {
        let _ = self
            .Document()
            .borrow()
            .GetDocument()
            .GetViewTransitionsIfExists()
            .map(|supplement| supplement.StartNavigationPreviewIfNeeded());
    }
    fn AbortNavigationPreview(&self) {
        let _ = self
            .Document()
            .borrow()
            .GetDocument()
            .GetViewTransitionsIfExists()
            .map(|supplement| supplement.AbortNavigationPreview());
    }
    fn ResetSameDocumentNavigationTasksIfTrackerPresent(&self) {
        if let Some(tracker) = self.task_tracker.borrow().as_ref() {
            tracker.ResetSameDocumentNavigationTasks();
        }
    }
    fn FrameIsAttached(&self) -> bool {
        self.Live()
    }
    fn DispatchNavigateError(&self, invocation: &dyn ScriptInvocation, error: Rc<dyn ScriptValue>) {
        let message = invocation.ExceptionMessage(&error);
        let mut event =
            interaction::dispatch::core::events::error_event::ErrorEvent::CreateForScript(
                String::from_utf16_lossy(&message.message).into(),
                message.location.unwrap_or(
                    interaction::dispatch::bindings::script_invocation::ScriptSourceLocation {
                        url: String::new(),
                        line: 0,
                        column: 0,
                    },
                ),
                error,
                invocation.WorldId(),
                self.clock.as_ref(),
            );
        event.initEvent("navigateerror", false, false);
        self.Dispatch(invocation, &self.target, EventRef::new(event));
    }
    fn DispatchNavigateSuccess(&self, invocation: &dyn ScriptInvocation) {
        self.Dispatch(
            invocation,
            &self.target,
            self.SimpleEvent("navigatesuccess"),
        );
    }
    fn NavigateEventDeferCrossDocumentCommitEnabled(&self) -> bool {
        false
    }
}
fn EqualIgnoringFragment(url: &str, current: &str) -> bool {
    let (Ok(mut url), Ok(mut current)) = (url::Url::parse(url), url::Url::parse(current)) else {
        return false;
    };
    url.set_fragment(None);
    current.set_fragment(None);
    url == current
}
struct NavigationLoadTask {
    window: Weak<RefCell<WindowJavaScriptBindings>>,
    identity: u64,
}
impl NavigateEventTask for NavigationLoadTask {
    fn Cancel(&self) {
        if let Some(window) = self.window.upgrade() {
            window.borrow_mut().CancelNativeTask(self.identity);
        }
    }
}
impl NavigateEventEnvironment for NavigationWindowEnvironment {
    fn HasWindow(&self) -> bool {
        self.Live()
    }
    fn Clock(&self) -> &dyn EventClock {
        self.clock.as_ref()
    }
    fn AbortContext(&self) -> Rc<dyn AbortSignalContext> {
        self.bindings
            .borrow()
            .upgrade()
            .expect("live NavigateEvent bindings")
            .borrow()
            .NavigationAbortContext()
    }
    fn CurrentURL(&self) -> String {
        self.URL()
    }
    fn OriginString(&self) -> String {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .ToString()
    }
    fn CompleteURL(&self, url: &str) -> Option<String> {
        self.Complete(url)
    }
    fn CanChangeToURLForHistoryApi(&self, url: &str) -> bool {
        self.CanChange(url)
    }
    fn ShouldMaintainTrivialSessionHistory(&self) -> bool {
        false
    }
    fn CaptureCurrentTaskState(&self) -> Option<Rc<dyn Any>> {
        self.task_tracker
            .borrow()
            .as_ref()
            .and_then(|tracker| tracker.CurrentTaskState())
            .map(|state| state as Rc<dyn Any>)
    }
    fn EnterTaskAttributionScope(&self, override_current: bool) -> Box<dyn Any> {
        if let Some(tracker) = self.task_tracker.borrow().clone() {
            Box::new(tracker.OverrideTopLevelCheck(override_current))
        } else {
            Box::new(())
        }
    }
    fn ConsoleWarning(&self, message: String) {
        eprintln!("rechrom-navigation-warning: {message}");
    }
    fn AddFocusedElementChangeObserver(&self, event: Weak<NavigateEventOwner>) {
        self.focused_observers.borrow_mut().push(event);
    }
    fn RemoveFocusedElementChangeObserver(&self, event: usize) {
        self.focused_observers.borrow_mut().retain(|observer| {
            observer
                .upgrade()
                .is_some_and(|observer| observer.Identity() != event)
        });
    }
    fn AutofocusDelegate(&self) -> Option<u64> {
        self.interaction
            .borrow()
            .upgrade()?
            .Engine()
            .DocumentAutofocusDelegate(&self.Document().Handle())
    }
    fn FocusAutofocusDelegate(&self, invocation: &dyn ScriptInvocation, element: u64, _: bool) {
        self.ChangeFocus(invocation, element, true);
    }
    fn ClearFocusedElement(&self, invocation: &dyn ScriptInvocation) {
        let focused = self
            .interaction
            .borrow()
            .upgrade()
            .and_then(|interaction| interaction.State().borrow().focused_node_id);
        if let Some(focused) = focused {
            self.ChangeFocus(invocation, focused, false);
        }
    }
    fn ClearSequentialFocusNavigationStartingPoint(&self) {
        self.Document()
            .borrow_mut()
            .GetDocumentMut()
            .SetSequentialFocusNavigationStartingPoint(None);
    }
    fn PostDelayedLoadStart(
        &self,
        event: Weak<NavigateEventOwner>,
        delay: Duration,
    ) -> Rc<dyn NavigateEventTask> {
        let window = self.window.upgrade().expect("live navigation task queue");
        let identity = window.borrow_mut().EnqueueDelayedNativeTask(
            delay,
            Box::new(move || {
                if let Some(event) = event.upgrade() {
                    event.DelayedLoadStartTimerFired();
                }
            }),
        );
        Rc::new(NavigationLoadTask {
            window: self.window.clone(),
            identity,
        })
    }
    fn ProgressStarted(&self) {
        if let Some(frame) = self.frame.upgrade() {
            frame.state.SetIsLoading(true);
        }
    }
    fn ProgressCompleted(&self) {
        if let Some(frame) = self.frame.upgrade() {
            frame.state.SetIsLoading(false);
        }
    }
    fn FrameIsLoading(&self) -> bool {
        self.frame.upgrade().map_or_else(
            || {
                self.page
                    .borrow()
                    .upgrade()
                    .is_some_and(|page| page.DocumentLoaderLifecycle().IsLoading())
            },
            |frame| frame.state.IsLoading(),
        )
    }
    fn DidFailAsyncSameDocumentCommit(&self) {}
    fn WillNavigateEventCommitNow(&self, interaction: Option<PerformanceTimelineEntryIdInfo>) {
        if let (Some(performance), Some(interaction)) =
            (self.performance.borrow().upgrade(), interaction)
        {
            performance.WillNavigateEventCommitNow(interaction);
        }
    }
    fn RunURLAndHistoryUpdateSteps(
        &self,
        invocation: &dyn ScriptInvocation,
        params: &Rc<NavigateEventDispatchParams>,
        load: FrameLoadType,
        fire_popstate: bool,
    ) {
        self.CommitSameDocument(invocation, params, load, true, fire_popstate);
    }
    fn DidNavigationApiIntercept(&self) {}
    fn FrameIsAttached(&self) -> bool {
        self.Live()
    }
    fn DidFinishNavigation(&self, _success: bool) {
        if let Some(page) = self.page.borrow().upgrade() {
            page.dirty.set(true);
        }
    }
    fn AXHandleLoadStart(&self) {}
    fn AXHandleLoadComplete(&self) {}
    fn StartLoadingForAsyncNavigationApiCommit(&self) {
        NavigateEventEnvironment::ProgressStarted(self);
    }
    fn NavigateEventDeferCrossDocumentCommitEnabled(&self) -> bool {
        false
    }
    fn BindResumeDeferredCommitListener(&self) -> Rc<dyn NavigationResumeDeferredCommitListener> {
        unreachable!(
            "deferPageSwap is disabled until the embedding supplies the deferred commit pipe"
        )
    }
    fn TryCreateFragmentAnchor(&self, url: &str) -> bool {
        self.ScrollToFragment(url)
    }
    fn SetProgrammaticScrollOffsetToOrigin(&self) {
        self.ScrollTo([0., 0.]);
    }
    fn ProcessScrollForSameDocumentNavigation(
        &self,
        params: &Rc<NavigateEventDispatchParams>,
        _load: FrameLoadType,
        _instant: bool,
    ) {
        if let Some(state) = params
            .destination_item
            .as_ref()
            .and_then(|item| item.ViewState())
        {
            self.ScrollTo(state.scroll_offset);
        } else {
            self.ScrollToFragment(&params.URL());
        }
    }
}
impl NavigationWindowEnvironment {
    fn ChangeFocus(&self, invocation: &dyn ScriptInvocation, id: u64, focus: bool) {
        let Some(interaction) = self.interaction.borrow().upgrade() else {
            return;
        };
        let listener_interaction = interaction.clone();
        let listeners = Rc::new(
            move |event: &interaction::event::EventListenerInvocation<'_>,
                  source_document: &interaction::ownership::InteractionDocument,
                  _: &interaction::ownership::InteractionDOMMutationEmitter| {
                let Some(bindings) = listener_interaction.BindingsForDocument(source_document)
                else {
                    return Default::default();
                };
                DOMJavaScriptBindings::DispatchNavigationDOMListeners(&bindings, event, invocation)
            },
        );
        interaction
            .Engine()
            .WithScopedListeners(listeners)
            .DispatchTargeted(
                &interaction::input_event::InputEvent::Focus(
                    interaction::input_event::FocusEvent {
                        r#type: if focus {
                            interaction::input_event::FocusEventType::kFocus
                        } else {
                            interaction::input_event::FocusEventType::kBlur
                        },
                        target_node_id: id,
                        related_target_node_id: None,
                    },
                ),
                &self.Document().Handle(),
            );
    }
    fn ScrollTo(&self, offset: [f32; 2]) {
        if let Some(frame) = self.frame.upgrade() {
            if let Some(runtime) = frame.runtime.borrow().clone() {
                let root = {
                    let owner = runtime.document.borrow();
                    let doc = owner.GetDocument();
                    let Some(root) = doc.DocumentElement() else {
                        return;
                    };
                    doc.Node(root).Id()
                };
                frame.SetProgrammaticScrollOffset(
                    &runtime,
                    root,
                    Offset {
                        x: offset[0] as f64,
                        y: offset[1] as f64,
                    },
                    ViewportScrollBehavior::Instant,
                    compositor::ScrollSourceType::kAbsoluteScroll,
                );
            }
            return;
        }
        if self.IsRoot() {
            if let Some(page) = self.page.borrow().upgrade() {
                let root = {
                    let document = self.Document();
                    let owner = document.borrow();
                    let doc = owner.GetDocument();
                    let Some(root) = doc.DocumentElement() else {
                        return;
                    };
                    doc.Node(root).Id()
                };
                page.SetProgrammaticScrollOffset(
                    root,
                    Offset {
                        x: offset[0] as f64,
                        y: offset[1] as f64,
                    },
                    ViewportScrollBehavior::Instant,
                    compositor::ScrollSourceType::kAbsoluteScroll,
                );
            }
        }
    }
    fn ScrollToFragment(&self, url: &str) -> bool {
        let Some(fragment) = url::Url::parse(url)
            .ok()
            .and_then(|url| url.fragment().map(str::to_owned))
        else {
            return false;
        };
        let target = {
            let document = self.Document();
            let owner = document.borrow();
            let document = owner.GetDocument();
            document
                .FindFragmentAnchor(&fragment)
                .map(|node| (document.Node(node).Id(), node == document.Root()))
        };
        let Some((target, is_root)) = target else {
            return false;
        };
        if is_root {
            self.ScrollTo([0., 0.]);
            return true;
        }
        if let Some(frame) = self.frame.upgrade() {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                return false;
            };
            let result = runtime
                .layout
                .borrow_mut()
                .NativeScrollSource(target)
                .map(|mut source| source.ScrollFragmentAnchorIntoView());
            if let Some(result) = result {
                frame.ApplyNativeScrollResult(&runtime, result);
                return true;
            }
        } else if let Some(page) = self.page.borrow().upgrade() {
            page.EnsureLayoutOutputForNode(target);
            let result = page
                .layout_engine
                .borrow_mut()
                .NativeScrollSource(target)
                .map(|mut source| source.ScrollFragmentAnchorIntoView());
            if let Some(result) = result {
                page.ApplyNativeScrollResult(result);
                return true;
            }
        }
        false
    }
}
impl FrameLoadRequestEnvironment for NavigationWindowEnvironment {
    fn InitiatorSecurityOriginOwner(&self) -> Rc<dyn foundation::security::SecurityOriginOwner> {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone()
    }
    fn Now(&self) -> TimeTicks {
        self.clock.Now()
    }
    fn CurrentInputStartTime(&self) -> Option<TimeTicks> {
        None
    }
    fn BaseURL(&self) -> String {
        self.Document()
            .borrow()
            .GetDocument()
            .BaseURL()
            .AsStr()
            .to_owned()
    }
    fn OutgoingReferrer(&self) -> String {
        self.URL()
    }
    fn DocumentReferrerPolicy(&self) -> ReferrerPolicy {
        self.Loader()
            .CurrentHistoryItem()
            .map_or(ReferrerPolicy::Default, |item| item.ReferrerPolicy())
    }
    fn GenerateReferrer(&self, policy: ReferrerPolicy, url: &str, source: &str) -> String {
        let policy = LoaderReferrerPolicy(policy);
        policy.GenerateReferrer(source, url)
    }
    fn HasTransientUserActivation(&self) -> bool {
        self.Active()
    }
    fn RemoveDanglingMarkupInTargetEnabled(&self) -> bool {
        true
    }
    fn TargetBlankImpliesNoOpener(&self) -> bool {
        true
    }
    fn RelOpenerBcgDependencyHintEnabled(&self) -> bool {
        false
    }
    fn BlobURLSiteDiffersFromTopLevelSite(&self, url: &str) -> bool {
        FrameLoadRequestEnvironment::BlobURLSiteDiffersFromTopLevelSite(
            self.AnchorLoaderHost().as_ref(),
            url,
        )
    }
    fn EnforceNoopenerOnBlobURLNavigation(&self) -> bool {
        true
    }
    fn BlobURLPartitioningDisabled(&self) -> bool {
        FrameLoadRequestEnvironment::BlobURLPartitioningDisabled(self.AnchorLoaderHost().as_ref())
    }
    fn ResolveBlobURLToken(
        &self,
        url: &str,
        top_level: bool,
    ) -> Arc<interaction::dispatch::core::fileapi::blob_url_registry::BlobURLToken> {
        FrameLoadRequestEnvironment::ResolveBlobURLToken(
            self.AnchorLoaderHost().as_ref(),
            url,
            top_level,
        )
    }
    fn CountUse(&self, _feature: &'static str) {}
    fn ReportPartitioningBlobURLIssue(&self, _url: &str) {}
}

pub(super) fn RequestorOriginFor(
    origin: &dom::persistent_document::SecurityOrigin,
) -> RequestorOrigin {
    if let Some(tuple) = origin.Tuple() {
        RequestorOrigin::Tuple {
            scheme: tuple.scheme.clone(),
            host: tuple.host.clone(),
            port: tuple.port,
        }
    } else {
        RequestorOrigin::Opaque(origin.OpaqueNonce().expect("opaque origin identity"))
    }
}

fn LoaderReferrerPolicy(policy: ReferrerPolicy) -> url_loader::ReferrerPolicy {
    match policy {
        ReferrerPolicy::Default | ReferrerPolicy::StrictOriginWhenCrossOrigin => {
            url_loader::ReferrerPolicy::StrictOriginWhenCrossOrigin
        }
        ReferrerPolicy::Never => url_loader::ReferrerPolicy::NoReferrer,
        ReferrerPolicy::NoReferrerWhenDowngrade => {
            url_loader::ReferrerPolicy::NoReferrerWhenDowngrade
        }
        ReferrerPolicy::Origin => url_loader::ReferrerPolicy::Origin,
        ReferrerPolicy::OriginWhenCrossOrigin => url_loader::ReferrerPolicy::OriginWhenCrossOrigin,
        ReferrerPolicy::SameOrigin => url_loader::ReferrerPolicy::SameOrigin,
        ReferrerPolicy::StrictOrigin => url_loader::ReferrerPolicy::StrictOrigin,
        ReferrerPolicy::Always => url_loader::ReferrerPolicy::UnsafeUrl,
    }
}
