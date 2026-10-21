// Copyright 2012 Google Inc. BSD-style license.
// Source: src/sfnt/SkOTTable_OS_2.h:20-25
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2 {
    pub version: Version,
}
impl SkOTTableOS2 {
    pub const TAG0: i8 = b'O' as i8;
    pub const TAG1: i8 = b'S' as i8;
    pub const TAG2: i8 = b'/' as i8;
    pub const TAG3: i8 = b'2' as i8;
    pub const TAG: u32 = super::SkOTTableTypes::SkOTTableTAG::<Self>::value;
}
// Source: src/sfnt/SkOTTable_OS_2.h:27-28
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union Version {
    pub version: u16,
    // Source: src/sfnt/SkOTTable_OS_2.h:31-31
    pub vA: VA,
    // Source: src/sfnt/SkOTTable_OS_2.h:32-32
    pub v0: V0,
    // Source: src/sfnt/SkOTTable_OS_2.h:33-33
    pub v1: V1,
    // Source: src/sfnt/SkOTTable_OS_2.h:34-34
    pub v2: V2,
    // Source: src/sfnt/SkOTTable_OS_2.h:36-36
    pub v3: V3,
    // Source: src/sfnt/SkOTTable_OS_2.h:38-38
    pub v4: V4,
}
// Native zero-field derived wrappers become aliases to the exact packed base
// type. Each complete version retains its own independently translated layout.
// Source: src/sfnt/SkOTTable_OS_2.h:31-31
pub use super::SkOTTable_OS_2_VA::SkOTTableOS2_VA as VA;
// Source: src/sfnt/SkOTTable_OS_2.h:32-32
pub use super::SkOTTable_OS_2_V0::SkOTTableOS2_V0 as V0;
// Source: src/sfnt/SkOTTable_OS_2.h:33-33
pub use super::SkOTTable_OS_2_V1::SkOTTableOS2_V1 as V1;
// Source: src/sfnt/SkOTTable_OS_2.h:34-34
pub use super::SkOTTable_OS_2_V2::SkOTTableOS2_V2 as V2;
// Source: src/sfnt/SkOTTable_OS_2.h:36-36
pub use super::SkOTTable_OS_2_V3::SkOTTableOS2_V3 as V3;
// Source: src/sfnt/SkOTTable_OS_2.h:38-38
pub use super::SkOTTable_OS_2_V4::SkOTTableOS2_V4 as V4;
impl super::SkOTTableTypes::SkOTTagBytes for SkOTTableOS2 {
    const TAG0: i8 = Self::TAG0;
    const TAG1: i8 = Self::TAG1;
    const TAG2: i8 = Self::TAG2;
    const TAG3: i8 = Self::TAG3;
}
// Source: src/sfnt/SkOTTable_OS_2.h:45-45
const _: () = assert!(core::mem::size_of::<VA>() == 68);
// Source: src/sfnt/SkOTTable_OS_2.h:46-46
const _: () = assert!(core::mem::size_of::<V0>() == 78);
// Source: src/sfnt/SkOTTable_OS_2.h:47-47
const _: () = assert!(core::mem::size_of::<V1>() == 86);
// Source: src/sfnt/SkOTTable_OS_2.h:48-48
const _: () = assert!(core::mem::size_of::<V2>() == 96);
// Source: src/sfnt/SkOTTable_OS_2.h:49-49
const _: () = assert!(core::mem::size_of::<V3>() == 96);
// Source: src/sfnt/SkOTTable_OS_2.h:50-50
const _: () = assert!(core::mem::size_of::<V4>() == 96);
