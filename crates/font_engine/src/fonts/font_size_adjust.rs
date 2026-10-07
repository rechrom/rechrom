// cpp: font_engine/fonts/font_size_adjust.h:13-49
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Metric {
    kExHeight,
    kCapHeight,
    kChWidth,
    kIcWidth,
    kIcHeight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    kNumber,
    kFromFont,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FontSizeAdjust {
    value_: f32,
    metric_: Metric,
    type_: ValueType,
}

const _: () = assert!(std::mem::size_of::<FontSizeAdjust>() == 12);

impl Default for FontSizeAdjust {
    fn default() -> Self {
        Self::new(Self::kFontSizeAdjustNone)
    }
}

impl PartialEq for FontSizeAdjust {
    fn eq(&self, other: &Self) -> bool {
        self.value_ == other.Value()
            && self.metric_ == other.GetMetric()
            && self.IsFromFont() == other.IsFromFont()
    }
}

impl FontSizeAdjust {
    pub const kFontSizeAdjustNone: f32 = -1.0;

    pub const fn new(value: f32) -> Self {
        Self::with_metric_and_type(value, Metric::kExHeight, ValueType::kNumber)
    }

    pub const fn with_type(value: f32, type_: ValueType) -> Self {
        Self::with_metric_and_type(value, Metric::kExHeight, type_)
    }

    pub const fn with_metric(value: f32, metric: Metric) -> Self {
        Self::with_metric_and_type(value, metric, ValueType::kNumber)
    }

    pub const fn with_metric_and_type(value: f32, metric: Metric, type_: ValueType) -> Self {
        Self {
            value_: value,
            metric_: metric,
            type_: type_,
        }
    }

    pub fn IsSet(&self) -> bool {
        self.value_ != Self::kFontSizeAdjustNone || self.type_ == ValueType::kFromFont
    }

    pub fn IsFromFont(&self) -> bool {
        self.type_ == ValueType::kFromFont
    }

    pub fn Value(&self) -> f32 {
        self.value_
    }

    pub fn GetMetric(&self) -> Metric {
        self.metric_
    }
}

// The GetHash and ToString declarations at font_size_adjust.h:41-42 have no
// definitions in the supplied C++ tree. This source file stays blocked.
