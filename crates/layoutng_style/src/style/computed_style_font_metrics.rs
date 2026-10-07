use font_engine::{Font, FontBaseline, FontHeight};
use foundation::{EDominantBaseline, LayoutUnit, Length, MinimumValueForLength};

use super::computed_style::{ComputedStyle, ComputedStyleBuilder};

#[allow(non_snake_case)]
impl ComputedStyleBuilder {
    // cpp: layoutng_style/style/computed_style_font_metrics.cc:31-36
    pub fn FontHeight(&self) -> LayoutUnit {
        let font_data = unsafe { (&*self.GetFont()).PrimaryFont() };
        if !font_data.is_null() {
            return LayoutUnit::from_signed(unsafe { (&*font_data).GetFontMetrics().Height() });
        }
        LayoutUnit::default()
    }
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:870
    // cpp: layoutng_style/style/computed_style_font_metrics.cc:38-65
    pub fn GetFontBaseline(&self) -> FontBaseline {
        match self.CssDominantBaseline() {
            EDominantBaseline::kAuto => {}
            EDominantBaseline::kMiddle => return FontBaseline::kXMiddleBaseline,
            EDominantBaseline::kAlphabetic => return FontBaseline::kAlphabeticBaseline,
            EDominantBaseline::kHanging => return FontBaseline::kHangingBaseline,
            EDominantBaseline::kCentral => return FontBaseline::kCentralBaseline,
            EDominantBaseline::kTextBeforeEdge => return FontBaseline::kTextOverBaseline,
            EDominantBaseline::kTextAfterEdge => return FontBaseline::kTextUnderBaseline,
            EDominantBaseline::kIdeographic => return FontBaseline::kIdeographicUnderBaseline,
            EDominantBaseline::kMathematical => return FontBaseline::kMathBaseline,
            EDominantBaseline::kUseScript
            | EDominantBaseline::kNoChange
            | EDominantBaseline::kResetSize => unreachable!("invalid dominant baseline"),
        }
        if !self.GetFontDescription().IsVerticalAnyUpright() {
            FontBaseline::kAlphabeticBaseline
        } else {
            FontBaseline::kCentralBaseline
        }
    }

    // cpp: layoutng_style/style/computed_style_font_metrics.cc:67-71
    // cpp: layoutng_style/style/computed_style.h:872
    pub fn GetFontHeight(&self, baseline: FontBaseline) -> FontHeight {
        let font_data = unsafe { (&*self.GetFont()).PrimaryFont() };
        if !font_data.is_null() {
            return unsafe { (&*font_data).GetFontMetrics().GetFontHeight(baseline) };
        }
        FontHeight::default()
    }

    // cpp: layoutng_style/style/computed_style_font_metrics.cc:73-86
    // cpp: layoutng_style/style/computed_style.h:1245
    pub fn ComputedLineHeightFromLength(line_height: &Length, font: &Font) -> f32 {
        if line_height.IsAuto() {
            let primary_font = font.PrimaryFont();
            if !primary_font.is_null() {
                return unsafe { (&*primary_font).GetFontMetrics().LineSpacing() } as f32;
            }
            return 0.0;
        }
        if line_height.HasPercent() {
            return MinimumValueForLength(
                line_height,
                LayoutUnit::from_f32(font.GetFontDescription().ComputedSize()),
            )
            .ToFloat();
        }
        debug_assert!(line_height.IsFixed());
        line_height.Pixels()
    }

    // cpp: layoutng_style/style/computed_style.h:1246
    // cpp: layoutng_style/style/computed_style_font_metrics.cc:88-90
    pub fn ComputedLineHeight(&self) -> f32 {
        Self::ComputedLineHeightFromLength(self.LineHeight(), unsafe { &*self.GetFont() })
    }

    // cpp: layoutng_style/style/computed_style.h:1248
    // cpp: layoutng_style/style/computed_style_font_metrics.cc:92-103
    pub fn ComputedLineHeightAsFixedForFont(&self, font: &Font) -> LayoutUnit {
        let line_height = self.LineHeight();
        if line_height.IsAuto() {
            let primary_font = font.PrimaryFont();
            if !primary_font.is_null() {
                return unsafe { (&*primary_font).GetFontMetrics().FixedLineSpacing() };
            }
            return LayoutUnit::default();
        }
        if line_height.HasPercent() {
            return MinimumValueForLength(line_height, Self::ComputedFontSizeAsFixed(font));
        }
        debug_assert!(line_height.IsFixed());
        LayoutUnit::FromFloatRound(line_height.Pixels())
    }

    // cpp: layoutng_style/style/computed_style.h:1247
    // cpp: layoutng_style/style/computed_style_font_metrics.cc:105-107
    pub fn ComputedLineHeightAsFixed(&self) -> LayoutUnit {
        self.ComputedLineHeightAsFixedForFont(unsafe { &*self.GetFont() })
    }
}
