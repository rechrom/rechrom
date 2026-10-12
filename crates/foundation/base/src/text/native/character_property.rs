// C++: src/foundation/text/native/character_property.h

use super::east_asian_spacing_type::EastAsianSpacingType;
use super::han_kerning_char_type::HanKerningCharType;

pub type CharacterPropertyType = u16;

// C++ struct CharacterProperty -> Rust struct + impl. Rust has no bitfields;
// the single u16 stores the source fields in declaration order.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CharacterProperty {
    value_: CharacterPropertyType,
}

impl CharacterProperty {
    // C++: CharacterProperty() : CharacterProperty(0).
    pub const fn new() -> Self {
        Self::from_unsigned(0)
    }

    // C++: explicit CharacterProperty(CharacterPropertyType value).
    pub const fn from_unsigned(value: CharacterPropertyType) -> Self {
        Self { value_: value }
    }

    // C++: AsUnsigned.
    pub const fn AsUnsigned(&self) -> CharacterPropertyType {
        self.value_
    }

    // C++ public bitfields, in the original declaration order.
    pub const fn is_cjk_ideograph_or_symbol(&self) -> bool {
        self.value_ & (1 << 0) != 0
    }

    pub fn set_is_cjk_ideograph_or_symbol(&mut self, value: bool) {
        self.set_flag(0, value);
    }

    pub const fn is_potential_custom_element_name_char(&self) -> bool {
        self.value_ & (1 << 1) != 0
    }

    pub fn set_is_potential_custom_element_name_char(&mut self, value: bool) {
        self.set_flag(1, value);
    }

    pub const fn is_bidi_control(&self) -> bool {
        self.value_ & (1 << 2) != 0
    }

    pub fn set_is_bidi_control(&mut self, value: bool) {
        self.set_flag(2, value);
    }

    pub const fn is_hangul(&self) -> bool {
        self.value_ & (1 << 3) != 0
    }

    pub fn set_is_hangul(&mut self, value: bool) {
        self.set_flag(3, value);
    }

    pub fn han_kerning(&self) -> HanKerningCharType {
        // All values currently stored in CharacterProperty are 0..=10.
        match (self.value_ >> 4) & 0x0f {
            0 => HanKerningCharType::kOther,
            1 => HanKerningCharType::kOpen,
            2 => HanKerningCharType::kClose,
            3 => HanKerningCharType::kMiddle,
            4 => HanKerningCharType::kOpenNarrow,
            5 => HanKerningCharType::kCloseNarrow,
            6 => HanKerningCharType::kDot,
            7 => HanKerningCharType::kColon,
            8 => HanKerningCharType::kSemicolon,
            9 => HanKerningCharType::kOpenQuote,
            10 => HanKerningCharType::kCloseQuote,
            _ => HanKerningCharType::kInvalid,
        }
    }

    pub fn set_han_kerning(&mut self, value: HanKerningCharType) {
        self.value_ = (self.value_ & !(0x0f << 4)) | (((value as u16) & 0x0f) << 4);
    }

    pub fn east_asian_spacing(&self) -> EastAsianSpacingType {
        match (self.value_ >> 8) & 0x03 {
            0 => EastAsianSpacingType::kOther,
            1 => EastAsianSpacingType::kNarrow,
            2 => EastAsianSpacingType::kConditional,
            _ => EastAsianSpacingType::kWide,
        }
    }

    pub fn set_east_asian_spacing(&mut self, value: EastAsianSpacingType) {
        self.value_ = (self.value_ & !(0x03 << 8)) | (((value as u16) & 0x03) << 8);
    }

    fn set_flag(&mut self, shift: u16, value: bool) {
        self.value_ = (self.value_ & !(1 << shift)) | ((value as u16) << shift);
    }
}

// C++: CharacterProperty::operator==.
impl PartialEq for CharacterProperty {
    fn eq(&self, other: &Self) -> bool {
        self.AsUnsigned() == other.AsUnsigned()
    }
}

impl Eq for CharacterProperty {}

impl Default for CharacterProperty {
    fn default() -> Self {
        Self::new()
    }
}
