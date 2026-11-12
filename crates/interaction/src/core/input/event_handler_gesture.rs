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
// Source: third_party/blink/renderer/core/input/event_handler.cc:1689-1749,1782-2182
// The LocalFrame, Document, clock and geometry ports name the original owners;
// all routing, target selection and lifecycle ordering below remain source logic.
use super::*;
use crate::core::input::{
    gesture_manager::GestureEventWithHitTestResults,
    touch_adjustment::{self, TouchAdjustmentCandidateType},
};
use foundation::{gfx, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize};
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_input_event::Modifiers,
    web_pointer_properties::PointerType,
};

type Frame<'scope, R> = Rc<dyn EventHandlerFrame<'scope, R> + 'scope>;
pub struct UnboundedGestureTarget<'scope, R: EventHandlerHitResult> {
    pub frame: Frame<'scope, R>,
    pub location: HitTestLocation,
    pub result: R,
}
/// Only source leaf operations cross this interface. Frame-chain construction,
/// adjustment policy and hover/mouse transitions are owned by EventHandler.
pub trait EventHandlerGestureTargeting<'scope, R: EventHandlerHitResult> {
    fn Now(&self) -> TimeTicks;
    fn InnerNodeFrame(&self, result: &R) -> Option<Frame<'scope, R>>;
    fn NodeFrame(&self, node: &NodeRef) -> Option<Frame<'scope, R>>;
    fn ParentLocalFrame(&self) -> Option<Frame<'scope, R>>;
    fn OwnerElement(&self) -> Option<NodeRef>;
    fn HoverElement(&self) -> Option<NodeRef>;
    fn ContentFrameForOwner(&self, node: &NodeRef) -> Option<Frame<'scope, R>>;
    fn SubframeForActiveUnboundedElement(
        &self,
        point: gfx::PointF,
    ) -> Option<UnboundedGestureTarget<'scope, R>>;
    fn AdjustmentScales(&self) -> (f32, f32, f32);
    fn IsOverResizer(&self, result: &R) -> bool;
    /// Assembles the native candidate/geometry context for this query only.
    /// A Layout loan must not escape through the retained source binding.
    fn FindBestTouchAdjustmentCandidate(
        &self,
        candidate: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &R,
    ) -> Option<(touch_adjustment::NodeRef, gfx::Point)>;
    fn ResolveRectBasedTest(
        &self,
        result: &mut R,
        node: &touch_adjustment::NodeRef,
        point: PhysicalOffset,
    ) -> HitTestLocation;
}
fn same_frame<'scope, R: EventHandlerHitResult>(
    a: &Frame<'scope, R>,
    b: &Frame<'scope, R>,
) -> bool {
    Rc::ptr_eq(&a.Identity(), &b.Identity())
}
fn same_node(a: &Option<NodeRef>, b: &Option<NodeRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
impl<'scope, R: EventHandlerHitResult> EventHandler<'scope, R> {
    // Source: event_handler.cc:1689-1704.
    pub fn HandleGestureEvent(&self, event: &WebGestureEvent) -> WebInputEventResult {
        debug_assert!(self.frame_.IsLocalRoot());
        debug_assert_ne!(event.FrameScale(), 0.0);
        debug_assert!(!event.IsScrollEvent());
        let targeted = self.TargetGestureEvent(event, false);
        self.HandleTargetedGestureEvent(&targeted)
    }
    // Source: event_handler.cc:1706-1749.
    pub fn HandleTargetedGestureEvent(
        &self,
        event: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        if !self.frame_.HasPage() {
            return WebInputEventResult::kNotHandled;
        }
        debug_assert!(self.frame_.IsLocalRoot());
        debug_assert!(!event.event.IsScrollEvent());
        if event.event.GetType() == Type::kGestureShowPress {
            self.last_show_press_timestamp_
                .set(Some(self.gesture_targeting_.Now()));
        }
        if event.event.GetType() == Type::kGestureTap {
            self.UpdateGestureTargetNodeForMouseEvent(event);
        }
        if let Some(target) = self
            .gesture_targeting_
            .SubframeForActiveUnboundedElement(event.event.PositionInRootFrame())
        {
            return target.frame.Handler().HandleGestureEventInFrame(
                &GestureEventWithHitTestResults {
                    event: event.event.clone(),
                    location: target.location,
                    result: target.result,
                },
            );
        }
        if let Some(frame) = self.gesture_targeting_.InnerNodeFrame(&event.result) {
            return frame.Handler().HandleGestureEventInFrame(event);
        }
        self.gesture_manager_.HandleGestureEventInFrame(event)
    }
    // Source: event_handler.cc:1782-1791.
    pub fn ShouldApplyTouchAdjustment(&self, event: &WebGestureEvent) -> bool {
        crate::core::input::gesture_targeting::ShouldApplyTouchAdjustment(event)
    }
    // Source: event_handler.cc:1793-1797.
    pub fn CacheTouchAdjustmentResult(&self, id: u32, point: gfx::PointF) {
        self.cursor_state_.CacheTouchAdjustmentResult(id, point);
    }
    // Source: event_handler.cc:1799-1824.
    pub fn GestureCorrespondsToAdjustedTouch(&self, event: &WebGestureEvent) -> bool {
        crate::core::input::gesture_targeting::GestureCorrespondsToAdjustedTouch(
            event,
            &self.should_use_touch_event_adjusted_point_,
            self.cursor_state_.TouchAdjustmentUniqueEventId(),
            self.cursor_state_.TouchAdjustmentPoint(),
        )
    }
    // Source: event_handler.cc:1826-1868.
    pub fn BestNodeForHitTestResult(
        &self,
        candidate: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &R,
    ) -> Option<(touch_adjustment::NodeRef, gfx::Point)> {
        assert!(location.IsRectBasedTest());
        if candidate != TouchAdjustmentCandidateType::kContextMenu
            && (result.GetScrollbar().is_some() || self.gesture_targeting_.IsOverResizer(result))
        {
            return None;
        }
        let root = self
            .frame_
            .HitTesting()
            .ConvertLocationToRoot(location, false);
        let area = root.ToEnclosingRect();
        if area.IsEmpty() {
            return None;
        }
        let bounds = location.BoundingBox();
        let point = location.Point();
        assert!(
            (bounds.X() <= point.left
                && point.left < bounds.Right()
                && bounds.Y() <= point.top
                && point.top < bounds.Bottom())
                || (bounds.Right() == LayoutUnit::Max() && point.left == LayoutUnit::Max())
                || (bounds.Bottom() == LayoutUnit::Max() && point.top == LayoutUnit::Max())
        );
        self.gesture_targeting_
            .FindBestTouchAdjustmentCandidate(candidate, location, result)
    }
    // Source: event_handler.cc:1875-1933.
    pub fn UpdateCrossFrameHoverActiveState(&self, active: bool, element: Option<NodeRef>) {
        debug_assert!(self.frame_.IsLocalRoot());
        let mut new_chain = Vec::new();
        let mut new_frame = element
            .as_ref()
            .and_then(|n| self.gesture_targeting_.NodeFrame(n));
        while let Some(frame) = new_frame {
            if same_frame(&frame, &self.frame_) {
                break;
            }
            new_chain.push(frame.clone());
            new_frame = frame.Handler().gesture_targeting_.ParentLocalFrame();
        }
        let mut old = self.gesture_targeting_.HoverElement();
        if !same_node(&element, &old) {
            let mut index = new_chain.len();
            while let Some(node) = old {
                let Some(old_frame) = self.gesture_targeting_.ContentFrameForOwner(&node) else {
                    break;
                };
                let new_frame = if index > 0 {
                    index -= 1;
                    Some(&new_chain[index])
                } else {
                    None
                };
                let handler = old_frame.Handler();
                old = handler.gesture_targeting_.HoverElement();
                if new_frame.is_none_or(|new| !same_frame(new, &old_frame)) {
                    old_frame
                        .HitTesting()
                        .UpdateHoverActiveState(active, true, None);
                }
            }
        }
        self.frame_
            .HitTesting()
            .UpdateHoverActiveState(active, true, element);
    }
    // Source: event_handler.cc:1938-2026.
    pub fn UpdateGestureTargetNodeForMouseEvent(&self, event: &GestureEventWithHitTestResults<R>) {
        debug_assert!(self.frame_.IsLocalRoot());
        let mut entered = Vec::new();
        let mut frame = self.gesture_targeting_.InnerNodeFrame(&event.result);
        while let Some(current) = frame {
            entered.push(current.clone());
            frame = current.Handler().gesture_targeting_.ParentLocalFrame();
        }
        let mut index = entered.len();
        let mut exited = Vec::new();
        let mut frame = Some(self.frame_.clone());
        while let Some(current) = frame {
            let handler = current.Handler();
            let Some(node) = handler.mouse_event_manager_.GetElementUnderMouse() else {
                break;
            };
            let next = handler.gesture_targeting_.ContentFrameForOwner(&node);
            if !exited.is_empty() {
                exited.push(current.clone());
            } else if index == 0 || !same_frame(&current, &entered[index - 1]) {
                exited.push(current.clone());
            } else if next.is_some() && index != 0 {
                index -= 1;
            }
            frame = next;
        }
        let mouse = WebMouseEvent::FromGesture(
            Type::kMouseMove,
            &event.event,
            Button::kNoButton,
            0,
            event.event.GetModifiers() | Modifiers::kIsCompatibilityEventForTouch as i32,
            event.event.TimeStamp(),
            PointerId::default(),
        );
        for frame in exited.into_iter().rev() {
            frame
                .Handler()
                .mouse_event_manager_
                .SetElementUnderMouse(self.EffectiveMouseEventTargetElement(None), &mouse);
        }
        while index != 0 {
            index -= 1;
            let target = entered[index].Handler();
            if let Some(parent) = target.gesture_targeting_.ParentLocalFrame() {
                parent.Handler().mouse_event_manager_.SetElementUnderMouse(
                    self.EffectiveMouseEventTargetElement(target.gesture_targeting_.OwnerElement()),
                    &mouse,
                );
            }
        }
    }
    // Source: event_handler.cc:2028-2075.
    pub fn TargetGestureEvent(
        &self,
        event: &WebGestureEvent,
        read_only: bool,
    ) -> GestureEventWithHitTestResults<R> {
        debug_assert!(self.frame_.IsLocalRoot());
        debug_assert!(!event.IsScrollEvent());
        match crate::core::input::gesture_targeting::TargetGestureEvent(
            &SourceGestureQuery(self),
            event,
            read_only,
        ) {
            Ok(targeted) => targeted,
            Err(impossible) => match impossible {},
        }
    }
    // Source: event_handler.cc:2077-2145.
    pub fn HitTestResultForGestureEvent(
        &self,
        event: &WebGestureEvent,
        hit_type: u32,
    ) -> GestureEventWithHitTestResults<R> {
        match crate::core::input::gesture_targeting::HitTestResultForGestureEvent(
            &SourceGestureQuery(self),
            event,
            hit_type,
        ) {
            Ok(result) => result,
            Err(impossible) => match impossible {},
        }
    }
    // Source: event_handler.cc:2146-2182.
    pub fn ApplyTouchAdjustment(
        &self,
        event: &mut WebGestureEvent,
        location: &mut HitTestLocation,
        result: &mut R,
    ) {
        let candidate = match event.GetType() {
            Type::kGestureTap
            | Type::kGestureTapUnconfirmed
            | Type::kGestureTapDown
            | Type::kGestureShowPress => TouchAdjustmentCandidateType::kClickable,
            Type::kGestureShortPress
            | Type::kGestureLongPress
            | Type::kGestureLongTap
            | Type::kGestureTwoFingerTap => TouchAdjustmentCandidateType::kContextMenu,
            _ => panic!("touch adjustment requires a tap or press gesture"),
        };
        if let Some((node, point)) = self.BestNodeForHitTestResult(candidate, location, result) {
            let local = self
                .frame_
                .HitTesting()
                .ConvertLocationFromRoot(&HitTestLocation::FromPoint(point));
            debug_assert!(location.ContainsPoint(*local.TransformedPoint()));
            debug_assert!(location.IsRectBasedTest());
            *location = self
                .gesture_targeting_
                .ResolveRectBasedTest(result, &node, *local.Point());
            event.ApplyTouchAdjustment(gfx::PointF::new(point.x() as f32, point.y() as f32));
        }
    }
}
impl<'scope, R: EventHandlerHitResult>
    crate::core::input::gesture_targeting::GestureTargetingQuery<R>
    for SourceGestureQuery<'_, 'scope, R>
{
    fn HitTypeForGesture(&self, kind: Type) -> u32 {
        self.0.gesture_manager_.GetHitTypeForGestureType(kind)
    }
    fn Now(&self) -> TimeTicks {
        self.0.gesture_targeting_.Now()
    }
    fn LastShowPressTimestamp(&self) -> Option<TimeTicks> {
        self.0.last_show_press_timestamp_.get()
    }
    fn UpdateHoverActiveState(&self, active: bool, result: &R) {
        self.0
            .UpdateCrossFrameHoverActiveState(active, result.InnerPossiblyPseudoElement());
    }
    fn DeferTapActiveState(&self, result: &R, delay: u64) -> Result<(), Self::Error> {
        self.0
            .last_deferred_tap_element_
            .replace(result.InnerElement());
        self.0
            .active_interval_timer_
            .StartOneShotMicroseconds(delay);
        Ok(())
    }
}

struct SourceGestureQuery<'a, 'scope, R: EventHandlerHitResult>(&'a EventHandler<'scope, R>);
impl<'scope, R: EventHandlerHitResult> crate::core::input::gesture_targeting::GestureTargetQuery<R>
    for SourceGestureQuery<'_, 'scope, R>
{
    type Error = std::convert::Infallible;
    fn ShouldApplyTouchAdjustment(&self, event: &WebGestureEvent) -> bool {
        self.0.ShouldApplyTouchAdjustment(event)
    }
    fn GestureCorrespondsToAdjustedTouch(&self, event: &WebGestureEvent) -> bool {
        self.0.GestureCorrespondsToAdjustedTouch(event)
    }
    fn AdjustedTouchPoint(&self) -> gfx::PointF {
        self.0.cursor_state_.touch_adjustment_point_.get()
    }
    fn HitTestRectSize(&self, width: f32, height: f32) -> PhysicalSize {
        let (device, inspector, page) = self.0.gesture_targeting_.AdjustmentScales();
        touch_adjustment::GetHitTestRectForAdjustment(
            PhysicalSize::new(LayoutUnit::from_f32(width), LayoutUnit::from_f32(height)),
            device,
            inspector,
            page,
        )
    }
    fn HitTest(&self, location: HitTestLocation, bits: u32) -> Result<R, Self::Error> {
        Ok(self
            .0
            .frame_
            .LocalFrameRoot()
            .Handler()
            .HitTestResultAtLocation(&location, bits, false))
    }
    fn BestNodeForHitTestResult(
        &self,
        candidate: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &R,
    ) -> Result<Option<gfx::Point>, Self::Error> {
        Ok(self
            .0
            .BestNodeForHitTestResult(candidate, location, result)
            .map(|(_, point)| point))
    }
}
