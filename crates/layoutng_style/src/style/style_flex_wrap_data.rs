use super::computed_style_constants::FlexWrapMode;

// cpp: layoutng_style/style/style_flex_wrap_data.h:13-36
/// Encodes the two C++ bitfields in their original 2+1-bit widths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleFlexWrapData {
    bits_: u8,
}

#[allow(non_snake_case)]
impl StyleFlexWrapData {
    // cpp: layoutng_style/style/style_flex_wrap_data.h:17-19
    pub const fn new(wrap_mode: FlexWrapMode) -> Self {
        Self::with_balance(wrap_mode, false)
    }

    // cpp: layoutng_style/style/style_flex_wrap_data.h:20-22
    pub const fn with_balance(wrap_mode: FlexWrapMode, is_balanced: bool) -> Self {
        Self {
            bits_: (wrap_mode as u8 & 0b11) | ((is_balanced as u8) << 2),
        }
    }

    // cpp: layoutng_style/style/style_flex_wrap_data.h:24-26
    pub const fn GetWrapMode(&self) -> FlexWrapMode {
        match self.bits_ & 0b11 {
            0 => FlexWrapMode::kNowrap,
            1 => FlexWrapMode::kWrap,
            2 => FlexWrapMode::kWrapReverse,
            _ => unreachable!(),
        }
    }

    // cpp: layoutng_style/style/style_flex_wrap_data.h:27
    pub const fn IsBalanced(&self) -> bool {
        (self.bits_ & 0b100) != 0
    }
}

// Derived equality compares the two encoded fields, which occupy exactly
// the low three bits; constructors always clear the remaining bits.
