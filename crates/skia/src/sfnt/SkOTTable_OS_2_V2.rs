// Packed native declarations; raw big-endian integer storage is retained.
// Transparent enum values admit every wire byte, including unknown values.
// Bit-field accessors use the source byte order and named low-to-high bit indices.
// No C++ object references are formed into unaligned font storage.
// Source: src/sfnt/SkOTTable_OS_2_V2.h:18-531
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:19-19
    pub version: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:22-22
    pub xAvgCharWidth: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:36-36
    pub usWeightClass: SkOTTableOS2_V2_WeightClass,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:49-49
    pub usWidthClass: SkOTTableOS2_V2_WidthClass,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:82-82
    pub fsType: SkOTTableOS2_V2_Type,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:83-83
    pub ySubscriptXSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:84-84
    pub ySubscriptYSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:85-85
    pub ySubscriptXOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:86-86
    pub ySubscriptYOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:87-87
    pub ySuperscriptXSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:88-88
    pub ySuperscriptYSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:89-89
    pub ySuperscriptXOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:90-90
    pub ySuperscriptYOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:91-91
    pub yStrikeoutSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:92-92
    pub yStrikeoutPosition: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:93-93
    pub sFamilyClass: crate::src::sfnt::SkIBMFamilyClass::SkIBMFamilyClass,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:94-94
    pub panose: crate::src::sfnt::SkPanose::SkPanose,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:354-354
    pub ulUnicodeRange: SkOTTableOS2_V2_UnicodeRange,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:355-355
    pub achVendID: [i8; 4],
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:389-389
    pub fsSelection: SkOTTableOS2_V2_Selection,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:390-390
    pub usFirstCharIndex: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:391-391
    pub usLastCharIndex: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:393-393
    pub sTypoAscender: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:394-394
    pub sTypoDescender: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:395-395
    pub sTypoLineGap: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:396-396
    pub usWinAscent: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:397-397
    pub usWinDescent: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:524-524
    pub ulCodePageRange: SkOTTableOS2_V2_CodePageRange,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:526-526
    pub sxHeight: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:527-527
    pub sCapHeight: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:528-528
    pub usDefaultChar: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:529-529
    pub usBreakChar: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:530-530
    pub usMaxContext: u16,
}
impl SkOTTableOS2_V2 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:20-20
    pub const VERSION: u16 = (2u16).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:23-36
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_WeightClass {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:35-35
    pub value: u16,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:24-34
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V2_WeightClass_Value(pub u16);
impl SkOTTableOS2_V2_WeightClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:25-25
    pub const Thin: Self = Self((100u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:26-26
    pub const ExtraLight: Self = Self((200u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:27-27
    pub const Light: Self = Self((300u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:28-28
    pub const Normal: Self = Self((400u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:29-29
    pub const Medium: Self = Self((500u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:30-30
    pub const SemiBold: Self = Self((600u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:31-31
    pub const Bold: Self = Self((700u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:32-32
    pub const ExtraBold: Self = Self((800u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:33-33
    pub const Black: Self = Self((900u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:37-49
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_WidthClass {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:48-48
    pub value: SkOTTableOS2_V2_WidthClass_Value,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:38-48
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V2_WidthClass_Value(pub u16);
impl SkOTTableOS2_V2_WidthClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:39-39
    pub const UltraCondensed: Self = Self((1u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:40-40
    pub const ExtraCondensed: Self = Self((2u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:41-41
    pub const Condensed: Self = Self((3u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:42-42
    pub const SemiCondensed: Self = Self((4u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:43-43
    pub const Medium: Self = Self((5u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:44-44
    pub const SemiExpanded: Self = Self((6u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:45-45
    pub const Expanded: Self = Self((7u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:46-46
    pub const ExtraExpanded: Self = Self((8u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:47-47
    pub const UltraExpanded: Self = Self((9u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:50-82
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V2_Type {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:72-72
    pub field: SkOTTableOS2_V2_Type_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:81-81
    pub raw: SkOTTableOS2_V2_Type_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:51-72
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-53
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-63
    pub byte1: u8,
}
impl SkOTTableOS2_V2_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn NoSubsetting(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_NoSubsetting(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Bitmap(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Bitmap(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:53-61
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Reserved00(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Reserved00(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Restricted(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Restricted(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn PreviewPrint(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_PreviewPrint(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Editable(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Editable(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Reserved04(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Reserved04(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Reserved05(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Reserved05(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Reserved06(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:63-71
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:73-81
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:80-80
    pub value: u16,
}
impl SkOTTableOS2_V2_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:74-74
    pub const Installable: u16 = 0;
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:75-75
    pub const RestrictedMask: u16 = (1u16 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:76-76
    pub const PreviewPrintMask: u16 = (1u16 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:77-77
    pub const EditableMask: u16 = (1u16 << (3)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:78-78
    pub const NoSubsettingMask: u16 = (1u16 << (8)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:79-79
    pub const BitmapMask: u16 = (1u16 << (9)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:95-354
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V2_UnicodeRange {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:260-260
    pub field: SkOTTableOS2_V2_UnicodeRange_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:353-353
    pub raw: SkOTTableOS2_V2_UnicodeRange_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:96-260
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_UnicodeRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-98
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-108
    pub byte1: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-118
    pub byte2: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-128
    pub byte3: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-139
    pub byte4: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-149
    pub byte5: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-159
    pub byte6: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-169
    pub byte7: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-180
    pub byte8: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-190
    pub byte9: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-200
    pub byte10: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-210
    pub byte11: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-221
    pub byte12: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-231
    pub byte13: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-241
    pub byte14: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-251
    pub byte15: u8,
}
impl SkOTTableOS2_V2_UnicodeRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn Thai(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Thai(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn Lao(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Lao(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn Georgian(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Georgian(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn Reserved027(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved027(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn HangulJamo(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_HangulJamo(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn LatinExtendedAdditional(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_LatinExtendedAdditional(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn GreekExtended(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_GreekExtended(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:98-106
    pub fn GeneralPunctuation(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_GeneralPunctuation(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Bengali(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Bengali(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Gurmukhi(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Gurmukhi(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Gujarati(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Gujarati(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Oriya(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Oriya(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Tamil(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Tamil(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Telugu(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Telugu(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Kannada(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Kannada(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:108-116
    pub fn Malayalam(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Malayalam(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Reserved008(&self) -> bool {
        self.byte2 & (1u8 << 0) != 0
    }
    pub fn set_Reserved008(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Cyrillic(&self) -> bool {
        self.byte2 & (1u8 << 1) != 0
    }
    pub fn set_Cyrillic(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Armenian(&self) -> bool {
        self.byte2 & (1u8 << 2) != 0
    }
    pub fn set_Armenian(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Hebrew(&self) -> bool {
        self.byte2 & (1u8 << 3) != 0
    }
    pub fn set_Hebrew(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Reserved012(&self) -> bool {
        self.byte2 & (1u8 << 4) != 0
    }
    pub fn set_Reserved012(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Arabic(&self) -> bool {
        self.byte2 & (1u8 << 5) != 0
    }
    pub fn set_Arabic(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Reserved014(&self) -> bool {
        self.byte2 & (1u8 << 6) != 0
    }
    pub fn set_Reserved014(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:118-126
    pub fn Devanagari(&self) -> bool {
        self.byte2 & (1u8 << 7) != 0
    }
    pub fn set_Devanagari(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn BasicLatin(&self) -> bool {
        self.byte3 & (1u8 << 0) != 0
    }
    pub fn set_BasicLatin(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn Latin1Supplement(&self) -> bool {
        self.byte3 & (1u8 << 1) != 0
    }
    pub fn set_Latin1Supplement(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn LatinExtendedA(&self) -> bool {
        self.byte3 & (1u8 << 2) != 0
    }
    pub fn set_LatinExtendedA(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn LatinExtendedB(&self) -> bool {
        self.byte3 & (1u8 << 3) != 0
    }
    pub fn set_LatinExtendedB(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn IPAExtensions(&self) -> bool {
        self.byte3 & (1u8 << 4) != 0
    }
    pub fn set_IPAExtensions(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn SpacingModifierLetters(&self) -> bool {
        self.byte3 & (1u8 << 5) != 0
    }
    pub fn set_SpacingModifierLetters(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn CombiningDiacriticalMarks(&self) -> bool {
        self.byte3 & (1u8 << 6) != 0
    }
    pub fn set_CombiningDiacriticalMarks(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:128-136
    pub fn Greek(&self) -> bool {
        self.byte3 & (1u8 << 7) != 0
    }
    pub fn set_Greek(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn Hangul(&self) -> bool {
        self.byte4 & (1u8 << 0) != 0
    }
    pub fn set_Hangul(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn Surrogates(&self) -> bool {
        self.byte4 & (1u8 << 1) != 0
    }
    pub fn set_Surrogates(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn Reserved058(&self) -> bool {
        self.byte4 & (1u8 << 2) != 0
    }
    pub fn set_Reserved058(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn CJKUnifiedIdeographs(&self) -> bool {
        self.byte4 & (1u8 << 3) != 0
    }
    pub fn set_CJKUnifiedIdeographs(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn PrivateUseArea(&self) -> bool {
        self.byte4 & (1u8 << 4) != 0
    }
    pub fn set_PrivateUseArea(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn CJKCompatibilityIdeographs(&self) -> bool {
        self.byte4 & (1u8 << 5) != 0
    }
    pub fn set_CJKCompatibilityIdeographs(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn AlphabeticPresentationForms(&self) -> bool {
        self.byte4 & (1u8 << 6) != 0
    }
    pub fn set_AlphabeticPresentationForms(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:139-147
    pub fn ArabicPresentationFormsA(&self) -> bool {
        self.byte4 & (1u8 << 7) != 0
    }
    pub fn set_ArabicPresentationFormsA(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn CJKSymbolsAndPunctuation(&self) -> bool {
        self.byte5 & (1u8 << 0) != 0
    }
    pub fn set_CJKSymbolsAndPunctuation(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn Hiragana(&self) -> bool {
        self.byte5 & (1u8 << 1) != 0
    }
    pub fn set_Hiragana(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn Katakana(&self) -> bool {
        self.byte5 & (1u8 << 2) != 0
    }
    pub fn set_Katakana(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn Bopomofo(&self) -> bool {
        self.byte5 & (1u8 << 3) != 0
    }
    pub fn set_Bopomofo(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn HangulCompatibilityJamo(&self) -> bool {
        self.byte5 & (1u8 << 4) != 0
    }
    pub fn set_HangulCompatibilityJamo(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn CJKMiscellaneous(&self) -> bool {
        self.byte5 & (1u8 << 5) != 0
    }
    pub fn set_CJKMiscellaneous(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn EnclosedCJKLettersAndMonths(&self) -> bool {
        self.byte5 & (1u8 << 6) != 0
    }
    pub fn set_EnclosedCJKLettersAndMonths(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:149-157
    pub fn CJKCompatibility(&self) -> bool {
        self.byte5 & (1u8 << 7) != 0
    }
    pub fn set_CJKCompatibility(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn ControlPictures(&self) -> bool {
        self.byte6 & (1u8 << 0) != 0
    }
    pub fn set_ControlPictures(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn OpticalCharacterRecognition(&self) -> bool {
        self.byte6 & (1u8 << 1) != 0
    }
    pub fn set_OpticalCharacterRecognition(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn EnclosedAlphanumerics(&self) -> bool {
        self.byte6 & (1u8 << 2) != 0
    }
    pub fn set_EnclosedAlphanumerics(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn BoxDrawing(&self) -> bool {
        self.byte6 & (1u8 << 3) != 0
    }
    pub fn set_BoxDrawing(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn BlockElements(&self) -> bool {
        self.byte6 & (1u8 << 4) != 0
    }
    pub fn set_BlockElements(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn GeometricShapes(&self) -> bool {
        self.byte6 & (1u8 << 5) != 0
    }
    pub fn set_GeometricShapes(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn MiscellaneousSymbols(&self) -> bool {
        self.byte6 & (1u8 << 6) != 0
    }
    pub fn set_MiscellaneousSymbols(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:159-167
    pub fn Dingbats(&self) -> bool {
        self.byte6 & (1u8 << 7) != 0
    }
    pub fn set_Dingbats(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn SuperscriptsAndSubscripts(&self) -> bool {
        self.byte7 & (1u8 << 0) != 0
    }
    pub fn set_SuperscriptsAndSubscripts(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn CurrencySymbols(&self) -> bool {
        self.byte7 & (1u8 << 1) != 0
    }
    pub fn set_CurrencySymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn CombiningDiacriticalMarksForSymbols(&self) -> bool {
        self.byte7 & (1u8 << 2) != 0
    }
    pub fn set_CombiningDiacriticalMarksForSymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn LetterlikeSymbols(&self) -> bool {
        self.byte7 & (1u8 << 3) != 0
    }
    pub fn set_LetterlikeSymbols(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn NumberForms(&self) -> bool {
        self.byte7 & (1u8 << 4) != 0
    }
    pub fn set_NumberForms(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn Arrows(&self) -> bool {
        self.byte7 & (1u8 << 5) != 0
    }
    pub fn set_Arrows(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn MathematicalOperators(&self) -> bool {
        self.byte7 & (1u8 << 6) != 0
    }
    pub fn set_MathematicalOperators(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:169-177
    pub fn MiscellaneousTechnical(&self) -> bool {
        self.byte7 & (1u8 << 7) != 0
    }
    pub fn set_MiscellaneousTechnical(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved088(&self) -> bool {
        self.byte8 & (1u8 << 0) != 0
    }
    pub fn set_Reserved088(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved089(&self) -> bool {
        self.byte8 & (1u8 << 1) != 0
    }
    pub fn set_Reserved089(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved090(&self) -> bool {
        self.byte8 & (1u8 << 2) != 0
    }
    pub fn set_Reserved090(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved091(&self) -> bool {
        self.byte8 & (1u8 << 3) != 0
    }
    pub fn set_Reserved091(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved092(&self) -> bool {
        self.byte8 & (1u8 << 4) != 0
    }
    pub fn set_Reserved092(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved093(&self) -> bool {
        self.byte8 & (1u8 << 5) != 0
    }
    pub fn set_Reserved093(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved094(&self) -> bool {
        self.byte8 & (1u8 << 6) != 0
    }
    pub fn set_Reserved094(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:180-188
    pub fn Reserved095(&self) -> bool {
        self.byte8 & (1u8 << 7) != 0
    }
    pub fn set_Reserved095(&mut self, value: bool) {
        self.byte8 = (self.byte8 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Khmer(&self) -> bool {
        self.byte9 & (1u8 << 0) != 0
    }
    pub fn set_Khmer(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Mongolian(&self) -> bool {
        self.byte9 & (1u8 << 1) != 0
    }
    pub fn set_Mongolian(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Braille(&self) -> bool {
        self.byte9 & (1u8 << 2) != 0
    }
    pub fn set_Braille(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Yi(&self) -> bool {
        self.byte9 & (1u8 << 3) != 0
    }
    pub fn set_Yi(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Reserved084(&self) -> bool {
        self.byte9 & (1u8 << 4) != 0
    }
    pub fn set_Reserved084(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Reserved085(&self) -> bool {
        self.byte9 & (1u8 << 5) != 0
    }
    pub fn set_Reserved085(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Reserved086(&self) -> bool {
        self.byte9 & (1u8 << 6) != 0
    }
    pub fn set_Reserved086(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:190-198
    pub fn Reserved087(&self) -> bool {
        self.byte9 & (1u8 << 7) != 0
    }
    pub fn set_Reserved087(&mut self, value: bool) {
        self.byte9 = (self.byte9 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Thaana(&self) -> bool {
        self.byte10 & (1u8 << 0) != 0
    }
    pub fn set_Thaana(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Sinhala(&self) -> bool {
        self.byte10 & (1u8 << 1) != 0
    }
    pub fn set_Sinhala(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Myanmar(&self) -> bool {
        self.byte10 & (1u8 << 2) != 0
    }
    pub fn set_Myanmar(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Ethiopic(&self) -> bool {
        self.byte10 & (1u8 << 3) != 0
    }
    pub fn set_Ethiopic(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Cherokee(&self) -> bool {
        self.byte10 & (1u8 << 4) != 0
    }
    pub fn set_Cherokee(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn UnifiedCanadianSyllabics(&self) -> bool {
        self.byte10 & (1u8 << 5) != 0
    }
    pub fn set_UnifiedCanadianSyllabics(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Ogham(&self) -> bool {
        self.byte10 & (1u8 << 6) != 0
    }
    pub fn set_Ogham(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:200-208
    pub fn Runic(&self) -> bool {
        self.byte10 & (1u8 << 7) != 0
    }
    pub fn set_Runic(&mut self, value: bool) {
        self.byte10 = (self.byte10 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn CombiningHalfMarks(&self) -> bool {
        self.byte11 & (1u8 << 0) != 0
    }
    pub fn set_CombiningHalfMarks(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn CJKCompatibilityForms(&self) -> bool {
        self.byte11 & (1u8 << 1) != 0
    }
    pub fn set_CJKCompatibilityForms(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn SmallFormVariants(&self) -> bool {
        self.byte11 & (1u8 << 2) != 0
    }
    pub fn set_SmallFormVariants(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn ArabicPresentationFormsB(&self) -> bool {
        self.byte11 & (1u8 << 3) != 0
    }
    pub fn set_ArabicPresentationFormsB(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn HalfwidthAndFullwidthForms(&self) -> bool {
        self.byte11 & (1u8 << 4) != 0
    }
    pub fn set_HalfwidthAndFullwidthForms(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn Specials(&self) -> bool {
        self.byte11 & (1u8 << 5) != 0
    }
    pub fn set_Specials(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn Tibetan(&self) -> bool {
        self.byte11 & (1u8 << 6) != 0
    }
    pub fn set_Tibetan(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:210-218
    pub fn Syriac(&self) -> bool {
        self.byte11 & (1u8 << 7) != 0
    }
    pub fn set_Syriac(&mut self, value: bool) {
        self.byte11 = (self.byte11 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved120(&self) -> bool {
        self.byte12 & (1u8 << 0) != 0
    }
    pub fn set_Reserved120(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved121(&self) -> bool {
        self.byte12 & (1u8 << 1) != 0
    }
    pub fn set_Reserved121(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved122(&self) -> bool {
        self.byte12 & (1u8 << 2) != 0
    }
    pub fn set_Reserved122(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved123(&self) -> bool {
        self.byte12 & (1u8 << 3) != 0
    }
    pub fn set_Reserved123(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved124(&self) -> bool {
        self.byte12 & (1u8 << 4) != 0
    }
    pub fn set_Reserved124(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved125(&self) -> bool {
        self.byte12 & (1u8 << 5) != 0
    }
    pub fn set_Reserved125(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved126(&self) -> bool {
        self.byte12 & (1u8 << 6) != 0
    }
    pub fn set_Reserved126(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:221-229
    pub fn Reserved127(&self) -> bool {
        self.byte12 & (1u8 << 7) != 0
    }
    pub fn set_Reserved127(&mut self, value: bool) {
        self.byte12 = (self.byte12 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved112(&self) -> bool {
        self.byte13 & (1u8 << 0) != 0
    }
    pub fn set_Reserved112(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved113(&self) -> bool {
        self.byte13 & (1u8 << 1) != 0
    }
    pub fn set_Reserved113(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved114(&self) -> bool {
        self.byte13 & (1u8 << 2) != 0
    }
    pub fn set_Reserved114(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved115(&self) -> bool {
        self.byte13 & (1u8 << 3) != 0
    }
    pub fn set_Reserved115(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved116(&self) -> bool {
        self.byte13 & (1u8 << 4) != 0
    }
    pub fn set_Reserved116(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved117(&self) -> bool {
        self.byte13 & (1u8 << 5) != 0
    }
    pub fn set_Reserved117(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved118(&self) -> bool {
        self.byte13 & (1u8 << 6) != 0
    }
    pub fn set_Reserved118(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:231-239
    pub fn Reserved119(&self) -> bool {
        self.byte13 & (1u8 << 7) != 0
    }
    pub fn set_Reserved119(&mut self, value: bool) {
        self.byte13 = (self.byte13 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved104(&self) -> bool {
        self.byte14 & (1u8 << 0) != 0
    }
    pub fn set_Reserved104(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved105(&self) -> bool {
        self.byte14 & (1u8 << 1) != 0
    }
    pub fn set_Reserved105(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved106(&self) -> bool {
        self.byte14 & (1u8 << 2) != 0
    }
    pub fn set_Reserved106(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved107(&self) -> bool {
        self.byte14 & (1u8 << 3) != 0
    }
    pub fn set_Reserved107(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved108(&self) -> bool {
        self.byte14 & (1u8 << 4) != 0
    }
    pub fn set_Reserved108(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved109(&self) -> bool {
        self.byte14 & (1u8 << 5) != 0
    }
    pub fn set_Reserved109(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved110(&self) -> bool {
        self.byte14 & (1u8 << 6) != 0
    }
    pub fn set_Reserved110(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:241-249
    pub fn Reserved111(&self) -> bool {
        self.byte14 & (1u8 << 7) != 0
    }
    pub fn set_Reserved111(&mut self, value: bool) {
        self.byte14 = (self.byte14 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved096(&self) -> bool {
        self.byte15 & (1u8 << 0) != 0
    }
    pub fn set_Reserved096(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved097(&self) -> bool {
        self.byte15 & (1u8 << 1) != 0
    }
    pub fn set_Reserved097(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved098(&self) -> bool {
        self.byte15 & (1u8 << 2) != 0
    }
    pub fn set_Reserved098(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved099(&self) -> bool {
        self.byte15 & (1u8 << 3) != 0
    }
    pub fn set_Reserved099(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved100(&self) -> bool {
        self.byte15 & (1u8 << 4) != 0
    }
    pub fn set_Reserved100(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved101(&self) -> bool {
        self.byte15 & (1u8 << 5) != 0
    }
    pub fn set_Reserved101(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved102(&self) -> bool {
        self.byte15 & (1u8 << 6) != 0
    }
    pub fn set_Reserved102(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:251-259
    pub fn Reserved103(&self) -> bool {
        self.byte15 & (1u8 << 7) != 0
    }
    pub fn set_Reserved103(&mut self, value: bool) {
        self.byte15 = (self.byte15 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:261-353
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_UnicodeRange_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:352-352
    pub value: [u32; 4],
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:262-295
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_UnicodeRange_Raw_l0 {}
impl SkOTTableOS2_V2_UnicodeRange_Raw_l0 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:263-263
    pub const BasicLatinMask: u32 = (1u32 << (0)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:264-264
    pub const Latin1SupplementMask: u32 = (1u32 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:265-265
    pub const LatinExtendedAMask: u32 = (1u32 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:266-266
    pub const LatinExtendedBMask: u32 = (1u32 << (3)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:267-267
    pub const IPAExtensionsMask: u32 = (1u32 << (4)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:268-268
    pub const SpacingModifierLettersMask: u32 = (1u32 << (5)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:269-269
    pub const CombiningDiacriticalMarksMask: u32 = (1u32 << (6)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:270-270
    pub const GreekMask: u32 = (1u32 << (7)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:272-272
    pub const CyrillicMask: u32 = (1u32 << (9)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:273-273
    pub const ArmenianMask: u32 = (1u32 << (10)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:274-274
    pub const HebrewMask: u32 = (1u32 << (11)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:276-276
    pub const ArabicMask: u32 = (1u32 << (13)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:278-278
    pub const DevanagariMask: u32 = (1u32 << (15)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:279-279
    pub const BengaliMask: u32 = (1u32 << (16)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:280-280
    pub const GurmukhiMask: u32 = (1u32 << (17)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:281-281
    pub const GujaratiMask: u32 = (1u32 << (18)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:282-282
    pub const OriyaMask: u32 = (1u32 << (19)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:283-283
    pub const TamilMask: u32 = (1u32 << (20)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:284-284
    pub const TeluguMask: u32 = (1u32 << (21)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:285-285
    pub const KannadaMask: u32 = (1u32 << (22)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:286-286
    pub const MalayalamMask: u32 = (1u32 << (23)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:287-287
    pub const ThaiMask: u32 = (1u32 << (24)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:288-288
    pub const LaoMask: u32 = (1u32 << (25)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:289-289
    pub const GeorgianMask: u32 = (1u32 << (26)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:291-291
    pub const HangulJamoMask: u32 = (1u32 << (28)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:292-292
    pub const LatinExtendedAdditionalMask: u32 = (1u32 << (29)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:293-293
    pub const GreekExtendedMask: u32 = (1u32 << (30)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:294-294
    pub const GeneralPunctuationMask: u32 = (1u32 << (31)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:296-329
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_UnicodeRange_Raw_l1 {}
impl SkOTTableOS2_V2_UnicodeRange_Raw_l1 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:297-297
    pub const SuperscriptsAndSubscriptsMask: u32 = (1u32 << (32 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:298-298
    pub const CurrencySymbolsMask: u32 = (1u32 << (33 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:299-299
    pub const CombiningDiacriticalMarksForSymbolsMask: u32 = (1u32 << (34 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:300-300
    pub const LetterlikeSymbolsMask: u32 = (1u32 << (35 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:301-301
    pub const NumberFormsMask: u32 = (1u32 << (36 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:302-302
    pub const ArrowsMask: u32 = (1u32 << (37 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:303-303
    pub const MathematicalOperatorsMask: u32 = (1u32 << (38 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:304-304
    pub const MiscellaneousTechnicalMask: u32 = (1u32 << (39 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:305-305
    pub const ControlPicturesMask: u32 = (1u32 << (40 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:306-306
    pub const OpticalCharacterRecognitionMask: u32 = (1u32 << (41 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:307-307
    pub const EnclosedAlphanumericsMask: u32 = (1u32 << (42 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:308-308
    pub const BoxDrawingMask: u32 = (1u32 << (43 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:309-309
    pub const BlockElementsMask: u32 = (1u32 << (44 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:310-310
    pub const GeometricShapesMask: u32 = (1u32 << (45 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:311-311
    pub const MiscellaneousSymbolsMask: u32 = (1u32 << (46 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:312-312
    pub const DingbatsMask: u32 = (1u32 << (47 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:313-313
    pub const CJKSymbolsAndPunctuationMask: u32 = (1u32 << (48 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:314-314
    pub const HiraganaMask: u32 = (1u32 << (49 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:315-315
    pub const KatakanaMask: u32 = (1u32 << (50 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:316-316
    pub const BopomofoMask: u32 = (1u32 << (51 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:317-317
    pub const HangulCompatibilityJamoMask: u32 = (1u32 << (52 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:318-318
    pub const CJKMiscellaneousMask: u32 = (1u32 << (53 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:319-319
    pub const EnclosedCJKLettersAndMonthsMask: u32 = (1u32 << (54 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:320-320
    pub const CJKCompatibilityMask: u32 = (1u32 << (55 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:321-321
    pub const HangulMask: u32 = (1u32 << (56 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:322-322
    pub const SurrogatesMask: u32 = (1u32 << (57 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:324-324
    pub const CJKUnifiedIdeographsMask: u32 = (1u32 << (59 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:325-325
    pub const PrivateUseAreaMask: u32 = (1u32 << (60 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:326-326
    pub const CJKCompatibilityIdeographsMask: u32 = (1u32 << (61 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:327-327
    pub const AlphabeticPresentationFormsMask: u32 = (1u32 << (62 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:328-328
    pub const ArabicPresentationFormsAMask: u32 = (1u32 << (63 - 32)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:330-351
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_UnicodeRange_Raw_l2 {}
impl SkOTTableOS2_V2_UnicodeRange_Raw_l2 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:331-331
    pub const CombiningHalfMarksMask: u32 = (1u32 << (64 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:332-332
    pub const CJKCompatibilityFormsMask: u32 = (1u32 << (65 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:333-333
    pub const SmallFormVariantsMask: u32 = (1u32 << (66 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:334-334
    pub const ArabicPresentationFormsBMask: u32 = (1u32 << (67 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:335-335
    pub const HalfwidthAndFullwidthFormsMask: u32 = (1u32 << (68 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:336-336
    pub const SpecialsMask: u32 = (1u32 << (69 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:337-337
    pub const TibetanMask: u32 = (1u32 << (70 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:338-338
    pub const SyriacMask: u32 = (1u32 << (71 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:339-339
    pub const ThaanaMask: u32 = (1u32 << (72 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:340-340
    pub const SinhalaMask: u32 = (1u32 << (73 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:341-341
    pub const MyanmarMask: u32 = (1u32 << (74 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:342-342
    pub const EthiopicMask: u32 = (1u32 << (75 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:343-343
    pub const CherokeeMask: u32 = (1u32 << (76 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:344-344
    pub const UnifiedCanadianSyllabicsMask: u32 = (1u32 << (77 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:345-345
    pub const OghamMask: u32 = (1u32 << (78 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:346-346
    pub const RunicMask: u32 = (1u32 << (79 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:347-347
    pub const KhmerMask: u32 = (1u32 << (80 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:348-348
    pub const MongolianMask: u32 = (1u32 << (81 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:349-349
    pub const BrailleMask: u32 = (1u32 << (82 - 64)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:350-350
    pub const YiMask: u32 = (1u32 << (83 - 64)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:356-389
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V2_Selection {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:378-378
    pub field: SkOTTableOS2_V2_Selection_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:388-388
    pub raw: SkOTTableOS2_V2_Selection_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:357-378
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-359
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-369
    pub byte1: u8,
}
impl SkOTTableOS2_V2_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved08(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved09(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:359-367
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Italic(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Italic(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Underscore(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Underscore(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Negative(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Negative(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Outlined(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Outlined(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Strikeout(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Strikeout(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Bold(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Bold(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Regular(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Regular(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:369-377
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:379-388
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:387-387
    pub value: u16,
}
impl SkOTTableOS2_V2_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:380-380
    pub const ItalicMask: u16 = (1u16 << (0)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:381-381
    pub const UnderscoreMask: u16 = (1u16 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:382-382
    pub const NegativeMask: u16 = (1u16 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:383-383
    pub const OutlinedMask: u16 = (1u16 << (3)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:384-384
    pub const StrikeoutMask: u16 = (1u16 << (4)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:385-385
    pub const BoldMask: u16 = (1u16 << (5)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:386-386
    pub const RegularMask: u16 = (1u16 << (6)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:399-524
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V2_CodePageRange {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:482-482
    pub field: SkOTTableOS2_V2_CodePageRange_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:523-523
    pub raw: SkOTTableOS2_V2_CodePageRange_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:400-482
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_CodePageRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-402
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-412
    pub byte1: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-422
    pub byte2: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-432
    pub byte3: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-443
    pub byte4: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-453
    pub byte5: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-463
    pub byte6: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-473
    pub byte7: u8,
}
impl SkOTTableOS2_V2_CodePageRange_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn Reserved24(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved24(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn Reserved25(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved25(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn Reserved26(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved26(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn Reserved27(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved27(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn Reserved28(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved28(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn MacintoshCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_MacintoshCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn OEMCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_OEMCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:402-410
    pub fn SymbolCharacterSet(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_SymbolCharacterSet(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn Thai_874(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Thai_874(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn JISJapan_932(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_JISJapan_932(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn ChineseSimplified_936(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_ChineseSimplified_936(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn KoreanWansung_949(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_KoreanWansung_949(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn ChineseTraditional_950(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_ChineseTraditional_950(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn KoreanJohab_1361(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_KoreanJohab_1361(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn Reserved22(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved22(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:412-420
    pub fn Reserved23(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved23(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Vietnamese(&self) -> bool {
        self.byte2 & (1u8 << 0) != 0
    }
    pub fn set_Vietnamese(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved09(&self) -> bool {
        self.byte2 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved10(&self) -> bool {
        self.byte2 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved11(&self) -> bool {
        self.byte2 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved12(&self) -> bool {
        self.byte2 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved13(&self) -> bool {
        self.byte2 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved14(&self) -> bool {
        self.byte2 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:422-430
    pub fn Reserved15(&self) -> bool {
        self.byte2 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte2 = (self.byte2 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Latin1_1252(&self) -> bool {
        self.byte3 & (1u8 << 0) != 0
    }
    pub fn set_Latin1_1252(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Latin2EasternEurope_1250(&self) -> bool {
        self.byte3 & (1u8 << 1) != 0
    }
    pub fn set_Latin2EasternEurope_1250(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Cyrillic_1251(&self) -> bool {
        self.byte3 & (1u8 << 2) != 0
    }
    pub fn set_Cyrillic_1251(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Greek_1253(&self) -> bool {
        self.byte3 & (1u8 << 3) != 0
    }
    pub fn set_Greek_1253(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Turkish_1254(&self) -> bool {
        self.byte3 & (1u8 << 4) != 0
    }
    pub fn set_Turkish_1254(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Hebrew_1255(&self) -> bool {
        self.byte3 & (1u8 << 5) != 0
    }
    pub fn set_Hebrew_1255(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn Arabic_1256(&self) -> bool {
        self.byte3 & (1u8 << 6) != 0
    }
    pub fn set_Arabic_1256(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:432-440
    pub fn WindowsBaltic_1257(&self) -> bool {
        self.byte3 & (1u8 << 7) != 0
    }
    pub fn set_WindowsBaltic_1257(&mut self, value: bool) {
        self.byte3 = (self.byte3 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn IBMTurkish_857(&self) -> bool {
        self.byte4 & (1u8 << 0) != 0
    }
    pub fn set_IBMTurkish_857(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn IBMCyrillic_855(&self) -> bool {
        self.byte4 & (1u8 << 1) != 0
    }
    pub fn set_IBMCyrillic_855(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn Latin2_852(&self) -> bool {
        self.byte4 & (1u8 << 2) != 0
    }
    pub fn set_Latin2_852(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn MSDOSBaltic_775(&self) -> bool {
        self.byte4 & (1u8 << 3) != 0
    }
    pub fn set_MSDOSBaltic_775(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn Greek_737(&self) -> bool {
        self.byte4 & (1u8 << 4) != 0
    }
    pub fn set_Greek_737(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn Arabic_708(&self) -> bool {
        self.byte4 & (1u8 << 5) != 0
    }
    pub fn set_Arabic_708(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn WELatin1_850(&self) -> bool {
        self.byte4 & (1u8 << 6) != 0
    }
    pub fn set_WELatin1_850(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:443-451
    pub fn US_437(&self) -> bool {
        self.byte4 & (1u8 << 7) != 0
    }
    pub fn set_US_437(&mut self, value: bool) {
        self.byte4 = (self.byte4 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn IBMGreek_869(&self) -> bool {
        self.byte5 & (1u8 << 0) != 0
    }
    pub fn set_IBMGreek_869(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn MSDOSRussian_866(&self) -> bool {
        self.byte5 & (1u8 << 1) != 0
    }
    pub fn set_MSDOSRussian_866(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn MSDOSNordic_865(&self) -> bool {
        self.byte5 & (1u8 << 2) != 0
    }
    pub fn set_MSDOSNordic_865(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn Arabic_864(&self) -> bool {
        self.byte5 & (1u8 << 3) != 0
    }
    pub fn set_Arabic_864(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn MSDOSCanadianFrench_863(&self) -> bool {
        self.byte5 & (1u8 << 4) != 0
    }
    pub fn set_MSDOSCanadianFrench_863(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn Hebrew_862(&self) -> bool {
        self.byte5 & (1u8 << 5) != 0
    }
    pub fn set_Hebrew_862(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn MSDOSIcelandic_861(&self) -> bool {
        self.byte5 & (1u8 << 6) != 0
    }
    pub fn set_MSDOSIcelandic_861(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:453-461
    pub fn MSDOSPortuguese_860(&self) -> bool {
        self.byte5 & (1u8 << 7) != 0
    }
    pub fn set_MSDOSPortuguese_860(&mut self, value: bool) {
        self.byte5 = (self.byte5 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved40(&self) -> bool {
        self.byte6 & (1u8 << 0) != 0
    }
    pub fn set_Reserved40(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved41(&self) -> bool {
        self.byte6 & (1u8 << 1) != 0
    }
    pub fn set_Reserved41(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved42(&self) -> bool {
        self.byte6 & (1u8 << 2) != 0
    }
    pub fn set_Reserved42(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved43(&self) -> bool {
        self.byte6 & (1u8 << 3) != 0
    }
    pub fn set_Reserved43(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved44(&self) -> bool {
        self.byte6 & (1u8 << 4) != 0
    }
    pub fn set_Reserved44(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved45(&self) -> bool {
        self.byte6 & (1u8 << 5) != 0
    }
    pub fn set_Reserved45(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved46(&self) -> bool {
        self.byte6 & (1u8 << 6) != 0
    }
    pub fn set_Reserved46(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:463-471
    pub fn Reserved47(&self) -> bool {
        self.byte6 & (1u8 << 7) != 0
    }
    pub fn set_Reserved47(&mut self, value: bool) {
        self.byte6 = (self.byte6 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved32(&self) -> bool {
        self.byte7 & (1u8 << 0) != 0
    }
    pub fn set_Reserved32(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved33(&self) -> bool {
        self.byte7 & (1u8 << 1) != 0
    }
    pub fn set_Reserved33(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved34(&self) -> bool {
        self.byte7 & (1u8 << 2) != 0
    }
    pub fn set_Reserved34(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved35(&self) -> bool {
        self.byte7 & (1u8 << 3) != 0
    }
    pub fn set_Reserved35(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved36(&self) -> bool {
        self.byte7 & (1u8 << 4) != 0
    }
    pub fn set_Reserved36(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved37(&self) -> bool {
        self.byte7 & (1u8 << 5) != 0
    }
    pub fn set_Reserved37(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved38(&self) -> bool {
        self.byte7 & (1u8 << 6) != 0
    }
    pub fn set_Reserved38(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:473-481
    pub fn Reserved39(&self) -> bool {
        self.byte7 & (1u8 << 7) != 0
    }
    pub fn set_Reserved39(&mut self, value: bool) {
        self.byte7 = (self.byte7 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:483-523
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_CodePageRange_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:522-522
    pub value: [u32; 2],
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:484-503
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_CodePageRange_Raw_l0 {}
impl SkOTTableOS2_V2_CodePageRange_Raw_l0 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:485-485
    pub const Latin1_1252Mask: u32 = (1u32 << (0)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:486-486
    pub const Latin2EasternEurope_1250Mask: u32 = (1u32 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:487-487
    pub const Cyrillic_1251Mask: u32 = (1u32 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:488-488
    pub const Greek_1253Mask: u32 = (1u32 << (3)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:489-489
    pub const Turkish_1254Mask: u32 = (1u32 << (4)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:490-490
    pub const Hebrew_1255Mask: u32 = (1u32 << (5)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:491-491
    pub const Arabic_1256Mask: u32 = (1u32 << (6)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:492-492
    pub const WindowsBaltic_1257Mask: u32 = (1u32 << (7)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:493-493
    pub const Vietnamese_1258Mask: u32 = (1u32 << (8)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:494-494
    pub const Thai_874Mask: u32 = (1u32 << (16)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:495-495
    pub const JISJapan_932Mask: u32 = (1u32 << (17)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:496-496
    pub const ChineseSimplified_936Mask: u32 = (1u32 << (18)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:497-497
    pub const KoreanWansung_949Mask: u32 = (1u32 << (19)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:498-498
    pub const ChineseTraditional_950Mask: u32 = (1u32 << (20)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:499-499
    pub const KoreanJohab_1361Mask: u32 = (1u32 << (21)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:500-500
    pub const MacintoshCharacterSetMask: u32 = (1u32 << (29)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:501-501
    pub const OEMCharacterSetMask: u32 = (1u32 << (30)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:502-502
    pub const SymbolCharacterSetMask: u32 = (1u32 << (31)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:504-521
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V2_CodePageRange_Raw_l1 {}
impl SkOTTableOS2_V2_CodePageRange_Raw_l1 {
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:505-505
    pub const IBMGreek_869Mask: u32 = (1u32 << (48 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:506-506
    pub const MSDOSRussian_866Mask: u32 = (1u32 << (49 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:507-507
    pub const MSDOSNordic_865Mask: u32 = (1u32 << (50 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:508-508
    pub const Arabic_864Mask: u32 = (1u32 << (51 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:509-509
    pub const MSDOSCanadianFrench_863Mask: u32 = (1u32 << (52 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:510-510
    pub const Hebrew_862Mask: u32 = (1u32 << (53 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:511-511
    pub const MSDOSIcelandic_861Mask: u32 = (1u32 << (54 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:512-512
    pub const MSDOSPortuguese_860Mask: u32 = (1u32 << (55 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:513-513
    pub const IBMTurkish_857Mask: u32 = (1u32 << (56 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:514-514
    pub const IBMCyrillic_855Mask: u32 = (1u32 << (57 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:515-515
    pub const Latin2_852Mask: u32 = (1u32 << (58 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:516-516
    pub const MSDOSBaltic_775Mask: u32 = (1u32 << (59 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:517-517
    pub const Greek_737Mask: u32 = (1u32 << (60 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:518-518
    pub const Arabic_708Mask: u32 = (1u32 << (61 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:519-519
    pub const WELatin1_850Mask: u32 = (1u32 << (62 - 32)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V2.h:520-520
    pub const US_437Mask: u32 = (1u32 << (63 - 32)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V2.h:536-536
const _: () = assert!(core::mem::size_of::<SkOTTableOS2_V2>() == 96);
