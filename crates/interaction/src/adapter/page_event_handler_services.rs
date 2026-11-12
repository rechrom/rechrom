//! Timer, wheel and scroll leaves for the source EventHandler binding.
//! Chromium: event_handler.cc:273-302,327-342,971-983,1646-1647,2384-2403.
use std::{rc::Rc, time::Duration};

use foundation::gfx;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_scroll::core::input::{
    event_handler::EventHandlerScrollRouting,
    scroll_manager::{
        ResizePaintLayer, ResizeScrollableArea, ResizeScrollableAreaRef, ScrollManager,
    },
};

use super::{
    page_event_handler_hit_test::PageEventHandlerHitResult,
    page_event_handler_node::PageEventHandlerNodes,
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    page_scroll_manager::PageScrollManagerOwners,
};
use crate::core::input::{
    event_handler::{
        EventHandlerHitResult, EventHandlerRegistry, EventHandlerScrollPort, EventHandlerTimer,
        EventHandlerWheel, NodeRef,
    },
    pointer_event_manager::{PointerResizeArea, PointerResizeTransform},
};

use crate::core::input::event_handler::EventHandlerState;
pub use crate::core::input::event_handler::{EventHandlerTimerKind, EventHandlerTimerTask};

/// The Page task-runner owns tokens, cancellation, document-generation checks
/// and callback delivery. The adapter owns neither a clock nor another timer.
pub trait PageEventHandlerTimerServices {
    fn Stop(&self, timer: EventHandlerTimerKind);
    fn IsActive(&self, timer: EventHandlerTimerKind) -> bool;
    fn StartOneShot(&self, task: EventHandlerTimerTask, delay: Duration);
}
pub struct PageEventHandlerTimer {
    services: Rc<dyn PageEventHandlerTimerServices>,
    kind: EventHandlerTimerKind,
    state: Rc<EventHandlerState>,
    frame_identity: Rc<()>,
}
impl PageEventHandlerTimer {
    pub fn new(
        services: Rc<dyn PageEventHandlerTimerServices>,
        kind: EventHandlerTimerKind,
        state: Rc<EventHandlerState>,
        frame_identity: Rc<()>,
    ) -> Rc<Self> {
        Rc::new(Self {
            services,
            kind,
            state,
            frame_identity,
        })
    }
}
impl EventHandlerTimer for PageEventHandlerTimer {
    fn Stop(&self) {
        self.state.InvalidateTimer(self.kind);
        self.services.Stop(self.kind);
    }
    fn IsActive(&self) -> bool {
        self.services.IsActive(self.kind)
    }
    fn StartOneShotMilliseconds(&self, milliseconds: u32) {
        self.services.StartOneShot(
            self.state
                .NextTimerTask(self.frame_identity.clone(), self.kind),
            Duration::from_millis(milliseconds.into()),
        );
    }
    fn StartOneShotMicroseconds(&self, microseconds: u64) {
        self.services.StartOneShot(
            self.state
                .NextTimerTask(self.frame_identity.clone(), self.kind),
            Duration::from_micros(microseconds),
        );
    }
}

// The translated registry is an identity-only port; listener and path state
// remain in the existing PageDispatchState, rather than another registry.
impl EventHandlerRegistry for super::page_event_dispatcher::PageDispatchState {}

pub struct PageEventHandlerWheel {
    routing: Rc<EventHandlerScrollRouting>,
    nodes: Rc<PageEventHandlerNodes>,
}
impl PageEventHandlerWheel {
    pub fn new(
        routing: Rc<EventHandlerScrollRouting>,
        nodes: Rc<PageEventHandlerNodes>,
    ) -> Rc<Self> {
        Rc::new(Self { routing, nodes })
    }
}
impl EventHandlerWheel for PageEventHandlerWheel {
    fn Clear(&self) {
        self.routing.ClearMouseWheelEventManager();
    }
    fn ElementRemoved(&self, target: &NodeRef) {
        let target = self
            .nodes
            .EventTarget(target)
            .expect("wheel removal retains canonical target identity");
        self.routing.WheelElementRemoved(Some(&target));
    }
}

pub trait PageEventHandlerResizeCoordinates {
    fn ConvertFromRootFrame(&self, point: gfx::Point) -> gfx::Point;
}

pub struct PageEventHandlerScroll<'scope> {
    manager: Rc<ScrollManager>,
    controls: Rc<PagePointerScrollControlOwners>,
    coordinates: Rc<dyn PageEventHandlerResizeCoordinates + 'scope>,
}
impl<'scope> PageEventHandlerScroll<'scope> {
    pub fn new(
        owners: &PageScrollManagerOwners,
        controls: Rc<PagePointerScrollControlOwners>,
        coordinates: Rc<dyn PageEventHandlerResizeCoordinates + 'scope>,
    ) -> Rc<Self> {
        Rc::new(Self {
            manager: owners.ScrollManager(),
            controls,
            coordinates,
        })
    }
}
struct ScrollResizeArea(Rc<dyn PointerResizeArea>);
struct ScrollResizeTransform(gfx::Transform);
impl PointerResizeTransform for ScrollResizeTransform {
    fn Transform(&self) -> &gfx::Transform {
        &self.0
    }
}
impl ResizeScrollableArea for ScrollResizeArea {
    fn InResizeMode(&self) -> bool {
        self.0.InResizeMode()
    }
    fn SetInResizeMode(&self, value: bool) {
        self.0.SetInResizeMode(value);
    }
    fn Resize(&self, position: gfx::Point, transform: &gfx::Transform) {
        self.0.Resize(
            gfx::PointF::new(position.x() as f32, position.y() as f32),
            &ScrollResizeTransform(transform.clone()),
        );
    }
    fn InitializeResizeTransform(&self, position: gfx::Point) -> gfx::Transform {
        self.0
            .InitializeResizeTransform(gfx::PointF::new(position.x() as f32, position.y() as f32))
            .Transform()
            .clone()
    }
}
struct ScrollResizeLayer(ResizeScrollableAreaRef);
impl ResizePaintLayer for ScrollResizeLayer {
    fn ScrollableArea(&self) -> Option<ResizeScrollableAreaRef> {
        Some(self.0.clone())
    }
}
impl EventHandlerScrollPort<PageEventHandlerHitResult> for PageEventHandlerScroll<'_> {
    fn Clear(&self) {
        self.manager.Clear();
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.manager.MiddleClickAutoscrollInProgress()
    }
    fn StopMiddleClickAutoscroll(&self) {
        self.manager.StopMiddleClickAutoscroll();
    }
    fn InResizeMode(&self) -> bool {
        self.manager.InResizeMode()
    }
    fn Resize(&self, event: &WebMouseEvent) {
        self.manager.Resize(event);
    }
    fn TrySetResizeScrollableArea(
        &self,
        result: &PageEventHandlerHitResult,
        event: &WebMouseEvent,
    ) -> bool {
        let Some(node) = result.InnerNode() else {
            return false;
        };
        let Some(area) = self
            .controls
            .EnclosingLayerScrollableArea(node.DomNodeId() as u64)
        else {
            return false;
        };
        let position = event.PositionInRootFrame();
        let position = self.coordinates.ConvertFromRootFrame(gfx::Point::new(
            position.x().floor() as i32,
            position.y().floor() as i32,
        ));
        if !area.IsAbsolutePointInResizeControl(gfx::PointF::new(
            position.x() as f32,
            position.y() as f32,
        )) {
            return false;
        }
        self.manager.SetResizeScrollableArea(
            &ScrollResizeLayer(Rc::new(ScrollResizeArea(area))),
            position,
        );
        true
    }
    fn ClearResizeScrollableArea(&self, destroyed: bool) {
        self.manager.ClearResizeScrollableArea(destroyed);
    }
}
