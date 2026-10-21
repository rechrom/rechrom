// Packed native declarations; raw big-endian integer storage is retained.
// Transparent enum values admit every wire byte, including unknown values.
// Bit-field accessors use the source byte order and named low-to-high bit indices.
// No C++ object references are formed into unaligned font storage.
// Source: src/sfnt/SkIBMFamilyClass.h:15-135
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkIBMFamilyClass {
    // Source: src/sfnt/SkIBMFamilyClass.h:31-31
    pub familyClass: SkIBMFamilyClass_Class,
    // Source: src/sfnt/SkIBMFamilyClass.h:134-134
    pub familySubClass: SkIBMFamilyClass_SubClass,
}
// Source: src/sfnt/SkIBMFamilyClass.h:16-31
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_Class(pub u8);
impl SkIBMFamilyClass_Class {
    // Source: src/sfnt/SkIBMFamilyClass.h:17-17
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:18-18
    pub const OldstyleSerifs: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:19-19
    pub const TransitionalSerifs: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:20-20
    pub const ModernSerifs: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:21-21
    pub const ClarendonSerifs: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:22-22
    pub const SlabSerifs: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:24-24
    pub const FreeformSerifs: Self = Self(7);
    // Source: src/sfnt/SkIBMFamilyClass.h:25-25
    pub const SansSerif: Self = Self(8);
    // Source: src/sfnt/SkIBMFamilyClass.h:26-26
    pub const Ornamentals: Self = Self(9);
    // Source: src/sfnt/SkIBMFamilyClass.h:27-27
    pub const Scripts: Self = Self(10);
    // Source: src/sfnt/SkIBMFamilyClass.h:29-29
    pub const Symbolic: Self = Self(12);
}
// Source: src/sfnt/SkIBMFamilyClass.h:32-134
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkIBMFamilyClass_SubClass {
    // Source: src/sfnt/SkIBMFamilyClass.h:45-45
    pub oldstyleSerifs: SkIBMFamilyClass_SubClass_OldstyleSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:52-52
    pub transitionalSerifs: SkIBMFamilyClass_SubClass_TransitionalSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:59-59
    pub modernSerifs: SkIBMFamilyClass_SubClass_ModernSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:71-71
    pub clarendonSerifs: SkIBMFamilyClass_SubClass_ClarendonSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:81-81
    pub slabSerifs: SkIBMFamilyClass_SubClass_SlabSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:87-87
    pub freeformSerifs: SkIBMFamilyClass_SubClass_FreeformSerifs,
    // Source: src/sfnt/SkIBMFamilyClass.h:101-101
    pub sansSerif: SkIBMFamilyClass_SubClass_SansSerif,
    // Source: src/sfnt/SkIBMFamilyClass.h:110-110
    pub ornamentals: SkIBMFamilyClass_SubClass_Ornamentals,
    // Source: src/sfnt/SkIBMFamilyClass.h:123-123
    pub scripts: SkIBMFamilyClass_SubClass_Scripts,
    // Source: src/sfnt/SkIBMFamilyClass.h:133-133
    pub symbolic: SkIBMFamilyClass_SubClass_Symbolic,
}
// Source: src/sfnt/SkIBMFamilyClass.h:33-45
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_OldstyleSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_OldstyleSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:34-34
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:35-35
    pub const IBMRoundedLegibility: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:36-36
    pub const Garalde: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:37-37
    pub const Venetian: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:38-38
    pub const ModifiedVenetian: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:39-39
    pub const DutchModern: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:40-40
    pub const DutchTraditional: Self = Self(6);
    // Source: src/sfnt/SkIBMFamilyClass.h:41-41
    pub const Contemporary: Self = Self(7);
    // Source: src/sfnt/SkIBMFamilyClass.h:42-42
    pub const Calligraphic: Self = Self(8);
    // Source: src/sfnt/SkIBMFamilyClass.h:44-44
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:46-52
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_TransitionalSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_TransitionalSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:47-47
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:48-48
    pub const DirectLine: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:49-49
    pub const Script: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:51-51
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:53-59
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_ModernSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_ModernSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:54-54
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:55-55
    pub const Italian: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:56-56
    pub const Script: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:58-58
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:60-71
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_ClarendonSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_ClarendonSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:61-61
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:62-62
    pub const Clarendon: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:63-63
    pub const Modern: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:64-64
    pub const Traditional: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:65-65
    pub const Newspaper: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:66-66
    pub const StubSerif: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:67-67
    pub const Monotone: Self = Self(6);
    // Source: src/sfnt/SkIBMFamilyClass.h:68-68
    pub const Typewriter: Self = Self(7);
    // Source: src/sfnt/SkIBMFamilyClass.h:70-70
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:72-81
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_SlabSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_SlabSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:73-73
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:74-74
    pub const Monotone: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:75-75
    pub const Humanist: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:76-76
    pub const Geometric: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:77-77
    pub const Swiss: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:78-78
    pub const Typewriter: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:80-80
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:82-87
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_FreeformSerifs(pub u8);
impl SkIBMFamilyClass_SubClass_FreeformSerifs {
    // Source: src/sfnt/SkIBMFamilyClass.h:83-83
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:84-84
    pub const Modern: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:86-86
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:88-101
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_SansSerif(pub u8);
impl SkIBMFamilyClass_SubClass_SansSerif {
    // Source: src/sfnt/SkIBMFamilyClass.h:89-89
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:90-90
    pub const IBMNeoGrotesqueGothic: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:91-91
    pub const Humanist: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:92-92
    pub const LowXRoundGeometric: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:93-93
    pub const HighXRoundGeometric: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:94-94
    pub const NeoGrotesqueGothic: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:95-95
    pub const ModifiedNeoGrotesqueGothic: Self = Self(6);
    // Source: src/sfnt/SkIBMFamilyClass.h:97-97
    pub const TypewriterGothic: Self = Self(9);
    // Source: src/sfnt/SkIBMFamilyClass.h:98-98
    pub const Matrix: Self = Self(10);
    // Source: src/sfnt/SkIBMFamilyClass.h:100-100
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:102-110
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_Ornamentals(pub u8);
impl SkIBMFamilyClass_SubClass_Ornamentals {
    // Source: src/sfnt/SkIBMFamilyClass.h:103-103
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:104-104
    pub const Engraver: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:105-105
    pub const BlackLetter: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:106-106
    pub const Decorative: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:107-107
    pub const ThreeDimensional: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:109-109
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:111-123
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_Scripts(pub u8);
impl SkIBMFamilyClass_SubClass_Scripts {
    // Source: src/sfnt/SkIBMFamilyClass.h:112-112
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:113-113
    pub const Uncial: Self = Self(1);
    // Source: src/sfnt/SkIBMFamilyClass.h:114-114
    pub const Brush_Joined: Self = Self(2);
    // Source: src/sfnt/SkIBMFamilyClass.h:115-115
    pub const Formal_Joined: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:116-116
    pub const Monotone_Joined: Self = Self(4);
    // Source: src/sfnt/SkIBMFamilyClass.h:117-117
    pub const Calligraphic: Self = Self(5);
    // Source: src/sfnt/SkIBMFamilyClass.h:118-118
    pub const Brush_Unjoined: Self = Self(6);
    // Source: src/sfnt/SkIBMFamilyClass.h:119-119
    pub const Formal_Unjoined: Self = Self(7);
    // Source: src/sfnt/SkIBMFamilyClass.h:120-120
    pub const Monotone_Unjoined: Self = Self(8);
    // Source: src/sfnt/SkIBMFamilyClass.h:122-122
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:124-133
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkIBMFamilyClass_SubClass_Symbolic(pub u8);
impl SkIBMFamilyClass_SubClass_Symbolic {
    // Source: src/sfnt/SkIBMFamilyClass.h:125-125
    pub const NoClassification: Self = Self(0);
    // Source: src/sfnt/SkIBMFamilyClass.h:127-127
    pub const MixedSerif: Self = Self(3);
    // Source: src/sfnt/SkIBMFamilyClass.h:129-129
    pub const OldstyleSerif: Self = Self(6);
    // Source: src/sfnt/SkIBMFamilyClass.h:130-130
    pub const NeoGrotesqueSansSerif: Self = Self(7);
    // Source: src/sfnt/SkIBMFamilyClass.h:132-132
    pub const Miscellaneous: Self = Self(15);
}
// Source: src/sfnt/SkIBMFamilyClass.h:140-140
const _: () = assert!(core::mem::size_of::<SkIBMFamilyClass>() == 2);
