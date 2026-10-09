// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/renderer/platform/graphics/color_space_gamut.h:16-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpaceGamut {
    SRGB = 3,
    P3 = 5,
    BT2020 = 8,
}
