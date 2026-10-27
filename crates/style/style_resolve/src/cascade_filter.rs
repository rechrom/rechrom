// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_filter.h:13-75

use crate::properties::css_property::{CSSProperty, Flags};

// Pass only properties with every required flag set. Flags with deliberately
// inverted counterparts allow rejection of all properties by requiring both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CascadeFilter {
    // cpp: cascade_filter.h:73
    required_bits_: Flags,
}

impl CascadeFilter {
    // cpp: cascade_filter.h:25
    pub const fn new() -> Self {
        Self { required_bits_: 0 }
    }

    // cpp: cascade_filter.h:34
    pub const fn FromFlag(flag: Flags) -> Self {
        Self {
            required_bits_: flag,
        }
    }

    // cpp: cascade_filter.h:54-57
    pub const fn Add(self, flag: Flags) -> Self {
        let required_bits = self.required_bits_ | flag;
        Self::FromRequiredBits(required_bits)
    }

    // cpp: cascade_filter.h:59-61
    pub fn Accepts(self, property: &CSSProperty) -> bool {
        (property.GetFlags() & self.required_bits_) == self.required_bits_
    }

    // cpp: cascade_filter.h:63-65
    pub const fn Requires(self, flag: Flags) -> bool {
        self.required_bits_ & flag != 0
    }

    // cpp: cascade_filter.h:67
    pub const fn IsEmpty(self) -> bool {
        self.required_bits_ == 0
    }

    // cpp: cascade_filter.h:70-71
    const fn FromRequiredBits(required_bits: Flags) -> Self {
        Self {
            required_bits_: required_bits,
        }
    }
}

#[cfg(test)]
#[path = "cascade_filter_test.rs"]
mod tests;
