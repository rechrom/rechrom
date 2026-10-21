// C++: foundation/text/native/hanging_punctuation.h. The declared
// IsHangingPunctuation(UChar32, mask) has no definition in the supplied tree.
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

// cpp: foundation/text/native/hanging_punctuation.h:14-20
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HangingPunctuation(u32);

#[allow(non_upper_case_globals)]
impl HangingPunctuation {
    pub const kNone: Self = Self(0);
    pub const kFirst: Self = Self(1);
    pub const kLast: Self = Self(2);
    pub const kAllowEnd: Self = Self(4);
    // C++ casts to the underlying bit-mask type preserve unnamed combinations.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
}

// cpp: foundation/text/native/hanging_punctuation.h:22-50
impl BitOr for HangingPunctuation {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for HangingPunctuation {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl BitXor for HangingPunctuation {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for HangingPunctuation {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}
impl BitAnd for HangingPunctuation {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitAndAssign for HangingPunctuation {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl Not for HangingPunctuation {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
