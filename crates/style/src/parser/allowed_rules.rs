// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:15-79

#![allow(non_camel_case_types)]

use super::css_at_rule_id::CSSAtRuleID;
use std::ops::BitOr;

// https://drafts.csswg.org/css-syntax/#qualified-rule
// cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:15-23
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualifiedRuleType {
    // A regular style rule, e.g. .foo:hover { ... }.
    kStyle,
    // A keyframe rule found within @keyframes, e.g. 50% { ... }.
    kKeyframe,
    kCount, // Must go last.
}

// This class represents which kinds of rules are valid in a certain context.
// For example, @namespace rules are only valid top-level, and @ornaments is
// only valid within @font-feature-values.
// cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:25-79
// Equality maps allowed_rules.h:50-52.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllowedRules {
    // Bits for CSSAtRuleID values are stored first (lower bits), then for
    // QualifiedRuleType values (higher bits).
    bits_: u64,
}

// C++ overloads Has/Remove for the two concrete enums. This sealed trait
// expresses those same accepted arguments without admitting other rule types.
mod sealed {
    pub trait Sealed {}
}
pub trait AllowedRule: sealed::Sealed {
    fn Bit(self) -> u64;
}
impl sealed::Sealed for CSSAtRuleID {}
impl AllowedRule for CSSAtRuleID {
    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:63-65
    fn Bit(self) -> u64 {
        self as u64
    }
}
impl sealed::Sealed for QualifiedRuleType {}
impl AllowedRule for QualifiedRuleType {
    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:66-69
    fn Bit(self) -> u64 {
        CSSAtRuleID::kCount as u64 + self as u64
    }
}

impl AllowedRules {
    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:39
    pub const fn new() -> Self {
        Self { bits_: 0 }
    }

    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:29-36,40-42
    pub const fn FromAtRules(list: &[CSSAtRuleID]) -> Self {
        let mut bits = 0;
        let mut index = 0;
        while index < list.len() {
            bits |= 1u64 << list[index] as u64;
            index += 1;
        }
        Self { bits_: bits }
    }

    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:29-36,43-45
    pub const fn FromQualifiedRules(list: &[QualifiedRuleType]) -> Self {
        let mut bits = 0;
        let mut index = 0;
        while index < list.len() {
            bits |= 1u64 << list[index] as u64;
            index += 1;
        }
        Self {
            bits_: bits << CSSAtRuleID::kCount as u64,
        }
    }

    // cpp: allowed_rules.h:46-48. Rust's BitOr trait cannot be invoked from a
    // const initializer, so mapped constexpr rule sets use this equivalent.
    pub const fn Union(self, other: Self) -> Self {
        Self {
            bits_: self.bits_ | other.bits_,
        }
    }

    // cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:54-55,57-58
    pub fn Remove<Rule: AllowedRule>(&mut self, id: Rule) {
        self.bits_ &= !(1u64 << id.Bit());
    }

    pub fn Has<Rule: AllowedRule>(&self, id: Rule) -> bool {
        ((self.bits_ >> id.Bit()) & 1) != 0
    }
}

// cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:46-48,61
impl BitOr for AllowedRules {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self {
            bits_: self.bits_ | other.bits_,
        }
    }
}

// TODO(andruud): A wider bitset can lift this restriction once needed.
// cpp: third_party/blink/renderer/core/css/parser/allowed_rules.h:71-74
const _: () = assert!(CSSAtRuleID::kCount as u64 + QualifiedRuleType::kCount as u64 <= 64);
