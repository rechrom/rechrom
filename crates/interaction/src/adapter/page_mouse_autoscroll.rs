//! Canonical EventHandler autoscroll owner, shared by mouse and ScrollManager.
use interaction_default::core::page::autoscroll_controller::{
    AutoscrollController, AutoscrollControllerState, AutoscrollLayout, AutoscrollStopServices,
    SelectionAutoscrollFrame,
};
use interaction_scroll::core::input::scroll_manager::{ScrollAutoscrollController, ScrollFrame};
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum AutoscrollHostEffect {
    AutoscrollSelectionActive(bool),
    AutoscrollStart { position: (f32, f32) },
    AutoscrollFling { velocity: (f32, f32) },
    AutoscrollEnd,
}
pub struct PageMouseAutoscroll {
    state: Rc<AutoscrollControllerState>,
    stop: Rc<dyn AutoscrollStopServices>,
}
impl PageMouseAutoscroll {
    pub fn new(
        state: Rc<AutoscrollControllerState>,
        stop: Rc<dyn AutoscrollStopServices>,
    ) -> Rc<Self> {
        Rc::new(Self { state, stop })
    }
    pub fn StopAutoscrollWithNodeRelease(&self, release: &dyn Fn(u64)) {
        struct Stop<'a> {
            host: &'a dyn AutoscrollStopServices,
            release: &'a dyn Fn(u64),
        }
        impl AutoscrollStopServices for Stop<'_> {
            fn StopNodeAutoscroll(&self, layout: &dyn AutoscrollLayout) {
                if let Some(node) = layout.NodeId() {
                    (self.release)(node);
                }
            }
            fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool) {
                self.host.NotifyAutoscrollForSelectionInMainFrame(active);
            }
            fn SystemAutoscrollEnd(&self) {
                self.host.SystemAutoscrollEnd();
            }
            fn SetCursorOverridden(&self, value: bool) {
                self.host.SetCursorOverridden(value);
            }
            fn UpdateCursor(&self) {
                self.host.UpdateCursor();
            }
        }
        self.state.StopAutoscroll(&Stop {
            host: self.stop.as_ref(),
            release,
        });
    }
    pub fn NeedsAnimation(&self) -> bool {
        matches!(self.state.autoscroll_type.get(),interaction_default::core::page::autoscroll_controller::AutoscrollType::Selection|interaction_default::core::page::autoscroll_controller::AutoscrollType::DragAndDrop)
    }
    pub fn StopServices(&self) -> &dyn AutoscrollStopServices {
        self.stop.as_ref()
    }
    pub fn SelectionAutoscrollInProgress(&self) -> bool {
        self.state.SelectionAutoscrollInProgress()
    }
    pub fn StopSelectionAutoscrollIfNeeded(&self) {
        if self.state.SelectionAutoscrollInProgress() {
            self.state.StopAutoscroll(self.stop.as_ref());
        }
    }
    pub fn StartAutoscrollForSelection(
        &self,
        frame: &dyn SelectionAutoscrollFrame,
        layout: Rc<dyn AutoscrollLayout>,
    ) {
        AutoscrollController::WithState(frame, &self.state).StartAutoscrollForSelection(layout);
    }
    pub fn Animate(
        &self,
        frame: &dyn SelectionAutoscrollFrame,
        now: interaction_input::base::time::time::TimeTicks,
    ) {
        AutoscrollController::WithState(frame, &self.state).Animate(now);
    }
}
impl ScrollAutoscrollController for PageMouseAutoscroll {
    fn StopAutoscroll(&self) {
        self.state.StopAutoscroll(self.stop.as_ref());
    }
    fn StopAutoscrollWithNodeRelease(&self, release: &dyn Fn(u64)) {
        PageMouseAutoscroll::StopAutoscrollWithNodeRelease(self, release);
    }
    fn StopMiddleClickAutoscroll(&self, _: &Rc<dyn ScrollFrame>) {
        self.state.StopMiddleClickAutoscroll(self.stop.as_ref());
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.state.MiddleClickAutoscrollInProgress()
    }
}

/// MouseEventManager's complete autoscroll port borrows the current selection
/// transaction; the retained controller never owns its listener dispatcher.
pub struct PageMouseAutoscrollPort<'a> {
    pub owner: &'a PageMouseAutoscroll,
    pub frame: &'a dyn SelectionAutoscrollFrame,
}
impl crate::core::input::mouse_event_manager::MouseAutoscrollPort for PageMouseAutoscrollPort<'_> {
    fn SelectionAutoscrollInProgress(&self) -> bool {
        self.owner.SelectionAutoscrollInProgress()
    }
    fn StopAutoscroll(&self) {
        self.owner.state.StopAutoscroll(self.frame);
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.owner.MiddleClickAutoscrollInProgress()
    }
    fn HasAutoscrollController(&self) -> bool {
        true
    }
    fn StartAutoscrollForSelection(
        &self,
        layout: &crate::core::input::mouse_event_manager::MouseDragLayoutRef,
    ) {
        self.owner
            .StartAutoscrollForSelection(self.frame, layout.AutoscrollLayout());
    }
}
