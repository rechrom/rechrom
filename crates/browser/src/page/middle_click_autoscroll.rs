//! ChromeClient/LocalFrame leaves for the source middle-click controller.
//! Source autoscroll_controller.cc:357-456,539-613; no controller state lives here.
use super::local_frame::LocalFrameDomain;
use super::*;
use interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect;
use interaction::default::core::page::autoscroll_controller::{
    AutoscrollFrame, AutoscrollStopServices, PanningCursor,
};

pub(super) struct MiddleClickFrame {
    domain: Weak<LocalFrameDomain>,
    token: u64,
}
impl MiddleClickFrame {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, token: u64) -> Rc<Self> {
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            token,
        })
    }
    fn Page(&self) -> Rc<PageState> {
        self.domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .expect("live middle-click Page")
    }
    fn Stop(&self) -> super::mouse_autoscroll::PageAutoscrollStop {
        super::mouse_autoscroll::PageAutoscrollStop(Rc::downgrade(&self.Page()))
    }
}
impl AutoscrollStopServices for MiddleClickFrame {
    fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool) {
        self.Stop().NotifyAutoscrollForSelectionInMainFrame(active);
    }
    fn SystemAutoscrollEnd(&self) {
        self.Stop().SystemAutoscrollEnd();
    }
    fn SetCursorOverridden(&self, value: bool) {
        self.Stop().SetCursorOverridden(value);
    }
    fn UpdateCursor(&self) {
        self.Stop().UpdateCursor();
    }
}
impl AutoscrollFrame for MiddleClickFrame {
    fn VisualViewportMaximumScrollOffset(&self) -> (f64, f64) {
        let page = self.Page();
        let Some(facts) = page.PageScaleViewportFacts() else {
            return (0.0, 0.0);
        };
        let maximum = page.web_view_zoom.borrow().MaximumScrollOffset(&facts);
        (maximum.x() as f64, maximum.y() as f64)
    }
    fn DevicePixelRatio(&self) -> f32 {
        let domain = self.domain.upgrade().expect("live Frame domain");
        if self.token == domain.root {
            return self.Page().constraints.borrow().device_pixel_ratio as f32;
        }
        let frame = domain.Frame(self.token).expect("attached child Frame");
        let runtime = frame.runtime.borrow().clone().expect("child Document");
        let dpr = runtime.constraints.borrow().device_pixel_ratio as f32;
        dpr
    }
    fn IsLocalRoot(&self) -> bool {
        self.domain
            .upgrade()
            .is_some_and(|domain| self.token == domain.root)
    }
    fn HasView(&self) -> bool {
        self.domain.upgrade().is_some_and(|domain| {
            if self.token != domain.root && domain.Frame(self.token).is_none() {
                return false;
            }
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, self.token)
                .View()
                .is_some()
        })
    }
    fn SetPanningCursor(&self, cursor: PanningCursor) {
        use interaction::core::input::event_handler_cursor::CursorType as C;
        let kind = match cursor {
            PanningCursor::NorthEast => C::kNorthEastPanning,
            PanningCursor::NorthWest => C::kNorthWestPanning,
            PanningCursor::North => C::kNorthPanning,
            PanningCursor::SouthEast => C::kSouthEastPanning,
            PanningCursor::SouthWest => C::kSouthWestPanning,
            PanningCursor::South => C::kSouthPanning,
            PanningCursor::East => C::kEastPanning,
            PanningCursor::West => C::kWestPanning,
            PanningCursor::Vertical => C::kMiddlePanningVertical,
            PanningCursor::Horizontal => C::kMiddlePanningHorizontal,
            PanningCursor::Middle => C::kMiddlePanning,
        };
        let page = self.Page();
        let cursor = interaction::Cursor::Standard(kind);
        *page.native_event_handler_host.cursor.borrow_mut() = cursor.clone();
        page.client.borrow_mut().DidChangeCursor(cursor);
    }
    fn SystemAutoscrollStart(
        &self,
        position: interaction::input::public::common::input::web_pointer_properties::PointF,
    ) {
        self.Page()
            .client
            .borrow_mut()
            .DidAutoscroll(AutoscrollHostEffect::AutoscrollStart {
                position: (position.x(), position.y()),
            });
    }
    fn SystemAutoscrollFling(&self, velocity: (f32, f32)) {
        self.Page()
            .client
            .borrow_mut()
            .DidAutoscroll(AutoscrollHostEffect::AutoscrollFling { velocity });
    }
}

impl LocalFrameDomain {
    pub(super) fn StopMiddleClickAutoscrollForFrame(&self, token: u64) {
        let Some(frame) = self.Frame(token) else {
            return;
        };
        let Some(root) = self.FocusInteraction(self.root) else {
            return;
        };
        let owners = root.SourceInputOwners();
        if owners
            .autoscroll
            .MiddleClickAutoscrollUsesFrame(Rc::as_ptr(&frame) as usize)
        {
            if let Some(service) = owners.autoscroll_services.borrow().as_ref().cloned() {
                owners
                    .autoscroll
                    .StopMiddleClickAutoscroll(service.StopServices());
            }
        }
    }
}
