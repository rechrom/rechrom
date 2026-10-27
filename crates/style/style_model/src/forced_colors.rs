// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/public/common/css/forced_colors.h:11-15
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForcedColors {
    kNone,
    kActive,
}
impl ForcedColors {
    pub const kMaxValue: Self = Self::kActive;
}
