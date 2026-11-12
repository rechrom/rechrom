// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: cc/input/scroll_utils.h:20-27,44
// Source: cc/input/scroll_utils.cc:50-56

pub const kPixelsPerLineStep: i32 = 40;
pub const kMinFractionToStepWhenPaging: f32 = 0.875;
#[cfg(target_os = "macos")]
pub const kMaxOverlapBetweenPages: i32 = 40;
#[cfg(not(target_os = "macos"))]
pub const kMaxOverlapBetweenPages: i32 = i32::MAX;

pub fn CalculatePageStep(length: i32) -> i32 {
    let min_page_step = (length as f32 * kMinFractionToStepWhenPaging) as i32;
    let page_step = min_page_step.max(length - kMaxOverlapBetweenPages);
    page_step.max(1)
}
