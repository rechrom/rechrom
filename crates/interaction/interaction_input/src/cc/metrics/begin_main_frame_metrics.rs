// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: cc/metrics/begin_main_frame_metrics.h:15-38
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BeginMainFrameReason {
    kOther = 0,
    kRAFOrServiceScriptedAnimations = 1,
    kVideoFrameCallback = 2,
    kAnimation = 3,
    kStylePaintOrLayoutInvalidation = 4,
    kScroll = 5,
    kInput = 6,
    kMainThreadScroll = 7,
    kDelayedTimerFired = 8,
}

impl BeginMainFrameReason {
    // Source: cc/metrics/begin_main_frame_metrics.h:22-23,26,29-31,36
    pub const kRAF: Self = Self::kRAFOrServiceScriptedAnimations;
    pub const kServiceScriptedAnimations: Self = Self::kRAFOrServiceScriptedAnimations;
    pub const kCSSAnimation: Self = Self::kAnimation;
    pub const kStyleInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kPaintInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kLayoutInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kMaxValue: Self = Self::kDelayedTimerFired;
}
