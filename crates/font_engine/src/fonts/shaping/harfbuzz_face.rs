// C++: font_engine/fonts/shaping/harfbuzz_face.h.
use super::super::font_platform_data::FontPlatformData;
use crate::text::native::opentype_font::HarfBuzzFont;
use foundation::{Member, Traceable, Visitor};

// cpp: font_engine/fonts/shaping/harfbuzz_face.h:16-29
pub struct HarfBuzzFace {
    platform_data_: Member<FontPlatformData>,
}

impl HarfBuzzFace {
    pub fn new(platform_data: &FontPlatformData) -> Self {
        Self {
            platform_data_: Member::from_ptr(
                platform_data as *const FontPlatformData as *mut FontPlatformData,
            ),
        }
    }

    pub fn GetScaledFont(&self) -> *mut HarfBuzzFont {
        let platform_data = self.platform_data_.Get();
        assert!(
            !platform_data.is_null(),
            "HarfBuzzFace has no platform data"
        );
        unsafe { (&*platform_data).RawFont().BorrowHarfBuzzFont() }
    }
}

impl Traceable for HarfBuzzFace {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.platform_data_);
    }
}
