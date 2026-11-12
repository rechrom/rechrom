// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/navigation_api/navigate_event_dispatch_params.h:28-73,
// .cc:15-32; public/web/web_frame_load_type.h:38-45.
use super::navigation_api::{HistoryItem, SerializedScriptValue};
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigateEventType {
    Fragment,
    HistoryApi,
    CrossDocument,
    PrerenderNoVarySearchActivation,
}
pub use crate::core::timing::performance_timeline_entry_id_generator::PerformanceTimelineEntryIdInfo;
pub use navigation_history::FrameLoadType;
pub use navigation_history::UserNavigationInvolvement;
pub trait NavigationResumeDeferredCommitListener {
    /// Immutable receiver endpoint transported with the original navigation request.
    fn EndpointId(&self) -> u64;
    fn ResumeDeferredCommit(&self);
}
pub struct NavigateEventDispatchParams {
    url: RefCell<String>,
    pub event_type: NavigateEventType,
    pub frame_load_type: FrameLoadType,
    pub involvement: UserNavigationInvolvement,
    pub source_element: RefCell<Option<EventTargetRef>>,
    pub initiator_origin: Option<Rc<dyn foundation::security::SecurityOriginOwner>>,
    pub state_object: Option<Rc<dyn SerializedScriptValue>>,
    pub destination_item: Option<Rc<HistoryItem>>,
    pub is_browser_initiated: bool,
    pub has_ua_visual_transition: bool,
    /// The cppgc allocation payload is zeroed before the source constructor
    /// (V8 cppgc-internal/memory.h:85-87; memory.cc:23-51). Fragment/history
    /// producers may override it; cross-document intercept starts at false.
    pub should_skip_screenshot: Cell<Option<bool>>,
    pub is_synchronously_committed_same_document: bool,
    pub download_filename: Option<String>,
    pub interaction_id: Cell<Option<PerformanceTimelineEntryIdInfo>>,
    /// Native pipe/promise owners are retained as their original identities.
    pub resume_deferred_commit_listener:
        RefCell<Option<Rc<dyn NavigationResumeDeferredCommitListener>>>,
}
impl NavigateEventDispatchParams {
    pub fn URL(&self) -> String {
        self.url.borrow().clone()
    }
    pub fn SetURL(&self, url: String) {
        *self.url.borrow_mut() = url;
    }
    pub fn new(url: String, event_type: NavigateEventType, frame_load_type: FrameLoadType) -> Self {
        Self {
            url: RefCell::new(url),
            event_type,
            frame_load_type,
            involvement: UserNavigationInvolvement::None,
            source_element: RefCell::new(None),
            initiator_origin: None,
            state_object: None,
            destination_item: None,
            is_browser_initiated: false,
            has_ua_visual_transition: false,
            should_skip_screenshot: Cell::new(Some(false)),
            is_synchronously_committed_same_document: true,
            download_filename: None,
            interaction_id: Cell::new(None),
            resume_deferred_commit_listener: RefCell::new(None),
        }
    }
}
