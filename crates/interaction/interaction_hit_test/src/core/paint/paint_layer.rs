// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: third_party/blink/renderer/core/paint/paint_layer.h:77-91
// An integer mask preserves the source's bitwise combinations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PaintLayerIteration(pub u32);

impl PaintLayerIteration {
    pub const kNegativeZOrderChildren: Self = Self(1);
    pub const kNormalFlowChildren: Self = Self(1 << 1);
    pub const kPositiveZOrderChildren: Self = Self(1 << 2);
    pub const kStackedChildren: Self = Self(1 | (1 << 2));
    pub const kNormalFlowAndPositiveZOrderChildren: Self = Self((1 << 1) | (1 << 2));
    pub const kAllChildren: Self = Self(1 | (1 << 1) | (1 << 2));
}
