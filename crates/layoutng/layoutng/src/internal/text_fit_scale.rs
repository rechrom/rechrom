use font_engine::Font;
use foundation::{Member, Visitor};

// cpp: layoutng/internal/text_fit_scale.h:15-22
pub struct TextFitScale {
    pub scale: f32,
    pub font: Member<Font>,
}

impl Default for TextFitScale {
    fn default() -> Self {
        Self {
            scale: 1.0,
            font: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl TextFitScale {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.font);
    }
}

// cpp: layoutng/internal/text_fit_scale.h:27-48
pub struct TextFitBlockScale {
    pub paint_scale: f32,
    pub scaled_font: *const Font,
}

impl Default for TextFitBlockScale {
    fn default() -> Self {
        Self {
            paint_scale: 1.0,
            scaled_font: std::ptr::null(),
        }
    }
}

#[allow(non_snake_case, non_upper_case_globals)]
impl TextFitBlockScale {
    pub const kFixed: *const Self = std::ptr::null();

    pub fn TotalScale(&self, original_font: &Font) -> f32 {
        if self.scaled_font.is_null() {
            return self.paint_scale;
        }
        let original_font_size = original_font.GetFontDescription().ComputedSize();
        if original_font_size > 0.0 {
            return unsafe { &*self.scaled_font }
                .GetFontDescription()
                .ComputedSize()
                / original_font_size
                * self.paint_scale;
        }
        self.paint_scale
    }
}
