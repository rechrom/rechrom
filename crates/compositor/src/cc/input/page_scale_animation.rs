// Translation status: complete
// Copyright 2011 The Chromium Authors
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/page_scale_animation.h:1-19,94,100,129-133;
//         cc/input/page_scale_animation.cc:1-16,40-45,241.
// Rust imports/module scope replace includes, namespaces and export guards.
use crate::adapter::cubic_bezier::CubicBezier;
use crate::adapter::input_geometry::{
    Point, PointAtOffsetFromOrigin, PointF, RectF, ScaleSize, ScaleVector2d, SizeF, Vector2dF,
};
use crate::adapter::input_services::Seconds;
use crate::adapter::time::{TimeDelta, TimeTicks};

// Source: cc/input/page_scale_animation.h:20-35.
// Used in cc to pass around a scale animation that has not yet been initialized.
#[derive(Clone, Copy, Debug)]
pub struct PendingPageScaleAnimation {
    pub target_offset: Point,
    pub use_anchor: bool,
    pub scale: f32,
    pub duration: TimeDelta,
}
impl PendingPageScaleAnimation {
    // Source: cc/input/page_scale_animation.h:23-30.
    pub fn new(target_offset: &Point, use_anchor: bool, scale: f32, duration: &TimeDelta) -> Self {
        Self {
            target_offset: *target_offset,
            use_anchor,
            scale,
            duration: *duration,
        }
    }
}

// Source: cc/input/page_scale_animation.cc:17-26.
// Return viewport-relative coordinates normalized to percentage position.
fn NormalizeFromViewport(denormalized: Vector2dF, viewport_size: SizeF) -> Vector2dF {
    debug_assert!(!viewport_size.IsEmpty());
    ScaleVector2d(
        denormalized,
        1.0 / viewport_size.width(),
        1.0 / viewport_size.height(),
    )
}

// Source: cc/input/page_scale_animation.cc:27-33.
fn DenormalizeToViewport(normalized: Vector2dF, viewport_size: SizeF) -> Vector2dF {
    ScaleVector2d(normalized, viewport_size.width(), viewport_size.height())
}

// Source: cc/input/page_scale_animation.cc:34-39.
fn InterpolateBetween(start: Vector2dF, end: Vector2dF, interp: f32) -> Vector2dF {
    start + ScaleVector2d(end - start, interp, interp)
}

// Source: cc/input/page_scale_animation.h:37-46,54-58,114-128.
// Zoom animation math, primarily double-tap zoom. Call ...AtTime each frame to
// obtain eased scroll/scale positions. All public sizes/vectors are in the root
// scroll layer's coordinate space. No Clone/Copy preserves deleted copy members;
// automatic Rust field destruction implements the source default destructor.
pub struct PageScaleAnimation {
    start_page_scale_factor_: f32,
    target_page_scale_factor_: f32,
    start_scroll_offset_: PointF,
    target_scroll_offset_: PointF,
    start_anchor_: PointF,
    target_anchor_: PointF,
    viewport_size_: SizeF,
    root_layer_size_: SizeF,
    start_time_: TimeTicks,
    duration_: TimeDelta,
    timing_function_: CubicBezier,
}
impl PageScaleAnimation {
    // Source: cc/input/page_scale_animation.h:47-53;
    //         cc/input/page_scale_animation.cc:46-55.
    pub fn Create(
        start_scroll_offset: &PointF,
        start_page_scale_factor: f32,
        viewport_size: &SizeF,
        root_layer_size: &SizeF,
    ) -> Box<Self> {
        Box::new(Self::new(
            start_scroll_offset,
            start_page_scale_factor,
            viewport_size,
            root_layer_size,
        ))
    }

    // Source: cc/input/page_scale_animation.h:95-99;
    //         cc/input/page_scale_animation.cc:56-71.
    fn new(
        start_scroll_offset: &PointF,
        start_page_scale_factor: f32,
        viewport_size: &SizeF,
        root_layer_size: &SizeF,
    ) -> Self {
        Self {
            start_page_scale_factor_: start_page_scale_factor,
            target_page_scale_factor_: 0.0,
            start_scroll_offset_: *start_scroll_offset,
            target_scroll_offset_: PointF::default(),
            start_anchor_: PointF::default(),
            target_anchor_: PointF::default(),
            viewport_size_: *viewport_size,
            root_layer_size_: *root_layer_size,
            start_time_: TimeTicks::default(),
            duration_: TimeDelta::default(),
            // Easing constants experimentally determined.
            timing_function_: CubicBezier::new(0.8, 0.0, 0.3, 0.9),
        }
    }

    // Source: cc/input/page_scale_animation.h:59-66;
    //         cc/input/page_scale_animation.cc:72-91.
    // Initialize final scroll/scale immediately after construction.
    // Zoom while explicitly specifying the top-left scroll position.
    pub fn ZoomTo(
        &mut self,
        target_scroll_offset: &PointF,
        target_page_scale_factor: f32,
        duration: f64,
    ) {
        self.target_page_scale_factor_ = target_page_scale_factor;
        self.target_scroll_offset_ = *target_scroll_offset;
        self.ClampTargetScrollOffset();
        self.duration_ = Seconds(duration);
        if self.start_page_scale_factor_ == target_page_scale_factor {
            self.start_anchor_ = self.start_scroll_offset_;
            self.target_anchor_ = *target_scroll_offset;
            return;
        }
        // Infer an anchor from start/target viewport rects for uniform zooming.
        self.InferTargetAnchorFromScrollOffsets();
        self.start_anchor_ = self.target_anchor_;
    }

    // Source: cc/input/page_scale_animation.h:67-74;
    //         cc/input/page_scale_animation.cc:92-112.
    // Keep the content-space anchor at the same physical-display position,
    // unless constrained by root layer edges.
    pub fn ZoomWithAnchor(
        &mut self,
        anchor: &PointF,
        target_page_scale_factor: f32,
        duration: f64,
    ) {
        self.start_anchor_ = *anchor;
        self.target_page_scale_factor_ = target_page_scale_factor;
        self.duration_ = Seconds(duration);
        // If the tapped anchor would collide with root edges, infer a target
        // anchor that does not. Interpolate between the anchors during animation.
        self.InferTargetScrollOffsetFromStartAnchor();
        self.ClampTargetScrollOffset();
        if self.start_page_scale_factor_ == self.target_page_scale_factor_ {
            self.target_anchor_ = self.start_anchor_;
            return;
        }
        self.InferTargetAnchorFromScrollOffsets();
    }

    // Source: cc/input/page_scale_animation.h:102;
    //         cc/input/page_scale_animation.cc:113-119.
    fn InferTargetScrollOffsetFromStartAnchor(&mut self) {
        let normalized = NormalizeFromViewport(
            self.start_anchor_ - self.start_scroll_offset_,
            self.StartViewportSize(),
        );
        self.target_scroll_offset_ =
            self.start_anchor_ - DenormalizeToViewport(normalized, self.TargetViewportSize());
    }

    // Source: cc/input/page_scale_animation.h:103;
    //         cc/input/page_scale_animation.cc:120-141.
    fn InferTargetAnchorFromScrollOffsets(&mut self) {
        // The anchor has the same normalized position in both viewport rects:
        // anchor = start_size * normalized + start_offset
        // anchor = target_size * normalized + target_offset.
        // Centered zoom has normalized anchor (0.5,0.5); bottom-right has (1,1).
        let width_scale =
            1.0 / (self.TargetViewportSize().width() - self.StartViewportSize().width());
        let height_scale =
            1.0 / (self.TargetViewportSize().height() - self.StartViewportSize().height());
        let normalized = ScaleVector2d(
            self.start_scroll_offset_ - self.target_scroll_offset_,
            width_scale,
            height_scale,
        );
        self.target_anchor_ = self.target_scroll_offset_
            + DenormalizeToViewport(normalized, self.TargetViewportSize());
    }

    // Source: cc/input/page_scale_animation.h:101;
    //         cc/input/page_scale_animation.cc:142-149.
    fn ClampTargetScrollOffset(&mut self) {
        let max_scroll_offset = PointAtOffsetFromOrigin(
            RectF::FromSize(self.root_layer_size_).bottom_right()
                - RectF::FromSize(self.TargetViewportSize()).bottom_right(),
        );
        self.target_scroll_offset_.SetToMin(max_scroll_offset);
        self.target_scroll_offset_.SetToMax(PointF::default());
    }

    // Source: cc/input/page_scale_animation.h:105;
    //         cc/input/page_scale_animation.cc:150-153.
    fn StartViewportSize(&self) -> SizeF {
        ScaleSize(self.viewport_size_, 1.0 / self.start_page_scale_factor_)
    }
    // Source: cc/input/page_scale_animation.h:106;
    //         cc/input/page_scale_animation.cc:154-157.
    fn TargetViewportSize(&self) -> SizeF {
        ScaleSize(self.viewport_size_, 1.0 / self.target_page_scale_factor_)
    }
    // Source: cc/input/page_scale_animation.h:108;
    //         cc/input/page_scale_animation.cc:158-161.
    fn ViewportSizeAt(&self, interp: f32) -> SizeF {
        ScaleSize(self.viewport_size_, 1.0 / self.PageScaleFactorAt(interp))
    }
    // Source: cc/input/page_scale_animation.h:75-77;
    //         cc/input/page_scale_animation.cc:162-165.
    pub fn IsAnimationStarted(&self) -> bool {
        self.start_time_ > TimeTicks::default()
    }
    // Source: cc/input/page_scale_animation.h:78-79;
    //         cc/input/page_scale_animation.cc:166-170.
    pub fn StartAnimation(&mut self, time: TimeTicks) {
        debug_assert!(self.start_time_.is_null());
        self.start_time_ = time;
    }
    // Source: cc/input/page_scale_animation.h:80-82;
    //         cc/input/page_scale_animation.cc:171-175.
    pub fn ScrollOffsetAtTime(&self, time: TimeTicks) -> PointF {
        debug_assert!(!self.start_time_.is_null());
        self.ScrollOffsetAt(self.InterpAtTime(time))
    }
    // Source: cc/input/page_scale_animation.h:83;
    //         cc/input/page_scale_animation.cc:176-180.
    pub fn PageScaleFactorAtTime(&self, time: TimeTicks) -> f32 {
        debug_assert!(!self.start_time_.is_null());
        self.PageScaleFactorAt(self.InterpAtTime(time))
    }
    // Source: cc/input/page_scale_animation.h:84-85;
    //         cc/input/page_scale_animation.cc:181-185.
    pub fn IsAnimationCompleteAtTime(&self, time: TimeTicks) -> bool {
        debug_assert!(!self.start_time_.is_null());
        time >= self.end_time()
    }
    // Source: cc/input/page_scale_animation.h:107;
    //         cc/input/page_scale_animation.cc:186-195.
    fn InterpAtTime(&self, monotonic_time: TimeTicks) -> f32 {
        debug_assert!(!self.start_time_.is_null());
        debug_assert!(monotonic_time >= self.start_time_);
        if self.IsAnimationCompleteAtTime(monotonic_time) {
            return 1.0;
        }
        let normalized_time = (monotonic_time - self.start_time_) / self.duration_;
        self.timing_function_.Solve(normalized_time) as f32
    }
    // Source: cc/input/page_scale_animation.h:109;
    //         cc/input/page_scale_animation.cc:196-204.
    fn ScrollOffsetAt(&self, interp: f32) -> PointF {
        if interp <= 0.0 {
            return self.start_scroll_offset_;
        }
        if interp >= 1.0 {
            return self.target_scroll_offset_;
        }
        self.AnchorAt(interp) - self.ViewportRelativeAnchorAt(interp)
    }
    // Source: cc/input/page_scale_animation.h:110;
    //         cc/input/page_scale_animation.cc:205-211.
    fn AnchorAt(&self, interp: f32) -> PointF {
        // Interpolate from start to target anchor in absolute space.
        PointAtOffsetFromOrigin(InterpolateBetween(
            self.start_anchor_.OffsetFromOrigin(),
            self.target_anchor_.OffsetFromOrigin(),
            interp,
        ))
    }
    // Source: cc/input/page_scale_animation.h:111;
    //         cc/input/page_scale_animation.cc:212-226.
    fn ViewportRelativeAnchorAt(&self, interp: f32) -> Vector2dF {
        // Interpolate from start to target anchor in normalized space.
        let start_normalized = NormalizeFromViewport(
            self.start_anchor_ - self.start_scroll_offset_,
            self.StartViewportSize(),
        );
        let target_normalized = NormalizeFromViewport(
            self.target_anchor_ - self.target_scroll_offset_,
            self.TargetViewportSize(),
        );
        let interp_normalized = InterpolateBetween(start_normalized, target_normalized, interp);
        DenormalizeToViewport(interp_normalized, self.ViewportSizeAt(interp))
    }
    // Source: cc/input/page_scale_animation.h:112-113;
    //         cc/input/page_scale_animation.cc:227-240.
    fn PageScaleFactorAt(&self, interp: f32) -> f32 {
        if interp <= 0.0 {
            return self.start_page_scale_factor_;
        }
        if interp >= 1.0 {
            return self.target_page_scale_factor_;
        }
        // Linearly interpolate the magnitude in log scale.
        let mut diff = self.target_page_scale_factor_ / self.start_page_scale_factor_;
        let mut log_diff = diff.ln();
        log_diff *= interp;
        diff = log_diff.exp();
        self.start_page_scale_factor_ * diff
    }
    // Source: cc/input/page_scale_animation.h:86-88.
    pub fn start_time(&self) -> TimeTicks {
        self.start_time_
    }
    // Source: cc/input/page_scale_animation.h:89.
    pub fn duration(&self) -> TimeDelta {
        self.duration_
    }
    // Source: cc/input/page_scale_animation.h:90.
    pub fn end_time(&self) -> TimeTicks {
        self.start_time_ + self.duration_
    }
    // Source: cc/input/page_scale_animation.h:91.
    pub fn target_scroll_offset(&self) -> PointF {
        self.target_scroll_offset_
    }
    // Source: cc/input/page_scale_animation.h:92-93.
    pub fn target_page_scale_factor(&self) -> f32 {
        self.target_page_scale_factor_
    }
}
