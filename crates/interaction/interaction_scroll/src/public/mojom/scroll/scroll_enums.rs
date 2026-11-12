// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/mojom/scroll/scroll_enums.mojom:7-25

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollType {
    kUser = 0,
    kProgrammatic,
    kClamping,
    kCompositor,
    kAnchoring,
    kScrollStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBehavior {
    kAuto = 0,
    kInstant,
    kSmooth,
}
