// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: ui/base/mojom/menu_source_type.mojom:7-29
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MenuSourceType {
    kNone = 0,
    kMouse = 1,
    kKeyboard = 2,
    kTouch = 3,
    kTouchEditMenu = 4,
    kLongPress = 5,
    kLongTap = 6,
    kTouchHandle = 7,
    kStylus = 8,
    kAdjustSelection = 9,
    kAdjustSelectionReset = 10,
}
