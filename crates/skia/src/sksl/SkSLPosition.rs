// Copyright 2021 Google LLC. BSD-style license.
impl Position {
    // Source: src/sksl/SkSLPosition.h:20-22
    pub const fn new() -> Self {
        Self { bits: 0x00ff_ffff }
    }
    // Source: src/sksl/SkSLPosition.h:24-32
    pub fn Range(startOffset: i32, endOffset: i32) -> Self {
        crate::SkASSERT!(@source b"src/sksl/SkSLPosition.h\0".as_ptr(),25; @condition b"startOffset <= endOffset\0".as_ptr(); startOffset <= endOffset);
        debug_assert!(startOffset <= Self::kMaxOffset);
        let length = endOffset - startOffset;
        let mut result = Self::new();
        result.bits = (startOffset as u32 & 0x00ff_ffff)
            | ((if length <= 0xff { length } else { 0xff } as u32 & 0xff) << 24);
        result
    }
    // Source: src/sksl/SkSLPosition.h:34-36
    pub fn valid(&self) -> bool {
        self.offset() != -1
    }
    // Source: src/sksl/SkSLPosition.h:40-43
    pub fn startOffset(&self) -> i32 {
        crate::SkASSERT!(@source b"src/sksl/SkSLPosition.h\0".as_ptr(),41; @condition b"this->valid()\0".as_ptr(); self.valid());
        self.offset()
    }
    // Source: src/sksl/SkSLPosition.h:45-48
    pub fn endOffset(&self) -> i32 {
        crate::SkASSERT!(@source b"src/sksl/SkSLPosition.h\0".as_ptr(),46; @condition b"this->valid()\0".as_ptr(); self.valid());
        self.offset() + (self.bits >> 24) as i32
    }
    // Source: src/sksl/SkSLPosition.h:51-59
    pub fn rangeThrough(&self, end: Self) -> Self {
        if self.offset() == -1 || end.offset() == -1 {
            return *self;
        }
        crate::SkASSERTF!(@source b"src/sksl/SkSLPosition.h\0".as_ptr(),55;
            @condition b"this->startOffset() <= end.startOffset() && this->endOffset() <= end.endOffset()\0".as_ptr();
            self.startOffset()<=end.startOffset()&&self.endOffset()<=end.endOffset(),
            "Invalid range: (%d-%d) - (%d-%d)\n",self.startOffset(),self.endOffset(),end.startOffset(),end.endOffset());
        Self::Range(self.startOffset(), end.endOffset())
    }
    // Source: src/sksl/SkSLPosition.h:62-65
    pub fn after(&self) -> Self {
        let endOffset = self.endOffset();
        Self::Range(endOffset, endOffset + 1)
    }
}
// Source: src/sksl/SkSLPosition.h:67-73
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.offset() == other.offset() && self.bits >> 24 == other.bits >> 24
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for Position {}
// Source: src/sksl/SkSLPosition.h:75-89
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.offset().cmp(&other.offset()))
    }
    fn gt(&self, other: &Self) -> bool {
        self.offset() > other.offset()
    }
    fn ge(&self, other: &Self) -> bool {
        self.offset() >= other.offset()
    }
    fn lt(&self, other: &Self) -> bool {
        self.offset() < other.offset()
    }
    fn le(&self, other: &Self) -> bool {
        self.offset() <= other.offset()
    }
}
// Source: src/sksl/SkSLPosition.h:91-91
impl Position {
    pub const kMaxOffset: i32 = 0x7fffff;
}
// Source: src/sksl/SkSLPosition.h:94-96
// Adapted: one integer stores the actual signed24/unsigned8 bitfields. Accesses
// sign-extend the low24 bits; native four-byte representation is retained.
#[derive(Clone, Copy, Debug)]
pub struct Position {
    bits: u32,
}
impl Position {
    fn offset(&self) -> i32 {
        ((self.bits << 8) as i32) >> 8
    }
}
impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/sksl/SkSLPosition.h:98-102
#[derive(Clone, Copy, Debug, Default)]
pub struct ForLoopPositions {
    pub initPosition: Position,
    pub conditionPosition: Position,
    pub nextPosition: Position,
}
impl Position {
    // Source: src/sksl/SkSLPosition.cpp:14-32
    // Option preserves native null string_view versus a nonnull empty byte view.
    pub fn line(&self, source: Option<&[u8]>) -> i32 {
        crate::SkASSERT!(@source b"src/sksl/SkSLPosition.cpp\0".as_ptr(),15; @condition b"this->valid()\0".as_ptr(); self.valid());
        if self.offset() == -1 {
            return -1;
        }
        let Some(source) = source else {
            return -1;
        };
        debug_assert!(self.offset() <= source.len() as i32);
        let offset = self.offset().min(source.len() as i32);
        let mut line = 1;
        for i in 0..offset {
            if source[i as usize] == b'\n' {
                line += 1;
            }
        }
        line
    }
}
