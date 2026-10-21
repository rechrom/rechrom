// Packed native declarations; raw big-endian integer storage is retained.
// Transparent enum values admit every wire byte, including unknown values.
// Bit-field accessors use the source byte order and named low-to-high bit indices.
// No C++ object references are formed into unaligned font storage.
// Source: src/sfnt/SkOTTable_OS_2_V0.h:18-139
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0 {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:19-19
    pub version: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:24-24
    pub xAvgCharWidth: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:38-38
    pub usWeightClass: SkOTTableOS2_V0_WeightClass,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:51-51
    pub usWidthClass: SkOTTableOS2_V0_WidthClass,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:82-82
    pub fsType: SkOTTableOS2_V0_Type,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:83-83
    pub ySubscriptXSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:84-84
    pub ySubscriptYSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:85-85
    pub ySubscriptXOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:86-86
    pub ySubscriptYOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:87-87
    pub ySuperscriptXSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:88-88
    pub ySuperscriptYSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:89-89
    pub ySuperscriptXOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:90-90
    pub ySuperscriptYOffset: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:91-91
    pub yStrikeoutSize: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:92-92
    pub yStrikeoutPosition: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:93-93
    pub sFamilyClass: crate::src::sfnt::SkIBMFamilyClass::SkIBMFamilyClass,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:94-94
    pub panose: crate::src::sfnt::SkPanose::SkPanose,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:95-95
    pub ulCharRange: [u32; 4],
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:96-96
    pub achVendID: [i8; 4],
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:130-130
    pub fsSelection: SkOTTableOS2_V0_Selection,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:131-131
    pub usFirstCharIndex: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:132-132
    pub usLastCharIndex: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:134-134
    pub sTypoAscender: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:135-135
    pub sTypoDescender: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:136-136
    pub sTypoLineGap: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:137-137
    pub usWinAscent: u16,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:138-138
    pub usWinDescent: u16,
}
impl SkOTTableOS2_V0 {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:22-22
    pub const VERSION: u16 = (0u16).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:25-38
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_WeightClass {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:37-37
    pub value: u16,
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:26-36
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V0_WeightClass_Value(pub u16);
impl SkOTTableOS2_V0_WeightClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:27-27
    pub const Thin: Self = Self((100u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:28-28
    pub const ExtraLight: Self = Self((200u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:29-29
    pub const Light: Self = Self((300u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:30-30
    pub const Normal: Self = Self((400u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:31-31
    pub const Medium: Self = Self((500u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:32-32
    pub const SemiBold: Self = Self((600u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:33-33
    pub const Bold: Self = Self((700u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:34-34
    pub const ExtraBold: Self = Self((800u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:35-35
    pub const Black: Self = Self((900u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:39-51
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_WidthClass {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:50-50
    pub value: SkOTTableOS2_V0_WidthClass_Value,
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:40-50
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkOTTableOS2_V0_WidthClass_Value(pub u16);
impl SkOTTableOS2_V0_WidthClass_Value {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:41-41
    pub const UltraCondensed: Self = Self((1u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:42-42
    pub const ExtraCondensed: Self = Self((2u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:43-43
    pub const Condensed: Self = Self((3u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:44-44
    pub const SemiCondensed: Self = Self((4u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:45-45
    pub const Medium: Self = Self((5u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:46-46
    pub const SemiExpanded: Self = Self((6u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:47-47
    pub const Expanded: Self = Self((7u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:48-48
    pub const ExtraExpanded: Self = Self((8u16).to_be());
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:49-49
    pub const UltraExpanded: Self = Self((9u16).to_be());
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:52-82
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V0_Type {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:74-74
    pub field: SkOTTableOS2_V0_Type_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:81-81
    pub raw: SkOTTableOS2_V0_Type_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:53-74
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-55
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-65
    pub byte1: u8,
}
impl SkOTTableOS2_V0_Type_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved08(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved09(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:55-63
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Reserved00(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Reserved00(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Restricted(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Restricted(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn PreviewPrint(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_PreviewPrint(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Editable(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Editable(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Reserved04(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Reserved04(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Reserved05(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Reserved05(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Reserved06(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Reserved06(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:65-73
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:75-81
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:80-80
    pub value: u16,
}
impl SkOTTableOS2_V0_Type_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:76-76
    pub const Installable: u16 = 0;
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:77-77
    pub const RestrictedMask: u16 = (1u16 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:78-78
    pub const PreviewPrintMask: u16 = (1u16 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:79-79
    pub const EditableMask: u16 = (1u16 << (3)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:97-130
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkOTTableOS2_V0_Selection {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:119-119
    pub field: SkOTTableOS2_V0_Selection_Field,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:129-129
    pub raw: SkOTTableOS2_V0_Selection_Raw,
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:98-119
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-100
    pub byte0: u8,
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-110
    pub byte1: u8,
}
impl SkOTTableOS2_V0_Selection_Field {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved08(&self) -> bool {
        self.byte0 & (1u8 << 0) != 0
    }
    pub fn set_Reserved08(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved09(&self) -> bool {
        self.byte0 & (1u8 << 1) != 0
    }
    pub fn set_Reserved09(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved10(&self) -> bool {
        self.byte0 & (1u8 << 2) != 0
    }
    pub fn set_Reserved10(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved11(&self) -> bool {
        self.byte0 & (1u8 << 3) != 0
    }
    pub fn set_Reserved11(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved12(&self) -> bool {
        self.byte0 & (1u8 << 4) != 0
    }
    pub fn set_Reserved12(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved13(&self) -> bool {
        self.byte0 & (1u8 << 5) != 0
    }
    pub fn set_Reserved13(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved14(&self) -> bool {
        self.byte0 & (1u8 << 6) != 0
    }
    pub fn set_Reserved14(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:100-108
    pub fn Reserved15(&self) -> bool {
        self.byte0 & (1u8 << 7) != 0
    }
    pub fn set_Reserved15(&mut self, value: bool) {
        self.byte0 = (self.byte0 & !(1u8 << 7)) | ((value as u8) << 7);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Italic(&self) -> bool {
        self.byte1 & (1u8 << 0) != 0
    }
    pub fn set_Italic(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 0)) | ((value as u8) << 0);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Underscore(&self) -> bool {
        self.byte1 & (1u8 << 1) != 0
    }
    pub fn set_Underscore(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 1)) | ((value as u8) << 1);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Negative(&self) -> bool {
        self.byte1 & (1u8 << 2) != 0
    }
    pub fn set_Negative(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 2)) | ((value as u8) << 2);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Outlined(&self) -> bool {
        self.byte1 & (1u8 << 3) != 0
    }
    pub fn set_Outlined(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 3)) | ((value as u8) << 3);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Strikeout(&self) -> bool {
        self.byte1 & (1u8 << 4) != 0
    }
    pub fn set_Strikeout(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 4)) | ((value as u8) << 4);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Bold(&self) -> bool {
        self.byte1 & (1u8 << 5) != 0
    }
    pub fn set_Bold(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 5)) | ((value as u8) << 5);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Regular(&self) -> bool {
        self.byte1 & (1u8 << 6) != 0
    }
    pub fn set_Regular(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 6)) | ((value as u8) << 6);
    }
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:110-118
    pub fn Reserved07(&self) -> bool {
        self.byte1 & (1u8 << 7) != 0
    }
    pub fn set_Reserved07(&mut self, value: bool) {
        self.byte1 = (self.byte1 & !(1u8 << 7)) | ((value as u8) << 7);
    }
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:120-129
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableOS2_V0_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:128-128
    pub value: u16,
}
impl SkOTTableOS2_V0_Selection_Raw {
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:121-121
    pub const ItalicMask: u16 = (1u16 << (0)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:122-122
    pub const UnderscoreMask: u16 = (1u16 << (1)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:123-123
    pub const NegativeMask: u16 = (1u16 << (2)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:124-124
    pub const OutlinedMask: u16 = (1u16 << (3)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:125-125
    pub const StrikeoutMask: u16 = (1u16 << (4)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:126-126
    pub const BoldMask: u16 = (1u16 << (5)).to_be();
    // Source: src/sfnt/SkOTTable_OS_2_V0.h:127-127
    pub const RegularMask: u16 = (1u16 << (6)).to_be();
}
// Source: src/sfnt/SkOTTable_OS_2_V0.h:144-144
const _: () = assert!(core::mem::size_of::<SkOTTableOS2_V0>() == 78);
