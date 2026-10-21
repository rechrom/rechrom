// cpp: layoutng_style/style/font_size_style.h:8-10
// Pending font_engine:api and foundation:blink_geometry_api connections.
use font_engine::Font;
use foundation::Length;

// cpp: layoutng_style/style/font_size_style.h:14-26
/// Borrowed font-size inputs used during CSS length resolution.
pub struct FontSizeStyle<'a> {
    font_: *const Font,
    specified_line_height_: &'a Length,
    effective_zoom_: f32,
}

#[allow(non_snake_case)]
impl<'a> FontSizeStyle<'a> {
    pub fn new(font: *const Font, specified_line_height: &'a Length, effective_zoom: f32) -> Self {
        Self {
            font_: font,
            specified_line_height_: specified_line_height,
            effective_zoom_: effective_zoom,
        }
    }

    // cpp: layoutng_style/style/font_size_style.h:34-39
    pub fn GetFont(&self) -> *const Font {
        self.font_
    }
    pub fn LineHeight(&self) -> &Length {
        self.specified_line_height_
    }
    pub fn SpecifiedFontSize(&self) -> f32 {
        assert!(
            !self.font_.is_null(),
            "FontSizeStyle requires a font for size lookup"
        );
        // C++ dereferences the borrowed const Font* without taking ownership.
        unsafe { (*self.font_).GetFontDescription().SpecifiedSize() }
    }
    pub fn EffectiveZoom(&self) -> f32 {
        self.effective_zoom_
    }
}

// cpp: layoutng_style/style/font_size_style.h:28-32
impl PartialEq for FontSizeStyle<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.font_ == other.font_
            && self.specified_line_height_ == other.specified_line_height_
            && self.effective_zoom_ == other.effective_zoom_
    }
}
