// Copyright 2014 Google Inc. BSD-style license.
// Source: include/core/SkBlurTypes.h:11-15
// Source: include/core/SkBlurTypes.h:17-17
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkBlurStyle {
    kNormal_SkBlurStyle,
    kSolid_SkBlurStyle,
    kOuter_SkBlurStyle,
    kInner_SkBlurStyle,
}
pub const kLastEnum_SkBlurStyle: SkBlurStyle = SkBlurStyle::kInner_SkBlurStyle;
