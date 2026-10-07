// C++: font_engine/fonts/font_variant_numeric.h. Its ToString declarations
// have no definitions in the supplied tree.

macro_rules! numeric_enum {
    ($name:ident { $($variant:ident = $value:expr),+ $(,)? }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(u8);
        #[allow(non_upper_case_globals)]
        impl $name {
            $(pub const $variant: Self = Self($value);)+
            pub const fn bits(self) -> u8 { self.0 }
        }
    };
}

// cpp: font_engine/fonts/font_variant_numeric.h:15-35
numeric_enum!(NumericFigure { kNormalFigure = 0, kLiningNums = 1, kOldstyleNums = 2 });
numeric_enum!(NumericSpacing { kNormalSpacing = 0, kProportionalNums = 1, kTabularNums = 2 });
numeric_enum!(NumericFraction { kNormalFraction = 0, kDiagonalFractions = 1, kStackedFractions = 2 });
numeric_enum!(Ordinal { kOrdinalOff = 0, kOrdinalOn = 1 });
numeric_enum!(SlashedZero { kSlashedZeroOff = 0, kSlashedZeroOn = 1 });

// cpp: font_engine/fonts/font_variant_numeric.h:12-98
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontVariantNumeric {
    fields_as_unsigned_: u32,
}

#[allow(non_snake_case)]
impl FontVariantNumeric {
    // cpp: font_engine/fonts/font_variant_numeric.h:37-41,78
    pub const fn InitializeFromUnsigned(init_value: u32) -> Self {
        Self {
            fields_as_unsigned_: init_value,
        }
    }
    pub(crate) const fn FieldsAsUnsigned(&self) -> u32 {
        self.fields_as_unsigned_
    }

    // cpp: font_engine/fonts/font_variant_numeric.h:43-56
    pub fn SetNumericFigure(&mut self, figure: NumericFigure) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !0b11) | u32::from(figure.bits() & 0b11);
    }
    pub fn SetNumericSpacing(&mut self, spacing: NumericSpacing) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !(0b11 << 2)) | (u32::from(spacing.bits() & 0b11) << 2);
    }
    pub fn SetNumericFraction(&mut self, fraction: NumericFraction) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !(0b11 << 4)) | (u32::from(fraction.bits() & 0b11) << 4);
    }
    pub fn SetOrdinal(&mut self, ordinal: Ordinal) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !(1 << 6)) | (u32::from(ordinal.bits() & 1) << 6);
    }
    pub fn SetSlashedZero(&mut self, slashed_zero: SlashedZero) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !(1 << 7)) | (u32::from(slashed_zero.bits() & 1) << 7);
    }

    // cpp: font_engine/fonts/font_variant_numeric.h:58-72
    pub fn NumericFigureValue(&self) -> NumericFigure {
        NumericFigure((self.fields_as_unsigned_ & 0b11) as u8)
    }
    pub fn NumericSpacingValue(&self) -> NumericSpacing {
        NumericSpacing(((self.fields_as_unsigned_ >> 2) & 0b11) as u8)
    }
    pub fn NumericFractionValue(&self) -> NumericFraction {
        NumericFraction(((self.fields_as_unsigned_ >> 4) & 0b11) as u8)
    }
    pub fn OrdinalValue(&self) -> Ordinal {
        Ordinal(((self.fields_as_unsigned_ >> 6) & 1) as u8)
    }
    pub fn SlashedZeroValue(&self) -> SlashedZero {
        SlashedZero(((self.fields_as_unsigned_ >> 7) & 1) as u8)
    }

    // cpp: font_engine/fonts/font_variant_numeric.h:74-78
    pub fn IsAllNormal(&self) -> bool {
        self.fields_as_unsigned_ == 0
    }
}
