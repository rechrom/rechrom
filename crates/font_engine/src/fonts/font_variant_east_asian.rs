// C++: font_engine/fonts/font_variant_east_asian.h. Its ToString declarations
// have no definitions in the supplied tree.

// cpp: font_engine/fonts/font_variant_east_asian.h:15-37
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EastAsianForm(u8);
#[allow(non_upper_case_globals)]
impl EastAsianForm {
    pub const kNormalForm: Self = Self(0);
    pub const kJis78: Self = Self(1);
    pub const kJis83: Self = Self(2);
    pub const kJis90: Self = Self(3);
    pub const kJis04: Self = Self(4);
    pub const kSimplified: Self = Self(5);
    pub const kTraditional: Self = Self(6);
    pub const fn bits(self) -> u8 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EastAsianWidth(u8);
#[allow(non_upper_case_globals)]
impl EastAsianWidth {
    pub const kNormalWidth: Self = Self(0);
    pub const kFullWidth: Self = Self(1);
    pub const kProportionalWidth: Self = Self(2);
    pub const fn bits(self) -> u8 {
        self.0
    }
}

// cpp: font_engine/fonts/font_variant_east_asian.h:12-82
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontVariantEastAsian {
    fields_as_unsigned_: u32,
}

#[allow(non_snake_case)]
impl FontVariantEastAsian {
    // cpp: font_engine/fonts/font_variant_east_asian.h:39-43,77
    pub const fn InitializeFromUnsigned(init_value: u32) -> Self {
        Self {
            fields_as_unsigned_: init_value,
        }
    }
    pub(crate) const fn FieldsAsUnsigned(&self) -> u32 {
        self.fields_as_unsigned_
    }

    // cpp: font_engine/fonts/font_variant_east_asian.h:45-52
    pub fn Form(&self) -> EastAsianForm {
        EastAsianForm((self.fields_as_unsigned_ & 0b111) as u8)
    }
    pub fn Width(&self) -> EastAsianWidth {
        EastAsianWidth(((self.fields_as_unsigned_ >> 3) & 0b11) as u8)
    }
    pub fn Ruby(&self) -> bool {
        (self.fields_as_unsigned_ & (1 << 5)) != 0
    }

    // cpp: font_engine/fonts/font_variant_east_asian.h:54-57
    pub fn SetForm(&mut self, form: EastAsianForm) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !0b111) | u32::from(form.bits() & 0b111);
    }
    pub fn SetWidth(&mut self, width: EastAsianWidth) {
        self.fields_as_unsigned_ =
            (self.fields_as_unsigned_ & !(0b11 << 3)) | (u32::from(width.bits() & 0b11) << 3);
    }
    pub fn SetRuby(&mut self, ruby: bool) {
        self.fields_as_unsigned_ = (self.fields_as_unsigned_ & !(1 << 5)) | (u32::from(ruby) << 5);
    }

    // cpp: font_engine/fonts/font_variant_east_asian.h:59-64
    pub fn IsAllNormal(&self) -> bool {
        self.fields_as_unsigned_ == 0
    }
}
