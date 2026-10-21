// Copyright 2011 Google Inc. BSD-style license.
// Source: src/core/SkBlitRow.h:16-19
// The native enum declaration is explicit; exported integer constants below
// retain the existing unsigned factory bitmask API and refer to these values.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flags32 {
    kGlobalAlpha_Flag32 = 1 << 0,
    kSrcPixelAlpha_Flag32 = 1 << 1,
}
// Source: src/core/SkBlitRow.h:14-19
pub struct SkBlitRow;
impl SkBlitRow {
    pub const kGlobalAlpha_Flag32: u32 = Flags32::kGlobalAlpha_Flag32 as u32;
    pub const kSrcPixelAlpha_Flag32: u32 = Flags32::kSrcPixelAlpha_Flag32 as u32;
}
// Source: src/core/SkBlitRow.h:27-27
pub type Proc32 = fn(&mut [u32], &[u32], usize, u32);

// Source: src/core/SkBlitRow.h:27-27
// Native pointer/count ABI, distinct from the existing borrowed-slice projection.
// The caller retains source/destination pixel leases for the entire invocation.
pub type RawProc32 = unsafe fn(*mut u32, *const u32, i32, u32);
