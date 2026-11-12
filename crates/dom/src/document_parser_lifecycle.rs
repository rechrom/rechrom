//! DocumentParser base lifecycle, shared with its original native parser.
//! Chromium document_parser.cc:40-92; document.cc:4020-4054,9775-9798;
//! document.h:2870,2892,2908. No parser tree/input/decoder is copied here.
use crate::persistent_document::{DOMOwnerHandle, PersistentDocument};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserState {
    Parsing,
    Stopping,
    Stopped,
    Detached,
}
pub trait DocumentParserClient {
    fn NotifyParserStopped(&self);
}
pub struct DocumentParserLifecycle {
    document: Cell<Option<DOMOwnerHandle>>,
    state: Cell<ParserState>,
    clients: RefCell<Vec<Rc<dyn DocumentParserClient>>>,
    source_line: RefCell<Option<foundation::parser_source_line::ParserSourceLine>>,
    waiting_for_scripts: Cell<bool>,
    executing_script: Cell<u32>,
}
impl DocumentParserLifecycle {
    pub fn new(document: DOMOwnerHandle) -> Rc<Self> {
        Rc::new(Self {
            document: Cell::new(Some(document)),
            state: Cell::new(ParserState::Parsing),
            clients: RefCell::new(Vec::new()),
            source_line: RefCell::new(None),
            waiting_for_scripts: Cell::new(false),
            executing_script: Cell::new(0),
        })
    }
    pub fn Document(&self) -> Option<DOMOwnerHandle> {
        self.document.get()
    }
    pub fn State(&self) -> ParserState {
        self.state.get()
    }
    pub fn IsParsing(&self) -> bool {
        self.state.get() == ParserState::Parsing
    }
    pub fn BindSourceLine(&self, line: foundation::parser_source_line::ParserSourceLine) {
        let mut source_line = self.source_line.borrow_mut();
        if let Some(old) = &*source_line {
            assert!(old.SameOwner(&line), "same parser input line owner");
        } else {
            *source_line = Some(line);
        }
    }
    pub fn IsScriptable(&self) -> bool {
        self.source_line.borrow().is_some()
    }
    pub fn IsWaitingForScripts(&self) -> bool {
        self.waiting_for_scripts.get()
    }
    pub fn SetWaitingForScripts(&self, waiting: bool) {
        self.waiting_for_scripts.set(waiting);
    }
    /// ScriptableDocumentParser.cc:41-43, capture_source_location.cc:67-88.
    pub fn IsParsingAtLineNumber(&self) -> bool {
        self.IsParsing() && !self.IsWaitingForScripts() && self.executing_script.get() == 0
    }
    pub fn LineNumber(&self) -> Option<u32> {
        self.source_line
            .borrow()
            .as_ref()
            .map(|line| (line.ZeroBasedLine() + 1) as u32)
    }
    pub fn EnterExecutingScript(self: &Rc<Self>) -> ParserScriptScope {
        self.executing_script.set(
            self.executing_script
                .get()
                .checked_add(1)
                .expect("parser script nesting"),
        );
        ParserScriptScope(self.clone())
    }
    pub fn IsStopped(&self) -> bool {
        matches!(
            self.state.get(),
            ParserState::Stopped | ParserState::Detached
        )
    }
    pub fn IsDetached(&self) -> bool {
        self.state.get() == ParserState::Detached
    }
    pub fn PrepareToStopParsing(&self) {
        assert!(self.IsParsing());
        self.state.set(ParserState::Stopping);
    }
    pub fn StopParsing(&self) {
        self.state.set(ParserState::Stopped);
        let clients = self.clients.borrow().clone();
        for client in clients {
            let present = self.clients.borrow().iter().any(|c| Rc::ptr_eq(c, &client));
            if present {
                client.NotifyParserStopped();
            }
        }
    }
    pub fn Detach(&self) {
        self.state.set(ParserState::Detached);
        self.document.set(None);
    }
    pub fn AddClient(&self, client: Rc<dyn DocumentParserClient>) {
        if !self.clients.borrow().iter().any(|c| Rc::ptr_eq(c, &client)) {
            self.clients.borrow_mut().push(client);
        }
    }
    pub fn RemoveClient(&self, client: &Rc<dyn DocumentParserClient>) {
        self.clients.borrow_mut().retain(|c| !Rc::ptr_eq(c, client));
    }
}
pub struct ParserScriptScope(Rc<DocumentParserLifecycle>);
impl Drop for ParserScriptScope {
    fn drop(&mut self) {
        self.0.executing_script.set(
            self.0
                .executing_script
                .get()
                .checked_sub(1)
                .expect("parser script nesting"),
        );
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DocumentReadyState {
    Loading,
    Interactive,
    #[default]
    Complete,
}
impl DocumentReadyState {
    pub fn Name(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Interactive => "interactive",
            Self::Complete => "complete",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DocumentParsingState {
    Parsing,
    InDOMContentLoaded,
    #[default]
    Finished,
}
#[derive(Clone, Copy, Debug)]
pub enum DocumentFrameLifecycleEffect {
    MaybeStopDeferringCommitsWithoutContentfulPaint,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocumentLoadEventProgress {
    NotRun,
    InProgress,
    #[default]
    Completed,
    BeforeUnloadEventInProgress,
    BeforeUnloadEventHandled,
    PageHideInProgress,
    UnloadVisibilityChangeInProgress,
    UnloadEventInProgress,
    UnloadEventHandled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageDismissalType {
    NoDismissal,
    BeforeUnload,
    PageHide,
    UnloadVisibilityChange,
    Unload,
}

/// A Document retains the original task handle. Implementations cancel that
/// precise Agent task; no task-runner/frame/service object crosses this port.
pub trait DocumentCancellableTask {
    fn IsActive(&self) -> bool;
    fn Cancel(&self);
}
#[derive(Clone)]
pub struct PendingJavaScriptURL {
    pub url: String,
    pub world: Option<u64>,
}
#[derive(Default)]
pub(crate) struct DocumentNavigationLifecycle {
    write_recursion_depth: Cell<u32>,
    frame_effects: RefCell<std::collections::VecDeque<DocumentFrameLifecycleEffect>>,
    parser: RefCell<Option<Rc<DocumentParserLifecycle>>>,
    ready: Cell<DocumentReadyState>,
    parsing: Cell<DocumentParsingState>,
    load: Cell<DocumentLoadEventProgress>,
    ignore_opens_and_writes_for_abort: Cell<bool>,
    ignore_opens_during_unload: Cell<u32>,
    subframe_loading_disabled_roots: RefCell<std::collections::HashMap<u64, u32>>,
    javascript_urls: RefCell<Vec<PendingJavaScriptURL>>,
    javascript_task: RefCell<Option<Rc<dyn DocumentCancellableTask>>>,
    http_refresh_task: RefCell<Option<Rc<dyn DocumentCancellableTask>>>,
    http_refresh: RefCell<Option<ScheduledHttpRefresh>>,
}
#[derive(Clone)]
pub struct ScheduledHttpRefresh {
    pub delay: std::time::Duration,
    pub url: String,
    pub from_header: bool,
    pub input_timestamp_micros: u64,
}
pub enum CancelParsingStage {
    RegisterParserAbortedStickyFeature,
    ReadyStateChanged,
    CancelledHttpRefresh,
    Done,
}
/// Document::CancelParsing's event boundary: consume stages synchronously and
/// release DOM/parser borrows for readystatechange listener reentry.
pub struct DocumentCancelParsing {
    lifecycle: Rc<DocumentNavigationLifecycle>,
    attribute_lifecycle: Rc<crate::attribute_lifecycle::AttributeLifecycleQueue>,
    step: u8,
    has_frame: bool,
}
impl DocumentCancelParsing {
    pub fn Next(&mut self) -> CancelParsingStage {
        loop {
            let step = self.step;
            self.step = self.step.saturating_add(1);
            match step {
                0 => {
                    let parsing = self
                        .lifecycle
                        .parser
                        .borrow()
                        .as_ref()
                        .is_some_and(|p| p.IsParsing());
                    if parsing {
                        self.lifecycle.ignore_opens_and_writes_for_abort.set(true);
                        if self.has_frame {
                            return CancelParsingStage::RegisterParserAbortedStickyFeature;
                        }
                    }
                }
                1 => {
                    if let Some(parser) = self.lifecycle.parser.borrow_mut().take() {
                        parser.Detach();
                    }
                    self.lifecycle
                        .SetParsingState(DocumentParsingState::Finished, &self.attribute_lifecycle);
                    if self.lifecycle.ready.replace(DocumentReadyState::Complete)
                        != DocumentReadyState::Complete
                    {
                        return CancelParsingStage::ReadyStateChanged;
                    }
                }
                2 => {
                    if self.lifecycle.load.get() < DocumentLoadEventProgress::Completed {
                        self.lifecycle
                            .load
                            .set(DocumentLoadEventProgress::Completed);
                    }
                    cancel_task(&self.lifecycle.javascript_task);
                    self.lifecycle.javascript_urls.borrow_mut().clear();
                    let was_active = self
                        .lifecycle
                        .http_refresh_task
                        .borrow()
                        .as_ref()
                        .is_some_and(|h| h.IsActive());
                    cancel_task(&self.lifecycle.http_refresh_task);
                    self.lifecycle.http_refresh.borrow_mut().take();
                    if was_active {
                        return CancelParsingStage::CancelledHttpRefresh;
                    }
                }
                _ => return CancelParsingStage::Done,
            }
        }
    }
}
fn cancel_task(task: &RefCell<Option<Rc<dyn DocumentCancellableTask>>>) {
    let task = task.borrow_mut().take();
    if let Some(task) = task {
        if task.IsActive() {
            task.Cancel();
        }
    }
}
impl PersistentDocument {
    /// Document.h:1690; Document::write cc:4847 uses a synchronous nesting
    /// scope. This reads the original Document state, not a parser snapshot.
    pub fn IsInDocumentWrite(&self) -> bool {
        self.navigation_lifecycle.write_recursion_depth.get() != 0
    }
    pub fn EnterDocumentWrite(&self) -> DocumentWriteScope {
        let depth = &self.navigation_lifecycle.write_recursion_depth;
        depth.set(depth.get().checked_add(1).expect("Document write nesting"));
        DocumentWriteScope(self.navigation_lifecycle.clone())
    }
    pub fn InstallDocumentParser(&self, parser: Rc<DocumentParserLifecycle>) {
        assert_eq!(parser.Document(), Some(self.RootHandle()));
        if let Some(old) = self
            .navigation_lifecycle
            .parser
            .borrow_mut()
            .replace(parser)
        {
            old.Detach();
        }
        self.navigation_lifecycle
            .parsing
            .set(DocumentParsingState::Parsing);
        self.navigation_lifecycle
            .ready
            .set(DocumentReadyState::Loading);
        self.navigation_lifecycle
            .load
            .set(DocumentLoadEventProgress::NotRun);
    }
    pub fn GetDocumentParser(&self) -> Option<Rc<DocumentParserLifecycle>> {
        self.navigation_lifecycle.parser.borrow().clone()
    }
    /// Document.cc4757-4776: reads the same load lifecycle progress owner.
    pub fn PageDismissalEventBeingDispatched(&self) -> PageDismissalType {
        match self.navigation_lifecycle.load.get() {
            DocumentLoadEventProgress::BeforeUnloadEventInProgress => {
                PageDismissalType::BeforeUnload
            }
            DocumentLoadEventProgress::PageHideInProgress => PageDismissalType::PageHide,
            DocumentLoadEventProgress::UnloadVisibilityChangeInProgress => {
                PageDismissalType::UnloadVisibilityChange
            }
            DocumentLoadEventProgress::UnloadEventInProgress => PageDismissalType::Unload,
            DocumentLoadEventProgress::NotRun
            | DocumentLoadEventProgress::InProgress
            | DocumentLoadEventProgress::Completed
            | DocumentLoadEventProgress::BeforeUnloadEventHandled
            | DocumentLoadEventProgress::UnloadEventHandled => PageDismissalType::NoDismissal,
        }
    }
    pub fn GetScriptableDocumentParser(&self) -> Option<Rc<DocumentParserLifecycle>> {
        self.GetDocumentParser()
            .filter(|parser| parser.IsScriptable())
    }
    pub fn DetachParser(&self) {
        if let Some(parser) = self.navigation_lifecycle.parser.borrow_mut().take() {
            parser.Detach();
        }
    }
    pub fn BeginCancelParsing(&self, has_frame: bool) -> DocumentCancelParsing {
        DocumentCancelParsing {
            lifecycle: self.navigation_lifecycle.clone(),
            attribute_lifecycle: self.AttributeLifecycle().clone(),
            step: 0,
            has_frame,
        }
    }
    pub fn IgnoreOpensAndWritesForAbort(&self) -> bool {
        self.navigation_lifecycle
            .ignore_opens_and_writes_for_abort
            .get()
    }
    pub fn ReadyState(&self) -> DocumentReadyState {
        self.navigation_lifecycle.ready.get()
    }
    pub fn SetReadyState(&self, state: DocumentReadyState) -> bool {
        self.navigation_lifecycle.ready.replace(state) != state
    }
    pub fn ParsingState(&self) -> DocumentParsingState {
        self.navigation_lifecycle.parsing.get()
    }
    pub fn TakeDocumentFrameLifecycleEffects(
        &self,
    ) -> std::collections::VecDeque<DocumentFrameLifecycleEffect> {
        self.navigation_lifecycle
            .frame_effects
            .borrow_mut()
            .drain(..)
            .collect()
    }
    pub fn SetParsingState(&self, state: DocumentParsingState) {
        let previous = self.ParsingState();
        self.navigation_lifecycle
            .SetParsingState(state, self.AttributeLifecycle().as_ref());
        if previous != DocumentParsingState::Finished && state == DocumentParsingState::Finished {
            // Document.cc:8078-8085, the normal parser completion route.
            // CancelParsing uses its private state transition and does not
            // manufacture Document::FinishedParsing's frame notification.
            self.navigation_lifecycle
                .frame_effects
                .borrow_mut()
                .push_back(
                    DocumentFrameLifecycleEffect::MaybeStopDeferringCommitsWithoutContentfulPaint,
                );
        }
    }
    pub fn LoadEventFinished(&self) -> bool {
        self.navigation_lifecycle.load.get() >= DocumentLoadEventProgress::Completed
    }
    pub fn SetLoadEventProgress(&self, progress: DocumentLoadEventProgress) {
        self.navigation_lifecycle.load.set(progress);
    }
    pub fn CancelPendingJavaScriptUrls(&self) {
        cancel_task(&self.navigation_lifecycle.javascript_task);
        self.navigation_lifecycle
            .javascript_urls
            .borrow_mut()
            .clear();
    }
    pub fn HasPendingJavaScriptUrls(&self) -> bool {
        !self
            .navigation_lifecycle
            .javascript_urls
            .borrow()
            .is_empty()
    }
    /// The typed JS URL producer posts a Networking task only when true is
    /// returned; it retains the task on this same Document with the next call.
    pub fn QueueJavaScriptUrl(
        &self,
        url: PendingJavaScriptURL,
        is_initial_empty_document: bool,
    ) -> bool {
        assert!(url
            .url
            .get(..11)
            .is_some_and(|s| s.eq_ignore_ascii_case("javascript:")));
        if is_initial_empty_document {
            self.navigation_lifecycle
                .load
                .set(DocumentLoadEventProgress::NotRun);
        }
        self.navigation_lifecycle
            .javascript_urls
            .borrow_mut()
            .push(url);
        !self
            .navigation_lifecycle
            .javascript_task
            .borrow()
            .as_ref()
            .is_some_and(|h| h.IsActive())
    }
    pub fn SetJavaScriptUrlTask(&self, task: Rc<dyn DocumentCancellableTask>) {
        *self.navigation_lifecycle.javascript_task.borrow_mut() = Some(task);
    }
    pub fn TakePendingJavaScriptUrls(&self) -> Vec<PendingJavaScriptURL> {
        self.navigation_lifecycle
            .javascript_task
            .borrow_mut()
            .take();
        std::mem::take(&mut *self.navigation_lifecycle.javascript_urls.borrow_mut())
    }
    pub fn IsHttpRefreshScheduledWithin(&self, interval: std::time::Duration) -> bool {
        self.navigation_lifecycle
            .http_refresh
            .borrow()
            .as_ref()
            .is_some_and(|r| r.delay <= interval)
    }
    /// HttpRefreshScheduler.cc:93-117. Scheduling policy stays here; the Agent
    /// stores only the chosen original delayed task via SetHttpRefreshTask.
    pub fn ScheduleHttpRefresh(
        &self,
        refresh: ScheduledHttpRefresh,
        navigation_allowed: bool,
    ) -> bool {
        if !navigation_allowed
            || refresh.delay > std::time::Duration::from_secs((i32::MAX / 1000) as u64)
            || refresh.url.is_empty()
        {
            return false;
        }
        if self
            .navigation_lifecycle
            .http_refresh
            .borrow()
            .as_ref()
            .is_some_and(|r| r.delay < refresh.delay)
        {
            return false;
        }
        self.CancelHttpRefresh();
        *self.navigation_lifecycle.http_refresh.borrow_mut() = Some(refresh);
        true
    }
    pub fn PendingHttpRefresh(&self) -> Option<ScheduledHttpRefresh> {
        self.navigation_lifecycle.http_refresh.borrow().clone()
    }
    pub fn ShouldStartHttpRefreshTimer(&self) -> bool {
        self.navigation_lifecycle.http_refresh.borrow().is_some()
            && self.LoadEventFinished()
            && !self
                .navigation_lifecycle
                .http_refresh_task
                .borrow()
                .as_ref()
                .is_some_and(|t| t.IsActive())
    }
    pub fn SetHttpRefreshTask(&self, task: Rc<dyn DocumentCancellableTask>) {
        *self.navigation_lifecycle.http_refresh_task.borrow_mut() = Some(task);
    }
    pub fn CancelHttpRefresh(&self) {
        cancel_task(&self.navigation_lifecycle.http_refresh_task);
        self.navigation_lifecycle.http_refresh.borrow_mut().take();
    }
    pub fn TakeHttpRefresh(&self) -> Option<ScheduledHttpRefresh> {
        self.navigation_lifecycle
            .http_refresh_task
            .borrow_mut()
            .take();
        self.navigation_lifecycle.http_refresh.borrow_mut().take()
    }
}
pub struct DocumentWriteScope(Rc<DocumentNavigationLifecycle>);
impl Drop for DocumentWriteScope {
    fn drop(&mut self) {
        let depth = &self.0.write_recursion_depth;
        depth.set(depth.get().checked_sub(1).expect("Document write nesting"));
    }
}

impl DocumentNavigationLifecycle {
    fn SetParsingState(
        &self,
        state: DocumentParsingState,
        lifecycle: &crate::attribute_lifecycle::AttributeLifecycleQueue,
    ) {
        let previous = self.parsing.replace(state);
        if previous != DocumentParsingState::Finished && state == DocumentParsingState::Finished {
            lifecycle.RecordFormStateDocumentFinishedParsing();
        }
    }
}

/// Document.cc:4641-4734. Each yielded operation is executed after releasing
/// the Document borrow; the next step rechecks its actual Window attachment.
#[derive(Clone, Copy)]
pub struct DocumentUnloadFacts {
    pub has_window: bool,
    pub dispatched_pagehide_persisted_and_still_hidden: bool,
    pub dispatched_pagehide_and_still_hidden: bool,
    pub page_visible: bool,
}
pub enum DocumentUnloadStage {
    StopParser(Rc<DocumentParserLifecycle>),
    EndFocusedInputEditing,
    PromoteNavigationDestinationAndDispatchPageHide,
    DispatchVisibilityChange,
    DispatchWebkitVisibilityChange,
    SaveScrollAnchor,
    DispatchUnload,
    Done,
}
pub struct DocumentUnload {
    lifecycle: Rc<DocumentNavigationLifecycle>,
    step: u8,
    page_visible: bool,
}
impl DocumentUnload {
    pub fn Next(&mut self, facts: DocumentUnloadFacts) -> DocumentUnloadStage {
        use DocumentLoadEventProgress as P;
        use DocumentUnloadStage as S;
        loop {
            let step = self.step;
            self.step = self.step.saturating_add(1);
            match step {
                0 => {
                    let parser = self.lifecycle.parser.borrow().clone();
                    if let Some(parser) = parser {
                        return S::StopParser(parser);
                    }
                }
                1 => {
                    let progress = self.lifecycle.load.get();
                    if progress == P::NotRun || progress > P::UnloadEventInProgress {
                        self.step = u8::MAX;
                        return S::Done;
                    }
                    return S::EndFocusedInputEditing;
                }
                2 => {
                    if self.lifecycle.load.get() >= P::PageHideInProgress
                        || facts.dispatched_pagehide_persisted_and_still_hidden
                    {
                        self.lifecycle.load.set(P::UnloadEventHandled);
                        self.step = u8::MAX;
                        return S::Done;
                    }
                    self.lifecycle.load.set(P::PageHideInProgress);
                    if facts.has_window && !facts.dispatched_pagehide_and_still_hidden {
                        return S::PromoteNavigationDestinationAndDispatchPageHide;
                    }
                }
                3 => {
                    if !facts.has_window {
                        self.step = u8::MAX;
                        return S::Done;
                    }
                    // Query before changing progress: IsPageVisible depends on it.
                    self.page_visible = facts.page_visible;
                    self.lifecycle.load.set(P::UnloadVisibilityChangeInProgress);
                    if self.page_visible {
                        return S::DispatchVisibilityChange;
                    }
                }
                4 => {
                    if self.page_visible {
                        return S::DispatchWebkitVisibilityChange;
                    }
                }
                5 => {
                    if !facts.has_window {
                        self.step = u8::MAX;
                        return S::Done;
                    }
                    return S::SaveScrollAnchor;
                }
                6 => {
                    self.lifecycle.load.set(P::UnloadEventInProgress);
                    return S::DispatchUnload;
                }
                7 => self.lifecycle.load.set(P::UnloadEventHandled),
                _ => return S::Done,
            }
        }
    }
}
impl PersistentDocument {
    pub fn BeginDispatchUnloadEvents(&self) -> DocumentUnload {
        DocumentUnload {
            lifecycle: self.navigation_lifecycle.clone(),
            step: 0,
            page_visible: false,
        }
    }
    pub fn LoadEventProgress(&self) -> DocumentLoadEventProgress {
        self.navigation_lifecycle.load.get()
    }
    /// Document.cc3315-3343: the canonical rendering lifecycle is the activity
    /// owner. Shutdown does not destroy LocalDOMWindow's ExecutionContext.
    pub fn BeginShutdown(&self) -> bool {
        if !self.IsActive() {
            return false;
        }
        assert!(
            self.HasDOMWindow(),
            "an active Document has its actual Window"
        );
        self.Lifecycle()
            .AdvanceTo(crate::document_lifecycle::LifecycleState::Stopping);
        true
    }
    /// Document.cc3425-3428,3474-3478, after actual Layout/Style/selection and
    /// Fetcher teardown. Window destruction/reuse is a separate source stage.
    pub fn FinishShutdown(&mut self) {
        assert_eq!(
            self.Lifecycle().GetState(),
            crate::document_lifecycle::LifecycleState::Stopping
        );
        self.Lifecycle()
            .AdvanceTo(crate::document_lifecycle::LifecycleState::Stopped);
        self.UnbindDOMWindow();
        self.ClearAnchorNavigationContextLifetime();
    }
}

/// FrameLoader.cc1451-1459 / Document.h:2375. Guards retain the original
/// Document's counters across unload callbacks and microtask reentry.
pub struct SubframeLoadingDisabler {
    lifecycle: Rc<DocumentNavigationLifecycle>,
    root: u64,
}
impl Drop for SubframeLoadingDisabler {
    fn drop(&mut self) {
        let mut roots = self.lifecycle.subframe_loading_disabled_roots.borrow_mut();
        let count = roots
            .get_mut(&self.root)
            .expect("original subframe loading root");
        *count -= 1;
        if *count == 0 {
            roots.remove(&self.root);
        }
    }
}
pub struct DocumentDetachScope {
    lifecycle: Rc<DocumentNavigationLifecycle>,
    _subframe_loading: SubframeLoadingDisabler,
}
impl Drop for DocumentDetachScope {
    fn drop(&mut self) {
        self.lifecycle.ignore_opens_during_unload.set(
            self.lifecycle
                .ignore_opens_during_unload
                .get()
                .checked_sub(1)
                .expect("balanced unload open guard"),
        );
    }
}
impl PersistentDocument {
    pub fn EnterSubframeLoadingDisabler(&self, root: usize) -> SubframeLoadingDisabler {
        let root = self.Node(root).Id();
        *self
            .navigation_lifecycle
            .subframe_loading_disabled_roots
            .borrow_mut()
            .entry(root)
            .or_default() += 1;
        SubframeLoadingDisabler {
            lifecycle: self.navigation_lifecycle.clone(),
            root,
        }
    }
    pub fn EnterDetachDocumentScope(&self) -> DocumentDetachScope {
        let lifecycle = self.navigation_lifecycle.clone();
        lifecycle.ignore_opens_during_unload.set(
            lifecycle
                .ignore_opens_during_unload
                .get()
                .checked_add(1)
                .expect("unload open guard"),
        );
        DocumentDetachScope {
            lifecycle,
            _subframe_loading: self.EnterSubframeLoadingDisabler(self.Root()),
        }
    }
    pub fn ShouldIgnoreOpensAndWrites(&self) -> bool {
        self.IgnoreOpensAndWritesForAbort()
            || self.navigation_lifecycle.ignore_opens_during_unload.get() != 0
    }
    pub fn CanLoadSubframe(&self, owner: usize) -> bool {
        let disabled = self
            .navigation_lifecycle
            .subframe_loading_disabled_roots
            .borrow();
        let mut next = Some(owner);
        while let Some(node) = next {
            if disabled.contains_key(&self.Node(node).Id()) {
                return false;
            }
            next = self.Node(node).Parent().or_else(|| self.ShadowHost(node));
        }
        true
    }
}
