// Copyright 2012 Google Inc. BSD-style license.
// Rust declaration grouping: native Head fields retain their original order;
// nested Flags/MacStyle unions retain both complete two-byte projections. The
// native eight-bit allocation units become high/low bytes with one named getter
// and setter per bit. Constants and nested definitions follow native order within
// their groups; typed transparent enum values preserve unknown wire integers.
use super::SkOTTableTypes::*;
// Source: src/sfnt/SkOTTable_head.h:16-16
// Source: src/sfnt/SkOTTable_head.h:23-23
// Source: src/sfnt/SkOTTable_head.h:25-25
// Source: src/sfnt/SkOTTable_head.h:27-28
// Source: src/sfnt/SkOTTable_head.h:74-80
// Source: src/sfnt/SkOTTable_head.h:115-116
// Source: src/sfnt/SkOTTable_head.h:125-125
// Source: src/sfnt/SkOTTable_head.h:131-131
// Source: src/sfnt/SkOTTable_head.h:136-136
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableHead {
    pub version: SK_OT_Fixed,
    pub fontRevision: SK_OT_Fixed,
    pub checksumAdjustment: SK_OT_ULONG,
    pub magicNumber: SK_OT_ULONG,
    pub flags: Flags,
    pub unitsPerEm: SK_OT_USHORT,
    pub created: SK_OT_LONGDATETIME,
    pub modified: SK_OT_LONGDATETIME,
    pub xMin: SK_OT_SHORT,
    pub yMin: SK_OT_SHORT,
    pub xMax: SK_OT_SHORT,
    pub yMax: SK_OT_SHORT,
    pub macStyle: MacStyle,
    pub lowestRecPPEM: SK_OT_USHORT,
    pub fontDirectionHint: FontDirectionHint,
    pub indexToLocFormat: IndexToLocFormat,
    pub glyphDataFormat: GlyphDataFormat,
}
// Source: src/sfnt/SkOTTable_head.h:17-21
impl SkOTTagBytes for SkOTTableHead {
    const TAG0: i8 = b'h' as i8;
    const TAG1: i8 = b'e' as i8;
    const TAG2: i8 = b'a' as i8;
    const TAG3: i8 = b'd' as i8;
}
impl SkOTTableHead {
    pub const TAG0: i8 = b'h' as i8;
    pub const TAG1: i8 = b'e' as i8;
    pub const TAG2: i8 = b'a' as i8;
    pub const TAG3: i8 = b'd' as i8;
    pub const TAG: u32 = SkOTTableTAG::<Self>::value;
    // Source: src/sfnt/SkOTTable_head.h:24-24
    pub const version1: i32 = (0x00010000i32).to_be();
    // Source: src/sfnt/SkOTTable_head.h:26-26
    pub const fontChecksum: u32 = 0xB1B0AFBA;
    // Source: src/sfnt/SkOTTable_head.h:29-29
    pub const magicNumberConst: u32 = (0x5F0F3CF5u32).to_be();
}
// Source: src/sfnt/SkOTTable_head.h:30-73
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union Flags {
    pub field: FlagsField,
    pub raw: FlagsRaw,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FlagsField {
    pub high: u8,
    pub low: u8,
}
impl FlagsField {
    pub fn GXMetamorphosis_Apple(&self) -> bool {
        self.high & (1 << 0) != 0
    }
    pub fn set_GXMetamorphosis_Apple(&mut self, v: bool) {
        self.high = (self.high & !(1 << 0)) | ((v as u8) << 0);
    }
    pub fn HasStrongRTL_Apple(&self) -> bool {
        self.high & (1 << 1) != 0
    }
    pub fn set_HasStrongRTL_Apple(&mut self, v: bool) {
        self.high = (self.high & !(1 << 1)) | ((v as u8) << 1);
    }
    pub fn HasIndicStyleRearrangement(&self) -> bool {
        self.high & (1 << 2) != 0
    }
    pub fn set_HasIndicStyleRearrangement(&mut self, v: bool) {
        self.high = (self.high & !(1 << 2)) | ((v as u8) << 2);
    }
    pub fn AgfaMicroTypeExpressProcessed(&self) -> bool {
        self.high & (1 << 3) != 0
    }
    pub fn set_AgfaMicroTypeExpressProcessed(&mut self, v: bool) {
        self.high = (self.high & !(1 << 3)) | ((v as u8) << 3);
    }
    pub fn FontConverted(&self) -> bool {
        self.high & (1 << 4) != 0
    }
    pub fn set_FontConverted(&mut self, v: bool) {
        self.high = (self.high & !(1 << 4)) | ((v as u8) << 4);
    }
    pub fn DesignedForClearType(&self) -> bool {
        self.high & (1 << 5) != 0
    }
    pub fn set_DesignedForClearType(&mut self, v: bool) {
        self.high = (self.high & !(1 << 5)) | ((v as u8) << 5);
    }
    pub fn LastResort(&self) -> bool {
        self.high & (1 << 6) != 0
    }
    pub fn set_LastResort(&mut self, v: bool) {
        self.high = (self.high & !(1 << 6)) | ((v as u8) << 6);
    }
    pub fn Reserved15(&self) -> bool {
        self.high & (1 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, v: bool) {
        self.high = (self.high & !(1 << 7)) | ((v as u8) << 7);
    }
    pub fn BaselineAtY0(&self) -> bool {
        self.low & (1 << 0) != 0
    }
    pub fn set_BaselineAtY0(&mut self, v: bool) {
        self.low = (self.low & !(1 << 0)) | ((v as u8) << 0);
    }
    pub fn LeftSidebearingAtX0(&self) -> bool {
        self.low & (1 << 1) != 0
    }
    pub fn set_LeftSidebearingAtX0(&mut self, v: bool) {
        self.low = (self.low & !(1 << 1)) | ((v as u8) << 1);
    }
    pub fn InstructionsDependOnPointSize(&self) -> bool {
        self.low & (1 << 2) != 0
    }
    pub fn set_InstructionsDependOnPointSize(&mut self, v: bool) {
        self.low = (self.low & !(1 << 2)) | ((v as u8) << 2);
    }
    pub fn IntegerScaling(&self) -> bool {
        self.low & (1 << 3) != 0
    }
    pub fn set_IntegerScaling(&mut self, v: bool) {
        self.low = (self.low & !(1 << 3)) | ((v as u8) << 3);
    }
    pub fn InstructionsAlterAdvanceWidth(&self) -> bool {
        self.low & (1 << 4) != 0
    }
    pub fn set_InstructionsAlterAdvanceWidth(&mut self, v: bool) {
        self.low = (self.low & !(1 << 4)) | ((v as u8) << 4);
    }
    pub fn VerticalCenteredGlyphs_Apple(&self) -> bool {
        self.low & (1 << 5) != 0
    }
    pub fn set_VerticalCenteredGlyphs_Apple(&mut self, v: bool) {
        self.low = (self.low & !(1 << 5)) | ((v as u8) << 5);
    }
    pub fn Reserved06(&self) -> bool {
        self.low & (1 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, v: bool) {
        self.low = (self.low & !(1 << 6)) | ((v as u8) << 6);
    }
    pub fn RequiresLayout_Apple(&self) -> bool {
        self.low & (1 << 7) != 0
    }
    pub fn set_RequiresLayout_Apple(&mut self, v: bool) {
        self.low = (self.low & !(1 << 7)) | ((v as u8) << 7);
    }
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FlagsRaw {
    pub value: SK_OT_USHORT,
}
impl FlagsRaw {
    // Source: src/sfnt/SkOTTable_head.h:54-54
    pub const BaselineAtY0Mask: u16 = (1u16 << 0).to_be();
    // Source: src/sfnt/SkOTTable_head.h:55-55
    pub const LeftSidebearingAtX0Mask: u16 = (1u16 << 1).to_be();
    // Source: src/sfnt/SkOTTable_head.h:56-56
    pub const InstructionsDependOnPointSizeMask: u16 = (1u16 << 2).to_be();
    // Source: src/sfnt/SkOTTable_head.h:57-57
    pub const IntegerScalingMask: u16 = (1u16 << 3).to_be();
    // Source: src/sfnt/SkOTTable_head.h:58-58
    pub const InstructionsAlterAdvanceWidthMask: u16 = (1u16 << 4).to_be();
    // Source: src/sfnt/SkOTTable_head.h:59-59
    pub const VerticalCenteredGlyphs_AppleMask: u16 = (1u16 << 5).to_be();
    // Source: src/sfnt/SkOTTable_head.h:61-61
    pub const RequiresLayout_AppleMask: u16 = (1u16 << 7).to_be();
    // Source: src/sfnt/SkOTTable_head.h:63-63
    pub const GXMetamorphosis_AppleMask: u16 = (1u16 << 8).to_be();
    // Source: src/sfnt/SkOTTable_head.h:64-64
    pub const HasStrongRTL_AppleMask: u16 = (1u16 << 9).to_be();
    // Source: src/sfnt/SkOTTable_head.h:65-65
    pub const HasIndicStyleRearrangementMask: u16 = (1u16 << 10).to_be();
    // Source: src/sfnt/SkOTTable_head.h:66-66
    pub const AgfaMicroTypeExpressProcessedMask: u16 = (1u16 << 11).to_be();
    // Source: src/sfnt/SkOTTable_head.h:67-67
    pub const FontConvertedMask: u16 = (1u16 << 12).to_be();
    // Source: src/sfnt/SkOTTable_head.h:68-68
    pub const DesignedForClearTypeMask: u16 = (1u16 << 13).to_be();
    // Source: src/sfnt/SkOTTable_head.h:69-69
    pub const LastResortMask: u16 = (1u16 << 14).to_be();
}
// Source: src/sfnt/SkOTTable_head.h:81-115
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union MacStyle {
    pub field: MacStyleField,
    pub raw: MacStyleRaw,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MacStyleField {
    pub high: u8,
    pub low: u8,
}
impl MacStyleField {
    pub fn Reserved08(&self) -> bool {
        self.high & (1 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, v: bool) {
        self.high = (self.high & !(1 << 0)) | ((v as u8) << 0);
    }
    pub fn Reserved09(&self) -> bool {
        self.high & (1 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, v: bool) {
        self.high = (self.high & !(1 << 1)) | ((v as u8) << 1);
    }
    pub fn Reserved10(&self) -> bool {
        self.high & (1 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, v: bool) {
        self.high = (self.high & !(1 << 2)) | ((v as u8) << 2);
    }
    pub fn Reserved11(&self) -> bool {
        self.high & (1 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, v: bool) {
        self.high = (self.high & !(1 << 3)) | ((v as u8) << 3);
    }
    pub fn Reserved12(&self) -> bool {
        self.high & (1 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, v: bool) {
        self.high = (self.high & !(1 << 4)) | ((v as u8) << 4);
    }
    pub fn Reserved13(&self) -> bool {
        self.high & (1 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, v: bool) {
        self.high = (self.high & !(1 << 5)) | ((v as u8) << 5);
    }
    pub fn Reserved14(&self) -> bool {
        self.high & (1 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, v: bool) {
        self.high = (self.high & !(1 << 6)) | ((v as u8) << 6);
    }
    pub fn Reserved15(&self) -> bool {
        self.high & (1 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, v: bool) {
        self.high = (self.high & !(1 << 7)) | ((v as u8) << 7);
    }
    pub fn Bold(&self) -> bool {
        self.low & (1 << 0) != 0
    }
    pub fn set_Bold(&mut self, v: bool) {
        self.low = (self.low & !(1 << 0)) | ((v as u8) << 0);
    }
    pub fn Italic(&self) -> bool {
        self.low & (1 << 1) != 0
    }
    pub fn set_Italic(&mut self, v: bool) {
        self.low = (self.low & !(1 << 1)) | ((v as u8) << 1);
    }
    pub fn Underline(&self) -> bool {
        self.low & (1 << 2) != 0
    }
    pub fn set_Underline(&mut self, v: bool) {
        self.low = (self.low & !(1 << 2)) | ((v as u8) << 2);
    }
    pub fn Outline(&self) -> bool {
        self.low & (1 << 3) != 0
    }
    pub fn set_Outline(&mut self, v: bool) {
        self.low = (self.low & !(1 << 3)) | ((v as u8) << 3);
    }
    pub fn Shadow(&self) -> bool {
        self.low & (1 << 4) != 0
    }
    pub fn set_Shadow(&mut self, v: bool) {
        self.low = (self.low & !(1 << 4)) | ((v as u8) << 4);
    }
    pub fn Condensed(&self) -> bool {
        self.low & (1 << 5) != 0
    }
    pub fn set_Condensed(&mut self, v: bool) {
        self.low = (self.low & !(1 << 5)) | ((v as u8) << 5);
    }
    pub fn Extended(&self) -> bool {
        self.low & (1 << 6) != 0
    }
    pub fn set_Extended(&mut self, v: bool) {
        self.low = (self.low & !(1 << 6)) | ((v as u8) << 6);
    }
    pub fn Reserved07(&self) -> bool {
        self.low & (1 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, v: bool) {
        self.low = (self.low & !(1 << 7)) | ((v as u8) << 7);
    }
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MacStyleRaw {
    pub value: SK_OT_USHORT,
}
impl MacStyleRaw {
    // Source: src/sfnt/SkOTTable_head.h:105-105
    pub const BoldMask: u16 = (1u16 << 0).to_be();
    // Source: src/sfnt/SkOTTable_head.h:106-106
    pub const ItalicMask: u16 = (1u16 << 1).to_be();
    // Source: src/sfnt/SkOTTable_head.h:107-107
    pub const UnderlineMask: u16 = (1u16 << 2).to_be();
    // Source: src/sfnt/SkOTTable_head.h:108-108
    pub const OutlineMask: u16 = (1u16 << 3).to_be();
    // Source: src/sfnt/SkOTTable_head.h:109-109
    pub const ShadowMask: u16 = (1u16 << 4).to_be();
    // Source: src/sfnt/SkOTTable_head.h:110-110
    pub const CondensedMask: u16 = (1u16 << 5).to_be();
    // Source: src/sfnt/SkOTTable_head.h:111-111
    pub const ExtendedMask: u16 = (1u16 << 6).to_be();
}
// Source: src/sfnt/SkOTTable_head.h:117-125
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FontDirectionHint {
    pub value: FontDirectionHintValue,
}
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FontDirectionHintValue(pub SK_OT_SHORT);
impl FontDirectionHintValue {
    pub const FullyMixedDirectionalGlyphs: Self = Self((0i16 as u16).to_be());
    pub const OnlyStronglyLTR: Self = Self((1i16 as u16).to_be());
    pub const StronglyLTR: Self = Self((2i16 as u16).to_be());
    pub const OnlyStronglyRTL: Self = Self((-1i16 as u16).to_be());
    pub const StronglyRTL: Self = Self((-2i16 as u16).to_be());
}
// Source: src/sfnt/SkOTTable_head.h:126-131
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IndexToLocFormat {
    pub value: IndexToLocFormatValue,
}
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IndexToLocFormatValue(pub SK_OT_SHORT);
impl IndexToLocFormatValue {
    pub const ShortOffsets: Self = Self((0i16 as u16).to_be());
    pub const LongOffsets: Self = Self((1i16 as u16).to_be());
}
// Source: src/sfnt/SkOTTable_head.h:132-136
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GlyphDataFormat {
    pub value: GlyphDataFormatValue,
}
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GlyphDataFormatValue(pub SK_OT_SHORT);
impl GlyphDataFormatValue {
    pub const CurrentFormat: Self = Self((0i16 as u16).to_be());
}
// Source: src/sfnt/SkOTTable_head.h:143-144
const _: () = assert!(core::mem::offset_of!(SkOTTableHead, glyphDataFormat) == 52);
const _: () = assert!(core::mem::size_of::<SkOTTableHead>() == 54);
