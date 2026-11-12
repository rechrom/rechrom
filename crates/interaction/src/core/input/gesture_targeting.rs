// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source: core/input/event_handler.cc:2077-2182.
// The original target algorithm is shared by LocalFrame EventHandler and the
// synchronous Page binder; each query lends its resident fragment owners.
use super::gesture_manager::GestureEventWithHitTestResults;
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use interaction_hit_test::core::{
    layout::{hit_test_location::HitTestLocation, hit_test_request::RequestType},
    page::touch_adjustment::TouchAdjustmentCandidateType,
};
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_input_event::Type,
};
use std::cell::Cell;

// Source EventHandler.cc:1782-1824, shared by the original source facade and
// its call-scoped production binding over the same touch adjustment cache.
pub fn ShouldApplyTouchAdjustment(event: &WebGestureEvent) -> bool {
    !matches!(event.primary_pointer_type,
        interaction_input::public::common::input::web_pointer_properties::PointerType::kPen |
        interaction_input::public::common::input::web_pointer_properties::PointerType::kEraser)
        && !event.TapAreaInRootFrame().IsEmpty()
}
pub fn GestureCorrespondsToAdjustedTouch(
    event: &WebGestureEvent,
    use_adjusted: &Cell<bool>,
    id: u32,
    point: gfx::PointF,
) -> bool {
    if event.GetType() == Type::kGestureTapDown {
        use_adjusted.set(event.unique_touch_event_id != 0 && event.unique_touch_event_id == id);
    }
    if use_adjusted.get() {
        let area = event.TapAreaInRootFrame();
        let position = event.PositionInRootFrame();
        let rect = gfx::RectF::new(
            gfx::PointF::new(
                position.x() - area.width() * 0.5,
                position.y() - area.height() * 0.5,
            ),
            area,
        );
        use_adjusted.set(rect.InclusiveContains(point));
    }
    use_adjusted.get()
}

pub trait GestureTargetingQuery<R>: GestureTargetQuery<R> {
    fn HitTypeForGesture(&self, kind: Type) -> u32;
    fn Now(&self) -> TimeTicks;
    fn LastShowPressTimestamp(&self) -> Option<TimeTicks>;
    fn UpdateHoverActiveState(&self, active: bool, result: &R);
    fn DeferTapActiveState(&self, result: &R, delay_microseconds: u64) -> Result<(), Self::Error>;
}
// Source EventHandler.cc:2028-2075. The source minimum interval and lifecycle
// order are identical for both callers; host services only schedule the task.
pub fn TargetGestureEvent<R, Q: GestureTargetingQuery<R>>(
    query: &Q,
    event: &WebGestureEvent,
    read_only: bool,
) -> Result<GestureEventWithHitTestResults<R>, Q::Error> {
    let mut bits = query.HitTypeForGesture(event.GetType());
    let mut remaining = None;
    if read_only {
        bits |= RequestType::kReadOnly as u32;
    } else if event.GetType() == Type::kGestureTap {
        if let Some(last) = query.LastShowPressTimestamp() {
            let elapsed = query.Now().ToInternalValue() - last.ToInternalValue();
            if elapsed < 150_000 {
                bits |= RequestType::kReadOnly as u32;
                remaining = Some((150_000 - elapsed) as u64);
            }
        }
    }
    let targeted = HitTestResultForGestureEvent(query, event, bits)?;
    if bits & RequestType::kReadOnly as u32 == 0 {
        query.UpdateHoverActiveState(bits & RequestType::kActive as u32 != 0, &targeted.result);
    }
    if let Some(delay) = remaining {
        query.DeferTapActiveState(&targeted.result, delay)?;
    }
    Ok(targeted)
}

pub trait GestureTargetQuery<R> {
    type Error;
    fn ShouldApplyTouchAdjustment(&self, event: &WebGestureEvent) -> bool;
    fn GestureCorrespondsToAdjustedTouch(&self, event: &WebGestureEvent) -> bool;
    fn AdjustedTouchPoint(&self) -> gfx::PointF;
    fn HitTestRectSize(&self, width: f32, height: f32) -> PhysicalSize;
    fn HitTest(&self, location: HitTestLocation, bits: u32) -> Result<R, Self::Error>;
    fn BestNodeForHitTestResult(
        &self,
        candidate: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &R,
    ) -> Result<Option<gfx::Point>, Self::Error>;
}

pub fn HitTestResultForGestureEvent<R, Q: GestureTargetQuery<R>>(
    query: &Q,
    event: &WebGestureEvent,
    mut bits: u32,
) -> Result<GestureEventWithHitTestResults<R>, Q::Error> {
    bits |= RequestType::kReadOnly as u32;
    let mut adjusted = event.clone();
    let mut size = PhysicalSize::default();
    if query.ShouldApplyTouchAdjustment(event) {
        if query.GestureCorrespondsToAdjustedTouch(event) {
            adjusted.ApplyTouchAdjustment(query.AdjustedTouchPoint());
        } else {
            let area = adjusted.TapAreaInRootFrame();
            size = query.HitTestRectSize(area.width(), area.height());
            if !size.IsEmpty() {
                bits |= RequestType::kListBased as u32;
            }
        }
    }
    let location;
    let result;
    if size.IsEmpty() {
        location = HitTestLocation::FromPointF(adjusted.PositionInRootFrame());
        result = query.HitTest(location.clone(), bits)?;
    } else {
        let point = PhysicalOffset::FromPointFRound(&adjusted.PositionInRootFrame());
        let rect = PhysicalRect::new(
            PhysicalOffset::new(point.left - size.width / 2, point.top - size.height / 2),
            size,
        );
        let rect_location = HitTestLocation::FromPhysicalRect(rect);
        let rect_result = query.HitTest(rect_location.clone(), bits)?;
        let candidate = match event.GetType() {
            Type::kGestureTap
            | Type::kGestureTapUnconfirmed
            | Type::kGestureTapDown
            | Type::kGestureShowPress => TouchAdjustmentCandidateType::kClickable,
            Type::kGestureShortPress
            | Type::kGestureLongPress
            | Type::kGestureLongTap
            | Type::kGestureTwoFingerTap => TouchAdjustmentCandidateType::kContextMenu,
            _ => unreachable!("touch adjustment requires a tap or press gesture"),
        };
        if let Some(point) =
            query.BestNodeForHitTestResult(candidate, &rect_location, &rect_result)?
        {
            adjusted.ApplyTouchAdjustment(gfx::PointF::new(point.x() as f32, point.y() as f32));
        }
        // Source performs the second point test after adjusting the event:
        // the rect candidate and the point hit are not interchangeable.
        location = HitTestLocation::FromPointF(adjusted.PositionInRootFrame());
        result = query.HitTest(
            location.clone(),
            (bits | RequestType::kReadOnly as u32) & !(RequestType::kListBased as u32),
        )?;
    }
    debug_assert!(!location.IsRectBasedTest());
    Ok(GestureEventWithHitTestResults {
        event: adjusted,
        location,
        result,
    })
}
