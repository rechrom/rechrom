// C++: font_engine/fonts/font_metrics.h/.cc. The declaration of
// AscentDescentWithHacks has no definition in the supplied source tree.
use super::font_baseline::FontBaseline;
use super::font_height::FontHeight;
use foundation::LayoutUnit;

// cpp: font_engine/fonts/font_metrics.h:43-45
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplyBaselineTable(pub bool);

// cpp: font_engine/fonts/font_metrics.h:37-269
#[derive(Clone, Debug, Default)]
pub struct FontMetrics {
    cap_height_: f32,
    float_ascent_: f32,
    float_descent_: f32,
    line_gap_: f32,
    line_spacing_: f32,
    x_height_: f32,
    zero_width_: f32,
    underline_thickness_: Option<f32>,
    underline_position_: Option<f32>,
    ideographic_baseline_position_: Option<f32>,
    alphabetic_baseline_position_: Option<f32>,
    hanging_baseline_position_: Option<f32>,
    int_ascent_: i32,
    int_descent_: i32,
    has_x_height_: bool,
    has_zero_width_: bool,
}

#[allow(non_snake_case)]
impl FontMetrics {
    // cpp: font_engine/fonts/font_metrics.h:47-65
    pub fn FloatAscent(&self) -> f32 {
        self.float_ascent_
    }
    pub fn FloatAscentFor(&self, baseline: FontBaseline) -> f32 {
        self.FloatAscentWithTable(baseline, ApplyBaselineTable(false))
    }
    pub fn FloatAscentWithTable(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> f32 {
        if baseline == FontBaseline::kAlphabeticBaseline {
            self.float_ascent_
        } else {
            self.FloatAscentInternal(baseline, apply)
        }
    }
    pub fn SetAscent(&mut self, ascent: f32) {
        self.float_ascent_ = ascent;
        self.int_ascent_ = ascent.round() as i32;
    }
    pub fn FloatDescent(&self) -> f32 {
        self.float_descent_
    }
    pub fn FloatDescentFor(&self, baseline: FontBaseline) -> f32 {
        self.FloatDescentWithTable(baseline, ApplyBaselineTable(false))
    }
    pub fn FloatDescentWithTable(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> f32 {
        if baseline == FontBaseline::kAlphabeticBaseline {
            self.float_descent_
        } else {
            self.FloatHeight() - self.FloatAscentInternal(baseline, apply)
        }
    }
    pub fn SetDescent(&mut self, descent: f32) {
        self.float_descent_ = descent;
        self.int_descent_ = descent.round() as i32;
    }
    pub fn FloatHeight(&self) -> f32 {
        self.float_ascent_ + self.float_descent_
    }

    // cpp: font_engine/fonts/font_metrics.h:67-101
    pub fn ConvertBaseline(&self, value: f32, to: FontBaseline, from: FontBaseline) -> f32 {
        if from == to {
            value
        } else {
            self.FloatAscentFor(to) - self.FloatAscentFor(from) + value
        }
    }
    pub fn Alphabetic(&self, baseline: FontBaseline) -> f32 {
        self.ConvertBaseline(0.0, baseline, FontBaseline::kAlphabeticBaseline)
    }
    pub fn CapHeight(&self) -> f32 {
        self.cap_height_
    }
    pub fn CapHeightFor(&self, baseline: FontBaseline) -> f32 {
        self.ConvertBaseline(
            self.CapHeight(),
            baseline,
            FontBaseline::kAlphabeticBaseline,
        )
    }
    pub fn SetCapHeight(&mut self, cap_height: f32) {
        self.cap_height_ = cap_height;
    }
    pub fn LineGap(&self) -> i32 {
        self.line_gap_.round() as i32
    }
    pub fn SetLineGap(&mut self, line_gap: f32) {
        self.line_gap_ = line_gap;
    }
    pub fn LineSpacing(&self) -> i32 {
        self.line_spacing_.round() as i32
    }
    pub fn SetLineSpacing(&mut self, line_spacing: f32) {
        self.line_spacing_ = line_spacing;
    }
    pub fn XHeight(&self) -> f32 {
        self.x_height_
    }
    pub fn XHeightFor(&self, baseline: FontBaseline) -> f32 {
        self.ConvertBaseline(self.XHeight(), baseline, FontBaseline::kAlphabeticBaseline)
    }
    pub fn SetXHeight(&mut self, x_height: f32) {
        self.x_height_ = x_height;
        self.has_x_height_ = true;
    }
    pub fn HasXHeight(&self) -> bool {
        self.has_x_height_ && self.x_height_ > 0.0
    }
    pub fn SetHasXHeight(&mut self, has_x_height: bool) {
        self.has_x_height_ = has_x_height;
    }

    // cpp: font_engine/fonts/font_metrics.h:103-144
    pub fn Ascent(&self) -> i32 {
        self.int_ascent_
    }
    pub fn AscentFor(&self, baseline: FontBaseline) -> i32 {
        self.AscentWithTable(baseline, ApplyBaselineTable(false))
    }
    pub fn AscentWithTable(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> i32 {
        if baseline == FontBaseline::kAlphabeticBaseline {
            self.int_ascent_
        } else {
            self.IntAscentInternal(baseline, apply)
        }
    }
    pub fn Descent(&self) -> i32 {
        self.int_descent_
    }
    pub fn DescentFor(&self, baseline: FontBaseline) -> i32 {
        self.DescentWithTable(baseline, ApplyBaselineTable(false))
    }
    pub fn DescentWithTable(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> i32 {
        if baseline == FontBaseline::kAlphabeticBaseline {
            self.int_descent_
        } else {
            self.Height() - self.IntAscentInternal(baseline, apply)
        }
    }
    pub fn Height(&self) -> i32 {
        self.int_ascent_ + self.int_descent_
    }
    pub fn FixedAscent(&self, baseline: FontBaseline) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.FloatAscentFor(baseline))
    }
    pub fn FixedDescent(&self, baseline: FontBaseline) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.FloatDescentFor(baseline))
    }
    pub fn FixedAlphabetic(&self, baseline: FontBaseline) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.Alphabetic(baseline))
    }
    pub fn FixedCapHeight(&self, baseline: FontBaseline) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.CapHeightFor(baseline))
    }
    pub fn FixedXHeight(&self, baseline: FontBaseline) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.XHeightFor(baseline))
    }
    pub fn FixedLineSpacing(&self) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.line_spacing_)
    }
    pub fn GetFloatFontHeight(&self, baseline: FontBaseline) -> FontHeight {
        FontHeight::new(self.FixedAscent(baseline), self.FixedDescent(baseline))
    }
    pub fn GetFontHeight(&self, baseline: FontBaseline) -> FontHeight {
        FontHeight::new(
            LayoutUnit::from_signed(self.AscentFor(baseline)),
            LayoutUnit::from_signed(self.DescentFor(baseline)),
        )
    }

    // cpp: font_engine/fonts/font_metrics.h:164-214
    pub fn ZeroWidth(&self) -> f32 {
        self.zero_width_
    }
    pub fn SetZeroWidth(&mut self, zero_width: f32) {
        self.zero_width_ = zero_width;
        self.has_zero_width_ = true;
    }
    pub fn HasZeroWidth(&self) -> bool {
        self.has_zero_width_
    }
    pub fn SetHasZeroWidth(&mut self, has_zero_width: bool) {
        self.has_zero_width_ = has_zero_width;
    }
    pub fn UnderlineThickness(&self) -> Option<f32> {
        self.underline_thickness_
    }
    pub fn SetUnderlineThickness(&mut self, value: f32) {
        self.underline_thickness_ = Some(value);
    }
    pub fn UnderlinePosition(&self) -> Option<f32> {
        self.underline_position_
    }
    pub fn SetUnderlinePosition(&mut self, value: f32) {
        self.underline_position_ = Some(value);
    }
    pub fn SetIdeographicBaseline(&mut self, value: Option<f32>) {
        self.ideographic_baseline_position_ = value;
    }
    pub fn IdeographicBaseline(&self) -> Option<f32> {
        self.ideographic_baseline_position_
    }
    pub fn SetAlphabeticBaseline(&mut self, value: Option<f32>) {
        self.alphabetic_baseline_position_ = value;
    }
    pub fn AlphabeticBaseline(&self) -> Option<f32> {
        self.alphabetic_baseline_position_
    }
    pub fn SetHangingBaseline(&mut self, value: Option<f32>) {
        self.hanging_baseline_position_ = value;
    }
    pub fn HangingBaseline(&self) -> Option<f32> {
        self.hanging_baseline_position_
    }

    // cpp: font_engine/fonts/font_metrics.h:234-251
    pub(crate) fn Reset(&mut self) {
        self.cap_height_ = 0.0;
        self.float_ascent_ = 0.0;
        self.float_descent_ = 0.0;
        self.int_ascent_ = 0;
        self.int_descent_ = 0;
        self.line_gap_ = 0.0;
        self.line_spacing_ = 0.0;
        self.x_height_ = 0.0;
        self.has_x_height_ = false;
        self.underline_thickness_ = None;
        self.underline_position_ = None;
        self.ideographic_baseline_position_ = None;
        self.alphabetic_baseline_position_ = None;
        self.hanging_baseline_position_ = None;
    }

    // cpp: font_engine/fonts/font_metrics.cc:6-24
    fn FloatAscentInternal(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> f32 {
        match baseline {
            FontBaseline::kAlphabeticBaseline => panic!("unreachable alphabetic baseline"),
            FontBaseline::kCentralBaseline => self.FloatHeight() / 2.0,
            FontBaseline::kTextUnderBaseline => self.FloatHeight(),
            FontBaseline::kIdeographicUnderBaseline => {
                if let (Some(position), true) = (self.ideographic_baseline_position_, apply.0) {
                    self.float_ascent_ - position
                } else {
                    self.FloatHeight()
                }
            }
            FontBaseline::kXMiddleBaseline => self.float_ascent_ - self.XHeight() / 2.0,
            FontBaseline::kMathBaseline => self.float_ascent_ * 0.5,
            FontBaseline::kHangingBaseline => {
                if let (Some(position), true) = (self.hanging_baseline_position_, apply.0) {
                    self.float_ascent_ - position
                } else {
                    self.float_ascent_ * 0.2
                }
            }
            FontBaseline::kTextOverBaseline => 0.0,
        }
    }

    // cpp: font_engine/fonts/font_metrics.cc:26-49
    fn IntAscentInternal(&self, baseline: FontBaseline, apply: ApplyBaselineTable) -> i32 {
        match baseline {
            FontBaseline::kAlphabeticBaseline => panic!("unreachable alphabetic baseline"),
            FontBaseline::kCentralBaseline => self.Height() - self.Height() / 2,
            FontBaseline::kTextUnderBaseline => self.Height(),
            FontBaseline::kIdeographicUnderBaseline => {
                if let (Some(position), true) = (self.ideographic_baseline_position_, apply.0) {
                    self.int_ascent_ - position.round() as i32
                } else {
                    self.Height()
                }
            }
            FontBaseline::kXMiddleBaseline => self.int_ascent_ - (self.XHeight() / 2.0) as i32,
            FontBaseline::kMathBaseline => {
                if let (Some(position), true) = (self.hanging_baseline_position_, apply.0) {
                    self.int_ascent_ - position.round() as i32
                } else {
                    self.int_ascent_ / 2
                }
            }
            FontBaseline::kHangingBaseline => self.int_ascent_ * 2 / 10,
            FontBaseline::kTextOverBaseline => 0,
        }
    }
}
