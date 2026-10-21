// cpp: layoutng_style/style/text_size_adjust.h:12-17
/// Computed CSS text-size-adjust multiplier; negative values mean auto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSizeAdjust {
    adjustment_: f32,
}

#[allow(non_snake_case)]
impl TextSizeAdjust {
    pub fn new(adjustment: f32) -> Self {
        Self {
            adjustment_: adjustment,
        }
    }

    // cpp: layoutng_style/style/text_size_adjust.h:19-22
    pub fn AdjustAuto() -> Self {
        Self::new(-1.0)
    }
    pub fn AdjustNone() -> Self {
        Self::new(1.0)
    }

    // cpp: layoutng_style/style/text_size_adjust.h:24-30
    pub fn IsAuto(&self) -> bool {
        self.adjustment_ < 0.0
    }
    pub fn Multiplier(&self) -> f32 {
        debug_assert!(!self.IsAuto());
        self.adjustment_
    }
}

// cpp: layoutng_style/style/text_size_adjust.h:32-40
// Fieldwise f32 PartialEq has the same NaN behavior as C++ float equality.
