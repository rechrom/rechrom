#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// cpp: third_party/blink/renderer/core/css/white_space.h:11
use foundation::{TextWrapMode, TextWrapStyle};

// cpp: third_party/blink/renderer/core/css/white_space.h:21-33
// C++ enum class with u8 underlying type permits any combined bit pattern.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WhiteSpaceCollapse(u8);

#[allow(non_upper_case_globals)]
impl WhiteSpaceCollapse {
    pub const kCollapse: Self = Self(0);
    pub const kPreserve: Self = Self(1);
    pub const kPreserveBreaks: Self = Self(2);
    pub const kBreakSpaces: Self = Self(1 | 2);
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
}

// cpp: third_party/blink/renderer/core/css/white_space.h:35-38
pub const K_WHITE_SPACE_COLLAPSE_BITS: u32 = 2;
pub const K_WHITE_SPACE_COLLAPSE_MASK: u8 = (1 << K_WHITE_SPACE_COLLAPSE_BITS) - 1;

// cpp: third_party/blink/renderer/core/css/white_space.h:40-43
#[allow(non_snake_case)]
pub fn IsWhiteSpaceCollapseAny(value: WhiteSpaceCollapse, flags: WhiteSpaceCollapse) -> bool {
    (value.bits() & flags.bits()) != 0
}

// cpp: third_party/blink/renderer/core/css/white_space.h:45-64
#[allow(non_snake_case)]
pub fn ShouldPreserveWhiteSpaces(collapse: WhiteSpaceCollapse) -> bool {
    IsWhiteSpaceCollapseAny(collapse, WhiteSpaceCollapse::kPreserve)
}
#[allow(non_snake_case)]
pub fn ShouldCollapseWhiteSpaces(collapse: WhiteSpaceCollapse) -> bool {
    !ShouldPreserveWhiteSpaces(collapse)
}
#[allow(non_snake_case)]
pub fn ShouldPreserveBreaks(collapse: WhiteSpaceCollapse) -> bool {
    collapse != WhiteSpaceCollapse::kCollapse
}
#[allow(non_snake_case)]
pub fn ShouldCollapseBreaks(collapse: WhiteSpaceCollapse) -> bool {
    !ShouldPreserveBreaks(collapse)
}
#[allow(non_snake_case)]
pub fn ShouldBreakSpaces(collapse: WhiteSpaceCollapse) -> bool {
    collapse == WhiteSpaceCollapse::kBreakSpaces
}

// cpp: third_party/blink/renderer/core/css/white_space.h:70-71
pub const K_TEXT_WRAP_MODE_BITS: u32 =
    u8::BITS - TextWrapMode::kMaxEnumValue.value().leading_zeros();

// cpp: third_party/blink/renderer/core/css/white_space.h:73-76
#[allow(non_snake_case)]
pub fn ShouldWrapLine(mode: TextWrapMode) -> bool {
    mode != TextWrapMode::kNowrap
}

// cpp: third_party/blink/renderer/core/css/white_space.h:83-86
#[allow(non_snake_case)]
pub fn ShouldWrapLineGreedy(style: TextWrapStyle) -> bool {
    style == TextWrapStyle::kAuto || style == TextWrapStyle::kStable
}

// cpp: third_party/blink/renderer/core/css/white_space.h:92-101
#[allow(non_snake_case)]
pub fn ToWhiteSpaceValue(collapse: WhiteSpaceCollapse, wrap: TextWrapMode) -> u8 {
    collapse.bits() | (wrap.value() << K_WHITE_SPACE_COLLAPSE_BITS)
}

// cpp: third_party/blink/renderer/core/css/white_space.h:103-116
// The C++ cast can produce unnamed values, so EWhiteSpace is also a u8 newtype.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EWhiteSpace(u8);

#[allow(non_upper_case_globals)]
impl EWhiteSpace {
    pub const kNormal: Self = Self(4);
    pub const kNowrap: Self = Self(0);
    pub const kPre: Self = Self(1);
    pub const kPreLine: Self = Self(6);
    pub const kPreWrap: Self = Self(5);
    pub const kBreakSpaces: Self = Self(7);
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
}

// cpp: third_party/blink/renderer/core/css/white_space.h:118-119
const _: () = assert!(K_WHITE_SPACE_COLLAPSE_BITS + K_TEXT_WRAP_MODE_BITS <= u8::BITS);

// cpp: third_party/blink/renderer/core/css/white_space.h:121-126
#[allow(non_snake_case)]
pub fn ToWhiteSpace(collapse: WhiteSpaceCollapse, wrap: TextWrapMode) -> EWhiteSpace {
    EWhiteSpace::from_bits(ToWhiteSpaceValue(collapse, wrap))
}

// cpp: third_party/blink/renderer/core/css/white_space.h:128-135
#[allow(non_snake_case)]
pub fn IsValidWhiteSpace(whitespace: EWhiteSpace) -> bool {
    whitespace == EWhiteSpace::kNormal
        || whitespace == EWhiteSpace::kNowrap
        || whitespace == EWhiteSpace::kPre
        || whitespace == EWhiteSpace::kPreLine
        || whitespace == EWhiteSpace::kPreWrap
        || whitespace == EWhiteSpace::kBreakSpaces
}

// cpp: third_party/blink/renderer/core/css/white_space.h:137-141
#[allow(non_snake_case)]
pub fn ToWhiteSpaceCollapse(whitespace: EWhiteSpace) -> WhiteSpaceCollapse {
    WhiteSpaceCollapse::from_bits(whitespace.bits() & K_WHITE_SPACE_COLLAPSE_MASK)
}

// cpp: third_party/blink/renderer/core/css/white_space.h:142-145
#[allow(non_snake_case)]
pub fn ToTextWrapMode(whitespace: EWhiteSpace) -> TextWrapMode {
    TextWrapMode::from_bits(whitespace.bits() >> K_WHITE_SPACE_COLLAPSE_BITS)
}

#[cfg(test)]
mod tests {
    use crate::white_space;
    #[test]
    fn whitespace_all_longhand_combinations() {
        use foundation::TextWrapMode;
        use white_space::*;
        for collapse in 0..4 {
            for wrap in 0..2 {
                let collapse = WhiteSpaceCollapse::from_bits(collapse);
                let wrap = TextWrapMode::from_bits(wrap);
                let ws = ToWhiteSpace(collapse, wrap);
                assert_eq!(ToWhiteSpaceCollapse(ws), collapse);
                assert_eq!(ToTextWrapMode(ws), wrap);
                assert_eq!(
                    ShouldPreserveWhiteSpaces(collapse),
                    collapse.bits() & 1 != 0
                );
                assert_eq!(ShouldPreserveBreaks(collapse), collapse.bits() != 0);
                assert_eq!(IsValidWhiteSpace(ws), ws.bits() != 2 && ws.bits() != 3);
            }
        }
    }
}
