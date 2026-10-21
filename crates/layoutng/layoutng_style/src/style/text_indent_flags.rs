// cpp: layoutng_style/style/text_indent_flags.h:12-19
// A transparent u8 newtype admits combined flags, unlike a closed Rust enum.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextIndentFlags(u8);

#[allow(non_upper_case_globals)]
impl TextIndentFlags {
    pub const kDefault: Self = Self(0);
    pub const kEachLine: Self = Self(1 << 0);
    pub const kHanging: Self = Self(1 << 1);
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
}

// cpp: layoutng_style/style/text_indent_flags.h:21-25
impl std::ops::BitOr for TextIndentFlags {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.bits() | other.bits())
    }
}

// cpp: layoutng_style/style/text_indent_flags.h:27-31
impl std::ops::BitAnd for TextIndentFlags {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.bits() & other.bits())
    }
}

// cpp: layoutng_style/style/text_indent_flags.h:33-37
impl std::ops::BitOrAssign for TextIndentFlags {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/text_indent_flags.h:39-42
impl std::ops::Not for TextIndentFlags {
    type Output = bool;
    fn not(self) -> bool {
        self.bits() == 0
    }
}
