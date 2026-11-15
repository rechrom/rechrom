//! Resolved CSS underline placement shared by Layout offset computation and Paint.

#![allow(non_camel_case_types)]

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:37-42.
// This scalar crosses the existing Layout/Paint package boundary, so its value
// identity lives in the shared foundation crate while Paint retains the
// resolving policy and public re-export.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResolvedUnderlinePosition {
    #[default]
    kNearAlphabeticBaselineAuto,
    kNearAlphabeticBaselineFromFont,
    kUnder,
    kOver,
}
