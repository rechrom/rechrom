// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/hit_test_phase.h:1-21
// Includes, include guards and namespaces are represented by Rust modules.

// Source: third_party/blink/renderer/core/layout/hit_test_phase.h:10-17
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTestPhase {
    kSelfBlockBackground,
    kDescendantBlockBackgrounds,
    kFloat,
    kForeground,
}
