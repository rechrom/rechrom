// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/scroll_source_type.h:1-20

// Source: cc/trees/scroll_source_type.h:10-16
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ScrollSourceType {
    kNone,
    kAbsoluteScroll,
    kRelativeScroll,
    kStationaryScroll,
}
