#![allow(non_snake_case)]

use font_engine::{Font, FontBaseline, SimpleFontData, TextFragmentPaintInfo};
use foundation::{gfx, LayoutUnit, Member, Visitor};

// cpp: layoutng/internal/used_font.h:28-34
// cpp: layoutng/internal/used_font.h:76-84
pub struct UsedFont {
    pub font_: Member<Font>,
    pub text_fit_scaling_factor_: f32,
}

impl UsedFont {
    // cpp: layoutng/internal/used_font.h:32-33
    pub fn new(font: &Font, scaling_factor: f32) -> Self {
        Self {
            font_: Member::from_ptr(font as *const Font as *mut Font),
            text_fit_scaling_factor_: scaling_factor,
        }
    }

    // cpp: layoutng/internal/used_font.h:34-34
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.font_);
    }

    // cpp: layoutng/internal/used_font.h:36-36
    pub fn ScalingFactor(&self) -> f32 {
        self.text_fit_scaling_factor_
    }

    // cpp: layoutng/internal/used_font.h:37-37
    pub fn GetFont(&self) -> &Font {
        unsafe { &*self.font_.Get() }
    }

    // cpp: layoutng/internal/used_font.h:38-38
    pub fn PrimaryFont(&self) -> *const SimpleFontData {
        self.GetFont().PrimaryFont()
    }

    // cpp: layoutng/internal/used_font.h:41-43
    pub fn ComputedSize(&self) -> f32 {
        self.GetFont().GetFontDescription().ComputedSize()
    }

    // cpp: layoutng/internal/used_font.h:45-45
    pub fn UsedSize(&self) -> f32 {
        self.ComputedSize() * self.text_fit_scaling_factor_
    }

    // cpp: layoutng/internal/used_font.h:47-53
    pub fn FloatAscent(&self) -> f32 {
        let font_data = self.PrimaryFont();
        if !font_data.is_null() {
            return unsafe { &*font_data }.GetFontMetrics().FloatAscent()
                * self.text_fit_scaling_factor_;
        }
        0.0
    }

    // cpp: layoutng/internal/used_font.h:55-55
    pub fn FixedAscent(&self) -> LayoutUnit {
        LayoutUnit::from_f32(self.FloatAscent())
    }

    // cpp: layoutng/internal/used_font.h:57-57
    // Defined in //src/layoutng_inline/used_font.cc.
    pub fn FixedAscentForBaseline(&self, baseline: FontBaseline) -> LayoutUnit {
        unsafe { UsedFontFixedAscentForBaseline(self, baseline) }
    }

    // cpp: layoutng/internal/used_font.h:59-59
    // Defined in //src/layoutng_inline/used_font.cc.
    pub fn FixedDescent(&self) -> LayoutUnit {
        unsafe { UsedFontFixedDescent(self) }
    }

    // cpp: layoutng/internal/used_font.h:61-61
    // Defined in //src/layoutng_inline/used_font.cc.
    pub fn FixedDescentForBaseline(&self, baseline: FontBaseline) -> LayoutUnit {
        unsafe { UsedFontFixedDescentForBaseline(self, baseline) }
    }

    // cpp: layoutng/internal/used_font.h:65-65
    // Defined in //src/layoutng_inline/used_font.cc.
    pub fn UnderlineThickness(&self) -> Option<f32> {
        unsafe { UsedFontUnderlineThickness(self) }
    }

    // cpp: layoutng/internal/used_font.h:68-74
    pub fn TextInkBounds(&self, text_info: &TextFragmentPaintInfo) -> gfx::RectF {
        let mut bounds = self.GetFont().TextInkBounds(text_info);
        if self.text_fit_scaling_factor_ != 1.0 {
            bounds.Scale(self.text_fit_scaling_factor_, self.text_fit_scaling_factor_);
        }
        bounds
    }
}

unsafe extern "Rust" {
    fn UsedFontFixedAscentForBaseline(font: &UsedFont, baseline: FontBaseline) -> LayoutUnit;
    fn UsedFontFixedDescent(font: &UsedFont) -> LayoutUnit;
    fn UsedFontFixedDescentForBaseline(font: &UsedFont, baseline: FontBaseline) -> LayoutUnit;
    fn UsedFontUnderlineThickness(font: &UsedFont) -> Option<f32>;
}
