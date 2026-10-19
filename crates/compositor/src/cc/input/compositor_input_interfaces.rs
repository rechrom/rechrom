// Translation status: complete
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/compositor_input_interfaces.h:1-42,209-212.
// Rust imports/traits replace includes, forward declarations, namespace and
// export guards. Host-associated types preserve class identity and ownership
// for yet-untranslated source owners; these are not placeholder implementations.
use crate::adapter::begin_frame::BeginFrameArgs;
use crate::adapter::input_geometry::{PointF, Size, Vector2dF};
use crate::adapter::input_handler::{InputHandlerTypes, InputHost};
use crate::adapter::input_services::InputHostTypes;
use crate::adapter::input_types::ElementId;
use crate::adapter::time::{TimeDelta, TimeTicks};
use crate::cc::input::actively_scrolling_type::ActivelyScrollingType;
use crate::cc::input::browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications;
use crate::cc::input::browser_controls_state::BrowserControlsState;

// Source: cc/input/compositor_input_interfaces.h:43-50,123-124.
// Two-way interface between LayerTreeHostImpl's graphics side and InputHandler.
// The associated concrete owner supplies virtual dispatch while each call
// borrows the live owner context; no second trait-object owner is retained.
pub trait InputDelegateForCompositor<H: InputHandlerTypes> {
    /// Source: cc/input/scrollbar_controller.cc:151-156. Typed weak callback
    /// delivery to the concrete source controller, with the current live Host.
    fn RunAutoscrollTask<C:InputHost<H>+?Sized>(&mut self,
        task:crate::adapter::scrollbar::AutoscrollTask,host:&mut C);

    // Source: cc/input/compositor_input_interfaces.h:51-56.
    // Fill changes since the last commit during commit processing.
    fn ProcessCommitDeltas<C: InputHost<H> + ?Sized>(
        &mut self,
        host: &mut C,
        commit_data: &mut H::CompositorCommitData,
        main_thread_mutator_host: Option<&H::MutatorHost>,
    );
    // Source: cc/input/compositor_input_interfaces.h:57-59.
    fn TickAnimations(&mut self, monotonic_time: TimeTicks);
    // Source: cc/input/compositor_input_interfaces.h:60-61.
    fn WillShutdown(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:62.
    fn WillDraw(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:63.
    fn WillBeginImplFrame<C: InputHost<H> + ?Sized>(&mut self, host: &mut C, args: &BeginFrameArgs);
    // Source: cc/input/compositor_input_interfaces.h:64.
    fn DidCommit<C: InputHost<H> + ?Sized>(&mut self, host: &mut C);
    // Source: cc/input/compositor_input_interfaces.h:65.
    fn DidImplSideInvalidate<C: InputHost<H> + ?Sized>(&mut self, host: &mut C);
    // Source: cc/input/compositor_input_interfaces.h:66.
    fn DidActivatePendingTree<C: InputHost<H> + ?Sized>(&mut self, host: &mut C);
    // Source: cc/input/compositor_input_interfaces.h:67.
    fn DidFinishImplFrame(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:68-69.
    fn OnBeginImplFrameDeadline<C: InputHost<H> + ?Sized>(&mut self, host: &mut C);
    // Source: cc/input/compositor_input_interfaces.h:70-74.
    // Root scroll offset, scrollable size/limits or page scale/limits changed
    // outside the input system.
    fn RootLayerStateMayHaveChanged<C: InputHost<H> + ?Sized>(&mut self, host: &mut C);
    // Source: cc/input/compositor_input_interfaces.h:75-78.
    fn DidRegisterScrollbar<C: InputHost<H> + ?Sized>(
        &mut self,
        host: &mut C,
        scroll_element_id: ElementId,
        orientation: H::ScrollbarOrientation,
    );
    // Source: cc/input/compositor_input_interfaces.h:79-83.
    fn DidUnregisterScrollbar<C: InputHost<H> + ?Sized>(
        &mut self,
        host: &mut C,
        scroll_element_id: ElementId,
        orientation: H::ScrollbarOrientation,
    );
    /// Borrow-only entry to the same source registration algorithm.
    fn DidRegisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        scroll_element_id: ElementId,
        orientation: H::ScrollbarOrientation,
    );
    fn DidUnregisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        scroll_element_id: ElementId,
        orientation: H::ScrollbarOrientation,
    );
    // Source: cc/input/compositor_input_interfaces.h:84-87.
    fn ScrollOffsetAnimationFinished<C: InputHost<H> + ?Sized>(
        &mut self,
        host: &mut C,
        element_id: ElementId,
    );
    // Source: cc/input/compositor_input_interfaces.h:88-91.
    fn ElasticOverscrollAnimationFinished<C: InputHost<H> + ?Sized>(
        &mut self,
        host: &mut C,
        element_id: ElementId,
    );
    // Source: cc/input/compositor_input_interfaces.h:92-94.
    fn SetPrefersReducedMotion(&mut self, prefers_reduced_motion: bool);
    // Source: cc/input/compositor_input_interfaces.h:95-101.
    // Between user GestureScrollBegin and GestureScrollEnd, including a deferred
    // end while still animating to the final snap position.
    fn IsCurrentlyScrolling<C: InputHost<H> + ?Sized>(&self, host: &mut C) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:102-110.
    // A latched node is active only after received ScrollUpdates consume delta.
    // A prevented touchmove may leave a latched node without active scrolling.
    fn GetActivelyScrollingType<C: InputHost<H> + ?Sized>(
        &self,
        host: &mut C,
    ) -> ActivelyScrollingType;
    // Source: cc/input/input_handler.cc:1616-1637. Rust-only borrowed output
    // for read-only selected-tree queries; no input field value is copied.
    fn View(&self) -> super::input_handler::InputHandlerView<'_>;
    /// Short field split only, preserving the source InputHandler owner.
    fn WithRegistrationAndView<Output>(
        &mut self,
        callback: impl FnOnce(
            &mut super::scrollbar_controller::ScrollbarController,
            super::input_handler::InputHandlerView<'_>,
        ) -> Output,
    ) -> Output;

    // Source: cc/input/compositor_input_interfaces.h:111-113.
    fn IsHandlingTouchSequence(&self) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:114-119.
    // Whether the current scroll must be realized on the main thread.
    fn IsCurrentScrollMainRepainted<C: InputHost<H> + ?Sized>(&self, host: &mut C) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:120-122.
    fn HasQueuedInput(&self) -> bool;
}

// Source: cc/input/compositor_input_interfaces.h:125-130,208.
// Interface exposed by LayerTreeHostImpl to InputHandler.
pub trait CompositorDelegateForInput<H: InputHostTypes> {
    // Source: cc/input/compositor_input_interfaces.h:131-133.
    fn BindToInputHandler(&mut self, delegate: Box<H::InputHandler>);
    // Source: cc/input/compositor_input_interfaces.h:134.
    // C++ const returns a mutable tree reference; an exclusive Rust borrow is
    // required to preserve that mutability without aliasing mutable references.
    fn GetScrollTree(&mut self) -> &mut H::ScrollTree;
    // Source: cc/input/compositor_input_interfaces.h:135.
    fn ScrollAnimationAbort(&mut self, element_id: ElementId);
    // Source: cc/input/compositor_input_interfaces.h:136.
    fn GetBrowserControlsTopOffset(&self) -> f32;
    // Source: cc/input/compositor_input_interfaces.h:137.
    fn ScrollBegin(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:138-139.
    fn ScrollEnd(&mut self, compensated_scroll_delta: &Vector2dF);
    // Source: cc/input/compositor_input_interfaces.h:140-142.
    fn StartScrollSequence(
        &mut self,
        r#type: H::FrameSequenceTrackerType,
        scrolling_thread: H::SmoothEffectDrivingThread,
    );
    // Source: cc/input/compositor_input_interfaces.h:143.
    fn StopSequence(&mut self, r#type: H::FrameSequenceTrackerType);
    // Source: cc/input/compositor_input_interfaces.h:144.
    fn ScrollbarAnimationMouseLeave(&mut self, element_id: ElementId);
    // Source: cc/input/compositor_input_interfaces.h:145-147.
    fn ScrollbarAnimationMouseMove(&mut self, element_id: ElementId, device_viewport_point: PointF);
    // Source: cc/input/compositor_input_interfaces.h:148.
    fn ScrollbarAnimationMouseDown(&mut self, element_id: ElementId) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:149.
    fn ScrollbarAnimationMouseUp(&mut self, element_id: ElementId) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:150.
    fn PinchBegin(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:151.
    fn PinchEnd(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:152.
    fn SetNeedsAnimateInput(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:153-155.
    fn ScrollAnimationCreate(
        &mut self,
        scroll_node: &H::ScrollNode,
        scroll_amount: &Vector2dF,
        delayed_by: TimeDelta,
    ) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:156.
    fn TickScrollAnimations(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:157-158.
    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut H::LatencyInfo>,
    ) -> Option<Box<H::LatencyInfoSwapPromiseMonitor>>;
    // Source: cc/input/compositor_input_interfaces.h:159-161.
    fn GetScopedEventMetricsMonitor(
        &mut self,
        done_callback: H::EventMetricsDoneCallback,
    ) -> Option<Box<H::EventMetricsScopedMonitor>>;
    // Source: cc/input/compositor_input_interfaces.h:162.
    fn DidScrollForMetrics(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:163-165.
    fn PredictViewportBoundsDelta(
        &self,
        current_bounds_delta: f64,
        scroll_distance: Vector2dF,
    ) -> f64;
    // Source: cc/input/compositor_input_interfaces.h:166.
    fn NotifyInputEvent(&mut self, is_fling: bool);
    // Source: cc/input/compositor_input_interfaces.h:167-168.
    fn ElementHasImplOnlyScrollAnimation(&self, element_id: ElementId) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:169-173.
    fn UpdateImplAnimationScrollTargetWithDelta(
        &mut self,
        adjusted_delta: Vector2dF,
        scroll_node_id: i32,
        delayed_by: TimeDelta,
        element_id: ElementId,
    ) -> Option<PointF>;
    // Source: cc/input/compositor_input_interfaces.h:174.
    fn HasAnimatedScrollbars(&self) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:175-176.
    fn SetNeedsCommit(&mut self, reason: H::BeginMainFrameReason);
    // Rust spelling of the source's default argument BeginMainFrameReason::kOther.
    fn SetNeedsCommitDefault(&mut self) {
        self.SetNeedsCommit(H::OtherBeginMainFrameReason());
    }
    // Source: cc/input/compositor_input_interfaces.h:177.
    fn SetDeferBeginMainFrame(&mut self, defer_begin_main_frame: bool);
    // Source: cc/input/compositor_input_interfaces.h:178.
    fn DidUpdateScrollAnimationCurve(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:179.
    fn DidStartPinchZoom(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:180.
    fn DidUpdatePinchZoom(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:181.
    fn DidEndPinchZoom(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:182.
    fn DidStartScroll(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:183.
    fn DidEndScroll(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:184.
    fn DidMouseEnterNonViewportScroller(&mut self, element_id: ElementId);
    // Source: cc/input/compositor_input_interfaces.h:185.
    fn DidMouseLeave(&mut self);
    // Source: cc/input/compositor_input_interfaces.h:186.
    fn IsInHighLatencyMode(&self) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:187.
    fn WillScrollContent(&mut self, element_id: ElementId);
    // Source: cc/input/compositor_input_interfaces.h:188-190.
    fn DidScrollContent(&mut self, element_id: ElementId, animated: bool, scroll_delta: &Vector2dF);
    // Source: cc/input/compositor_input_interfaces.h:191.
    fn DeviceScaleFactor(&self) -> f32;
    // Source: cc/input/compositor_input_interfaces.h:192.
    fn PageScaleFactor(&self) -> f32;
    // Source: cc/input/compositor_input_interfaces.h:193.
    fn VisualDeviceViewportSize(&self) -> Size;
    // Source: cc/input/compositor_input_interfaces.h:194.
    fn GetSettings(&self) -> &H::LayerTreeSettings;
    // Source: cc/input/compositor_input_interfaces.h:195-200.
    fn UpdateBrowserControlsState(
        &mut self,
        constraints: BrowserControlsState,
        current: BrowserControlsState,
        animate: bool,
        offset_tag_modifications: Option<&BrowserControlsOffsetTagModifications>,
    );
    // Source: cc/input/compositor_input_interfaces.h:201-202.
    fn HasScrollLinkedAnimation(&self, for_scroller: ElementId) -> bool;
    // Source: cc/input/compositor_input_interfaces.h:203-206.
    // Temporary source escape hatch pending migration to the input interface.
    fn GetImplDeprecated(&mut self) -> &mut H::LayerTreeHostImpl;
    // Source: cc/input/compositor_input_interfaces.h:207.
    // Rust gives the const overload a distinct name.
    fn GetImplDeprecatedConst(&self) -> &H::LayerTreeHostImpl;
}
