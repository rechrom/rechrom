// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: ui/events/types/scroll_types.h:12-20
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollGranularity {
    kScrollByPrecisePixel = 0,
    kScrollByPixel,
    kScrollByLine,
    kScrollByPage,
    kScrollByDocument,
}

impl ScrollGranularity {
    // Source: ui/events/types/scroll_types.h:14,19
    pub const kFirstScrollGranularity: Self = Self::kScrollByPrecisePixel;
    pub const kMaxValue: Self = Self::kScrollByDocument;
}
