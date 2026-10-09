// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/public/mojom/css/preferred_contrast.mojom:8-13
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferredContrast {
    kMore,
    kLess,
    kNoPreference,
    kCustom,
}
