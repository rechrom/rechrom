// Copyright 2025 Google LLC. BSD-style license.
// Source: src/core/SkDrawTypes.h:13-19
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkDrawCoverage {
    kNo = 0,
    kYes = 1,
}
pub const kSkBlitterContextSize: usize = 3332;
