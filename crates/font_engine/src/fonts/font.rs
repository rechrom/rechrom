// C++: font_engine/fonts/font.h, font_data.cc, and font_services.cc.
// FontSelector and FontFallbackList are only forward-declared in the supplied
// source. Their pointer identities are retained, with the selector constructor
// explicitly left as a link-time dependency instead of invented behavior.
use foundation::{LayoutUnit, Member, TabSize, Traceable, Visitor};

use super::font_description::FontDescription;
use super::shaping::font_features::FontFeatureRange;
use super::simple_font_data::SimpleFontData;
use super::text_fragment_paint_info::TextFragmentPaintInfo;
use foundation::gfx;

// C++ forward declarations; these uninhabited Rust types cannot be created.
pub enum FontFallbackList {}
pub enum FontSelector {}

// cpp: font_engine/fonts/font.h:67-229
#[derive(Clone)]
pub struct Font {
    font_description_: FontDescription,
    font_fallback_list_: Member<FontFallbackList>,
    primary_font_: Member<SimpleFontData>,
    fallback_fonts_: Vec<Member<SimpleFontData>>,
}

// cpp: font_engine/fonts/font_data.cc:31
impl Default for Font {
    fn default() -> Self {
        Self {
            font_description_: FontDescription::default(),
            font_fallback_list_: Member::default(),
            primary_font_: Member::default(),
            fallback_fonts_: Vec::new(),
        }
    }
}

#[allow(non_snake_case)]
impl Font {
    // cpp: font_engine/fonts/font_services.cc:111-116
    pub fn TextInkBounds(&self, text_info: &TextFragmentPaintInfo) -> gfx::RectF {
        if self.ShouldSkipDrawing() || text_info.shape_result.is_null() {
            return gfx::RectF::default();
        }
        unsafe { &*text_info.shape_result }.ComputeInkBounds()
    }
    // cpp: font_engine/fonts/font_data.cc:33
    pub fn new(description: FontDescription) -> Self {
        Self {
            font_description_: description,
            ..Self::default()
        }
    }

    // cpp: font_engine/fonts/font_data.cc:35-40
    pub fn new_with_primary(
        description: FontDescription,
        primary_font: *const SimpleFontData,
    ) -> Self {
        assert!(!primary_font.is_null());
        let member = Member::from_ptr(primary_font.cast_mut());
        Self {
            font_description_: description,
            font_fallback_list_: Member::default(),
            primary_font_: member,
            fallback_fonts_: vec![member],
        }
    }

    // cpp: font_engine/fonts/font_data.cc:42-44
    pub fn new_with_ordered_fonts(
        description: FontDescription,
        ordered_fonts: &[*const SimpleFontData],
    ) -> Self {
        let primary_font = *ordered_fonts
            .first()
            .expect("ordered fonts must be nonempty");
        Self::new_with_ordered_fonts_and_primary(description, ordered_fonts, primary_font)
    }

    // cpp: font_engine/fonts/font_data.cc:46-59
    pub fn new_with_ordered_fonts_and_primary(
        description: FontDescription,
        ordered_fonts: &[*const SimpleFontData],
        primary_font: *const SimpleFontData,
    ) -> Self {
        assert!(!ordered_fonts.is_empty());
        assert!(!primary_font.is_null());
        let mut fallback_fonts = Vec::with_capacity(ordered_fonts.len());
        for font in ordered_fonts {
            assert!(!font.is_null());
            fallback_fonts.push(Member::from_ptr(font.cast_mut()));
        }
        Self {
            font_description_: description,
            font_fallback_list_: Member::default(),
            primary_font_: Member::from_ptr(primary_font.cast_mut()),
            fallback_fonts_: fallback_fonts,
        }
    }

    // cpp: font_engine/fonts/font.h:70-71
    // No definition is supplied for Font(const FontDescription&, FontSelector*).
    pub fn new_with_selector(description: FontDescription, selector: *mut FontSelector) -> Self {
        unsafe { FontNewWithSelector(description, selector) }
    }

    // cpp: font_engine/fonts/font.h:83-88
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.font_fallback_list_);
        visitor.Trace(&self.primary_font_);
        for font in &self.fallback_fonts_ {
            visitor.Trace(font);
        }
    }

    // cpp: font_engine/fonts/font.h:92-94
    pub fn GetFontDescription(&self) -> &FontDescription {
        &self.font_description_
    }

    // cpp: font_engine/fonts/font.h:168-170
    pub fn OrderedFontData(&self) -> &[Member<SimpleFontData>] {
        &self.fallback_fonts_
    }

    // cpp: font_engine/fonts/font.h:207
    pub fn NullifyForTesting(&mut self) {
        self.font_fallback_list_.Clear();
    }

    // cpp: font_engine/fonts/font_services.cc:29
    pub fn PrimaryFont(&self) -> *const SimpleFontData {
        self.primary_font_.Get()
    }

    // cpp: font_engine/fonts/font_services.cc:31-35
    pub fn PrimaryFontWithDigitZero(&self) -> *const SimpleFontData {
        let primary = self.PrimaryFont();
        if !primary.is_null() && unsafe { (&*primary).GlyphForCharacter(0x0030) } != 0 {
            primary
        } else {
            std::ptr::null()
        }
    }

    // cpp: font_engine/fonts/font_services.cc:37-41
    pub fn PrimaryFontWithCjkWater(&self) -> *const SimpleFontData {
        let primary = self.PrimaryFont();
        if !primary.is_null() && unsafe { (&*primary).GlyphForCharacter(0x6C34) } != 0 {
            primary
        } else {
            std::ptr::null()
        }
    }

    // cpp: font_engine/fonts/font_services.cc:43-45
    pub fn PrimaryFontForTabSize(&self) -> *const SimpleFontData {
        self.PrimaryFont()
    }

    // cpp: font_engine/fonts/font_services.cc:47
    pub fn GetFontSelector(&self) -> *mut FontSelector {
        std::ptr::null_mut()
    }

    // cpp: font_engine/fonts/font_services.cc:49-53
    pub fn SpaceWidth(&self) -> f32 {
        let primary = self.PrimaryFont();
        if primary.is_null() {
            0.0
        } else {
            (unsafe { (&*primary).SpaceWidth() }) + self.font_description_.LetterSpacing()
        }
    }

    // cpp: font_engine/fonts/font_services.cc:55-63
    pub fn TabWidthInternal(
        &self,
        font_data: *const SimpleFontData,
        tab_size: &TabSize,
    ) -> (f32, bool) {
        if font_data.is_null() {
            return (0.0, false);
        }
        (
            tab_size.GetPixelSize(
                unsafe { &*font_data }.SpaceWidth(),
                self.font_description_.LetterSpacing(),
                self.font_description_.WordSpacing(),
            ),
            true,
        )
    }

    // cpp: font_engine/fonts/font_services.cc:65-82
    pub fn TabWidth(&self, font_data: *const SimpleFontData, tab_size: &TabSize) -> f32 {
        self.TabWidthInternal(font_data, tab_size).0
    }

    pub fn TabWidthAt(
        &self,
        font_data: *const SimpleFontData,
        tab_size: &TabSize,
        position: f32,
    ) -> f32 {
        let width = self.TabWidth(font_data, tab_size);
        if width <= 0.0 {
            return 0.0;
        }
        let remainder = position % width;
        if remainder != 0.0 {
            width - remainder
        } else {
            width
        }
    }

    pub fn TabWidthLayoutUnit(&self, tab_size: &TabSize, position: LayoutUnit) -> LayoutUnit {
        LayoutUnit::FromFloatRound(self.TabWidthAt(
            self.primary_font_.Get(),
            tab_size,
            position.ToFloat(),
        ))
    }

    // cpp: font_engine/fonts/font_services.cc:84-86
    pub fn TextAutoSpaceInlineSize(&self) -> f32 {
        let primary = self.PrimaryFont();
        if primary.is_null() {
            0.0
        } else {
            unsafe { (&*primary).TextAutoSpaceInlineSize() }
        }
    }

    // cpp: font_engine/fonts/font_services.cc:93
    pub fn GetFontFeatures(&self) -> &[FontFeatureRange] {
        &[]
    }

    // cpp: font_engine/fonts/font_services.cc:95
    pub fn HasSimpleFontFeatures(&self) -> bool {
        true
    }

    // cpp: font_engine/fonts/font_services.cc:97-101
    pub fn CanShapeWordByWord(&self) -> bool {
        false
    }

    // cpp: font_engine/fonts/font_services.cc:103-109
    pub fn SetCanShapeWordByWordForTesting(&mut self, _value: bool) {}
    pub fn NullifyPrimaryFontForTesting(&mut self) {
        self.primary_font_.Clear();
    }
    pub fn ShouldSkipDrawing(&self) -> bool {
        false
    }
    pub fn HasCustomFont(&self) -> bool {
        let primary = self.PrimaryFont();
        !primary.is_null() && unsafe { (&*primary).IsCustomFont() }
    }
    pub fn IsFallbackValid(&self) -> bool {
        !self.PrimaryFont().is_null()
    }
}

// cpp: font_engine/fonts/font_services.cc:24-27
impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.primary_font_ == other.primary_font_
            && self.font_description_ == other.font_description_
    }
}

impl Traceable for Font {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        Font::Trace(self, visitor);
    }
}

// cpp: font_engine/fonts/font_data.cc:46-59
// The layoutng selector boundary calls this source-owned constructor.
#[unsafe(no_mangle)]
pub extern "Rust" fn FontNewFromResolvedData(
    description: &FontDescription,
    ordered_fonts: &[*const SimpleFontData],
    primary_font: *const SimpleFontData,
) -> Font {
    Font::new_with_ordered_fonts_and_primary(description.clone(), ordered_fonts, primary_font)
}

unsafe extern "Rust" {
    fn FontNewWithSelector(description: FontDescription, selector: *mut FontSelector) -> Font;
}
