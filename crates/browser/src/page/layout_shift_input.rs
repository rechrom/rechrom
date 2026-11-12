//! LocalFrameView's retained LayoutShiftTracker input owner and its document
//! kInternalDefault timer. Chromium layout_shift_tracker.cc:708-788,900-909.
use super::{local_frame::LocalFrameDomain, native_event_handler_host, popup_tasks, PageState};
use foundation::popup_types::PopupTaskId;
use interaction::{
    default::core::layout::layout_shift_tracker::{LayoutShiftInputTimer, LayoutShiftInputTracker},
    input::public::common::input::web_input_event::WebInputEventPayload,
};
use std::{
    cell::Cell,
    rc::{Rc, Weak},
    time::Duration,
};

pub(super) struct PageLayoutShiftInputTracker {
    domain: Weak<LocalFrameDomain>,
    frame: u64,
    document: dom::persistent_document::DOMOwnerHandle,
    tracker: LayoutShiftInputTracker,
}

impl PageLayoutShiftInputTracker {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, frame: u64) -> Rc<Self> {
        let page = domain.page.upgrade().expect("LayoutShiftTracker Page");
        let document = domain
            .Document(frame)
            .expect("LayoutShiftTracker Document")
            .borrow()
            .GetDocument()
            .RootHandle();
        let timer = Rc::new(PageLayoutShiftInputTimer {
            page: Rc::downgrade(&page),
            root_document: page.document.borrow().GetDocument().RootHandle(),
            active: Cell::new(None),
        });
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            frame,
            document,
            // TimestampBasedCLSTracking is experimental in Chromium. Preserve
            // its default document-task timer branch for normal Page input.
            tracker: LayoutShiftInputTracker::new(page.InteractionEventClock(), timer, false),
        })
    }
}

impl native_event_handler_host::LayoutShiftInputTracker for PageLayoutShiftInputTracker {
    fn NotifyInput(&self, event: &dyn WebInputEventPayload) {
        assert!(
            self.CurrentDocumentMatches(),
            "LayoutShiftTracker belongs to current Document"
        );
        self.tracker.NotifyInput(event);
    }

    fn CurrentDocumentMatches(&self) -> bool {
        self.domain
            .upgrade()
            .and_then(|domain| domain.Document(self.frame))
            .is_some_and(|document| document.borrow().GetDocument().RootHandle() == self.document)
    }
}

struct PageLayoutShiftInputTimer {
    page: Weak<PageState>,
    root_document: dom::persistent_document::DOMOwnerHandle,
    active: Cell<Option<PopupTaskId>>,
}

impl LayoutShiftInputTimer for PageLayoutShiftInputTimer {
    fn StartOneShot(&self, delay: Duration) {
        self.Stop();
        let page = self
            .page
            .upgrade()
            .expect("attached LayoutShiftTracker task runner");
        let id = page.popup_tasks.Post(
            self.root_document.clone(),
            delay.as_secs_f64(),
            popup_tasks::PageInteractionTask::LayoutShiftInputTimer,
        );
        self.active.set(Some(id));
    }
    fn IsActive(&self) -> bool {
        self.active.get().is_some_and(|id| {
            self.page
                .upgrade()
                .is_some_and(|page| page.popup_tasks.IsScheduled(id))
        })
    }
    fn Stop(&self) {
        if let Some(id) = self.active.take() {
            if let Some(page) = self.page.upgrade() {
                page.popup_tasks.Cancel(id);
            }
        }
    }
}
impl Drop for PageLayoutShiftInputTimer {
    fn drop(&mut self) {
        self.Stop();
    }
}
