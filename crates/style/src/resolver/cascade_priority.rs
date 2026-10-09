// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_priority.h:15-244
#![allow(non_upper_case_globals)]

use super::cascade_origin::CascadeOrigin;
use std::cmp::Ordering;

// cpp: cascade_priority.h:21-27
pub const fn EncodeOriginImportance(origin: CascadeOrigin, important: bool) -> u32 {
    if important {
        origin as u32 ^ 0xF
    } else {
        origin as u32
    }
}

// cpp: cascade_priority.h:33-39
pub const fn EncodeTreeOrder(tree_order: u16, important: bool) -> u32 {
    if important {
        tree_order as u32 ^ 0xFFFF
    } else {
        tree_order as u32
    }
}

// cpp: cascade_priority.h:45-51
pub const fn EncodeLayerOrder(layer_order: u16, important: bool) -> u64 {
    if important {
        layer_order as u64 ^ 0xFFFF
    } else {
        layer_order as u64
    }
}

// Priorities contain origin/importance, tree order, inline/try style, layer
// order, matched-rule/declaration indices, and the already-applied bit.
// Rule indices address sorted matched rules, retaining specificity, proximity,
// style sheet order, and rule order in the cascade comparison.
#[derive(Clone, Copy, Debug, Default)]
pub struct CascadePriority {
    // cpp: cascade_priority.h:227-240
    pub(crate) low_bits_: u64,
    pub(crate) high_bits_: u32,
}

impl CascadePriority {
    // cpp: cascade_priority.h:81-100
    pub const kImportantBit: u32 = 19;
    pub const kOriginImportanceOffset: u32 = 16;
    pub const kIsTryTacticsStyleOffset: u32 = 51;
    pub const kIsTryStyleOffset: u32 = 50;
    pub const kIsInlineStyleOffset: u32 = 49;
    pub const kLayerOrderOffset: u32 = 33;
    pub const kDeclarationIndexOffset: u32 = 1;
    pub const kRuleIndexOffset: u32 = 17;

    pub const kOriginImportanceMask: u64 = 0xF << Self::kOriginImportanceOffset;
    pub const kTreeOrderMask: u64 = 0xFFFF;
    pub const kLayerOrderMask: u64 = 0xFFFF << Self::kLayerOrderOffset;
    pub const kDeclarationIndexMask: u64 = 0xFFFF << Self::kDeclarationIndexOffset;
    pub const kRuleIndexMask: u64 = 0xFFFF << Self::kRuleIndexOffset;
    pub const kAlreadyAppliedMask: u64 = 0x1;

    // cpp: cascade_priority.h:102
    pub const fn new() -> Self {
        Self::FromBits(0, 0)
    }

    // Rust names the three C++ constructor overloads separately.
    // cpp: cascade_priority.h:103-112
    pub const fn FromOrigin(origin: CascadeOrigin) -> Self {
        Self::FromParts(origin, false, 0, false, false, false, 0, 0, 0)
    }

    // cpp: cascade_priority.h:113-122
    pub const fn FromOriginImportance(origin: CascadeOrigin, important: bool) -> Self {
        Self::FromParts(origin, important, 0, false, false, false, 0, 0, 0)
    }

    // cpp: cascade_priority.h:123-132
    pub const fn FromOriginImportanceTreeOrder(
        origin: CascadeOrigin,
        important: bool,
        tree_order: u16,
    ) -> Self {
        Self::FromParts(origin, important, tree_order, false, false, false, 0, 0, 0)
    }

    // cpp: cascade_priority.h:136-156
    #[allow(clippy::too_many_arguments)]
    pub const fn FromParts(
        origin: CascadeOrigin,
        important: bool,
        tree_order: u16,
        is_inline_style: bool,
        is_try_style: bool,
        is_try_tactics_style: bool,
        layer_order: u16,
        rule_index: u16,
        declaration_index: u16,
    ) -> Self {
        Self::FromBits(
            ((rule_index as u64) << Self::kRuleIndexOffset)
                | ((declaration_index as u64) << Self::kDeclarationIndexOffset)
                | (EncodeLayerOrder(layer_order, important) << Self::kLayerOrderOffset)
                | ((is_inline_style as u64) << Self::kIsInlineStyleOffset)
                | ((is_try_style as u64) << Self::kIsTryStyleOffset)
                | ((is_try_tactics_style as u64) << Self::kIsTryTacticsStyleOffset),
            EncodeTreeOrder(tree_order, important)
                | (EncodeOriginImportance(origin, important) << Self::kOriginImportanceOffset),
        )
    }

    // cpp: cascade_priority.h:157-159
    pub const fn WithAlreadyApplied(o: Self, already_applied: bool) -> Self {
        Self::FromBits(
            (o.low_bits_ & !Self::kAlreadyAppliedMask) | already_applied as u64,
            o.high_bits_,
        )
    }

    // cpp: cascade_priority.h:161
    pub const fn IsImportant(self) -> bool {
        (self.high_bits_ >> Self::kImportantBit) & 1 != 0
    }

    // cpp: cascade_priority.h:162-167
    pub const fn GetOrigin(self) -> CascadeOrigin {
        let important_xor = (((!self.high_bits_ >> Self::kImportantBit) & 1).wrapping_sub(1)
            as u64)
            & Self::kOriginImportanceMask;
        let origin =
            ((self.high_bits_ as u64 ^ important_xor) >> Self::kOriginImportanceOffset) as u8;
        // C++ casts back to its enum; safe Rust reconstructs its valid variants.
        match origin {
            0 => CascadeOrigin::kNone,
            1 => CascadeOrigin::kUserAgent,
            2 => CascadeOrigin::kUser,
            3 => CascadeOrigin::kAuthorPresentationalHint,
            4 => CascadeOrigin::kAuthor,
            5 => CascadeOrigin::kAnimation,
            16 => CascadeOrigin::kTransition,
            _ => panic!("invalid encoded cascade origin"),
        }
    }

    // cpp: cascade_priority.h:168-179
    pub const fn HasOrigin(self) -> bool {
        self.GetOrigin() as u8 != CascadeOrigin::kNone as u8
    }
    pub const fn GetDeclarationIndex(self) -> usize {
        ((self.low_bits_ & Self::kDeclarationIndexMask) >> Self::kDeclarationIndexOffset) as usize
    }
    pub const fn GetRuleIndex(self) -> usize {
        ((self.low_bits_ & Self::kRuleIndexMask) >> Self::kRuleIndexOffset) as usize
    }
    pub const fn IsAlreadyApplied(self) -> bool {
        self.low_bits_ & Self::kAlreadyAppliedMask != 0
    }
    pub const fn IsInlineStyle(self) -> bool {
        (self.low_bits_ >> Self::kIsInlineStyleOffset) & 1 != 0
    }
    pub const fn IsTryStyle(self) -> bool {
        (self.low_bits_ >> Self::kIsTryStyleOffset) & 1 != 0
    }

    // cpp: cascade_priority.h:181-206
    pub const fn ForLayerComparison(self) -> u64 {
        let mut bits = (self.low_bits_ >> 32) | ((self.high_bits_ as u64) << 32);
        if bits & (1u64 << (Self::kImportantBit + 32)) != 0 {
            // Reverse importance's flips while retaining transition and inline bits.
            bits ^= Self::kOriginImportanceMask << 32;
            bits ^= Self::kTreeOrderMask << 32;
            bits ^= Self::kLayerOrderMask >> 32;
        }
        bits >>= Self::kLayerOrderOffset - 32;
        bits
    }

    // cpp: cascade_priority.h:224-225
    pub(crate) const fn FromBits(low_bits: u64, high_bits: u32) -> Self {
        Self {
            low_bits_: low_bits,
            high_bits_: high_bits,
        }
    }
}

// cpp: cascade_priority.h:208-218
impl PartialEq for CascadePriority {
    fn eq(&self, o: &Self) -> bool {
        self.high_bits_ == o.high_bits_ && self.low_bits_ == o.low_bits_
    }
}
impl Eq for CascadePriority {}
impl PartialOrd for CascadePriority {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for CascadePriority {
    fn cmp(&self, o: &Self) -> Ordering {
        self.high_bits_
            .cmp(&o.high_bits_)
            .then_with(|| self.low_bits_.cmp(&o.low_bits_))
    }
}

#[cfg(test)]
#[path = "cascade_priority_test.rs"]
mod tests;
