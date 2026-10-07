// C++: src/foundation/style_values/style/computed_style_base_constants.h:769-810,
// 916-998. These C++ enum classes have `unsigned` storage and support bitwise
// combinations. Rust enum discriminants cannot represent unnamed combinations,
// so repr(transparent) value types preserve the full underlying bit pattern.
#![allow(non_upper_case_globals)]

macro_rules! style_flags {
    ($name:ident, $bits_name:ident, $width:expr, {$($variant:ident = $value:expr),+ $(,)?}) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name(u32);

        impl $name {
            $(pub const $variant: Self = Self($value);)+

            pub const fn from_bits(bits: u32) -> Self { Self(bits) }
            pub const fn bits(self) -> u32 { self.0 }
        }

        pub const $bits_name: i32 = $width;

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, other: Self) -> Self { Self(self.0 | other.0) }
        }
        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) { self.0 |= other.0; }
        }
        impl std::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, other: Self) -> Self { Self(self.0 ^ other.0) }
        }
        impl std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, other: Self) { self.0 ^= other.0; }
        }
        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, other: Self) -> Self { Self(self.0 & other.0) }
        }
        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, other: Self) { self.0 &= other.0; }
        }
        impl std::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self { Self(!self.0) }
        }
    };
}

// C++: computed_style_base_constants.h:769-810.
style_flags!(ETextTransform, kETextTransformBits, 6, {
    kNone = 0,
    kCapitalize = 1,
    kUppercase = 2,
    kLowercase = 4,
    kFullWidth = 8,
    kFullSizeKana = 16,
    kMathAuto = 32,
});

// C++: computed_style_base_constants.h:916-957.
style_flags!(TextDecorationLine, kTextDecorationLineBits, 6, {
    kNone = 0,
    kUnderline = 1,
    kOverline = 2,
    kLineThrough = 4,
    kBlink = 8,
    kSpellingError = 16,
    kGrammarError = 32,
});

// C++: computed_style_base_constants.h:959-997.
style_flags!(TextDecorationSkipSpaces, kTextDecorationSkipSpacesBits, 3, {
    kNone = 0,
    kStart = 1,
    kEnd = 2,
    kAll = 4,
});

// The style and layout crates keep these bit-pattern conversions at their
// foundation source boundary. Unnamed C++ flag combinations remain intact.
#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextTransformFromBits(bits: u32) -> ETextTransform {
    ETextTransform::from_bits(bits)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextTransformBits(value: ETextTransform) -> u32 {
    value.bits()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextDecorationLineFromBits(bits: u32) -> TextDecorationLine {
    TextDecorationLine::from_bits(bits)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextDecorationLineBits(value: TextDecorationLine) -> u32 {
    value.bits()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextDecorationSkipSpacesFromBits(
    bits: u32,
) -> TextDecorationSkipSpaces {
    TextDecorationSkipSpaces::from_bits(bits)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn FoundationTextDecorationSkipSpacesBits(
    value: TextDecorationSkipSpaces,
) -> u32 {
    value.bits()
}
