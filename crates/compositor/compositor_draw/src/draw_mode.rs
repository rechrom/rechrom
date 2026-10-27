// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/draw_mode.h:1-19.
// Source: cc/layers/draw_mode.h:10-15.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawMode {
    #[default]
    DRAW_MODE_NONE = 0,
    DRAW_MODE_HARDWARE = 1,
    DRAW_MODE_SOFTWARE = 2,
    DRAW_MODE_RESOURCELESS_SOFTWARE = 3,
}
