// Copyright 2023 Google LLC. BSD-style license.

// Source: include/core/SkTextureCompressionType.h:20-25
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkTextureCompressionType {
    kNone,
    kETC2_RGB8_UNORM,
    kBC1_RGB8_UNORM,
    kBC1_RGBA8_UNORM,
}
impl SkTextureCompressionType {
    // Source: include/core/SkTextureCompressionType.h:26-26
    pub const kLast: Self = Self::kBC1_RGBA8_UNORM;
    // Source: include/core/SkTextureCompressionType.h:27-27
    pub const kETC1_RGB8: Self = Self::kETC2_RGB8_UNORM;
}
