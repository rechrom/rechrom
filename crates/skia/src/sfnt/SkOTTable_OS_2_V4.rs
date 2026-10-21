// Copyright 2012 Google Inc. BSD-style license.
// Direct packed native declarations. Rust contiguous fields and out-of-line
// nested definitions retain source order within each declaration group. Every
// native union keeps its Field and Raw projection; each native byte bitfield
// allocation unit has exact named low-to-high bit getters/setters. Transparent
// enum wrappers preserve all raw big-endian wire values, including unknown ones.
use super::SkOTTableTypes::*;
// Source: src/sfnt/SkOTTable_OS_2_V4.h:18-18
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:19-19
    pub version: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:22-22
    pub xAvgCharWidth: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:36-36
    pub usWeightClass: SkOTTableOS2_V4_WeightClass,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:49-49
    pub usWidthClass: SkOTTableOS2_V4_WidthClass,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:82-82
    pub fsType: SkOTTableOS2_V4_Type,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:83-83
    pub ySubscriptXSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:84-84
    pub ySubscriptYSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:85-85
    pub ySubscriptXOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:86-86
    pub ySubscriptYOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:87-87
    pub ySuperscriptXSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:88-88
    pub ySuperscriptYSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:89-89
    pub ySuperscriptXOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:90-90
    pub ySuperscriptYOffset: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:91-91
    pub yStrikeoutSize: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:92-92
    pub yStrikeoutPosition: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:93-93
    pub sFamilyClass: crate::src::sfnt::SkIBMFamilyClass::SkIBMFamilyClass,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:94-94
    pub panose: crate::src::sfnt::SkPanose::SkPanose,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:395-395
    pub ulUnicodeRange: SkOTTableOS2_V4_UnicodeRange,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:396-396
    pub achVendID: [SK_OT_CHAR; 4],
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:433-433
    pub fsSelection: SkOTTableOS2_V4_Selection,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:434-434
    pub usFirstCharIndex: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:435-435
    pub usLastCharIndex: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:437-437
    pub sTypoAscender: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:438-438
    pub sTypoDescender: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:439-439
    pub sTypoLineGap: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:440-440
    pub usWinAscent: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:441-441
    pub usWinDescent: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:568-568
    pub ulCodePageRange: SkOTTableOS2_V4_CodePageRange,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:570-570
    pub sxHeight: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:571-571
    pub sCapHeight: SK_OT_SHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:572-572
    pub usDefaultChar: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:573-573
    pub usBreakChar: SK_OT_USHORT,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:574-574
    pub usMaxContext: SK_OT_USHORT,
}
impl SkOTTableOS2_V4 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:20-20
    pub const VERSION: SK_OT_USHORT = ((4) as u16).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:23-23
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_WeightClass {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:35-35
    pub value: SK_OT_USHORT,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:24-24
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V4_WeightClass_Value(pub SK_OT_USHORT);
impl SkOTTableOS2_V4_WeightClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:25-25
    pub const Thin: Self = Self(((100) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:26-26
    pub const ExtraLight: Self = Self(((200) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:27-27
    pub const Light: Self = Self(((300) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:28-28
    pub const Normal: Self = Self(((400) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:29-29
    pub const Medium: Self = Self(((500) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:30-30
    pub const SemiBold: Self = Self(((600) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:31-31
    pub const Bold: Self = Self(((700) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:32-32
    pub const ExtraBold: Self = Self(((800) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:33-33
    pub const Black: Self = Self(((900) as u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:37-37
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_WidthClass {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:48-48
    pub value: SkOTTableOS2_V4_WidthClass_Value,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:38-38
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V4_WidthClass_Value(pub SK_OT_USHORT);
impl SkOTTableOS2_V4_WidthClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:39-39
    pub const UltraCondensed: Self = Self(((1) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:40-40
    pub const ExtraCondensed: Self = Self(((2) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:41-41
    pub const Condensed: Self = Self(((3) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:42-42
    pub const SemiCondensed: Self = Self(((4) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:43-43
    pub const Medium: Self = Self(((5) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:44-44
    pub const SemiExpanded: Self = Self(((6) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:45-45
    pub const Expanded: Self = Self(((7) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:46-46
    pub const ExtraExpanded: Self = Self(((8) as u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:47-47
    pub const UltraExpanded: Self = Self(((9) as u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:50-50
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V4_Type {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:72-72
    pub field: SkOTTableOS2_V4_Type_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:81-81
    pub raw: SkOTTableOS2_V4_Type_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:51-51
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:53-53
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:63-63
    pub byte1: u8,
}
impl SkOTTableOS2_V4_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:54-54
    pub fn NoSubsetting(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_NoSubsetting(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:55-55
    pub fn Bitmap(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Bitmap(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:56-56
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:57-57
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:58-58
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:59-59
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:60-60
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:61-61
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:64-64
    pub fn Reserved00(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Reserved00(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:65-65
    pub fn Restricted(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Restricted(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:66-66
    pub fn PreviewPrint(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_PreviewPrint(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:67-67
    pub fn Editable(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Editable(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:68-68
    pub fn Reserved04(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Reserved04(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:69-69
    pub fn Reserved05(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Reserved05(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:70-70
    pub fn Reserved06(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:71-71
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:73-73
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:80-80
    pub value: SK_OT_USHORT,
}
impl SkOTTableOS2_V4_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:74-74
    pub const Installable: SK_OT_USHORT = 0;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:75-75
    pub const RestrictedMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:76-76
    pub const PreviewPrintMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:77-77
    pub const EditableMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 3 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:78-78
    pub const NoSubsettingMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 8 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:79-79
    pub const BitmapMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 9 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:95-95
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V4_UnicodeRange {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:260-260
    pub field: SkOTTableOS2_V4_UnicodeRange_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:394-394
    pub raw: SkOTTableOS2_V4_UnicodeRange_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:96-96
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:98-98
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:108-108
    pub byte1: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:118-118
    pub byte2: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:128-128
    pub byte3: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:139-139
    pub byte4: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:149-149
    pub byte5: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:159-159
    pub byte6: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:169-169
    pub byte7: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:180-180
    pub byte8: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:190-190
    pub byte9: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:200-200
    pub byte10: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:210-210
    pub byte11: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:221-221
    pub byte12: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:231-231
    pub byte13: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:241-241
    pub byte14: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:251-251
    pub byte15: u8,
}
impl SkOTTableOS2_V4_UnicodeRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:99-99
    pub fn Thai(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Thai(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:100-100
    pub fn Lao(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Lao(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:101-101
    pub fn Georgian(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Georgian(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:102-102
    pub fn Balinese(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Balinese(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:103-103
    pub fn HangulJamo(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_HangulJamo(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:104-104
    pub fn LatinExtendedAdditional(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_LatinExtendedAdditional(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:105-105
    pub fn GreekExtended(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_GreekExtended(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:106-106
    pub fn GeneralPunctuation(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_GeneralPunctuation(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:109-109
    pub fn Bengali(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Bengali(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:110-110
    pub fn Gurmukhi(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Gurmukhi(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:111-111
    pub fn Gujarati(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Gujarati(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:112-112
    pub fn Oriya(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Oriya(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:113-113
    pub fn Tamil(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Tamil(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:114-114
    pub fn Telugu(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Telugu(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:115-115
    pub fn Kannada(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Kannada(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:116-116
    pub fn Malayalam(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Malayalam(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:119-119
    pub fn Coptic(&self) -> bool {
        self.byte2 & (1u8 << 0) != 0
    }
    pub fn set_Coptic(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:120-120
    pub fn Cyrillic(&self) -> bool {
        self.byte2 & (1u8 << 1) != 0
    }
    pub fn set_Cyrillic(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:121-121
    pub fn Armenian(&self) -> bool {
        self.byte2 & (1u8 << 2) != 0
    }
    pub fn set_Armenian(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:122-122
    pub fn Hebrew(&self) -> bool {
        self.byte2 & (1u8 << 3) != 0
    }
    pub fn set_Hebrew(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:123-123
    pub fn Vai(&self) -> bool {
        self.byte2 & (1u8 << 4) != 0
    }
    pub fn set_Vai(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:124-124
    pub fn Arabic(&self) -> bool {
        self.byte2 & (1u8 << 5) != 0
    }
    pub fn set_Arabic(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:125-125
    pub fn NKo(&self) -> bool {
        self.byte2 & (1u8 << 6) != 0
    }
    pub fn set_NKo(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:126-126
    pub fn Devanagari(&self) -> bool {
        self.byte2 & (1u8 << 7) != 0
    }
    pub fn set_Devanagari(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:129-129
    pub fn BasicLatin(&self) -> bool {
        self.byte3 & (1u8 << 0) != 0
    }
    pub fn set_BasicLatin(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:130-130
    pub fn Latin1Supplement(&self) -> bool {
        self.byte3 & (1u8 << 1) != 0
    }
    pub fn set_Latin1Supplement(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:131-131
    pub fn LatinExtendedA(&self) -> bool {
        self.byte3 & (1u8 << 2) != 0
    }
    pub fn set_LatinExtendedA(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:132-132
    pub fn LatinExtendedB(&self) -> bool {
        self.byte3 & (1u8 << 3) != 0
    }
    pub fn set_LatinExtendedB(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:133-133
    pub fn IPAExtensions(&self) -> bool {
        self.byte3 & (1u8 << 4) != 0
    }
    pub fn set_IPAExtensions(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:134-134
    pub fn SpacingModifierLetters(&self) -> bool {
        self.byte3 & (1u8 << 5) != 0
    }
    pub fn set_SpacingModifierLetters(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:135-135
    pub fn CombiningDiacriticalMarks(&self) -> bool {
        self.byte3 & (1u8 << 6) != 0
    }
    pub fn set_CombiningDiacriticalMarks(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:136-136
    pub fn GreekAndCoptic(&self) -> bool {
        self.byte3 & (1u8 << 7) != 0
    }
    pub fn set_GreekAndCoptic(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:140-140
    pub fn Hangul(&self) -> bool {
        self.byte4 & (1u8 << 0) != 0
    }
    pub fn set_Hangul(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:141-141
    pub fn NonPlane0(&self) -> bool {
        self.byte4 & (1u8 << 1) != 0
    }
    pub fn set_NonPlane0(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:142-142
    pub fn Phoenician(&self) -> bool {
        self.byte4 & (1u8 << 2) != 0
    }
    pub fn set_Phoenician(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:143-143
    pub fn CJKUnifiedIdeographs(&self) -> bool {
        self.byte4 & (1u8 << 3) != 0
    }
    pub fn set_CJKUnifiedIdeographs(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:144-144
    pub fn PrivateUseArea(&self) -> bool {
        self.byte4 & (1u8 << 4) != 0
    }
    pub fn set_PrivateUseArea(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:145-145
    pub fn CJKCompatibilityIdeographs(&self) -> bool {
        self.byte4 & (1u8 << 5) != 0
    }
    pub fn set_CJKCompatibilityIdeographs(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:146-146
    pub fn AlphabeticPresentationForms(&self) -> bool {
        self.byte4 & (1u8 << 6) != 0
    }
    pub fn set_AlphabeticPresentationForms(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:147-147
    pub fn ArabicPresentationFormsA(&self) -> bool {
        self.byte4 & (1u8 << 7) != 0
    }
    pub fn set_ArabicPresentationFormsA(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:150-150
    pub fn CJKSymbolsAndPunctuation(&self) -> bool {
        self.byte5 & (1u8 << 0) != 0
    }
    pub fn set_CJKSymbolsAndPunctuation(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:151-151
    pub fn Hiragana(&self) -> bool {
        self.byte5 & (1u8 << 1) != 0
    }
    pub fn set_Hiragana(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:152-152
    pub fn Katakana(&self) -> bool {
        self.byte5 & (1u8 << 2) != 0
    }
    pub fn set_Katakana(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:153-153
    pub fn Bopomofo(&self) -> bool {
        self.byte5 & (1u8 << 3) != 0
    }
    pub fn set_Bopomofo(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:154-154
    pub fn HangulCompatibilityJamo(&self) -> bool {
        self.byte5 & (1u8 << 4) != 0
    }
    pub fn set_HangulCompatibilityJamo(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:155-155
    pub fn PhagsPa(&self) -> bool {
        self.byte5 & (1u8 << 5) != 0
    }
    pub fn set_PhagsPa(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:156-156
    pub fn EnclosedCJKLettersAndMonths(&self) -> bool {
        self.byte5 & (1u8 << 6) != 0
    }
    pub fn set_EnclosedCJKLettersAndMonths(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:157-157
    pub fn CJKCompatibility(&self) -> bool {
        self.byte5 & (1u8 << 7) != 0
    }
    pub fn set_CJKCompatibility(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:160-160
    pub fn ControlPictures(&self) -> bool {
        self.byte6 & (1u8 << 0) != 0
    }
    pub fn set_ControlPictures(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:161-161
    pub fn OpticalCharacterRecognition(&self) -> bool {
        self.byte6 & (1u8 << 1) != 0
    }
    pub fn set_OpticalCharacterRecognition(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:162-162
    pub fn EnclosedAlphanumerics(&self) -> bool {
        self.byte6 & (1u8 << 2) != 0
    }
    pub fn set_EnclosedAlphanumerics(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:163-163
    pub fn BoxDrawing(&self) -> bool {
        self.byte6 & (1u8 << 3) != 0
    }
    pub fn set_BoxDrawing(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:164-164
    pub fn BlockElements(&self) -> bool {
        self.byte6 & (1u8 << 4) != 0
    }
    pub fn set_BlockElements(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:165-165
    pub fn GeometricShapes(&self) -> bool {
        self.byte6 & (1u8 << 5) != 0
    }
    pub fn set_GeometricShapes(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:166-166
    pub fn MiscellaneousSymbols(&self) -> bool {
        self.byte6 & (1u8 << 6) != 0
    }
    pub fn set_MiscellaneousSymbols(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:167-167
    pub fn Dingbats(&self) -> bool {
        self.byte6 & (1u8 << 7) != 0
    }
    pub fn set_Dingbats(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:170-170
    pub fn SuperscriptsAndSubscripts(&self) -> bool {
        self.byte7 & (1u8 << 0) != 0
    }
    pub fn set_SuperscriptsAndSubscripts(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:171-171
    pub fn CurrencySymbols(&self) -> bool {
        self.byte7 & (1u8 << 1) != 0
    }
    pub fn set_CurrencySymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:172-172
    pub fn CombiningDiacriticalMarksForSymbols(&self) -> bool {
        self.byte7 & (1u8 << 2) != 0
    }
    pub fn set_CombiningDiacriticalMarksForSymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:173-173
    pub fn LetterlikeSymbols(&self) -> bool {
        self.byte7 & (1u8 << 3) != 0
    }
    pub fn set_LetterlikeSymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:174-174
    pub fn NumberForms(&self) -> bool {
        self.byte7 & (1u8 << 4) != 0
    }
    pub fn set_NumberForms(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:175-175
    pub fn Arrows(&self) -> bool {
        self.byte7 & (1u8 << 5) != 0
    }
    pub fn set_Arrows(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:176-176
    pub fn MathematicalOperators(&self) -> bool {
        self.byte7 & (1u8 << 6) != 0
    }
    pub fn set_MathematicalOperators(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:177-177
    pub fn MiscellaneousTechnical(&self) -> bool {
        self.byte7 & (1u8 << 7) != 0
    }
    pub fn set_MiscellaneousTechnical(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:181-181
    pub fn MusicalSymbols(&self) -> bool {
        self.byte8 & (1u8 << 0) != 0
    }
    pub fn set_MusicalSymbols(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:182-182
    pub fn MathematicalAlphanumericSymbols(&self) -> bool {
        self.byte8 & (1u8 << 1) != 0
    }
    pub fn set_MathematicalAlphanumericSymbols(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:183-183
    pub fn PrivateUse(&self) -> bool {
        self.byte8 & (1u8 << 2) != 0
    }
    pub fn set_PrivateUse(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:184-184
    pub fn VariationSelectors(&self) -> bool {
        self.byte8 & (1u8 << 3) != 0
    }
    pub fn set_VariationSelectors(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:185-185
    pub fn Tags(&self) -> bool {
        self.byte8 & (1u8 << 4) != 0
    }
    pub fn set_Tags(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:186-186
    pub fn Limbu(&self) -> bool {
        self.byte8 & (1u8 << 5) != 0
    }
    pub fn set_Limbu(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:187-187
    pub fn TaiLe(&self) -> bool {
        self.byte8 & (1u8 << 6) != 0
    }
    pub fn set_TaiLe(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:188-188
    pub fn NewTaiLue(&self) -> bool {
        self.byte8 & (1u8 << 7) != 0
    }
    pub fn set_NewTaiLue(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:191-191
    pub fn Khmer(&self) -> bool {
        self.byte9 & (1u8 << 0) != 0
    }
    pub fn set_Khmer(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:192-192
    pub fn Mongolian(&self) -> bool {
        self.byte9 & (1u8 << 1) != 0
    }
    pub fn set_Mongolian(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:193-193
    pub fn Braille(&self) -> bool {
        self.byte9 & (1u8 << 2) != 0
    }
    pub fn set_Braille(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:194-194
    pub fn Yi(&self) -> bool {
        self.byte9 & (1u8 << 3) != 0
    }
    pub fn set_Yi(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:195-195
    pub fn Tagalog_Hanunoo_Buhid_Tagbanwa(&self) -> bool {
        self.byte9 & (1u8 << 4) != 0
    }
    pub fn set_Tagalog_Hanunoo_Buhid_Tagbanwa(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:196-196
    pub fn OldItalic(&self) -> bool {
        self.byte9 & (1u8 << 5) != 0
    }
    pub fn set_OldItalic(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:197-197
    pub fn Gothic(&self) -> bool {
        self.byte9 & (1u8 << 6) != 0
    }
    pub fn set_Gothic(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:198-198
    pub fn Deseret(&self) -> bool {
        self.byte9 & (1u8 << 7) != 0
    }
    pub fn set_Deseret(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:201-201
    pub fn Thaana(&self) -> bool {
        self.byte10 & (1u8 << 0) != 0
    }
    pub fn set_Thaana(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:202-202
    pub fn Sinhala(&self) -> bool {
        self.byte10 & (1u8 << 1) != 0
    }
    pub fn set_Sinhala(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:203-203
    pub fn Myanmar(&self) -> bool {
        self.byte10 & (1u8 << 2) != 0
    }
    pub fn set_Myanmar(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:204-204
    pub fn Ethiopic(&self) -> bool {
        self.byte10 & (1u8 << 3) != 0
    }
    pub fn set_Ethiopic(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:205-205
    pub fn Cherokee(&self) -> bool {
        self.byte10 & (1u8 << 4) != 0
    }
    pub fn set_Cherokee(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:206-206
    pub fn UnifiedCanadianSyllabics(&self) -> bool {
        self.byte10 & (1u8 << 5) != 0
    }
    pub fn set_UnifiedCanadianSyllabics(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:207-207
    pub fn Ogham(&self) -> bool {
        self.byte10 & (1u8 << 6) != 0
    }
    pub fn set_Ogham(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:208-208
    pub fn Runic(&self) -> bool {
        self.byte10 & (1u8 << 7) != 0
    }
    pub fn set_Runic(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:211-211
    pub fn CombiningHalfMarks(&self) -> bool {
        self.byte11 & (1u8 << 0) != 0
    }
    pub fn set_CombiningHalfMarks(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:212-212
    pub fn CJKCompatibilityForms(&self) -> bool {
        self.byte11 & (1u8 << 1) != 0
    }
    pub fn set_CJKCompatibilityForms(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:213-213
    pub fn SmallFormVariants(&self) -> bool {
        self.byte11 & (1u8 << 2) != 0
    }
    pub fn set_SmallFormVariants(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:214-214
    pub fn ArabicPresentationFormsB(&self) -> bool {
        self.byte11 & (1u8 << 3) != 0
    }
    pub fn set_ArabicPresentationFormsB(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:215-215
    pub fn HalfwidthAndFullwidthForms(&self) -> bool {
        self.byte11 & (1u8 << 4) != 0
    }
    pub fn set_HalfwidthAndFullwidthForms(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:216-216
    pub fn Specials(&self) -> bool {
        self.byte11 & (1u8 << 5) != 0
    }
    pub fn set_Specials(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:217-217
    pub fn Tibetan(&self) -> bool {
        self.byte11 & (1u8 << 6) != 0
    }
    pub fn set_Tibetan(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:218-218
    pub fn Syriac(&self) -> bool {
        self.byte11 & (1u8 << 7) != 0
    }
    pub fn set_Syriac(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:222-222
    pub fn PhaistosDisc(&self) -> bool {
        self.byte12 & (1u8 << 0) != 0
    }
    pub fn set_PhaistosDisc(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:223-223
    pub fn Carian_Lycian_Lydian(&self) -> bool {
        self.byte12 & (1u8 << 1) != 0
    }
    pub fn set_Carian_Lycian_Lydian(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:224-224
    pub fn DominoTiles_MahjongTiles(&self) -> bool {
        self.byte12 & (1u8 << 2) != 0
    }
    pub fn set_DominoTiles_MahjongTiles(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:225-225
    pub fn Reserved123(&self) -> bool {
        self.byte12 & (1u8 << 3) != 0
    }
    pub fn set_Reserved123(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:226-226
    pub fn Reserved124(&self) -> bool {
        self.byte12 & (1u8 << 4) != 0
    }
    pub fn set_Reserved124(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:227-227
    pub fn Reserved125(&self) -> bool {
        self.byte12 & (1u8 << 5) != 0
    }
    pub fn set_Reserved125(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:228-228
    pub fn Reserved126(&self) -> bool {
        self.byte12 & (1u8 << 6) != 0
    }
    pub fn set_Reserved126(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:229-229
    pub fn Reserved127(&self) -> bool {
        self.byte12 & (1u8 << 7) != 0
    }
    pub fn set_Reserved127(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:232-232
    pub fn Sundanese(&self) -> bool {
        self.byte13 & (1u8 << 0) != 0
    }
    pub fn set_Sundanese(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:233-233
    pub fn Lepcha(&self) -> bool {
        self.byte13 & (1u8 << 1) != 0
    }
    pub fn set_Lepcha(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:234-234
    pub fn OlChiki(&self) -> bool {
        self.byte13 & (1u8 << 2) != 0
    }
    pub fn set_OlChiki(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:235-235
    pub fn Saurashtra(&self) -> bool {
        self.byte13 & (1u8 << 3) != 0
    }
    pub fn set_Saurashtra(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:236-236
    pub fn KayahLi(&self) -> bool {
        self.byte13 & (1u8 << 4) != 0
    }
    pub fn set_KayahLi(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:237-237
    pub fn Rejang(&self) -> bool {
        self.byte13 & (1u8 << 5) != 0
    }
    pub fn set_Rejang(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:238-238
    pub fn Cham(&self) -> bool {
        self.byte13 & (1u8 << 6) != 0
    }
    pub fn set_Cham(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:239-239
    pub fn AncientSymbols(&self) -> bool {
        self.byte13 & (1u8 << 7) != 0
    }
    pub fn set_AncientSymbols(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:242-242
    pub fn OldPersian(&self) -> bool {
        self.byte14 & (1u8 << 0) != 0
    }
    pub fn set_OldPersian(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:243-243
    pub fn Shavian(&self) -> bool {
        self.byte14 & (1u8 << 1) != 0
    }
    pub fn set_Shavian(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:244-244
    pub fn Osmanya(&self) -> bool {
        self.byte14 & (1u8 << 2) != 0
    }
    pub fn set_Osmanya(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:245-245
    pub fn CypriotSyllabary(&self) -> bool {
        self.byte14 & (1u8 << 3) != 0
    }
    pub fn set_CypriotSyllabary(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:246-246
    pub fn Kharoshthi(&self) -> bool {
        self.byte14 & (1u8 << 4) != 0
    }
    pub fn set_Kharoshthi(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:247-247
    pub fn TaiXuanJingSymbols(&self) -> bool {
        self.byte14 & (1u8 << 5) != 0
    }
    pub fn set_TaiXuanJingSymbols(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:248-248
    pub fn Cuneiform(&self) -> bool {
        self.byte14 & (1u8 << 6) != 0
    }
    pub fn set_Cuneiform(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:249-249
    pub fn CountingRodNumerals(&self) -> bool {
        self.byte14 & (1u8 << 7) != 0
    }
    pub fn set_CountingRodNumerals(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:252-252
    pub fn Buginese(&self) -> bool {
        self.byte15 & (1u8 << 0) != 0
    }
    pub fn set_Buginese(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:253-253
    pub fn Glagolitic(&self) -> bool {
        self.byte15 & (1u8 << 1) != 0
    }
    pub fn set_Glagolitic(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:254-254
    pub fn Tifinagh(&self) -> bool {
        self.byte15 & (1u8 << 2) != 0
    }
    pub fn set_Tifinagh(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:255-255
    pub fn YijingHexagramSymbols(&self) -> bool {
        self.byte15 & (1u8 << 3) != 0
    }
    pub fn set_YijingHexagramSymbols(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:256-256
    pub fn SylotiNagri(&self) -> bool {
        self.byte15 & (1u8 << 4) != 0
    }
    pub fn set_SylotiNagri(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:257-257
    pub fn LinearB_AegeanNumbers(&self) -> bool {
        self.byte15 & (1u8 << 5) != 0
    }
    pub fn set_LinearB_AegeanNumbers(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:258-258
    pub fn AncientGreekNumbers(&self) -> bool {
        self.byte15 & (1u8 << 6) != 0
    }
    pub fn set_AncientGreekNumbers(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:259-259
    pub fn Ugaritic(&self) -> bool {
        self.byte15 & (1u8 << 7) != 0
    }
    pub fn set_Ugaritic(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:261-261
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:393-393
    pub value: [SK_OT_ULONG; 4],
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:262-262
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Raw_l0 {}
impl SkOTTableOS2_V4_UnicodeRange_Raw_l0 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:263-263
    pub const BasicLatinMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 0 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:264-264
    pub const Latin1SupplementMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:265-265
    pub const LatinExtendedAMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:266-266
    pub const LatinExtendedBMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 3 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:267-267
    pub const IPAExtensionsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 4 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:268-268
    pub const SpacingModifierLettersMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 5 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:269-269
    pub const CombiningDiacriticalMarksMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 6 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:270-270
    pub const GreekAndCopticMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 7 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:271-271
    pub const CopticMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 8 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:272-272
    pub const CyrillicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 9 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:273-273
    pub const ArmenianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 10 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:274-274
    pub const HebrewMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 11 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:275-275
    pub const VaiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 12 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:276-276
    pub const ArabicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 13 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:277-277
    pub const NKoMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 14 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:278-278
    pub const DevanagariMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 15 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:279-279
    pub const BengaliMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 16 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:280-280
    pub const GurmukhiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 17 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:281-281
    pub const GujaratiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 18 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:282-282
    pub const OriyaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 19 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:283-283
    pub const TamilMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 20 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:284-284
    pub const TeluguMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 21 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:285-285
    pub const KannadaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 22 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:286-286
    pub const MalayalamMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 23 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:287-287
    pub const ThaiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 24 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:288-288
    pub const LaoMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 25 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:289-289
    pub const GeorgianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 26 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:290-290
    pub const BalineseMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 27 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:291-291
    pub const HangulJamoMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 28 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:292-292
    pub const LatinExtendedAdditionalMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 29 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:293-293
    pub const GreekExtendedMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 30 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:294-294
    pub const GeneralPunctuationMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 31 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:296-296
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Raw_l1 {}
impl SkOTTableOS2_V4_UnicodeRange_Raw_l1 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:297-297
    pub const SuperscriptsAndSubscriptsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 32 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:298-298
    pub const CurrencySymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 33 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:299-299
    pub const CombiningDiacriticalMarksForSymbolsMask: SK_OT_ULONG =
        SkOTSetULONGBit::<{ 34 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:300-300
    pub const LetterlikeSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 35 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:301-301
    pub const NumberFormsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 36 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:302-302
    pub const ArrowsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 37 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:303-303
    pub const MathematicalOperatorsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 38 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:304-304
    pub const MiscellaneousTechnicalMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 39 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:305-305
    pub const ControlPicturesMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 40 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:306-306
    pub const OpticalCharacterRecognitionMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 41 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:307-307
    pub const EnclosedAlphanumericsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 42 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:308-308
    pub const BoxDrawingMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 43 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:309-309
    pub const BlockElementsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 44 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:310-310
    pub const GeometricShapesMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 45 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:311-311
    pub const MiscellaneousSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 46 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:312-312
    pub const DingbatsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 47 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:313-313
    pub const CJKSymbolsAndPunctuationMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 48 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:314-314
    pub const HiraganaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 49 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:315-315
    pub const KatakanaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 50 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:316-316
    pub const BopomofoMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 51 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:317-317
    pub const HangulCompatibilityJamoMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 52 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:318-318
    pub const PhagsPaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 53 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:319-319
    pub const EnclosedCJKLettersAndMonthsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 54 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:320-320
    pub const CJKCompatibilityMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 55 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:321-321
    pub const HangulMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 56 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:322-322
    pub const NonPlane0Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 57 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:323-323
    pub const PhoenicianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 58 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:324-324
    pub const CJKUnifiedIdeographsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 59 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:325-325
    pub const PrivateUseAreaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 60 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:326-326
    pub const CJKCompatibilityIdeographsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 61 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:327-327
    pub const AlphabeticPresentationFormsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 62 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:328-328
    pub const ArabicPresentationFormsAMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 63 - 32 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:330-330
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Raw_l2 {}
impl SkOTTableOS2_V4_UnicodeRange_Raw_l2 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:331-331
    pub const CombiningHalfMarksMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 64 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:332-332
    pub const CJKCompatibilityFormsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 65 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:333-333
    pub const SmallFormVariantsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 66 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:334-334
    pub const ArabicPresentationFormsBMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 67 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:335-335
    pub const HalfwidthAndFullwidthFormsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 68 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:336-336
    pub const SpecialsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 69 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:337-337
    pub const TibetanMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 70 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:338-338
    pub const SyriacMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 71 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:339-339
    pub const ThaanaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 72 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:340-340
    pub const SinhalaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 73 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:341-341
    pub const MyanmarMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 74 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:342-342
    pub const EthiopicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 75 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:343-343
    pub const CherokeeMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 76 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:344-344
    pub const UnifiedCanadianSyllabicsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 77 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:345-345
    pub const OghamMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 78 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:346-346
    pub const RunicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 79 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:347-347
    pub const KhmerMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 80 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:348-348
    pub const MongolianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 81 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:349-349
    pub const BrailleMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 82 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:350-350
    pub const YiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 83 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:351-351
    pub const Tagalog_Hanunoo_Buhid_TagbanwaMask: SK_OT_ULONG =
        SkOTSetULONGBit::<{ 84 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:352-352
    pub const OldItalicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 85 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:353-353
    pub const GothicMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 86 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:354-354
    pub const DeseretMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 87 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:355-355
    pub const MusicalSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 88 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:356-356
    pub const MathematicalAlphanumericSymbolsMask: SK_OT_ULONG =
        SkOTSetULONGBit::<{ 89 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:357-357
    pub const PrivateUseMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 90 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:358-358
    pub const VariationSelectorsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 91 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:359-359
    pub const TagsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 92 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:360-360
    pub const LimbuMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 93 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:361-361
    pub const TaiLeMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 94 - 64 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:362-362
    pub const NewTaiLueMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 95 - 64 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:364-364
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_UnicodeRange_Raw_l3 {}
impl SkOTTableOS2_V4_UnicodeRange_Raw_l3 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:365-365
    pub const BugineseMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 96 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:366-366
    pub const GlagoliticMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 97 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:367-367
    pub const TifinaghMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 98 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:368-368
    pub const YijingHexagramSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 99 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:369-369
    pub const SylotiNagriMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 100 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:370-370
    pub const LinearB_AegeanNumbersMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 101 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:371-371
    pub const AncientGreekNumbersMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 102 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:372-372
    pub const UgariticMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 103 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:373-373
    pub const OldPersianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 104 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:374-374
    pub const ShavianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 105 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:375-375
    pub const OsmanyaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 106 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:376-376
    pub const CypriotSyllabaryMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 107 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:377-377
    pub const KharoshthiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 108 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:378-378
    pub const TaiXuanJingSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 109 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:379-379
    pub const CuneiformMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 110 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:380-380
    pub const CountingRodNumeralsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 111 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:381-381
    pub const SundaneseMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 112 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:382-382
    pub const LepchaMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 113 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:383-383
    pub const OlChikiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 114 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:384-384
    pub const SaurashtraMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 115 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:385-385
    pub const KayahLiMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 116 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:386-386
    pub const RejangMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 117 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:387-387
    pub const ChamMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 118 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:388-388
    pub const AncientSymbolsMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 119 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:389-389
    pub const PhaistosDiscMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 120 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:390-390
    pub const Carian_Lycian_LydianMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 121 - 96 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:391-391
    pub const DominoTiles_MahjongTilesMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 122 - 96 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:397-397
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V4_Selection {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:419-419
    pub field: SkOTTableOS2_V4_Selection_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:432-432
    pub raw: SkOTTableOS2_V4_Selection_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:398-398
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:400-400
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:410-410
    pub byte1: u8,
}
impl SkOTTableOS2_V4_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:401-401
    pub fn WWS(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_WWS(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:402-402
    pub fn Oblique(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Oblique(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:403-403
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:404-404
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:405-405
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:406-406
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:407-407
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:408-408
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:411-411
    pub fn Italic(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Italic(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:412-412
    pub fn Underscore(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Underscore(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:413-413
    pub fn Negative(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Negative(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:414-414
    pub fn Outlined(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Outlined(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:415-415
    pub fn Strikeout(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Strikeout(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:416-416
    pub fn Bold(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Bold(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:417-417
    pub fn Regular(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Regular(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:418-418
    pub fn UseTypoMetrics(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_UseTypoMetrics(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:420-420
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:431-431
    pub value: SK_OT_USHORT,
}
impl SkOTTableOS2_V4_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:421-421
    pub const ItalicMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 0 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:422-422
    pub const UnderscoreMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:423-423
    pub const NegativeMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:424-424
    pub const OutlinedMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 3 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:425-425
    pub const StrikeoutMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 4 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:426-426
    pub const BoldMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 5 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:427-427
    pub const RegularMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 6 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:428-428
    pub const UseTypoMetricsMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 7 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:429-429
    pub const WWSMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 8 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:430-430
    pub const ObliqueMask: SK_OT_USHORT = SkOTSetUSHORTBit::<{ 9 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:443-443
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V4_CodePageRange {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:526-526
    pub field: SkOTTableOS2_V4_CodePageRange_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:567-567
    pub raw: SkOTTableOS2_V4_CodePageRange_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:444-444
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_CodePageRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:446-446
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:456-456
    pub byte1: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:466-466
    pub byte2: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:476-476
    pub byte3: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:487-487
    pub byte4: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:497-497
    pub byte5: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:507-507
    pub byte6: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:517-517
    pub byte7: u8,
}
impl SkOTTableOS2_V4_CodePageRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:447-447
    pub fn Reserved24(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved24(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:448-448
    pub fn Reserved25(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved25(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:449-449
    pub fn Reserved26(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved26(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:450-450
    pub fn Reserved27(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved27(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:451-451
    pub fn Reserved28(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved28(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:452-452
    pub fn MacintoshCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_MacintoshCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:453-453
    pub fn OEMCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_OEMCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:454-454
    pub fn SymbolCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_SymbolCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:457-457
    pub fn Thai_874(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Thai_874(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:458-458
    pub fn JISJapan_932(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_JISJapan_932(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:459-459
    pub fn ChineseSimplified_936(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_ChineseSimplified_936(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:460-460
    pub fn KoreanWansung_949(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_KoreanWansung_949(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:461-461
    pub fn ChineseTraditional_950(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_ChineseTraditional_950(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:462-462
    pub fn KoreanJohab_1361(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_KoreanJohab_1361(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:463-463
    pub fn Reserved22(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved22(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:464-464
    pub fn Reserved23(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved23(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:467-467
    pub fn Vietnamese(&self) -> bool {
        self.byte2 & (1u8 << 0) != 0
    }
    pub fn set_Vietnamese(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:468-468
    pub fn Reserved09(&self) -> bool {
        self.byte2 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:469-469
    pub fn Reserved10(&self) -> bool {
        self.byte2 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:470-470
    pub fn Reserved11(&self) -> bool {
        self.byte2 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:471-471
    pub fn Reserved12(&self) -> bool {
        self.byte2 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:472-472
    pub fn Reserved13(&self) -> bool {
        self.byte2 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:473-473
    pub fn Reserved14(&self) -> bool {
        self.byte2 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:474-474
    pub fn Reserved15(&self) -> bool {
        self.byte2 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:477-477
    pub fn Latin1_1252(&self) -> bool {
        self.byte3 & (1u8 << 0) != 0
    }
    pub fn set_Latin1_1252(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:478-478
    pub fn Latin2EasternEurope_1250(&self) -> bool {
        self.byte3 & (1u8 << 1) != 0
    }
    pub fn set_Latin2EasternEurope_1250(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:479-479
    pub fn Cyrillic_1251(&self) -> bool {
        self.byte3 & (1u8 << 2) != 0
    }
    pub fn set_Cyrillic_1251(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:480-480
    pub fn Greek_1253(&self) -> bool {
        self.byte3 & (1u8 << 3) != 0
    }
    pub fn set_Greek_1253(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:481-481
    pub fn Turkish_1254(&self) -> bool {
        self.byte3 & (1u8 << 4) != 0
    }
    pub fn set_Turkish_1254(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:482-482
    pub fn Hebrew_1255(&self) -> bool {
        self.byte3 & (1u8 << 5) != 0
    }
    pub fn set_Hebrew_1255(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:483-483
    pub fn Arabic_1256(&self) -> bool {
        self.byte3 & (1u8 << 6) != 0
    }
    pub fn set_Arabic_1256(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:484-484
    pub fn WindowsBaltic_1257(&self) -> bool {
        self.byte3 & (1u8 << 7) != 0
    }
    pub fn set_WindowsBaltic_1257(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:488-488
    pub fn IBMTurkish_857(&self) -> bool {
        self.byte4 & (1u8 << 0) != 0
    }
    pub fn set_IBMTurkish_857(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:489-489
    pub fn IBMCyrillic_855(&self) -> bool {
        self.byte4 & (1u8 << 1) != 0
    }
    pub fn set_IBMCyrillic_855(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:490-490
    pub fn Latin2_852(&self) -> bool {
        self.byte4 & (1u8 << 2) != 0
    }
    pub fn set_Latin2_852(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:491-491
    pub fn MSDOSBaltic_775(&self) -> bool {
        self.byte4 & (1u8 << 3) != 0
    }
    pub fn set_MSDOSBaltic_775(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:492-492
    pub fn Greek_737(&self) -> bool {
        self.byte4 & (1u8 << 4) != 0
    }
    pub fn set_Greek_737(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:493-493
    pub fn Arabic_708(&self) -> bool {
        self.byte4 & (1u8 << 5) != 0
    }
    pub fn set_Arabic_708(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:494-494
    pub fn WELatin1_850(&self) -> bool {
        self.byte4 & (1u8 << 6) != 0
    }
    pub fn set_WELatin1_850(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:495-495
    pub fn US_437(&self) -> bool {
        self.byte4 & (1u8 << 7) != 0
    }
    pub fn set_US_437(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:498-498
    pub fn IBMGreek_869(&self) -> bool {
        self.byte5 & (1u8 << 0) != 0
    }
    pub fn set_IBMGreek_869(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:499-499
    pub fn MSDOSRussian_866(&self) -> bool {
        self.byte5 & (1u8 << 1) != 0
    }
    pub fn set_MSDOSRussian_866(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:500-500
    pub fn MSDOSNordic_865(&self) -> bool {
        self.byte5 & (1u8 << 2) != 0
    }
    pub fn set_MSDOSNordic_865(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:501-501
    pub fn Arabic_864(&self) -> bool {
        self.byte5 & (1u8 << 3) != 0
    }
    pub fn set_Arabic_864(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:502-502
    pub fn MSDOSCanadianFrench_863(&self) -> bool {
        self.byte5 & (1u8 << 4) != 0
    }
    pub fn set_MSDOSCanadianFrench_863(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:503-503
    pub fn Hebrew_862(&self) -> bool {
        self.byte5 & (1u8 << 5) != 0
    }
    pub fn set_Hebrew_862(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:504-504
    pub fn MSDOSIcelandic_861(&self) -> bool {
        self.byte5 & (1u8 << 6) != 0
    }
    pub fn set_MSDOSIcelandic_861(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:505-505
    pub fn MSDOSPortuguese_860(&self) -> bool {
        self.byte5 & (1u8 << 7) != 0
    }
    pub fn set_MSDOSPortuguese_860(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:508-508
    pub fn Reserved40(&self) -> bool {
        self.byte6 & (1u8 << 0) != 0
    }
    pub fn set_Reserved40(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:509-509
    pub fn Reserved41(&self) -> bool {
        self.byte6 & (1u8 << 1) != 0
    }
    pub fn set_Reserved41(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:510-510
    pub fn Reserved42(&self) -> bool {
        self.byte6 & (1u8 << 2) != 0
    }
    pub fn set_Reserved42(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:511-511
    pub fn Reserved43(&self) -> bool {
        self.byte6 & (1u8 << 3) != 0
    }
    pub fn set_Reserved43(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:512-512
    pub fn Reserved44(&self) -> bool {
        self.byte6 & (1u8 << 4) != 0
    }
    pub fn set_Reserved44(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:513-513
    pub fn Reserved45(&self) -> bool {
        self.byte6 & (1u8 << 5) != 0
    }
    pub fn set_Reserved45(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:514-514
    pub fn Reserved46(&self) -> bool {
        self.byte6 & (1u8 << 6) != 0
    }
    pub fn set_Reserved46(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:515-515
    pub fn Reserved47(&self) -> bool {
        self.byte6 & (1u8 << 7) != 0
    }
    pub fn set_Reserved47(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:518-518
    pub fn Reserved32(&self) -> bool {
        self.byte7 & (1u8 << 0) != 0
    }
    pub fn set_Reserved32(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:519-519
    pub fn Reserved33(&self) -> bool {
        self.byte7 & (1u8 << 1) != 0
    }
    pub fn set_Reserved33(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:520-520
    pub fn Reserved34(&self) -> bool {
        self.byte7 & (1u8 << 2) != 0
    }
    pub fn set_Reserved34(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:521-521
    pub fn Reserved35(&self) -> bool {
        self.byte7 & (1u8 << 3) != 0
    }
    pub fn set_Reserved35(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:522-522
    pub fn Reserved36(&self) -> bool {
        self.byte7 & (1u8 << 4) != 0
    }
    pub fn set_Reserved36(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:523-523
    pub fn Reserved37(&self) -> bool {
        self.byte7 & (1u8 << 5) != 0
    }
    pub fn set_Reserved37(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:524-524
    pub fn Reserved38(&self) -> bool {
        self.byte7 & (1u8 << 6) != 0
    }
    pub fn set_Reserved38(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:525-525
    pub fn Reserved39(&self) -> bool {
        self.byte7 & (1u8 << 7) != 0
    }
    pub fn set_Reserved39(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:527-527
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_CodePageRange_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:566-566
    pub value: [SK_OT_ULONG; 2],
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:528-528
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_CodePageRange_Raw_l0 {}
impl SkOTTableOS2_V4_CodePageRange_Raw_l0 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:529-529
    pub const Latin1_1252Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 0 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:530-530
    pub const Latin2EasternEurope_1250Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 1 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:531-531
    pub const Cyrillic_1251Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 2 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:532-532
    pub const Greek_1253Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 3 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:533-533
    pub const Turkish_1254Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 4 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:534-534
    pub const Hebrew_1255Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 5 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:535-535
    pub const Arabic_1256Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 6 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:536-536
    pub const WindowsBaltic_1257Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 7 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:537-537
    pub const Vietnamese_1258Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 8 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:538-538
    pub const Thai_874Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 16 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:539-539
    pub const JISJapan_932Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 17 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:540-540
    pub const ChineseSimplified_936Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 18 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:541-541
    pub const KoreanWansung_949Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 19 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:542-542
    pub const ChineseTraditional_950Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 20 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:543-543
    pub const KoreanJohab_1361Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 21 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:544-544
    pub const MacintoshCharacterSetMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 29 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:545-545
    pub const OEMCharacterSetMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 30 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:546-546
    pub const SymbolCharacterSetMask: SK_OT_ULONG = SkOTSetULONGBit::<{ 31 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:548-548
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V4_CodePageRange_Raw_l1 {}
impl SkOTTableOS2_V4_CodePageRange_Raw_l1 {
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:549-549
    pub const IBMGreek_869Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 48 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:550-550
    pub const MSDOSRussian_866Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 49 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:551-551
    pub const MSDOSNordic_865Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 50 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:552-552
    pub const Arabic_864Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 51 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:553-553
    pub const MSDOSCanadianFrench_863Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 52 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:554-554
    pub const Hebrew_862Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 53 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:555-555
    pub const MSDOSIcelandic_861Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 54 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:556-556
    pub const MSDOSPortuguese_860Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 55 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:557-557
    pub const IBMTurkish_857Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 56 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:558-558
    pub const IBMCyrillic_855Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 57 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:559-559
    pub const Latin2_852Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 58 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:560-560
    pub const MSDOSBaltic_775Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 59 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:561-561
    pub const Greek_737Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 60 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:562-562
    pub const Arabic_708Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 61 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:563-563
    pub const WELatin1_850Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 62 - 32 }>::value;
    // Source: src/sfnt/SkOTTable_OS_2_V4.h:564-564
    pub const US_437Mask: SK_OT_ULONG = SkOTSetULONGBit::<{ 63 - 32 }>::value;
}
// Source: src/sfnt/SkOTTable_OS_2_V4.h:580-580
const _: () = assert!(core::mem::size_of::<SkOTTableOS2_V4>() == 96);
