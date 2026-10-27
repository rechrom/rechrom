#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: third_party/blink/renderer/core/css/valid_property_filter.h:14-46
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum ValidPropertyFilter {
    #[default]
    kNoFilter,
    kCue,
    kFirstLetter,
    kFirstLine,
    kMarker,
    kHighlightLegacy,
    kHighlight,
    kPositionTry,
    kPageContext,
}
