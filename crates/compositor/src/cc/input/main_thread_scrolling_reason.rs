// Translation status: complete
// Copyright 2016 The Chromium Authors
// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/main_thread_scrolling_reason.h:1-24,96-98;
//         cc/input/main_thread_scrolling_reason.cc:1-15,31-33,99-100.
// Rust module scope replaces header guards, namespace and export macros;
// typed imports replace forward declarations and dependency includes.
use crate::adapter::input_types::{EnumSet, EnumSetValue, TraceArraySink, TracedValueJSON};

// Source: cc/input/main_thread_scrolling_reason.h:25-38.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MainThreadRepaintReason {
    // See InputHandler::ScrollStatus::main_thread_repaint_reasons.
    // They are set in ScrollNode::main_thread_scrolling_reasons, or
    // kNoScrollingLayer is used for a ScrollNode that doesn't have
    // main_thread_scrolling_reasons but is_composited is false.
    kHasBackgroundAttachmentFixedObjects = 0,
    // Subpixel (LCD) text rendering requires blending glyphs with an opaque
    // background.
    kNotOpaqueForTextAndLCDText,
    kPreferNonCompositedScrolling,
    kBackgroundNeedsRepaintOnScroll,
}
impl MainThreadRepaintReason {
    pub const kMinValue: Self = Self::kHasBackgroundAttachmentFixedObjects;
    pub const kMaxValue: Self = Self::kBackgroundNeedsRepaintOnScroll;
}
impl EnumSetValue for MainThreadRepaintReason {
    fn ordinal(self) -> u32 {
        self as u32
    }
}

// Source: cc/input/main_thread_scrolling_reason.h:39-41.
pub type MainThreadRepaintReasons = EnumSet<MainThreadRepaintReason>;

// Source: cc/input/main_thread_scrolling_reason.h:42-49.
// See InputHandler::ScrollStatus::main_thread_hit_test_reasons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MainThreadHitTestReason {
    kScrollbarScrolling = 0,
    kMainThreadScrollHitTestRegion,
    kFailedHitTest,
}
impl MainThreadHitTestReason {
    pub const kMinValue: Self = Self::kScrollbarScrolling;
    pub const kMaxValue: Self = Self::kFailedHitTest;
}
impl EnumSetValue for MainThreadHitTestReason {
    fn ordinal(self) -> u32 {
        self as u32
    }
}

// Source: cc/input/main_thread_scrolling_reason.h:50-52.
pub type MainThreadHitTestReasons = EnumSet<MainThreadHitTestReason>;

// Source: cc/input/main_thread_scrolling_reason.h:53-69.
// The following reasons are neither repaint reasons nor hit-test reasons.
// They don't go through InputHandler::ScrollBegin() or set in
// InputHandler::ScrollStatus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MainThreadScrollingOtherReason {
    // We need main thread scrolling in a popup because it doesn't have a
    // threaded input handler. This flag is for metrics only, see
    // blink::WebPagePopupImpl::HandleGestureEvent.
    kPopupNoThreadedInput = 0,
    // Scrolling can be handled on the compositor thread but it might be
    // blocked on the main thread waiting for non-passive event handlers to
    // process the wheel/touch events (i.e. were they preventDefaulted?).
    kWheelEventHandlerRegion,
    kTouchEventHandlerRegion,
}
impl MainThreadScrollingOtherReason {
    pub const kMinValue: Self = Self::kPopupNoThreadedInput;
    pub const kMaxValue: Self = Self::kTouchEventHandlerRegion;
}
impl EnumSetValue for MainThreadScrollingOtherReason {
    fn ordinal(self) -> u32 {
        self as u32
    }
}

// Source: cc/input/main_thread_scrolling_reason.h:70-73.
pub type MainThreadScrollingOtherReasons = EnumSet<MainThreadScrollingOtherReason>;

// Source: cc/input/main_thread_scrolling_reason.h:74-78,88-89.
// Empty enum preserves the deleted C++ constructor: it has no instances.
pub enum MainThreadScrollingReason {}

// Source: cc/input/main_thread_scrolling_reason.cc:16-30.
fn AsTextImpl<R: MainThreadReasonSet>(reasons: R) -> String {
    let mut traced_value = TracedValueJSON::default();
    MainThreadScrollingReason::AddToTracedValue(reasons, &mut traced_value);
    let mut result = traced_value.ToJSON();
    // Remove '{main_thread_scrolling_reasons:[', ']}', and any '"' chars.
    let array_start_pos = result.find('[').expect("trace array start");
    let array_end_pos = result.find(']').expect("trace array end");
    result = result[array_start_pos + 1..array_end_pos].to_owned();
    result.retain(|c| c != '"');
    // Add spaces after all commas.
    result = result.replace(',', ", ");
    result
}

impl MainThreadScrollingReason {
    // Source: cc/input/main_thread_scrolling_reason.h:79-81;
    //         cc/input/main_thread_scrolling_reason.cc:34-48.
    // A typed generic resolves the three C++ overloads without runtime dispatch.
    pub fn AsText<R: MainThreadReasonSet>(reasons: R) -> String {
        AsTextImpl(reasons)
    }

    // Source: cc/input/main_thread_scrolling_reason.h:82-87;
    //         cc/input/main_thread_scrolling_reason.cc:57-98.
    // Each overload's complete control flow is in its trait implementation below.
    pub fn AddToTracedValue<R: MainThreadReasonSet>(
        reasons: R,
        traced_value: &mut dyn TraceArraySink,
    ) {
        reasons.AddToTracedValue(traced_value);
    }
}

// Rust overload resolution support; no Chromium behavior is implemented here.
pub trait MainThreadReasonSet {
    fn AddToTracedValue(self, traced_value: &mut dyn TraceArraySink);
}

// Source: cc/input/main_thread_scrolling_reason.cc:49-56.
macro_rules! ADD_REASON {
    ($reasons:ident, $traced_value:ident, $reason:expr, $string:expr) => {
        if $reasons.Has($reason) {
            $traced_value.AppendString($string);
            $reasons.Remove($reason);
        }
    };
}

// Source: cc/input/main_thread_scrolling_reason.cc:57-72.
impl MainThreadReasonSet for MainThreadRepaintReasons {
    fn AddToTracedValue(mut self, traced_value: &mut dyn TraceArraySink) {
        traced_value.BeginArray("main_thread_scrolling_reasons");
        ADD_REASON!(
            self,
            traced_value,
            MainThreadRepaintReason::kHasBackgroundAttachmentFixedObjects,
            "Has background-attachment:fixed"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadRepaintReason::kNotOpaqueForTextAndLCDText,
            "Not opaque for text and LCD text"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadRepaintReason::kPreferNonCompositedScrolling,
            "Prefer non-composited scrolling"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadRepaintReason::kBackgroundNeedsRepaintOnScroll,
            "Background needs repaint on scroll"
        );
        debug_assert!(self.empty());
        traced_value.EndArray();
    }
}

// Source: cc/input/main_thread_scrolling_reason.cc:73-85.
impl MainThreadReasonSet for MainThreadHitTestReasons {
    fn AddToTracedValue(mut self, traced_value: &mut dyn TraceArraySink) {
        traced_value.BeginArray("main_thread_scrolling_reasons");
        ADD_REASON!(
            self,
            traced_value,
            MainThreadHitTestReason::kScrollbarScrolling,
            "Scrollbar scrolling"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadHitTestReason::kMainThreadScrollHitTestRegion,
            "Main thread scroll hit test region"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadHitTestReason::kFailedHitTest,
            "Failed hit test"
        );
        debug_assert!(self.empty());
        traced_value.EndArray();
    }
}

// Source: cc/input/main_thread_scrolling_reason.cc:86-98.
impl MainThreadReasonSet for MainThreadScrollingOtherReasons {
    fn AddToTracedValue(mut self, traced_value: &mut dyn TraceArraySink) {
        traced_value.BeginArray("main_thread_scrolling_reasons");
        ADD_REASON!(
            self,
            traced_value,
            MainThreadScrollingOtherReason::kPopupNoThreadedInput,
            "Popup scrolling (no threaded input handler)"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadScrollingOtherReason::kWheelEventHandlerRegion,
            "Wheel event handler region"
        );
        ADD_REASON!(
            self,
            traced_value,
            MainThreadScrollingOtherReason::kTouchEventHandlerRegion,
            "Touch event handler region"
        );
        debug_assert!(self.empty());
        traced_value.EndArray();
    }
}

// Source: cc/input/main_thread_scrolling_reason.h:90-95.
// PrintTo overloads are declarations only, defined by Chromium's cc:test_support
// target. This production translation likewise has no test-support definitions.
