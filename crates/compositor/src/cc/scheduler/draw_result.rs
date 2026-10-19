// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/scheduler/draw_result.h:10-21
// These values are reported in UMA. Entries must never be renumbered, and
// numeric values must never be reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawResult {
    kInvalidResult = 0,
    kSuccess = 1,
    kAbortedCheckerboardAnimations = 2,
    kAbortedMissingHighResContent = 3,
    kAbortedCantDraw = 4,
    kAbortedDrainingPipeline = 5,
}

// Source: cc/scheduler/draw_result.h:20
impl DrawResult {
    pub const kMaxValue: Self = Self::kAbortedDrainingPipeline;
}
