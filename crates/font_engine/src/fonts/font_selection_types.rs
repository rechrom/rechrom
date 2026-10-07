// C++: font_engine/fonts/font_selection_types.h. The source unit remains
// blocked by the undeclared implementations of GetHash and ToString.

// cpp: font_engine/fonts/font_selection_types.h:40-104
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct FontSelectionValue {
    backing_: i16,
}

impl FontSelectionValue {
    const FRACTIONAL_ENTROPY: i32 = 4;

    pub const fn from_raw(raw: i16) -> Self {
        Self { backing_: raw }
    }

    // C++'s ClampTo<int16_t> narrows after multiplying by four.
    pub const fn from_int(value: i32) -> Self {
        let scaled = (value as i64) * (Self::FRACTIONAL_ENTROPY as i64);
        let bounded = if scaled < i16::MIN as i64 {
            i16::MIN as i64
        } else if scaled > i16::MAX as i64 {
            i16::MAX as i64
        } else {
            scaled
        };
        Self::from_raw(bounded as i16)
    }

    pub fn from_float(value: f32) -> Self {
        Self::from_raw((value * Self::FRACTIONAL_ENTROPY as f32) as i16)
    }

    pub fn from_double(value: f64) -> Self {
        Self::from_raw((value * Self::FRACTIONAL_ENTROPY as f64) as i16)
    }

    pub const fn ToFloat(self) -> f32 {
        self.backing_ as f32 / Self::FRACTIONAL_ENTROPY as f32
    }

    pub const fn RawValue(self) -> i16 {
        self.backing_
    }

    pub const fn MaximumValue() -> Self {
        Self::from_raw(i16::MAX)
    }

    pub const fn MinimumValue() -> Self {
        Self::from_raw(i16::MIN)
    }

    // cpp: font_engine/fonts/font_selection_types.h:176-179
    pub const fn ClampToObliqueRange(self) -> Self {
        if self.backing_ < kMinObliqueValue.backing_ {
            kMinObliqueValue
        } else if self.backing_ > kMaxObliqueValue.backing_ {
            kMaxObliqueValue
        } else {
            self
        }
    }
}

// cpp: font_engine/fonts/font_selection_types.h:106-109
impl std::ops::Add for FontSelectionValue {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::from_raw(((self.backing_ as i32) + (other.backing_ as i32)) as i16)
    }
}

// cpp: font_engine/fonts/font_selection_types.h:111-114
impl std::ops::Sub for FontSelectionValue {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::from_raw(((self.backing_ as i32) - (other.backing_ as i32)) as i16)
    }
}

// cpp: font_engine/fonts/font_selection_types.h:116-121
impl std::ops::Mul for FontSelectionValue {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::from_raw(
            ((self.backing_ as i32 * other.backing_ as i32) / Self::FRACTIONAL_ENTROPY) as i16,
        )
    }
}

// cpp: font_engine/fonts/font_selection_types.h:123-128
impl std::ops::Div for FontSelectionValue {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::from_raw(
            ((self.backing_ as i32 / other.backing_ as i32) * Self::FRACTIONAL_ENTROPY) as i16,
        )
    }
}

// cpp: font_engine/fonts/font_selection_types.h:130-132
impl std::ops::Neg for FontSelectionValue {
    type Output = Self;
    fn neg(self) -> Self {
        Self::from_raw(-(self.backing_ as i32) as i16)
    }
}

// cpp: font_engine/fonts/font_selection_types.h:134-157
// The five comparison operators use the derived i16 order/equality above.

// cpp: font_engine/fonts/font_selection_types.h:159-174
pub const kItalicThreshold: FontSelectionValue = FontSelectionValue::from_int(14);
pub const kFontSelectionZeroValue: FontSelectionValue = FontSelectionValue::from_int(0);
pub const kNormalSlopeValue: FontSelectionValue = FontSelectionValue::from_raw(0);
pub const kItalicSlopeValue: FontSelectionValue = FontSelectionValue::from_int(14);
pub const kMaxObliqueValue: FontSelectionValue = FontSelectionValue::from_int(90);
pub const kMinObliqueValue: FontSelectionValue = FontSelectionValue::from_int(-90);

// cpp: font_engine/fonts/font_selection_types.h:161-163
pub const fn isItalic(font_style: FontSelectionValue) -> bool {
    font_style.RawValue() >= kItalicThreshold.RawValue()
}

// cpp: font_engine/fonts/font_selection_types.h:181-244
pub const kBoldThreshold: FontSelectionValue = FontSelectionValue::from_int(600);
pub const kMinWeightValue: FontSelectionValue = FontSelectionValue::from_int(1);
pub const kMaxWeightValue: FontSelectionValue = FontSelectionValue::from_int(1000);
pub const kBlackWeightValue: FontSelectionValue = FontSelectionValue::from_int(900);
pub const kExtraBoldWeightValue: FontSelectionValue = FontSelectionValue::from_int(800);
pub const kBoldWeightValue: FontSelectionValue = FontSelectionValue::from_int(700);
pub const kSemiBoldWeightValue: FontSelectionValue = FontSelectionValue::from_int(600);
pub const kMediumWeightValue: FontSelectionValue = FontSelectionValue::from_int(500);
pub const kNormalWeightValue: FontSelectionValue = FontSelectionValue::from_int(400);
pub const kLightWeightValue: FontSelectionValue = FontSelectionValue::from_int(300);
pub const kExtraLightWeightValue: FontSelectionValue = FontSelectionValue::from_int(200);
pub const kThinWeightValue: FontSelectionValue = FontSelectionValue::from_int(100);
pub const kUpperWeightSearchThreshold: FontSelectionValue = FontSelectionValue::from_int(500);
pub const kLowerWeightSearchThreshold: FontSelectionValue = FontSelectionValue::from_int(400);
pub const kUltraCondensedWidthValue: FontSelectionValue = FontSelectionValue::from_int(50);
pub const kExtraCondensedWidthValue: FontSelectionValue = FontSelectionValue::from_raw(250);
pub const kCondensedWidthValue: FontSelectionValue = FontSelectionValue::from_int(75);
pub const kSemiCondensedWidthValue: FontSelectionValue = FontSelectionValue::from_raw(350);
pub const kNormalWidthValue: FontSelectionValue = FontSelectionValue::from_int(100);
pub const kSemiExpandedWidthValue: FontSelectionValue = FontSelectionValue::from_raw(450);
pub const kExpandedWidthValue: FontSelectionValue = FontSelectionValue::from_int(125);
pub const kExtraExpandedWidthValue: FontSelectionValue = FontSelectionValue::from_int(150);
pub const kUltraExpandedWidthValue: FontSelectionValue = FontSelectionValue::from_int(200);

// cpp: font_engine/fonts/font_selection_types.h:210-212
pub const fn isFontWeightBold(font_weight: FontSelectionValue) -> bool {
    font_weight.RawValue() >= kBoldThreshold.RawValue()
}

// cpp: font_engine/fonts/font_selection_types.h:246-298
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeType {
    kSetFromAuto,
    kSetExplicitly,
}

#[derive(Clone, Copy, Debug)]
pub struct FontSelectionRange {
    pub minimum: FontSelectionValue,
    pub maximum: FontSelectionValue,
    pub type_: RangeType,
}

impl Default for FontSelectionRange {
    fn default() -> Self {
        Self::new(
            FontSelectionValue::from_int(1),
            FontSelectionValue::from_int(0),
        )
    }
}

impl PartialEq for FontSelectionRange {
    fn eq(&self, other: &Self) -> bool {
        self.minimum == other.minimum && self.maximum == other.maximum
    }
}

impl Eq for FontSelectionRange {}

impl FontSelectionRange {
    pub const fn single(value: FontSelectionValue) -> Self {
        Self::new(value, value)
    }

    pub const fn new(minimum: FontSelectionValue, maximum: FontSelectionValue) -> Self {
        Self::with_type(minimum, maximum, RangeType::kSetFromAuto)
    }

    pub const fn with_type(
        minimum: FontSelectionValue,
        maximum: FontSelectionValue,
        type_: RangeType,
    ) -> Self {
        Self {
            minimum,
            maximum,
            type_,
        }
    }

    pub fn IsValid(&self) -> bool {
        self.minimum <= self.maximum
    }

    pub fn IsRange(&self) -> bool {
        self.maximum > self.minimum
    }

    pub fn IsRangeSetFromAuto(&self) -> bool {
        self.type_ == RangeType::kSetFromAuto
    }

    pub fn Expand(&mut self, other: &Self) {
        debug_assert!(other.IsValid());
        if !self.IsValid() {
            *self = *other;
        } else {
            self.minimum = self.minimum.min(other.minimum);
            self.maximum = self.maximum.max(other.maximum);
        }
        debug_assert!(self.IsValid());
    }

    pub fn Includes(&self, target: FontSelectionValue) -> bool {
        target >= self.minimum && target <= self.maximum
    }

    pub fn UniqueValue(&self) -> u32 {
        ((self.minimum.RawValue() as i32) << 16 | self.maximum.RawValue() as i32) as u32
    }

    pub fn clampToRange(&self, selection_value: FontSelectionValue) -> FontSelectionValue {
        selection_value.clamp(self.minimum, self.maximum)
    }
}

// cpp: font_engine/fonts/font_selection_types.h:300-320
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontSelectionRequest {
    pub weight: FontSelectionValue,
    pub width: FontSelectionValue,
    pub slope: FontSelectionValue,
}

impl FontSelectionRequest {
    pub const fn new(
        weight: FontSelectionValue,
        width: FontSelectionValue,
        slope: FontSelectionValue,
    ) -> Self {
        Self {
            weight,
            width,
            slope,
        }
    }
}

// cpp: font_engine/fonts/font_selection_types.h:322-341
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontSelectionRequestKey {
    pub request: FontSelectionRequest,
    pub isDeletedValue: bool,
}

impl FontSelectionRequestKey {
    pub const fn new(request: FontSelectionRequest) -> Self {
        Self {
            request,
            isDeletedValue: false,
        }
    }

    pub const fn deleted() -> Self {
        Self {
            request: FontSelectionRequest {
                weight: FontSelectionValue::from_raw(0),
                width: FontSelectionValue::from_raw(0),
                slope: FontSelectionValue::from_raw(0),
            },
            isDeletedValue: true,
        }
    }

    pub const fn IsHashTableDeletedValue(&self) -> bool {
        self.isDeletedValue
    }
}

// cpp: font_engine/fonts/font_selection_types.h:348-386
#[derive(Clone, Copy, Debug)]
pub struct FontSelectionCapabilities {
    pub width: FontSelectionRange,
    pub slope: FontSelectionRange,
    pub weight: FontSelectionRange,
    is_deleted_value_: bool,
}

impl Default for FontSelectionCapabilities {
    fn default() -> Self {
        let zero = FontSelectionRange::single(kFontSelectionZeroValue);
        Self::new(zero, zero, zero)
    }
}

impl PartialEq for FontSelectionCapabilities {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.slope == other.slope
            && self.weight == other.weight
            && self.is_deleted_value_ == other.is_deleted_value_
    }
}

impl Eq for FontSelectionCapabilities {}

impl FontSelectionCapabilities {
    pub const fn new(
        width: FontSelectionRange,
        slope: FontSelectionRange,
        weight: FontSelectionRange,
    ) -> Self {
        Self {
            width,
            slope,
            weight,
            is_deleted_value_: false,
        }
    }

    pub fn deleted() -> Self {
        Self {
            is_deleted_value_: true,
            ..Self::default()
        }
    }

    pub const fn IsHashTableDeletedValue(&self) -> bool {
        self.is_deleted_value_
    }

    pub fn Expand(&mut self, capabilities: &Self) {
        self.width.Expand(&capabilities.width);
        self.slope.Expand(&capabilities.slope);
        self.weight.Expand(&capabilities.weight);
    }

    pub fn IsValid(&self) -> bool {
        self.width.IsValid()
            && self.slope.IsValid()
            && self.weight.IsValid()
            && !self.is_deleted_value_
    }

    pub fn HasRange(&self) -> bool {
        self.width.IsRange() || self.slope.IsRange() || self.weight.IsRange()
    }
}

// cpp: font_engine/fonts/font_selection_types.h:401-410
pub const fn DefaultMinimumForClampFontSelectionValue() -> FontSelectionValue {
    FontSelectionValue::MinimumValue()
}

pub const fn DefaultMaximumForClampFontSelectionValue() -> FontSelectionValue {
    FontSelectionValue::MaximumValue()
}

// GetHash/ToString declarations at 82,308,315,343-346,388-399 have no
// definitions in the supplied C++ tree. Hash trait integration remains pending.
