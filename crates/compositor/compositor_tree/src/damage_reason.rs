// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/damage_reason.h:1-30
use crate::enum_set::{EnumSet, EnumValue};

// Source: cc/trees/damage_reason.h:12-23
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DamageReason {
    kUntracked,
    kAnimatedImage,
    kScrollbarFadeOutAnimation,
    kVideoLayer,
    kCompositorScroll,
}

impl DamageReason {
    // Source: cc/trees/damage_reason.h:22
    pub const kMaxValue: Self = Self::kCompositorScroll;
}

// Rust enum conversion for the source's base::EnumSet typed boundary.
impl EnumValue for DamageReason {
    fn to_integer(self) -> i32 {
        self as i32
    }
    fn from_integer(value: i32) -> Self {
        match value {
            0 => Self::kUntracked,
            1 => Self::kAnimatedImage,
            2 => Self::kScrollbarFadeOutAnimation,
            3 => Self::kVideoLayer,
            4 => Self::kCompositorScroll,
            _ => panic!("damage reason outside the source enum range"),
        }
    }
}

// Source: cc/trees/damage_reason.h:25-26
pub type DamageReasonSet =
    EnumSet<DamageReason, { DamageReason::kUntracked as i32 }, { DamageReason::kMaxValue as i32 }>;
