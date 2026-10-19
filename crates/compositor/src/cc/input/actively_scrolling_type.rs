// Translation status: complete
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/actively_scrolling_type.h:1-9,16-18.
// Header guards are represented by this Rust module; namespace cc is its parent.

// Source: cc/input/actively_scrolling_type.h:10-14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ActivelyScrollingType {
    kNone,
    kPrecise,
    kAnimated,
}
