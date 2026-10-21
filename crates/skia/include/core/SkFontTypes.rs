// Source: include/core/SkFontTypes.h:11-16
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkTextEncoding(pub core::ffi::c_int);
impl SkTextEncoding {
    pub const kUTF8: Self = Self(0);
    pub const kUTF16: Self = Self(1);
    pub const kUTF32: Self = Self(2);
    pub const kGlyphID: Self = Self(3);
}
// Source: include/core/SkFontTypes.h:18-23
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkFontHinting(pub core::ffi::c_int);
impl SkFontHinting {
    pub const kNone: Self = Self(0);
    pub const kSlight: Self = Self(1);
    pub const kNormal: Self = Self(2);
    pub const kFull: Self = Self(3);
}
