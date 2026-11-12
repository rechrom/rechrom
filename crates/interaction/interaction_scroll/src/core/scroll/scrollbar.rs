//! Scrollbar's pointer input and injected-gesture source methods.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/scroll/scrollbar.cc:306-357,386-429,655-737.
use super::scroll_types::{ScrollDirectionPhysical, ToScrollDelta};
pub use foundation::scrollbar_part::ScrollbarPart;
use foundation::{
    gfx::{Point, Vector2dF},
    scrollbar_input_state::ScrollbarInputState,
    CompositorElementId,
};
use interaction_input::{
    public::common::input::{
        web_gesture_event::WebGestureEvent,
        web_input_event::{Type, WebInputEvent},
        web_mouse_event::WebMouseEvent,
        web_pointer_event::WebPointerEvent,
    },
    ui::events::types::scroll_types::ScrollGranularity,
};
use std::time::Duration;
#[derive(Clone, Copy, Debug)]
pub struct ScrollbarGesture {
    pub scroll_element_id: CompositorElementId,
    pub delta: Vector2dF,
    pub granularity: ScrollGranularity,
    pub gesture_type: Type,
}
// Preserve the pointer facade path for the owning PaintLayerScrollableArea methods.
pub use crate::core::paint::paint_layer_scrollable_area::{
    PressedPartUseFeatures, ScrollbarThumbUse, ThumbUseFeature,
};
#[derive(Clone, Copy)]
pub struct ThumbGeometry {
    pub position: i32,
    pub length: i32,
    pub track_length: i32,
}
#[derive(Clone, Copy)]
pub struct ScrollOffsets {
    pub current: Vector2dF,
    pub minimum: Vector2dF,
    pub maximum: Vector2dF,
}
/// Required leaves of the same native Scrollbar/ScrollableArea. This port
/// owns no duplicate position, pressed state, geometry, target or gesture queue.
pub trait ScrollbarPointerSource {
    fn InputState(&self) -> &ScrollbarInputState;
    fn Connected(&self) -> bool;
    fn Horizontal(&self) -> bool;
    fn ThumbPressed(&self) -> bool;
    fn SetThumbPressed(&self, pressed: bool, event_type: Type);
    fn IsThumbAtRootFramePoint(&self, point: Point) -> bool;
    fn ConvertFromRootFrame(&self, point: Point) -> Point;
    fn ThumbGeometry(&self) -> ThumbGeometry;
    fn ScrollOffsets(&self) -> ScrollOffsets;
    fn ScrollElementId(&self) -> CompositorElementId;
    fn InputEventsScaleFactor(&self) -> f32;
    fn InjectScrollbarGesture(&self, gesture: ScrollbarGesture);
}
/// Additional source theme/part leaves for taps and presses. The part is the
/// native Scrollbar's same shared enum, with no adapter-side shadow state.
pub trait ScrollbarGestureSource: ScrollbarPointerSource {
    fn PressedPart(&self) -> ScrollbarPart;
    fn SetPressedPart(&self, part: ScrollbarPart, event_type: Type);
    fn HitTestRootFramePosition(&self, point: Point) -> ScrollbarPart;
}
/// Source timer services are injected; no clock or OS timer belongs to this
/// engine. TargetThumbGeometry reads the animator's real DesiredTargetOffset.
pub trait ScrollbarAutoscrollSource: ScrollbarGestureSource {
    fn TargetThumbGeometry(&self) -> (i32, i32);
    fn SetHoveredPart(&self, part: ScrollbarPart);
    fn StartAutoscrollTimer(&self, delay: Duration);
    fn StopAutoscrollTimer(&self);
    fn AutoscrollTimerDelay(&self) -> Duration;
}
pub trait ScrollbarMouseSource: ScrollbarAutoscrollSource {
    fn HoveredPart(&self) -> ScrollbarPart;
    fn CurrentPosition(&self) -> f32;
    fn TrackPosition(&self) -> i32;
    fn MiddleClickAutoscrollEnabled(&self) -> bool;
    fn ShouldCenterOnThumb(&self, event: &WebMouseEvent) -> bool;
    fn ShouldSnapBackToDragOrigin(&self, event: &WebMouseEvent) -> bool;
    fn ShouldDragDocumentInsteadOfThumb(&self, event: &WebMouseEvent) -> bool;
    fn InitialAutoscrollTimerDelay(&self) -> Duration;
    fn MouseEnteredScrollbar(&self);
    fn MouseExitedScrollbar(&self);
    fn MouseCapturedScrollbar(&self);
    fn MouseReleasedScrollbar(&self);
    fn UsesFluentOverlayScrollbars(&self) -> bool;
    fn InvalidateThumb(&self);
    fn SetPaintArtifactCompositorNeedsUpdate(&self);
}
fn MousePosition(event: &WebMouseEvent) -> Point {
    let point = event.PositionInRootFrame();
    Point::new(point.x().floor() as i32, point.y().floor() as i32)
}
// Source: scrollbar.cc:488-538.
pub fn MouseMoved(source: &dyn ScrollbarMouseSource, event: &WebMouseEvent) {
    use ScrollbarPart::*;
    let position = MousePosition(event);
    let part = source.HitTestRootFramePosition(position);
    let state = source.InputState();
    if state.manipulation_on_compositor.get() {
        source.SetHoveredPart(part);
        return;
    }
    if source.PressedPart() == kThumbPart {
        if source.ShouldSnapBackToDragOrigin(event) {
            if source.Connected() {
                let minimum = source.ScrollOffsets().minimum;
                InjectGestureScrollUpdateForThumbMove(
                    source,
                    state.drag_origin.get()
                        + if source.Horizontal() {
                            minimum.x()
                        } else {
                            minimum.y()
                        },
                );
            }
        } else {
            let local = source.ConvertFromRootFrame(position);
            MoveThumb(
                source,
                if source.Horizontal() {
                    local.x()
                } else {
                    local.y()
                },
                source.ShouldDragDocumentInsteadOfThumb(event),
            );
        }
        return;
    }
    if source.PressedPart() != kNoPart {
        let local = source.ConvertFromRootFrame(position);
        state.pressed_pos.set(if source.Horizontal() {
            local.x()
        } else {
            local.y()
        });
    }
    if part != source.HoveredPart() {
        if source.PressedPart() != kNoPart {
            if part == source.PressedPart() {
                StartTimerIfNeeded(source, source.AutoscrollTimerDelay());
            } else if source.HoveredPart() == source.PressedPart() {
                StopTimerIfNeeded(source);
            }
        }
        source.SetHoveredPart(part);
    }
}
// Source: scrollbar.cc:540-564.
pub fn MouseEntered(source: &dyn ScrollbarMouseSource) {
    if source.Connected() {
        source.MouseEnteredScrollbar();
    }
    if source.UsesFluentOverlayScrollbars() && source.Connected() {
        source.SetPaintArtifactCompositorNeedsUpdate();
    }
}
pub fn MouseExited(source: &dyn ScrollbarMouseSource) {
    if source.Connected() {
        source.MouseExitedScrollbar();
    }
    source.SetHoveredPart(ScrollbarPart::kNoPart);
    if source.UsesFluentOverlayScrollbars() && source.Connected() {
        source.InvalidateThumb();
        source.SetPaintArtifactCompositorNeedsUpdate();
    }
}
// Source: scrollbar.cc:566-591. This is not HandlePointerEvent(PointerUp).
pub fn MouseUp(source: &dyn ScrollbarMouseSource, event: &WebMouseEvent) {
    use ScrollbarPart::*;
    let captured = source.PressedPart() == kThumbPart;
    source.SetPressedPart(kNoPart, event.GetType());
    let state = source.InputState();
    if state.manipulation_on_compositor.replace(false) {
        return;
    }
    state.pressed_pos.set(0);
    state.dragging_document.set(false);
    StopTimerIfNeeded(source);
    if source.Connected() {
        if captured {
            source.MouseReleasedScrollbar();
        }
        if source.HitTestRootFramePosition(MousePosition(event)) == kNoPart {
            source.SetHoveredPart(kNoPart);
            source.MouseExitedScrollbar();
        }
        InjectScrollGestureForPressedPart(source, Type::kGestureScrollEnd);
    }
}
// Source: scrollbar.cc:53-65,593-641.
pub fn MouseDown(source: &dyn ScrollbarMouseSource, event: &WebMouseEvent) {
    use interaction_input::public::common::input::web_pointer_properties::Button;
    use ScrollbarPart::*;
    let button = event.pointer_properties.button;
    if if button == Button::kMiddle {
        source.MiddleClickAutoscrollEnabled()
    } else {
        button != Button::kLeft
    } {
        return;
    }
    let position = MousePosition(event);
    source.SetPressedPart(source.HitTestRootFramePosition(position), event.GetType());
    let state = source.InputState();
    state.manipulation_on_compositor.set(
        event.GetModifiers() & WebInputEvent::kScrollbarManipulationHandledOnCompositorThread != 0,
    );
    if state.manipulation_on_compositor.get() {
        return;
    }
    let local = source.ConvertFromRootFrame(position);
    let pressed_pos = if source.Horizontal() {
        local.x()
    } else {
        local.y()
    };
    if matches!(source.PressedPart(), kBackTrackPart | kForwardTrackPart)
        && source.ShouldCenterOnThumb(event)
    {
        source.SetHoveredPart(kThumbPart);
        source.SetPressedPart(kThumbPart, event.GetType());
        state.drag_origin.set(source.CurrentPosition());
        let geometry = source.ThumbGeometry();
        state
            .pressed_pos
            .set(source.TrackPosition() + geometry.position + geometry.length / 2);
        MoveThumb(source, pressed_pos, false);
        return;
    }
    if source.PressedPart() == kThumbPart {
        state.drag_origin.set(source.CurrentPosition());
        if source.Connected() {
            source.MouseCapturedScrollbar();
        }
    }
    state.pressed_pos.set(pressed_pos);
    AutoscrollPressedPart(source, source.InitialAutoscrollTimerDelay());
}
// Source scrollbar.cc:226-281.
pub fn ThumbWillBeUnderMouse(source: &dyn ScrollbarAutoscrollSource) -> bool {
    let (position, length) = source.TargetThumbGeometry();
    source.InputState().pressed_pos.get() >= position
        && source.InputState().pressed_pos.get() < position + length
}
pub fn AutoscrollPressedPart(source: &dyn ScrollbarAutoscrollSource, delay: Duration) {
    use ScrollbarPart::*;
    if !source.Connected() {
        return;
    }
    let part = source.PressedPart();
    if matches!(part, kThumbPart | kNoPart) {
        return;
    }
    if matches!(part, kBackTrackPart | kForwardTrackPart) && ThumbWillBeUnderMouse(source) {
        source.SetHoveredPart(kThumbPart);
        return;
    }
    InjectScrollGestureForPressedPart(source, Type::kGestureScrollUpdate);
    if matches!(
        part,
        kBackButtonStartPart
            | kForwardButtonStartPart
            | kBackButtonEndPart
            | kForwardButtonEndPart
            | kBackTrackPart
            | kForwardTrackPart
    ) {
        StartTimerIfNeeded(source, delay);
    }
}
pub fn StartTimerIfNeeded(source: &dyn ScrollbarAutoscrollSource, delay: Duration) {
    use ScrollbarPart::*;
    if source.PressedPart() == kThumbPart {
        return;
    }
    if matches!(source.PressedPart(), kBackTrackPart | kForwardTrackPart)
        && ThumbWillBeUnderMouse(source)
    {
        source.SetHoveredPart(kThumbPart);
        return;
    }
    source.StartAutoscrollTimer(delay);
}
pub fn StopTimerIfNeeded(source: &dyn ScrollbarAutoscrollSource) {
    source.StopAutoscrollTimer();
}
pub fn AutoscrollTimerFired(source: &dyn ScrollbarAutoscrollSource) {
    AutoscrollPressedPart(source, source.AutoscrollTimerDelay());
}
// Source scrollbar.cc:283-304.
pub fn PressedPartScrollDirectionPhysical(
    source: &dyn ScrollbarGestureSource,
) -> ScrollDirectionPhysical {
    use ScrollDirectionPhysical::*;
    use ScrollbarPart::*;
    let back = matches!(
        source.PressedPart(),
        kBackButtonStartPart | kBackButtonEndPart | kBackTrackPart
    );
    match (source.Horizontal(), back) {
        (true, true) => kScrollLeft,
        (true, false) => kScrollRight,
        (false, true) => kScrollUp,
        (false, false) => kScrollDown,
    }
}
pub fn PressedPartScrollGranularity(source: &dyn ScrollbarGestureSource) -> ScrollGranularity {
    use ScrollbarPart::*;
    if matches!(
        source.PressedPart(),
        kBackButtonStartPart | kBackButtonEndPart | kForwardButtonStartPart | kForwardButtonEndPart
    ) {
        ScrollGranularity::kScrollByLine
    } else {
        ScrollGranularity::kScrollByPage
    }
}
// Source scrollbar.cc:433-487. A tap performs one begin/update/end sequence;
// long press releases the pressed part and never starts the mouse repeat timer.
pub fn HandleGestureTapOrPress(
    source: &dyn ScrollbarGestureSource,
    event: &WebGestureEvent,
) -> bool {
    use ScrollbarPart::*;
    debug_assert!(!event.IsScrollEvent());
    let state = source.InputState();
    match event.GetType() {
        Type::kGestureTapDown => {
            let position = event.PositionInRootFrame();
            let point = Point::new(position.x().floor() as i32, position.y().floor() as i32);
            source.SetPressedPart(source.HitTestRootFramePosition(point), event.GetType());
            let local = source.ConvertFromRootFrame(point);
            state.pressed_pos.set(if source.Horizontal() {
                local.x()
            } else {
                local.y()
            });
            true
        }
        Type::kGestureTapCancel => {
            if source.PressedPart() != kThumbPart {
                return false;
            }
            state.scroll_pos.set(state.pressed_pos.get() as f32);
            true
        }
        Type::kGestureShortPress | Type::kGestureLongPress => {
            state.scroll_pos.set(0.0);
            state.pressed_pos.set(0);
            source.SetPressedPart(kNoPart, event.GetType());
            false
        }
        Type::kGestureTap => HandleTapGesture(source),
        _ => true,
    }
}
pub fn HandleTapGesture(source: &dyn ScrollbarGestureSource) -> bool {
    if !matches!(
        source.PressedPart(),
        ScrollbarPart::kThumbPart | ScrollbarPart::kNoPart
    ) && source.Connected()
    {
        let delta = ToScrollDelta(PressedPartScrollDirectionPhysical(source), 1.0);
        if DeltaWillScroll(source, delta) {
            InjectScrollGestureForPressedPart(source, Type::kGestureScrollBegin);
            InjectScrollGestureForPressedPart(source, Type::kGestureScrollUpdate);
            InjectScrollGestureForPressedPart(source, Type::kGestureScrollEnd);
            return true;
        }
    }
    source.InputState().scroll_pos.set(0.0);
    source.InputState().pressed_pos.set(0);
    source.SetPressedPart(ScrollbarPart::kNoPart, Type::kGestureTap);
    true
}
// Source scrollbar.cc:644-652,728-736.
pub fn InjectScrollGestureForPressedPart(source: &dyn ScrollbarGestureSource, kind: Type) {
    InjectScrollGesture(
        source,
        kind,
        ToScrollDelta(PressedPartScrollDirectionPhysical(source), 1.0),
        PressedPartScrollGranularity(source),
    );
}
pub fn DeltaWillScroll(source: &dyn ScrollbarPointerSource, delta: Vector2dF) -> bool {
    let offsets = source.ScrollOffsets();
    let target = offsets.current + delta;
    Vector2dF::new(
        target.x().clamp(offsets.minimum.x(), offsets.maximum.x()),
        target.y().clamp(offsets.minimum.y(), offsets.maximum.y()),
    ) != offsets.current
}
pub fn HandlePointerEvent(source: &dyn ScrollbarPointerSource, event: &WebPointerEvent) -> bool {
    let point = event.pointer_properties.PositionInWidget();
    let point = Point::new(point.x().floor() as i32, point.y().floor() as i32);
    let state = source.InputState();
    match event.GetType() {
        Type::kPointerDown => {
            if !source.IsThumbAtRootFramePoint(point) {
                return false;
            }
            source.SetThumbPressed(true, event.GetType());
            let local = source.ConvertFromRootFrame(point);
            let pos = if source.Horizontal() {
                local.x()
            } else {
                local.y()
            };
            state.pressed_pos.set(pos);
            state.scroll_pos.set(pos as f32);
            true
        }
        Type::kPointerMove if source.ThumbPressed() => {
            let local = source.ConvertFromRootFrame(point);
            let pos = if source.Horizontal() {
                local.x()
            } else {
                local.y()
            };
            state.scroll_pos.set(pos as f32);
            MoveThumb(source, pos, false);
            true
        }
        Type::kPointerUp if source.ThumbPressed() => {
            source.SetThumbPressed(false, event.GetType());
            state.pressed_pos.set(0);
            InjectScrollGesture(
                source,
                Type::kGestureScrollEnd,
                Vector2dF::default(),
                ScrollGranularity::kScrollByPrecisePixel,
            );
            true
        }
        _ => false,
    }
}
pub fn MoveThumb(source: &dyn ScrollbarPointerSource, pos: i32, dragging_document: bool) {
    if !source.Connected() {
        return;
    }
    let state = source.InputState();
    let mut delta = pos - state.pressed_pos.get();
    let offsets = source.ScrollOffsets();
    let (minimum, maximum, current) = if source.Horizontal() {
        (
            offsets.minimum.x(),
            offsets.maximum.x(),
            offsets.current.x(),
        )
    } else {
        (
            offsets.minimum.y(),
            offsets.maximum.y(),
            offsets.current.y(),
        )
    };
    if dragging_document {
        if state.dragging_document.get() {
            delta = pos - state.document_drag_pos.get();
        }
        state.dragging_document.set(true);
        InjectGestureScrollUpdateForThumbMove(
            source,
            (current + delta as f32).clamp(minimum, maximum),
        );
        state.document_drag_pos.set(pos);
        return;
    }
    if state.dragging_document.replace(false) {
        delta += state.pressed_pos.get() - state.document_drag_pos.get();
    }
    let geometry = source.ThumbGeometry();
    debug_assert!(geometry.length <= geometry.track_length);
    if geometry.length == geometry.track_length {
        return;
    }
    if delta > 0 {
        delta = delta.min(geometry.track_length - geometry.length - geometry.position);
    } else if delta < 0 {
        delta = delta.max(-geometry.position);
    }
    if delta != 0 {
        let target = (geometry.position + delta) as f32 * (maximum - minimum)
            / (geometry.track_length - geometry.length) as f32
            + minimum;
        InjectGestureScrollUpdateForThumbMove(source, target);
    }
}
pub fn InjectGestureScrollUpdateForThumbMove(source: &dyn ScrollbarPointerSource, target: f32) {
    debug_assert!(source.ThumbPressed());
    let current = source.ScrollOffsets().current + source.InputState().pending_injected_delta.get();
    let desired = if source.Horizontal() {
        Vector2dF::new(target, current.y())
    } else {
        Vector2dF::new(current.x(), target)
    };
    InjectScrollGesture(
        source,
        Type::kGestureScrollUpdate,
        desired - current,
        ScrollGranularity::kScrollByPrecisePixel,
    );
}
pub fn InjectScrollGesture(
    source: &dyn ScrollbarPointerSource,
    kind: Type,
    delta: Vector2dF,
    granularity: ScrollGranularity,
) {
    if !source.Connected() {
        return;
    }
    let state = source.InputState();
    if kind == Type::kGestureScrollEnd && !state.injected_gesture_scroll_begin.get() {
        return;
    }
    if matches!(kind, Type::kGestureScrollBegin | Type::kGestureScrollUpdate) {
        if !DeltaWillScroll(source, delta) {
            return;
        }
    }
    if kind == Type::kGestureScrollUpdate && !state.injected_gesture_scroll_begin.get() {
        InjectScrollGesture(source, Type::kGestureScrollBegin, delta, granularity);
    }
    state
        .pending_injected_delta
        .set(state.pending_injected_delta.get() + delta);
    super::scrollable_area::InjectScrollbarGestureScroll(source, kind, delta, granularity);
    if kind == Type::kGestureScrollBegin {
        state.injected_gesture_scroll_begin.set(true);
    } else if kind == Type::kGestureScrollEnd {
        state.injected_gesture_scroll_begin.set(false);
    }
}
