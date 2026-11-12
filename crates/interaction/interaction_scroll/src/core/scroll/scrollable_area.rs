/*
 * Copyright (C) 2008, 2011 Apple Inc. All Rights Reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. ``AS IS'' AND ANY EXPRESS OR IMPLIED
 * WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL APPLE INC. OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT,
 * INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/scroll/scrollable_area.h:88-115,122-125,145-148,169-170,348-389,403-404,432-447,548,580,638-640,645-646,674-675,710-712,722-725
// Source: third_party/blink/renderer/core/scroll/scrollable_area.cc:90-110,202-274,501-506,583-604,669-680,1032-1073,1430-1435
// Source: third_party/blink/renderer/core/scroll/scrollable_area.h:97-104
use foundation::gfx::{PointF, Rect, Size, ToRoundedVector2d, Vector2d};
use foundation::PhysicalRect;
use interaction_input::ui::events::types::scroll_types::ScrollGranularity;

use super::scroll_types::{
    ScrollConsumption, ScrollDirectionPhysical, ScrollOffset, ScrollbarOrientation,
};
use crate::cc::input::scroll_utils::{kPixelsPerLineStep, CalculatePageStep};
use crate::cc::trees::scroll_source_type::ScrollSourceType;
use crate::core::frame::scroll_options::ScrollBehavior as V8ScrollBehavior;
pub use crate::public::mojom::scroll::scroll_enums::{ScrollBehavior, ScrollType};

// Source: scrollable_area.h:88-91
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncludeScrollbarsInRect {
    kExcludeScrollbars,
    kIncludeScrollbars,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollCompletionMode {
    kFinished,
    kZeroDelta,
    kInterruptedByScroll,
}

pub type ScrollCallback = Box<dyn FnOnce(ScrollCompletionMode)>;

// These operations are applied synchronously to this resident area/animator
// owner at the source call point. No effects may be buffered across callbacks.
// Source: scrollable_area.cc:256,261-264,501-506,587,602-603
#[derive(Clone, Copy, Debug)]
pub enum ScrollMutation {
    CancelProgrammaticScrollAnimation,
    UnPinSelectedMarker,
    UpdateScrollMarkers,
    SetScrollOffset {
        offset: ScrollOffset,
        scroll_type: ScrollType,
        source_type: ScrollSourceType,
        behavior: ScrollBehavior,
    },
    ScrollToOffsetWithoutAnimation {
        offset: ScrollOffset,
        source_type: ScrollSourceType,
    },
}

pub trait ScrollMutationSink {
    fn Apply(&self, mutation: ScrollMutation);
}

// Source: scrollable_area.h:348-389,432; scrollable_area.cc:1048-1073.
// Every value is read from the canonical resident area. The projection has no
// writable engine access, tree storage or cached geometry of its own.
pub trait ScrollAreaGeometry {
    fn GetScrollOffset(&self) -> ScrollOffset;
    fn MinimumScrollOffsetInt(&self) -> Vector2d;
    fn MaximumScrollOffsetInt(&self) -> Vector2d;
    fn VisibleContentRect(&self, inclusion: IncludeScrollbarsInRect) -> Rect;
    fn VisibleScrollSnapportRect(&self, inclusion: IncludeScrollbarsInRect) -> PhysicalRect {
        self.VisibleContentRect(inclusion).into()
    }
    fn UserInputScrollable(&self, orientation: ScrollbarOrientation) -> bool;
    fn ScrollSize(&self, orientation: ScrollbarOrientation) -> i32;
    // The embedding owns WindowToViewportScalar and the frame lifetime.
    fn WindowToViewportLineStep(&self) -> Option<f32>;
    fn ShouldUseIntegerScrollOffset(&self) -> bool;
    fn HasScrollMarkerGroup(&self) -> bool;
    fn ScrollOffsetToPosition(&self, offset: ScrollOffset) -> PointF {
        PointF::new(offset.x(), offset.y())
    }
    fn ScrollPositionToOffset(&self, position: PointF) -> ScrollOffset {
        ScrollOffset::new(position.x(), position.y())
    }
    fn MinimumScrollOffset(&self) -> ScrollOffset {
        self.MinimumScrollOffsetInt().into()
    }
    fn MaximumScrollOffset(&self) -> ScrollOffset {
        self.MaximumScrollOffsetInt().into()
    }
}

// Source: scrollable_area.cc:246-259,589-603. This is the same area-owned
// ScrollAnimatorBase, injected independently from Layout/Paint geometry.
pub trait UserScrollAnimator {
    fn ComputeDeltaToConsume(&self, delta: ScrollOffset) -> ScrollOffset;
    fn UserScroll(
        &self,
        granularity: ScrollGranularity,
        delta: ScrollOffset,
        source_type: ScrollSourceType,
        on_finish: Option<ScrollCallback>,
        mutations: &dyn ScrollMutationSink,
    ) -> ScrollConsumption;
    fn CurrentOffset(&self) -> ScrollOffset;
}

// The facade borrows the live owner only for the synchronous transaction.
pub struct ScrollableArea<'a> {
    geometry: &'a dyn ScrollAreaGeometry,
    animator: &'a dyn UserScrollAnimator,
    mutations: &'a dyn ScrollMutationSink,
}

impl<'a> ScrollableArea<'a> {
    pub fn new(
        geometry: &'a dyn ScrollAreaGeometry,
        animator: &'a dyn UserScrollAnimator,
        mutations: &'a dyn ScrollMutationSink,
    ) -> Self {
        Self {
            geometry,
            animator,
            mutations,
        }
    }

    // Source: scrollable_area.h:109; scrollable_area.cc:90-95
    pub fn PixelsPerLineStep(&self) -> i32 {
        self.geometry
            .WindowToViewportLineStep()
            .unwrap_or(kPixelsPerLineStep as f32) as i32
    }

    // Source: scrollable_area.h:111-115
    pub fn NormalizeNonFiniteScroll(value: f32) -> f32 {
        if value.is_finite() {
            value
        } else {
            0.0
        }
    }

    // Source: scrollable_area.cc:98-110
    pub fn DetermineScrollBehavior(
        parameter: ScrollBehavior,
        style: ScrollBehavior,
    ) -> ScrollBehavior {
        if parameter == ScrollBehavior::kSmooth
            || (parameter == ScrollBehavior::kAuto && style == ScrollBehavior::kSmooth)
        {
            ScrollBehavior::kSmooth
        } else {
            ScrollBehavior::kInstant
        }
    }

    // Source: scrollable_area.cc:202-207
    pub fn ScrollbarOrientationFromDirection(
        direction: ScrollDirectionPhysical,
    ) -> ScrollbarOrientation {
        if matches!(
            direction,
            ScrollDirectionPhysical::kScrollUp | ScrollDirectionPhysical::kScrollDown
        ) {
            ScrollbarOrientation::kVerticalScrollbar
        } else {
            ScrollbarOrientation::kHorizontalScrollbar
        }
    }

    // Source: scrollable_area.cc:209-224
    pub fn ScrollStep(
        &self,
        granularity: ScrollGranularity,
        orientation: ScrollbarOrientation,
    ) -> f32 {
        match granularity {
            ScrollGranularity::kScrollByLine => self.LineStep(orientation) as f32,
            ScrollGranularity::kScrollByPage => self.PageStep(orientation) as f32,
            ScrollGranularity::kScrollByDocument => self.DocumentStep(orientation) as f32,
            ScrollGranularity::kScrollByPixel | ScrollGranularity::kScrollByPrecisePixel => {
                self.PixelStep(orientation)
            }
        }
    }

    // Source: scrollable_area.cc:226-232
    pub fn ResolveScrollDelta(
        &self,
        granularity: ScrollGranularity,
        delta: ScrollOffset,
    ) -> ScrollOffset {
        ScrollOffset::new(
            delta.x() * self.ScrollStep(granularity, ScrollbarOrientation::kHorizontalScrollbar),
            delta.y() * self.ScrollStep(granularity, ScrollbarOrientation::kVerticalScrollbar),
        )
    }

    // Source: scrollable_area.h:122-125; scrollable_area.cc:234-274
    pub fn UserScroll(
        &self,
        granularity: ScrollGranularity,
        delta: ScrollOffset,
        source_type: ScrollSourceType,
        on_finish: Option<ScrollCallback>,
    ) -> ScrollConsumption {
        // TODO: scrollable_area.cc:238-239 input trace instrumentation.
        let pixel_delta = self.ResolveScrollDelta(granularity, delta);
        let axis_delta = ScrollOffset::new(
            if self
                .geometry
                .UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar)
            {
                pixel_delta.x()
            } else {
                0.0
            },
            if self
                .geometry
                .UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar)
            {
                pixel_delta.y()
            } else {
                0.0
            },
        );
        let to_consume = self.animator.ComputeDeltaToConsume(axis_delta);
        if to_consume.x() == 0.0 && to_consume.y() == 0.0 {
            if let Some(callback) = on_finish {
                callback(ScrollCompletionMode::kZeroDelta);
            }
            return ScrollConsumption::new(false, false, pixel_delta.x(), pixel_delta.y());
        }
        self.mutations
            .Apply(ScrollMutation::CancelProgrammaticScrollAnimation);
        let mut result = self.animator.UserScroll(
            granularity,
            axis_delta,
            source_type,
            on_finish,
            self.mutations,
        );
        if result.DidScroll() {
            if self.geometry.HasScrollMarkerGroup() {
                self.mutations.Apply(ScrollMutation::UnPinSelectedMarker);
            }
            self.mutations.Apply(ScrollMutation::UpdateScrollMarkers);
        }
        let unscrollable = pixel_delta - axis_delta;
        result.unused_scroll_delta_x += unscrollable.x();
        result.unused_scroll_delta_y += unscrollable.y();
        result
    }

    // Source: scrollable_area.h:145-148; scrollable_area.cc:501-506
    pub fn ScrollBy(&self, delta: ScrollOffset, scroll_type: ScrollType, behavior: ScrollBehavior) {
        self.mutations.Apply(ScrollMutation::SetScrollOffset {
            offset: self.geometry.GetScrollOffset() + delta,
            scroll_type,
            source_type: ScrollSourceType::kRelativeScroll,
            behavior,
        });
    }

    // Source: scrollable_area.cc:583-604
    pub fn UserScrollHelper(
        &self,
        offset: ScrollOffset,
        behavior: ScrollBehavior,
        source_type: ScrollSourceType,
    ) {
        self.mutations
            .Apply(ScrollMutation::CancelProgrammaticScrollAnimation);
        let x = if self
            .geometry
            .UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar)
        {
            offset.x()
        } else {
            self.animator.CurrentOffset().x()
        };
        let y = if self
            .geometry
            .UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar)
        {
            offset.y()
        } else {
            self.animator.CurrentOffset().y()
        };
        debug_assert_eq!(behavior, ScrollBehavior::kInstant);
        self.mutations
            .Apply(ScrollMutation::ScrollToOffsetWithoutAnimation {
                offset: ScrollOffset::new(x, y),
                source_type,
            });
    }

    // Source: scrollable_area.h:169-170; scrollable_area.cc:669-680.
    pub fn V8EnumToScrollBehavior(behavior: V8ScrollBehavior) -> ScrollBehavior {
        match behavior {
            V8ScrollBehavior::kAuto => ScrollBehavior::kAuto,
            V8ScrollBehavior::kInstant => ScrollBehavior::kInstant,
            V8ScrollBehavior::kSmooth => ScrollBehavior::kSmooth,
        }
        // Rust's exhaustive enum match excludes the source NOTREACHED branch.
    }

    // Source: scrollable_area.h:348-357
    pub fn ScrollPosition(&self) -> PointF {
        self.geometry
            .ScrollOffsetToPosition(self.geometry.GetScrollOffset())
    }
    pub fn ScrollOffsetToPosition(&self, offset: ScrollOffset) -> PointF {
        self.geometry.ScrollOffsetToPosition(offset)
    }
    pub fn ScrollPositionToOffset(&self, position: PointF) -> ScrollOffset {
        self.geometry.ScrollPositionToOffset(position)
    }
    // Source: scrollable_area.h:364-379
    pub fn MinimumScrollOffset(&self) -> ScrollOffset {
        self.geometry.MinimumScrollOffset()
    }
    pub fn MaximumScrollOffset(&self) -> ScrollOffset {
        self.geometry.MaximumScrollOffset()
    }
    pub fn VisibleHeight(&self) -> i32 {
        self.geometry
            .VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars)
            .height()
    }
    pub fn VisibleWidth(&self) -> i32 {
        self.geometry
            .VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars)
            .width()
    }

    // Source: scrollable_area.h:403-404; scrollable_area.cc:1032-1046
    pub fn ClampScrollOffsetInt(&self, offset: Vector2d) -> Vector2d {
        let mut result = offset;
        result.SetToMin(&self.geometry.MaximumScrollOffsetInt());
        result.SetToMax(&self.geometry.MinimumScrollOffsetInt());
        result
    }
    pub fn ClampScrollOffset(&self, offset: ScrollOffset) -> ScrollOffset {
        ClampScrollOffsetForGeometry(self.geometry, offset)
    }

    // Source: scrollable_area.h:435-447
    pub fn MinimumScrollOffsetForOrientation(&self, orientation: ScrollbarOrientation) -> f32 {
        let offset = self.MinimumScrollOffset();
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            offset.x()
        } else {
            offset.y()
        }
    }
    pub fn MaximumScrollOffsetForOrientation(&self, orientation: ScrollbarOrientation) -> f32 {
        let offset = self.MaximumScrollOffset();
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            offset.x()
        } else {
            offset.y()
        }
    }
    pub fn ClampScrollOffsetForOrientation(
        &self,
        orientation: ScrollbarOrientation,
        offset: f32,
    ) -> f32 {
        let minimum = self.MinimumScrollOffsetForOrientation(orientation);
        let maximum = self.MaximumScrollOffsetForOrientation(orientation);
        if offset < minimum {
            minimum
        } else if maximum < offset {
            maximum
        } else {
            offset
        }
    }

    // Source: scrollable_area.cc:1048-1073
    pub fn LineStep(&self, _orientation: ScrollbarOrientation) -> i32 {
        self.PixelsPerLineStep()
    }
    pub fn PageSize(&self) -> Size {
        self.geometry
            .VisibleScrollSnapportRect(IncludeScrollbarsInRect::kExcludeScrollbars)
            .PixelSnappedSize()
    }
    pub fn PageStep(&self, orientation: ScrollbarOrientation) -> i32 {
        let size = self.PageSize();
        CalculatePageStep(
            if orientation == ScrollbarOrientation::kHorizontalScrollbar {
                size.width()
            } else {
                size.height()
            },
        )
    }
    pub fn DocumentStep(&self, orientation: ScrollbarOrientation) -> i32 {
        self.geometry.ScrollSize(orientation)
    }
    pub fn PixelStep(&self, _orientation: ScrollbarOrientation) -> f32 {
        1.0
    }

    // Source: scrollable_area.cc:1430-1435
    pub fn ScrollOffsetIsNoop(&self, offset: ScrollOffset) -> bool {
        self.geometry.GetScrollOffset()
            == if self.geometry.ShouldUseIntegerScrollOffset() {
                ScrollOffset::from(ToRoundedVector2d(&offset))
            } else {
                offset
            }
    }
}

/// Same source clamping for compound RootFrameViewport distribution, which
/// computes both endpoints before routing either viewport mutation.
pub fn ClampScrollOffsetForGeometry(
    geometry: &dyn ScrollAreaGeometry,
    offset: ScrollOffset,
) -> ScrollOffset {
    let mut result = offset;
    result.SetToMin(&geometry.MaximumScrollOffset());
    result.SetToMax(&geometry.MinimumScrollOffset());
    result
}

// Source: scrollable_area.cc:1374-1402. The source area scales pixel gestures
// before the host injects them; Scrollbar owns its injected-sequence state.
pub(crate) fn InjectScrollbarGestureScroll(
    source: &dyn super::scrollbar::ScrollbarPointerSource,
    kind: interaction_input::public::common::input::web_input_event::Type,
    delta: foundation::gfx::Vector2dF,
    granularity: ScrollGranularity,
) {
    use super::scrollbar::ScrollbarGesture;
    use foundation::gfx::Vector2dF;
    use interaction_input::public::common::input::web_input_event::Type;
    let scale = if matches!(
        granularity,
        ScrollGranularity::kScrollByPrecisePixel | ScrollGranularity::kScrollByPixel
    ) {
        source.InputEventsScaleFactor()
    } else {
        1.0
    };
    source.InjectScrollbarGesture(ScrollbarGesture {
        scroll_element_id: source.ScrollElementId(),
        delta: Vector2dF::new(delta.x() * scale, delta.y() * scale),
        granularity,
        gesture_type: kind,
    });
}
