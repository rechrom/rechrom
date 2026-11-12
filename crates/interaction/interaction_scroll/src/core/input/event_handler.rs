/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2012 Digia Plc. and/or its subsidiary(-ies)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/event_handler.h:80-85,102,109,142,144-145,149-153,170,233,278-280,456-458
// Source: third_party/blink/renderer/core/input/event_handler.cc:284,287-289,317,319,339,342,353-369,453-475,1646-1647,1671-1674,2414-2416
// This is EventHandler's scroll-owned slice. The Page-facing facade retains
// the remaining EventHandler state and calls the two clear stages at their
// original positions among pointer/gesture/mouse clearing.

use std::rc::Rc;

use foundation::gfx::PointF;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;

use super::mouse_wheel_event_manager::{
    MouseWheelEventManager, MouseWheelEventManagerContext, WebInputEventResult, WheelFrame,
    WheelTargetRoute,
};
use super::scroll_manager::{
    ScrollAutoscrollController, ScrollDirection, ScrollGranularity, ScrollLayoutBoxRef,
    ScrollManager, ScrollNodeRef,
};

// Source: event_handler.cc:365. This retains the layout object's own frame;
// the frame owner is independent of the EventHandler's execution placement.
pub trait AutoscrollFrame {}
pub type AutoscrollFrameRef = Rc<dyn AutoscrollFrame>;

// Source: event_handler.cc:355,365. The query borrows the resident LayoutObject
// identity; it does not expose Layout storage or trigger its lifecycle.
pub trait AutoscrollLayoutObject {
    fn IsBox(&self) -> bool;
    fn Frame(&self) -> AutoscrollFrameRef;
}

// Source: event_handler.cc:361-367. Finding an autoscrollable LayoutBox remains
// the owning Layout algorithm. The operation below invokes the already-owned
// controller with that same box and frame; it contains no target policy.
pub trait MiddleClickAutoscrollOperations {
    fn FindAutoscrollable(
        &self,
        layout_object: &dyn AutoscrollLayoutObject,
        is_middle_click_autoscroll: bool,
    ) -> Option<ScrollLayoutBoxRef>;
    fn StartMiddleClickAutoscroll(
        &self,
        controller: &Rc<dyn ScrollAutoscrollController>,
        frame: AutoscrollFrameRef,
        scrollable: Option<ScrollLayoutBoxRef>,
        position_in_root_frame: PointF,
        position_in_screen: PointF,
    );
}

// Source: event_handler.cc:366-368,465,468-475. These four values/operations
// project the existing MouseEventManager owner; no mouse state is copied here.
pub trait EventHandlerMouseScrollState {
    fn MousePressNode(&self) -> Option<ScrollNodeRef>;
    fn LastKnownMousePositionInViewport(&self) -> PointF;
    fn LastKnownMouseScreenPosition(&self) -> PointF;
    fn InvalidateClick(&self);
}

// Source: event_handler.cc:468-470. This pure conversion borrows the existing
// VisualViewport's current transform. It performs no rendering lifecycle.
pub trait EventHandlerViewportCoordinates {
    fn ViewportToRootFrame(&self, point: PointF) -> PointF;
}

// Source: event_handler.h:456-458; event_handler.cc:284,287-289,317,319.
// The wheel manager is constructed from the exact same ScrollManager Rc that
// the EventHandler retains. Rc/Drop express the two source traced Members.
pub struct EventHandlerScrollRouting {
    scroll_manager_: Rc<ScrollManager>,
    mouse_wheel_event_manager_: MouseWheelEventManager<ScrollManager>,
    mouse_: Rc<dyn EventHandlerMouseScrollState>,
    viewport_: Rc<dyn EventHandlerViewportCoordinates>,
}

impl EventHandlerScrollRouting {
    // Source: event_handler.h:80-83; event_handler.cc:284,287-289.
    // The facade constructs the source ScrollManager first, then passes its
    // retained owner here while assembling the remaining event managers.
    pub fn new(
        scroll_manager: Rc<ScrollManager>,
        wheel_frame: Rc<dyn WheelFrame>,
        wheel_context: MouseWheelEventManagerContext,
        mouse: Rc<dyn EventHandlerMouseScrollState>,
        viewport: Rc<dyn EventHandlerViewportCoordinates>,
    ) -> Self {
        let mouse_wheel_event_manager =
            MouseWheelEventManager::new(wheel_frame, Rc::clone(&scroll_manager), wheel_context);
        Self {
            scroll_manager_: scroll_manager,
            mouse_wheel_event_manager_: mouse_wheel_event_manager,
            mouse_: mouse,
            viewport_: viewport,
        }
    }

    // Source: event_handler.h:85; event_handler.cc:339.
    // Called after PointerEventManager::Clear in the facade's source Clear.
    pub fn ClearScrollManager(&self) {
        self.scroll_manager_.Clear();
    }

    // Source: event_handler.h:85; event_handler.cc:342.
    // Called after GestureManager::Clear and MouseEventManager::Clear.
    pub fn ClearMouseWheelEventManager(&self) {
        self.mouse_wheel_event_manager_.Clear();
    }

    // Source: event_handler.h:102; event_handler.cc:353-369.
    pub fn StartMiddleClickAutoscroll(
        &self,
        layout_object: &dyn AutoscrollLayoutObject,
        middle_click_autoscroll_enabled: bool,
        operations: &dyn MiddleClickAutoscrollOperations,
    ) {
        debug_assert!(middle_click_autoscroll_enabled);
        if !layout_object.IsBox() {
            return;
        }
        let Some(controller) = self.scroll_manager_.GetAutoscrollController() else {
            return;
        };
        let scrollable = operations.FindAutoscrollable(layout_object, true);
        operations.StartMiddleClickAutoscroll(
            &controller,
            layout_object.Frame(),
            scrollable,
            self.LastKnownMousePositionInRootFrame(),
            self.mouse_.LastKnownMouseScreenPosition(),
        );
        self.mouse_.InvalidateClick();
    }

    // Source: event_handler.h:109; event_handler.cc:453-456.
    pub fn StopAutoscroll(&self) {
        self.scroll_manager_.StopMiddleClickAutoscroll();
        self.scroll_manager_.StopAutoscroll();
    }

    // Source: event_handler.h:149-153; event_handler.cc:458-466.
    pub fn BubblingScroll(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
        starting_node: Option<ScrollNodeRef>,
    ) -> bool {
        // Source: scroll_manager.h:82-86 defaults scrolling_via_key to false.
        self.scroll_manager_.BubblingScroll(
            direction,
            granularity,
            starting_node,
            self.mouse_.MousePressNode(),
            false,
        )
    }

    // Source: event_handler.h:144; event_handler.cc:468-471.
    pub fn LastKnownMousePositionInRootFrame(&self) -> PointF {
        self.viewport_
            .ViewportToRootFrame(self.mouse_.LastKnownMousePositionInViewport())
    }

    // Source: event_handler.h:145; event_handler.cc:473-475.
    pub fn LastKnownMouseScreenPosition(&self) -> PointF {
        self.mouse_.LastKnownMouseScreenPosition()
    }

    // Source: event_handler.h:233; event_handler.cc:1646-1647.
    // The facade applies the source's pointer-manager removal branch first.
    pub fn WheelElementRemoved(&self, target: Option<&EventTargetRef>) {
        if let Some(target) = target {
            self.mouse_wheel_event_manager_.ElementRemoved(target);
        }
    }

    // Source: event_handler.h:170; event_handler.cc:1671-1674.
    // The manager owns DOM cancellation, subframe target routing and scrollbar
    // fade order. EventHandler returns its result without a second decision.
    pub fn HandleWheelEvent(&self, event: &WebMouseWheelEvent) -> WebInputEventResult {
        self.mouse_wheel_event_manager_.HandleWheelEvent(event)
    }

    // Source: event_handler.h:142; event_handler.cc:2414-2416.
    pub fn ResizeScrollableAreaDestroyed(&self) {
        self.scroll_manager_.ClearResizeScrollableArea(true);
    }

    // Source: event_handler.h:278-280.
    pub fn GetMouseWheelEventManager(&self) -> &MouseWheelEventManager<ScrollManager> {
        &self.mouse_wheel_event_manager_
    }
}

// Source: mouse_wheel_event_manager.cc:81-85,167-171,180-184.
// A frame route reaches precisely these existing EventHandler operations.
impl WheelTargetRoute for EventHandlerScrollRouting {
    fn HandleWheelEvent(&self, event: &WebMouseWheelEvent) -> WebInputEventResult {
        EventHandlerScrollRouting::HandleWheelEvent(self, event)
    }

    fn ClearWheelTarget(&self) {
        self.GetMouseWheelEventManager().Clear();
    }
}
