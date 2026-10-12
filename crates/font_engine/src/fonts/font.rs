// C++: font_engine/fonts/font.h, font_data.cc, and font_services.cc.
// FontSelector and FontFallbackList are only forward-declared in the supplied
// source. Their pointer identities are retained, with the selector constructor
// explicitly left as a link-time dependency instead of invented behavior.
use foundation::{LayoutUnit, Member, TabSize, Traceable, Visitor};

use super::font_description::FontDescription;
use super::font_fallback_metadata::{FontFallbackMetadata, FontFallbackRole};
use super::font_fallback_priority::{
    FontFallbackPriority, IsEmojiPresentationEmoji, IsTextPresentationEmoji,
};
use super::shaping::font_features::{FontFeatureRange, FontFeatureRanges};
use super::simple_font_data::SimpleFontData;
use super::text_fragment_paint_info::TextFragmentPaintInfo;
use crate::text::native::character::Character;
use foundation::gfx;

// C++ forward declarations; these uninhabited Rust types cannot be created.
pub enum FontFallbackList {}
pub enum FontSelector {}

fn LocaleRank(candidate_locales: &[String], requested: &str) -> u8 {
    if candidate_locales.is_empty() {
        return 2;
    }
    let requested = requested.replace('_', "-").to_ascii_lowercase();
    let requested_language = requested.split('-').next().unwrap_or("");
    let mut rank = 3;
    for candidate in candidate_locales {
        let candidate = candidate.replace('_', "-").to_ascii_lowercase();
        if candidate == requested
            || (!candidate.is_empty()
                && (requested.starts_with(&(candidate.clone() + "-"))
                    || candidate.starts_with(&(requested.clone() + "-"))))
        {
            return 0;
        }
        if !requested_language.is_empty()
            && candidate.split('-').next().unwrap_or("") == requested_language
        {
            rank = rank.min(1);
        }
    }
    rank
}

fn IsMathCharacter(character: u32) -> bool {
    matches!(
        character,
        0x2190..=0x21ff
            | 0x2200..=0x22ff
            | 0x27c0..=0x27ef
            | 0x2980..=0x2aff
            | 0x1d400..=0x1d7ff
    )
}

fn IsSymbolCharacter(character: u32) -> bool {
    matches!(
        character,
        0x2000..=0x206f
            | 0x20a0..=0x20cf
            | 0x2100..=0x214f
            | 0x2300..=0x27bf
            | 0x2b00..=0x2bff
    )
}

fn RoleRank(role: FontFallbackRole, hint: u32, priority: FontFallbackPriority) -> u8 {
    let order = if IsEmojiPresentationEmoji(priority) {
        [
            FontFallbackRole::kEmoji,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
        ]
    } else if IsTextPresentationEmoji(priority) {
        [
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    } else if IsMathCharacter(hint) {
        [
            FontFallbackRole::kMath,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kEmoji,
        ]
    } else if IsSymbolCharacter(hint) {
        [
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    } else {
        [
            FontFallbackRole::kText,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    };
    order
        .iter()
        .position(|candidate| *candidate == role)
        .map_or(5, |rank| rank as u8)
}

// CSS Fonts directional weight matching, shared with the face-group binding.
fn FallbackWeightRank(candidate: f32, requested: f32) -> (u8, f32) {
    if requested < 400.0 {
        if candidate <= requested {
            (0, requested - candidate)
        } else {
            (1, candidate - requested)
        }
    } else if requested <= 500.0 {
        if candidate >= requested && candidate <= 500.0 {
            (0, candidate - requested)
        } else if candidate < requested {
            (1, requested - candidate)
        } else {
            (2, candidate - requested)
        }
    } else if candidate >= requested {
        (0, candidate - requested)
    } else {
        (1, requested - candidate)
    }
}

fn FallbackCandidateRank(
    font: &SimpleFontData,
    metadata: &FontFallbackMetadata,
    hint: u32,
    locale: &str,
    priority: FontFallbackPriority,
    requested_weight: f32,
    requested_italic: bool,
) -> (u8, u8, u8, bool, u8, f32) {
    let (weight_direction, weight_distance) = FallbackWeightRank(metadata.weight, requested_weight);
    (
        (!font.PlatformData().FontContainsCharacter(hint)) as u8,
        RoleRank(metadata.role, hint, priority),
        LocaleRank(&metadata.locales, locale),
        metadata.italic != requested_italic,
        weight_direction,
        weight_distance,
    )
}

// cpp: font_engine/fonts/font.h:67-229
#[derive(Clone)]
pub struct Font {
    font_description_: FontDescription,
    font_fallback_list_: Member<FontFallbackList>,
    primary_font_: Member<SimpleFontData>,
    fallback_fonts_: Vec<Member<SimpleFontData>>,
    fallback_metadata_: Vec<FontFallbackMetadata>,
    font_group_length_: usize,
}

// cpp: font_engine/fonts/font_data.cc:31
impl Default for Font {
    fn default() -> Self {
        Self {
            font_description_: FontDescription::default(),
            font_fallback_list_: Member::default(),
            primary_font_: Member::default(),
            fallback_fonts_: Vec::new(),
            fallback_metadata_: Vec::new(),
            font_group_length_: 0,
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
            fallback_metadata_: vec![FontFallbackMetadata::default()],
            font_group_length_: 1,
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
        let metadata = vec![FontFallbackMetadata::default(); ordered_fonts.len()];
        Self::new_with_fallback_metadata(
            description,
            ordered_fonts,
            primary_font,
            &metadata,
            ordered_fonts.len(),
        )
    }

    /// Constructor used by the Browser -> LayoutNG -> FontEngine catalog
    /// boundary. Family membership is request-specific; the immutable role and
    /// locale metadata remains attached to each resolved face.
    pub fn new_with_fallback_metadata(
        description: FontDescription,
        ordered_fonts: &[*const SimpleFontData],
        primary_font: *const SimpleFontData,
        fallback_metadata: &[FontFallbackMetadata],
        font_group_length: usize,
    ) -> Self {
        assert!(!ordered_fonts.is_empty());
        assert!(!primary_font.is_null());
        assert_eq!(ordered_fonts.len(), fallback_metadata.len());
        assert!(font_group_length <= ordered_fonts.len());
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
            fallback_metadata_: fallback_metadata.to_vec(),
            font_group_length_: font_group_length,
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

    pub fn FallbackMetadata(&self) -> &[FontFallbackMetadata] {
        &self.fallback_metadata_
    }

    pub fn FontGroupLength(&self) -> usize {
        self.font_group_length_
    }

    // cpp: font_engine/fonts/font_fallback_iterator.cc:99-198
    //
    // The concrete platform faces cross the package boundary once, but their
    // order after the CSS family group is selected here for each shaping
    // segment. This keeps character/locale/presentation policy in FontEngine
    // rather than baking a global filesystem order into Layout.
    pub fn FallbackOrderForHint(
        &self,
        hint: u32,
        priority: FontFallbackPriority,
    ) -> Vec<Member<SimpleFontData>> {
        if self.fallback_fonts_.is_empty() {
            return Vec::new();
        }
        let group_length = self.font_group_length_.min(self.fallback_fonts_.len());
        let mut result = self.fallback_fonts_[..group_length].to_vec();
        let locale = self.font_description_.LocaleOrDefault().Ascii();
        let requested_weight = self.font_description_.Weight().RawValue() as f32 / 4.0;
        let requested_italic = self.font_description_.Style().RawValue() >= 14 * 4;
        let skip_system =
            Character::IsPrivateUse(hint as i32) || Character::IsNonCharacter(hint as i32);
        let mut system: Vec<usize> = (group_length..self.fallback_fonts_.len())
            .filter(|&index| {
                !matches!(
                    self.fallback_metadata_[index].role,
                    FontFallbackRole::kFamilyOnly | FontFallbackRole::kLastResort
                )
            })
            .collect();
        if !skip_system {
            system.sort_by(|&left, &right| {
                let left_font = unsafe { &*self.fallback_fonts_[left].Get() };
                let right_font = unsafe { &*self.fallback_fonts_[right].Get() };
                let left_metadata = &self.fallback_metadata_[left];
                let right_metadata = &self.fallback_metadata_[right];
                FallbackCandidateRank(
                    left_font,
                    left_metadata,
                    hint,
                    &locale,
                    priority,
                    requested_weight,
                    requested_italic,
                )
                .partial_cmp(&FallbackCandidateRank(
                    right_font,
                    right_metadata,
                    hint,
                    &locale,
                    priority,
                    requested_weight,
                    requested_italic,
                ))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.cmp(&right))
            });
            result.extend(system.into_iter().map(|index| self.fallback_fonts_[index]));
        }
        result.extend(
            (group_length..self.fallback_fonts_.len())
                .filter(|&index| {
                    self.fallback_metadata_[index].role == FontFallbackRole::kLastResort
                })
                .map(|index| self.fallback_fonts_[index]),
        );

        // FontFallbackIterator remembers its first candidate and retries that
        // exact face last so unresolved text receives the expected .notdef.
        // Keep the duplicate: ShapeWithFallback commits the final candidate.
        let first = result.first().copied().unwrap_or(self.fallback_fonts_[0]);
        result.push(first);
        result
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
    pub fn GetFontFeatures(&self) -> FontFeatureRanges {
        let mut features = FontFeatureRanges::new();
        FontFeatureRange::FromFontDescription(&self.font_description_, &mut features);
        features
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
    fallback_metadata: &[FontFallbackMetadata],
    font_group_length: usize,
) -> Font {
    Font::new_with_fallback_metadata(
        description.clone(),
        ordered_fonts,
        primary_font,
        fallback_metadata,
        font_group_length,
    )
}

unsafe extern "Rust" {
    fn FontNewWithSelector(description: FontDescription, selector: *mut FontSelector) -> Font;
}
