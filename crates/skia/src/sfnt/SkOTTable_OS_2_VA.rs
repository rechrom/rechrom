// Copyright 2012 Google Inc. BSD-style license.
// Direct packed native declarations. Rust contiguous fields and out-of-line
// nested definitions retain source order within each declaration group. Every
// native union keeps its Field and Raw projection; each native byte bitfield
// allocation unit has exact named low-to-high bit getters/setters. Transparent
// enum wrappers preserve all raw big-endian wire values, including unknown ones.
use super::SkOTTableTypes::*;
// Source: src/sfnt/SkOTTable_OS_2_VA.h:19-19
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:20-20
    pub version: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:25-25
    pub xAvgCharWidth: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:39-39
    pub usWeightClass: SkOTTableOS2_VA_WeightClass,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:53-53
    pub usWidthClass: SkOTTableOS2_VA_WidthClass,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:84-84
    pub fsType: SkOTTableOS2_VA_Type,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:85-85
    pub ySubscriptXSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:86-86
    pub ySubscriptYSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:87-87
    pub ySubscriptXOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:88-88
    pub ySubscriptYOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:89-89
    pub ySuperscriptXSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:90-90
    pub ySuperscriptYSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:91-91
    pub ySuperscriptXOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:92-92
    pub ySuperscriptYOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:93-93
    pub yStrikeoutSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:94-94
    pub yStrikeoutPosition: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:95-95
    pub sFamilyClass: crate::src::sfnt::SkIBMFamilyClass::SkIBMFamilyClass,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:96-96
    pub panose: crate::src::sfnt::SkPanose::SkPanose,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:97-97
    pub ulCharRange: [SK_OT_ULONG; 4],
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:98-98
    pub achVendID: [SK_OT_CHAR; 4],
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:131-131
    pub fsSelection: SkOTTableOS2_VA_Selection,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:132-132
    pub usFirstCharIndex: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:133-133
    pub usLastCharIndex: SK_OT_USHORT,
}
impl SkOTTableOS2_VA {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:23-23
    pub const VERSION: SK_OT_USHORT = ((0) as u16).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:26-26
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_WeightClass {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:38-38
    pub value: SkOTTableOS2_VA_WeightClass_Value,
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:27-27
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_VA_WeightClass_Value(pub SK_OT_USHORT);
impl SkOTTableOS2_VA_WeightClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:28-28
    pub const UltraLight: Self = Self(((1) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:29-29
    pub const ExtraLight: Self = Self(((2) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:30-30
    pub const Light: Self = Self(((3) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:31-31
    pub const SemiLight: Self = Self(((4) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:32-32
    pub const Medium: Self = Self(((5) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:33-33
    pub const SemiBold: Self = Self(((6) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:34-34
    pub const Bold: Self = Self(((7) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:35-35
    pub const ExtraBold: Self = Self(((8) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:36-36
    pub const UltraBold: Self = Self(((9) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:37-37
    pub const SK_SEQ_END: Self = Self(Self::UltraBold.0.wrapping_add(1));
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:40-40
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_WidthClass {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:52-52
    pub value: SkOTTableOS2_VA_WidthClass_Value,
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:41-41
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_VA_WidthClass_Value(pub SK_OT_USHORT);
impl SkOTTableOS2_VA_WidthClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:42-42
    pub const UltraCondensed: Self = Self(((1) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:43-43
    pub const ExtraCondensed: Self = Self(((2) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:44-44
    pub const Condensed: Self = Self(((3) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:45-45
    pub const SemiCondensed: Self = Self(((4) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:46-46
    pub const Medium: Self = Self(((5) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:47-47
    pub const SemiExpanded: Self = Self(((6) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:48-48
    pub const Expanded: Self = Self(((7) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:49-49
    pub const ExtraExpanded: Self = Self(((8) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:50-50
    pub const UltraExpanded: Self = Self(((9) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:51-51
    pub const SK_SEQ_END: Self = Self(Self::UltraExpanded.0.wrapping_add(1));
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:54-54
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_VA_Type {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:76-76
    pub field: SkOTTableOS2_VA_Type_Field,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:83-83
    pub raw: SkOTTableOS2_VA_Type_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:55-55
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:57-57
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:67-67
    pub byte1: u8,
}
impl SkOTTableOS2_VA_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:58-58
    pub fn Reserved08(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:59-59
    pub fn Reserved09(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:60-60
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:61-61
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:62-62
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:63-63
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:64-64
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:65-65
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:68-68
    pub fn Reserved00(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Reserved00(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:69-69
    pub fn Restricted(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Restricted(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:70-70
    pub fn PreviewPrint(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_PreviewPrint(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:71-71
    pub fn Editable(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Editable(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:72-72
    pub fn Reserved04(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Reserved04(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:73-73
    pub fn Reserved05(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Reserved05(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:74-74
    pub fn Reserved06(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:75-75
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:77-77
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:82-82
    pub value: SK_OT_USHORT,
}
impl SkOTTableOS2_VA_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:78-78
    pub const Installable: SK_OT_USHORT = 0;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:79-79
    pub const RestrictedMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:80-80
    pub const PreviewPrintMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:81-81
    pub const EditableMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 3 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:99-99
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_VA_Selection {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:121-121
    pub field: SkOTTableOS2_VA_Selection_Field,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:130-130
    pub raw: SkOTTableOS2_VA_Selection_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:100-100
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:102-102
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:112-112
    pub byte1: u8,
}
impl SkOTTableOS2_VA_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:103-103
    pub fn Reserved08(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:104-104
    pub fn Reserved09(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:105-105
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:106-106
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:107-107
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:108-108
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:109-109
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:110-110
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:113-113
    pub fn Italic(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Italic(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:114-114
    pub fn Underscore(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Underscore(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:115-115
    pub fn Negative(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Negative(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:116-116
    pub fn Outlined(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Outlined(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:117-117
    pub fn Strikeout(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Strikeout(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:118-118
    pub fn Bold(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Bold(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:119-119
    pub fn Reserved06(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:120-120
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:122-122
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_VA_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:129-129
    pub value: SK_OT_USHORT,
}
impl SkOTTableOS2_VA_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:123-123
    pub const ItalicMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 0 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:124-124
    pub const UnderscoreMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:125-125
    pub const NegativeMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:126-126
    pub const OutlinedMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 3 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:127-127
    pub const StrikeoutMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 4 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_VA.h:128-128
    pub const BoldMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 5 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_VA.h:139-139
const _: () = assert!(core::mem::size_of::<SkOTTableOS2_VA>() == 68);
