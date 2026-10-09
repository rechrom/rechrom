// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/public/mojom/webpreferences/web_preferences.mojom:15-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerType {
    kPointerNone = 1,
    kPointerCoarseType = 2,
    kPointerFineType = 4,
}
impl PointerType {
    pub const kPointerFirstType: Self = Self::kPointerNone;
}

// cpp: third_party/blink/public/mojom/webpreferences/web_preferences.mojom:22-26
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverType {
    kHoverNone = 1,
    kHoverHoverType = 2,
}
impl HoverType {
    pub const kHoverFirstType: Self = Self::kHoverNone;
}

// cpp: third_party/blink/public/mojom/webpreferences/web_preferences.mojom:33-36
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputDeviceUpdateAbilityType {
    kSlowType,
    kFastType,
}
