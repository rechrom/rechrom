// C++: src/foundation/blink_geometry/geometry/length.h/.cc

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::calculation_value::CalculationValue;
use crate::evaluation_input::EvaluationInput;
use crate::hash_functions::{AddFloatToHash, AddIntToHash};
use crate::LayoutUnit;

// C++: length.h:112. The nested enum is lifted because Length itself is
// cpp: foundation/blink_geometry/geometry/length.h:112
// pending CalculationValue and expression-node ownership integration.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LengthValueRange {
    kAll,
    kNonNegative,
}

// C++: length.h:116-130. The unsigned-char discriminants are preserved.
// cpp: foundation/blink_geometry/geometry/length.h:116-130
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LengthType {
    kAuto,
    kPercent,
    kFixed,
    kMinContent,
    kMaxContent,
    kMinIntrinsic,
    kStretch,
    kFitContent,
    kCalculated,
    kFlex,
    kNone,
    kContent,
    kOverlapJoin,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PixelsAndPercent {
    pub pixels: f32,
    pub percent: f32,
    pub has_explicit_pixels: bool,
    pub has_explicit_percent: bool,
}

impl PixelsAndPercent {
    // C++: length.h:50-54
    // cpp: foundation/blink_geometry/geometry/length.h:50-54
    pub fn from_pixels(pixels: f32) -> Self {
        Self {
            pixels,
            percent: 0.0,
            has_explicit_pixels: true,
            has_explicit_percent: false,
        }
    }

    // C++: length.h:55-62
    // cpp: foundation/blink_geometry/geometry/length.h:55-62
    pub fn new(
        pixels: f32,
        percent: f32,
        has_explicit_pixels: bool,
        has_explicit_percent: bool,
    ) -> Self {
        Self {
            pixels,
            percent,
            has_explicit_pixels,
            has_explicit_percent,
        }
    }
}

// C++: length.h:64-70
// cpp: foundation/blink_geometry/geometry/length.h:64-70
impl std::ops::AddAssign for PixelsAndPercent {
    fn add_assign(&mut self, rhs: Self) {
        self.pixels += rhs.pixels;
        self.percent += rhs.percent;
        self.has_explicit_pixels |= rhs.has_explicit_pixels;
        self.has_explicit_percent |= rhs.has_explicit_percent;
    }
}

// C++: length.h:71-75
// cpp: foundation/blink_geometry/geometry/length.h:71-75
impl std::ops::Add for PixelsAndPercent {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

// C++: length.h:76-82
// cpp: foundation/blink_geometry/geometry/length.h:76-82
impl std::ops::SubAssign for PixelsAndPercent {
    fn sub_assign(&mut self, rhs: Self) {
        self.pixels -= rhs.pixels;
        self.percent -= rhs.percent;
        self.has_explicit_pixels |= rhs.has_explicit_pixels;
        self.has_explicit_percent |= rhs.has_explicit_percent;
    }
}

// C++: length.h:83-87
// cpp: foundation/blink_geometry/geometry/length.h:83-87
impl std::ops::MulAssign<f32> for PixelsAndPercent {
    fn mul_assign(&mut self, number: f32) {
        self.pixels *= number;
        self.percent *= number;
    }
}

// C++: length.cc:65-109. A handle, rather than a pointer, preserves Length's
// cpp: foundation/blink_geometry/geometry/length.cc:65-109
// 8-byte representation and independent copy count. Arc replaces the C++
// persistent GC root while the entry is live.
struct CalculationValueHandleEntry {
    value: Arc<CalculationValue>,
    count: u32,
}

#[derive(Default)]
struct CalculationValueHandleMap {
    index: u32,
    map: HashMap<u32, CalculationValueHandleEntry>,
}

impl CalculationValueHandleMap {
    fn insert(&mut self, value: Arc<CalculationValue>) -> u32 {
        loop {
            self.index = self.index.wrapping_add(1);
            if self.index != 0 && self.index != u32::MAX && !self.map.contains_key(&self.index) {
                self.map
                    .insert(self.index, CalculationValueHandleEntry { value, count: 1 });
                return self.index;
            }
        }
    }

    fn increment(&mut self, handle: u32) {
        let entry = self
            .map
            .get_mut(&handle)
            .expect("missing calculation handle");
        entry.count = entry
            .count
            .checked_add(1)
            .expect("calculation count overflow");
    }

    fn decrement(&mut self, handle: u32) {
        let entry = self
            .map
            .get_mut(&handle)
            .expect("missing calculation handle");
        entry.count -= 1;
        if entry.count == 0 {
            self.map.remove(&handle);
        }
    }
}

static CALC_HANDLES: OnceLock<Mutex<CalculationValueHandleMap>> = OnceLock::new();

fn calc_handles() -> &'static Mutex<CalculationValueHandleMap> {
    CALC_HANDLES.get_or_init(|| Mutex::new(CalculationValueHandleMap::default()))
}

// cpp: foundation/blink_geometry/geometry/length.h:105-413
#[repr(C)]
#[derive(Debug)]
pub struct Length {
    // C++ uses a union of float value_ and unsigned calculation_handle_.
    value_bits_: u32,
    quirk_: bool,
    type_: LengthType,
    padding_: [u8; 2],
}

const _: [(); 8] = [(); std::mem::size_of::<Length>()];

static AUTO_LENGTH: Length = Length::constant(LengthType::kAuto);
static STRETCH_LENGTH: Length = Length::constant(LengthType::kStretch);
static FIT_CONTENT_LENGTH: Length = Length::constant(LengthType::kFitContent);
static MAX_CONTENT_LENGTH: Length = Length::constant(LengthType::kMaxContent);
static MIN_CONTENT_LENGTH: Length = Length::constant(LengthType::kMinContent);
static MIN_INTRINSIC_LENGTH: Length = Length::constant(LengthType::kMinIntrinsic);

impl Default for Length {
    fn default() -> Self {
        Self::from_type(LengthType::kAuto)
    }
}

// C++: length.h:155-172. Clone/Drop mirror copy construction and destruction.
// cpp: foundation/blink_geometry/geometry/length.h:155-172
impl Clone for Length {
    fn clone(&self) -> Self {
        if self.IsCalculated() {
            calc_handles().lock().unwrap().increment(self.value_bits_);
        }
        Self {
            value_bits_: self.value_bits_,
            quirk_: self.quirk_,
            type_: self.type_,
            padding_: [0; 2],
        }
    }
}

impl Drop for Length {
    fn drop(&mut self) {
        if self.IsCalculated() {
            calc_handles().lock().unwrap().decrement(self.value_bits_);
        }
    }
}

// C++: length.h:174-187
// cpp: foundation/blink_geometry/geometry/length.h:174-187
impl PartialEq for Length {
    fn eq(&self, other: &Self) -> bool {
        if self.type_ != other.type_ || self.quirk_ != other.quirk_ {
            return false;
        }
        if self.IsCalculated() {
            self.IsCalculatedEqual(other)
        } else {
            f32::from_bits(self.value_bits_) == f32::from_bits(other.value_bits_)
        }
    }
}

impl Length {
    const fn constant(type_: LengthType) -> Self {
        Self {
            value_bits_: 0,
            quirk_: false,
            type_,
            padding_: [0; 2],
        }
    }

    // C++: length.cc:44-59. Rust statics are initialized before first use.
    // cpp: foundation/blink_geometry/geometry/length.cc:44-59
    pub fn Initialize() {}

    // C++: length.h:133-153
    // cpp: foundation/blink_geometry/geometry/length.h:133-153
    pub fn from_type(type_: LengthType) -> Self {
        assert_ne!(type_, LengthType::kCalculated);
        Self::constant(type_)
    }

    pub fn new(value: impl Into<f64>, type_: LengthType) -> Self {
        assert_ne!(type_, LengthType::kCalculated);
        let value = value.into();
        assert!(value.is_finite());
        Self {
            value_bits_: (value.clamp(f32::MIN as f64, f32::MAX as f64) as f32).to_bits(),
            quirk_: false,
            type_,
            padding_: [0; 2],
        }
    }

    pub fn from_layout_unit(value: LayoutUnit, type_: LengthType) -> Self {
        Self::new(value.ToFloat(), type_)
    }

    // C++: length.cc:113-117
    // cpp: foundation/blink_geometry/geometry/length.cc:106-109
    pub fn from_calculation_value(value: Arc<CalculationValue>) -> Self {
        let handle = calc_handles().lock().unwrap().insert(value);
        Self {
            value_bits_: handle,
            quirk_: false,
            type_: LengthType::kCalculated,
            padding_: [0; 2],
        }
    }

    // C++: length.h:189-211. Rust cannot overload Fixed() and Fixed(number),
    // cpp: foundation/blink_geometry/geometry/length.h:189-211
    // so C++'s no-argument Fixed() call sites use Fixed(0).
    pub fn Auto() -> &'static Self {
        &AUTO_LENGTH
    }
    pub fn Stretch() -> &'static Self {
        &STRETCH_LENGTH
    }
    pub fn FitContent() -> &'static Self {
        &FIT_CONTENT_LENGTH
    }
    pub fn MaxContent() -> &'static Self {
        &MAX_CONTENT_LENGTH
    }
    pub fn MinContent() -> &'static Self {
        &MIN_CONTENT_LENGTH
    }
    pub fn MinIntrinsic() -> &'static Self {
        &MIN_INTRINSIC_LENGTH
    }
    pub fn Content() -> Self {
        Self::from_type(LengthType::kContent)
    }
    pub fn Fixed(number: impl Into<f64>) -> Self {
        Self::new(number, LengthType::kFixed)
    }
    pub fn None() -> Self {
        Self::from_type(LengthType::kNone)
    }
    pub fn Percent(number: impl Into<f64>) -> Self {
        Self::new(number, LengthType::kPercent)
    }
    pub fn Flex(value: f32) -> Self {
        Self::new(value, LengthType::kFlex)
    }

    // C++: length.h:213-280
    // cpp: foundation/blink_geometry/geometry/length.h:213-280
    fn GetFloatValue(&self) -> f32 {
        assert!(!self.IsNone() && !self.IsCalculated());
        f32::from_bits(self.value_bits_)
    }
    pub fn Pixels(&self) -> f32 {
        assert!(self.IsFixed());
        self.GetFloatValue()
    }
    pub fn PercentValue(&self) -> f32 {
        assert!(self.IsPercent());
        self.GetFloatValue()
    }
    pub fn FlexValue(&self) -> f32 {
        assert!(self.IsFlex());
        self.GetFloatValue()
    }
    pub fn GetType(&self) -> LengthType {
        self.type_
    }
    pub fn Quirk(&self) -> bool {
        self.quirk_
    }
    pub fn SetQuirk(&mut self, quirk: bool) {
        self.quirk_ = quirk;
    }
    pub fn IsNone(&self) -> bool {
        self.type_ == LengthType::kNone
    }
    pub fn IsZero(&self) -> bool {
        assert!(!self.IsNone());
        !self.IsCalculated() && self.GetFloatValue() == 0.0
    }
    pub fn IsAuto(&self) -> bool {
        self.type_ == LengthType::kAuto
    }
    pub fn IsFixed(&self) -> bool {
        self.type_ == LengthType::kFixed
    }
    pub fn IsCalculated(&self) -> bool {
        self.type_ == LengthType::kCalculated
    }
    pub fn IsMinContent(&self) -> bool {
        self.type_ == LengthType::kMinContent
    }
    pub fn IsMaxContent(&self) -> bool {
        self.type_ == LengthType::kMaxContent
    }
    pub fn IsMinIntrinsic(&self) -> bool {
        self.type_ == LengthType::kMinIntrinsic
    }
    pub fn IsStretch(&self) -> bool {
        self.type_ == LengthType::kStretch
    }
    pub fn IsFitContent(&self) -> bool {
        self.type_ == LengthType::kFitContent
    }
    pub fn IsPercent(&self) -> bool {
        self.type_ == LengthType::kPercent
    }
    pub fn IsOverlapJoin(&self) -> bool {
        self.type_ == LengthType::kOverlapJoin
    }
    pub fn IsFlex(&self) -> bool {
        self.type_ == LengthType::kFlex
    }
    pub fn MayHavePercentDependence(&self) -> bool {
        self.IsPercent() || self.IsCalculated()
    }
    pub fn CanConvertToCalculation(&self) -> bool {
        self.IsFixed() || self.IsPercent() || self.IsCalculated()
    }

    // C++: length.cc:146-166, 206-218
    // cpp: foundation/blink_geometry/geometry/length.cc:138-157
    pub fn GetPixelsAndPercent(&self) -> PixelsAndPercent {
        match self.type_ {
            LengthType::kFixed => PixelsAndPercent::from_pixels(self.Pixels()),
            LengthType::kPercent => PixelsAndPercent::new(0.0, self.PercentValue(), false, true),
            LengthType::kCalculated => self.GetCalculationValue().GetPixelsAndPercent(),
            _ => panic!("Length cannot be converted to PixelsAndPercent"),
        }
    }

    pub fn GetCalculationValue(&self) -> Arc<CalculationValue> {
        assert!(self.IsCalculated());
        calc_handles()
            .lock()
            .unwrap()
            .map
            .get(&self.value_bits_)
            .expect("missing calculation handle")
            .value
            .clone()
    }

    pub fn AsCalculationValue(&self) -> Arc<CalculationValue> {
        if self.IsCalculated() {
            self.GetCalculationValue()
        } else {
            CalculationValue::new(self.GetPixelsAndPercent(), LengthValueRange::kAll)
        }
    }

    // C++: length.cc:223-310
    // cpp: foundation/blink_geometry/geometry/length.cc:223-310
    pub fn NonNanCalculatedValue(&self, max_value: f32, input: &EvaluationInput<'_>) -> f32 {
        assert!(self.IsCalculated());
        self.GetCalculationValue().Evaluate(max_value, input)
    }
    pub fn HasOnlyFixedAndPercent(&self) -> bool {
        if self.IsFixed() || self.IsPercent() {
            true
        } else if self.IsCalculated() {
            self.GetCalculationValue().HasOnlyFixedAndPercent()
        } else {
            false
        }
    }
    pub fn HasAuto(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasAuto()
        } else {
            self.IsAuto()
        }
    }
    pub fn HasContentOrIntrinsic(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasContentOrIntrinsicSize()
        } else {
            matches!(
                self.type_,
                LengthType::kMinContent
                    | LengthType::kMaxContent
                    | LengthType::kFitContent
                    | LengthType::kMinIntrinsic
                    | LengthType::kContent
            )
        }
    }
    pub fn HasAutoOrContentOrIntrinsic(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasAutoOrContentOrIntrinsicSize()
        } else {
            self.IsAuto() || self.HasContentOrIntrinsic()
        }
    }
    pub fn HasPercent(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasPercent()
        } else {
            self.IsPercent()
        }
    }
    pub fn HasPercentOrStretch(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasPercentOrStretch()
        } else {
            self.IsPercent() || self.IsStretch()
        }
    }
    pub fn HasStretch(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasStretch()
        } else {
            self.IsStretch()
        }
    }
    pub fn HasMinContent(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasMinContent()
        } else {
            self.IsMinContent()
        }
    }
    pub fn HasMaxContent(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasMaxContent()
        } else {
            self.IsMaxContent()
        }
    }
    pub fn HasMinIntrinsic(&self) -> bool {
        self.IsMinIntrinsic()
    }
    pub fn HasFitContent(&self) -> bool {
        if self.IsCalculated() {
            self.GetCalculationValue().HasFitContent()
        } else {
            self.IsFitContent()
        }
    }
    pub fn IsCalculatedEqual(&self, other: &Self) -> bool {
        self.IsCalculated()
            && other.IsCalculated()
            && (self.value_bits_ == other.value_bits_
                || self.GetCalculationValue() == other.GetCalculationValue())
    }

    // C++: length.h:355-382, length.cc:119-144
    // cpp: foundation/blink_geometry/geometry/length.h:355-382
    // cpp: foundation/blink_geometry/geometry/length.cc:111-136
    pub fn Blend(&self, from: &Self, progress: f64, range: LengthValueRange) -> Self {
        assert!(self.CanConvertToCalculation() && from.CanConvertToCalculation());
        if progress == 0.0 {
            return from.clone();
        }
        if progress == 1.0 {
            return self.clone();
        }
        if from.IsCalculated()
            || self.IsCalculated()
            || (!from.IsZero() && !self.IsZero() && from.type_ != self.type_)
        {
            return Self::from_calculation_value(self.AsCalculationValue().Blend(
                &from.AsCalculationValue(),
                progress,
                range,
            ));
        }
        if from.IsZero() && self.IsZero() {
            return self.clone();
        }
        let result_type = if self.IsZero() {
            from.type_
        } else {
            self.type_
        };
        let mut value = (from.GetFloatValue() as f64
            + (self.GetFloatValue() - from.GetFloatValue()) as f64 * progress)
            as f32;
        if value.is_infinite() {
            value = if value.is_sign_positive() {
                f32::MAX
            } else {
                f32::MIN
            };
        }
        if range == LengthValueRange::kNonNegative && value < 0.0 {
            value = 0.0;
        }
        Self::new(value, result_type)
    }

    // C++: length.cc:168-205
    // cpp: foundation/blink_geometry/geometry/length.cc:159-203
    pub fn SubtractFromOneHundredPercent(&self) -> Self {
        if self.IsPercent() {
            return Self::Percent(100.0 - self.PercentValue());
        }
        assert!(self.CanConvertToCalculation());
        Self::from_calculation_value(self.AsCalculationValue().SubtractFromOneHundredPercent())
    }
    pub fn Add(&self, other: &Self) -> Self {
        assert!(self.CanConvertToCalculation());
        if self.IsFixed() && other.IsFixed() {
            return Self::Fixed(self.Pixels() + other.Pixels());
        }
        if self.IsPercent() && other.IsPercent() {
            return Self::Percent(self.PercentValue() + other.PercentValue());
        }
        Self::from_calculation_value(self.AsCalculationValue().Add(&other.AsCalculationValue()))
    }
    pub fn Zoom(&self, factor: f64) -> Self {
        if self.IsFixed() {
            Self::Fixed(self.GetFloatValue() as f64 * factor)
        } else if self.IsCalculated() {
            Self::from_calculation_value(self.GetCalculationValue().Zoom(factor))
        } else {
            self.clone()
        }
    }
    pub fn Multiplied(&self, max_value: f32, factor: f64) -> Self {
        if self.IsCalculated() {
            let value = self.NonNanCalculatedValue(max_value, &EvaluationInput::default());
            Self::Fixed(value as f64 * factor)
        } else {
            let mut copy = self.clone();
            let value = f32::from_bits(copy.value_bits_);
            copy.value_bits_ = ((value as f64 * factor) as f32).to_bits();
            copy
        }
    }

    // C++: length.cc:211-220, 311-348
    // cpp: foundation/blink_geometry/geometry/length.cc:214-220
    pub fn GetCalculatedCountForTest(&self) -> u32 {
        assert!(self.IsCalculated());
        calc_handles()
            .lock()
            .unwrap()
            .map
            .get(&self.value_bits_)
            .expect("missing calculation handle")
            .count
    }
    pub fn GetCalcHandleMapSizeForTest() -> usize {
        calc_handles().lock().unwrap().map.len()
    }
    // cpp: foundation/blink_geometry/geometry/length.cc:311-317
    pub fn GetHash(&self) -> u32 {
        let mut hash = 0u32;
        AddFloatToHash(&mut hash, f32::from_bits(self.value_bits_));
        AddIntToHash(&mut hash, self.type_ as u32);
        AddIntToHash(&mut hash, self.quirk_ as u32);
        AddIntToHash(&mut hash, self.value_bits_);
        hash
    }
    // cpp: foundation/blink_geometry/geometry/length.cc:327-348
    pub fn ToString(&self) -> String {
        // C++'s type-name table is preserved, including its legacy entries.
        const NAMES: [&str; 16] = [
            "Auto",
            "Percent",
            "Fixed",
            "MinContent",
            "MaxContent",
            "MinIntrinsic",
            "FillAvailable",
            "Stretch",
            "FitContent",
            "Calculated",
            "Flex",
            "ExtendToZoom",
            "DeviceWidth",
            "DeviceHeight",
            "None",
            "Content",
        ];
        let name = NAMES.get(self.type_ as usize).copied().unwrap_or("?");
        let value = if self.IsCalculated() {
            self.value_bits_.to_string()
        } else {
            f32::from_bits(self.value_bits_).to_string()
        };
        if self.quirk_ {
            format!("Length({name}, {value}, Quirk)")
        } else {
            format!("Length({name}, {value})")
        }
    }
}
