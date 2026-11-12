//! LocalFrame navigation guards, Frame's versioned form task, and the
//! FrameLoader ClientNavigationState. Chromium local_frame.h:496,1046-1047;
//! local_frame.cc:2043,2541-2548; frame.cc:639-668;
//! frame_loader.cc:1376-1381,1835-1854.
use super::{form_submission::FormSubmissionOwner, frame_load_request::FrameLoadRequest};
use crate::core::html::forms::html_form_element::CancelSubmission;
use navigation_history::FrameLoadType;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
/// Borrowed current-dispatch capability. CancelParsing releases the Document
/// borrow before invoking this source lifecycle event in the original realm.
pub trait FrameLifecycleEventDispatcher {
    fn DispatchReadyStateChanged(&self);
}
pub trait CancellableNavigationTask {
    fn IsActive(&self) -> bool;
    fn Cancel(&self);
}
pub trait FormSubmissionTaskRunner {
    fn PostDOMManipulationTask(&self, task: Box<dyn FnOnce()>)
        -> Rc<dyn CancellableNavigationTask>;
}
struct ClientNavigationState {
    url: String,
}
#[derive(Clone)]
pub struct ClientNavigationIdentity(Rc<ClientNavigationState>);
impl PartialEq for ClientNavigationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ClientNavigationIdentity {}
#[derive(Default)]
pub struct FrameNavigationLifecycle {
    navigation_disable_count: Cell<u32>,
    form_task: RefCell<Option<Rc<dyn CancellableNavigationTask>>>,
    form_version: Cell<u64>,
    client_navigation: RefCell<Option<Rc<ClientNavigationState>>>,
    parser_aborted_sticky_feature: Cell<bool>,
}
pub struct FrameNavigationDisabler {
    frame: Rc<FrameNavigationLifecycle>,
}
impl FrameNavigationDisabler {
    pub fn new(frame: Rc<FrameNavigationLifecycle>) -> Self {
        frame.navigation_disable_count.set(
            frame
                .navigation_disable_count
                .get()
                .checked_add(1)
                .expect("source navigation guard"),
        );
        Self { frame }
    }
}
impl Drop for FrameNavigationDisabler {
    fn drop(&mut self) {
        self.frame.navigation_disable_count.set(
            self.frame
                .navigation_disable_count
                .get()
                .checked_sub(1)
                .expect("balanced navigation guard"),
        );
    }
}
impl FrameNavigationLifecycle {
    pub fn IsNavigationAllowed(&self) -> bool {
        self.navigation_disable_count.get() == 0
    }
    pub fn RegisterParserAbortedStickyFeature(&self) {
        self.parser_aborted_sticky_feature.set(true);
    }
    pub fn ParserAbortedBlocksBackForwardCache(&self) -> bool {
        self.parser_aborted_sticky_feature.get()
    }
    pub fn BeginClientNavigation(&self, url: String) {
        *self.client_navigation.borrow_mut() = Some(Rc::new(ClientNavigationState { url }));
    }
    pub fn ClientNavigationURL(&self) -> Option<String> {
        self.client_navigation
            .borrow()
            .as_ref()
            .map(|state| state.url.clone())
    }
    pub fn ClientNavigationIdentity(&self) -> Option<ClientNavigationIdentity> {
        self.client_navigation
            .borrow()
            .as_ref()
            .map(|state| ClientNavigationIdentity(state.clone()))
    }
    pub fn ClearClientNavigation(&self) -> bool {
        self.client_navigation.borrow_mut().take().is_some()
    }
    pub fn ScheduleFormSubmission(
        self: &Rc<Self>,
        runner: &dyn FormSubmissionTaskRunner,
        submission: Rc<FormSubmissionOwner>,
    ) -> CancelSubmission {
        submission.NotifyInspector();
        self.ScheduleFormSubmissionTask(runner, Box::new(move || submission.Navigate()))
    }
    /// Frame::ScheduleFormSubmission's task/version ownership is shared by
    /// embeddings whose immutable request representation crosses a package
    /// boundary. The callback navigates that same frozen submission.
    pub fn ScheduleFormSubmissionTask(
        self: &Rc<Self>,
        runner: &dyn FormSubmissionTaskRunner,
        navigate: Box<dyn FnOnce()>,
    ) -> CancelSubmission {
        self.ScheduleFormSubmissionTaskWithToken(runner, None, navigate)
    }
    pub fn ScheduleFormSubmissionTaskWithToken(
        self: &Rc<Self>,
        runner: &dyn FormSubmissionTaskRunner,
        token: Option<crate::core::html::forms::html_form_element::FormSubmissionTaskToken>,
        navigate: Box<dyn FnOnce()>,
    ) -> CancelSubmission {
        let task = Self::PostFormSubmissionTask(runner, token, navigate);
        let old = self.form_task.borrow_mut().replace(task);
        if let Some(old) = old {
            old.Cancel();
        }
        let version = self
            .form_version
            .get()
            .checked_add(1)
            .expect("form task version");
        self.form_version.set(version);
        let frame: Weak<Self> = Rc::downgrade(self);
        Box::new(move || {
            if let Some(frame) = frame.upgrade() {
                frame.CancelFormSubmissionWithVersion(version);
            }
        })
    }
    /// A new auxiliary context has no target Frame until its host creates it.
    /// Its originating form token supplies cancellation without replacing an
    /// unrelated pending task on the source frame.
    pub fn PostFormSubmissionTask(
        runner: &dyn FormSubmissionTaskRunner,
        token: Option<crate::core::html::forms::html_form_element::FormSubmissionTaskToken>,
        navigate: Box<dyn FnOnce()>,
    ) -> Rc<dyn CancellableNavigationTask> {
        runner.PostDOMManipulationTask(Box::new(move || {
            if token.as_ref().is_none_or(|token| token.IsCurrent()) {
                navigate();
            }
        }))
    }
    pub fn CancelFormSubmission(&self) {
        let task = self.form_task.borrow_mut().take();
        if let Some(task) = task {
            task.Cancel();
        }
    }
    pub fn CancelFormSubmissionWithVersion(&self, version: u64) {
        if self.form_version.get() == version {
            self.CancelFormSubmission();
        }
    }
    pub fn IsFormSubmissionPending(&self) -> bool {
        self.form_task
            .borrow()
            .as_ref()
            .is_some_and(|task| task.IsActive())
    }
}
impl Drop for FrameNavigationLifecycle {
    fn drop(&mut self) {
        if let Some(task) = self.form_task.get_mut().take() {
            task.Cancel();
        }
    }
}
/// local_frame.cc:646-699 with the source Form() branch (same-URL replacement
/// is unconditionally excluded for forms at cc:620-623).
pub fn NavigationShouldReplaceFormHistory(
    request: &FrameLoadRequest,
    load: FrameLoadType,
    trivial_session_history: bool,
    origin_window_is_target: bool,
    load_event_finished: bool,
    has_transient_user_activation: bool,
) -> bool {
    if load != FrameLoadType::Standard {
        return false;
    }
    if request.force_history_push.get() {
        assert!(!trivial_session_history);
        return false;
    }
    if trivial_session_history {
        return true;
    }
    if !origin_window_is_target {
        return false;
    }
    if load_event_finished || has_transient_user_activation {
        return false;
    }
    true
}

/// FrameLoader::ShouldPerformFragmentNavigation, frame_loader.cc:1636-1653.
/// Facts are read from the original Frame/Document before navigate dispatch.
pub fn ShouldPerformFragmentNavigation(
    http_method: &str,
    load: FrameLoadType,
    url: &str,
    current_url: &str,
    is_provisional: bool,
    is_frameset: bool,
) -> bool {
    if !http_method.eq_ignore_ascii_case("GET")
        || load.IsReload()
        || load.IsTraversal()
        || is_provisional
        || is_frameset
    {
        return false;
    }
    let (Ok(mut destination), Ok(mut current)) =
        (url::Url::parse(url), url::Url::parse(current_url))
    else {
        return false;
    };
    if destination.fragment().is_none() {
        return false;
    }
    destination.set_fragment(None);
    current.set_fragment(None);
    destination == current
}
