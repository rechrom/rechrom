// C++: font_engine/fonts/shaping/han_kerning.h and han_kerning_font_data.cc.
// The supplied .cc defines FontData construction and GetCharType, but not
// HanKerning's AppendFontFeatures, PrepareFallback, or ApplyKerning methods.
#![allow(non_snake_case, dead_code)]

use super::harfbuzz_shaper::{GlyphData, GlyphDataList, HarfBuzzShaper};
use crate::fonts::glyph::Glyph;
use crate::fonts::simple_font_data::SimpleFontData;
use crate::text::native::character::Character;
use crate::text::native::harfbuzz as hb;
use crate::text::native::layout_locale::LayoutLocale;
use foundation::gfx::RectF;
use foundation::{HanKerningCharType as CharType, String, TextDirection};

const USCRIPT_HAN: i32 = 17;
const HB_OT_TAG_GPOS: u32 = u32::from_be_bytes(*b"GPOS");

// HarfBuzz is a source package dependency. These are ABI declarations, not
// substituted implementations of its feature table query.
unsafe extern "C" {
    fn hb_ot_layout_table_get_feature_tags(
        face: *mut hb::HbFace,
        table_tag: u32,
        start_offset: u32,
        feature_count: *mut u32,
        feature_tags: *mut u32,
    ) -> u32;
}

// cpp: font_engine/fonts/shaping/han_kerning.h:50-55
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub is_horizontal: bool,
    pub is_line_start: bool,
    pub apply_start: bool,
    pub apply_end: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            is_horizontal: true,
            is_line_start: false,
            apply_start: false,
            apply_end: false,
        }
    }
}

// cpp: font_engine/fonts/shaping/han_kerning.h:126,139-149
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Priority {
    kText,
    kCache,
}

pub struct HanKerning {
    may_apply_: bool,
    is_start_prev_used_: bool,
    is_end_next_used_: bool,
    segment_start_: u32,
    segment_end_: u32,
    last_start_: u32,
    last_end_: u32,
    last_font_data_: *const FontData,
    char_types_: Vec<CharType>,
    unsafe_to_break_before_: Vec<u32>,
    changed_indexes_: Vec<u32>,
}

impl HanKerning {
    // cpp: font_engine/fonts/shaping/han_kerning.h:67,70-73
    pub fn MayApply(&self) -> bool {
        self.may_apply_
    }
    pub fn UnsafeToBreakBefore(&self) -> &[u32] {
        &self.unsafe_to_break_before_
    }
    pub fn ClearUnsafeToBreakBefore(&mut self) {
        self.unsafe_to_break_before_.clear();
    }

    // cpp: font_engine/fonts/shaping/han_kerning.h:164-174
    pub fn ShouldKern(kind: CharType, last_type: CharType) -> bool {
        kind == CharType::kOpen
            && matches!(
                last_type,
                CharType::kOpen | CharType::kMiddle | CharType::kClose | CharType::kOpenNarrow
            )
    }
    pub fn ShouldKernLast(kind: CharType, last_type: CharType) -> bool {
        last_type == CharType::kClose
            && matches!(
                kind,
                CharType::kClose | CharType::kMiddle | CharType::kCloseNarrow
            )
    }

    // The constructor and inline MayApply/DidShapeSegment bodies at
    // han_kerning.h:57-65,152-162 remain blocked: StringView lacks Is8Bit,
    // FontDescription lacks GetTextSpacingTrim, and ApplyKerning has no source
    // definition. Declarations at :75-85 and :128-137 also lack definitions.
}

// Rust cannot nest a struct in an impl. This module's FontData corresponds to
// HanKerning::FontData and retains the source field defaults and constructor.
// cpp: font_engine/fonts/shaping/han_kerning.h:89-116
#[derive(Clone, Debug)]
pub struct FontData {
    pub has_alternate_spacing: bool,
    pub has_contextual_spacing: bool,
    pub is_quote_fullwidth: bool,
    pub type_for_dot: CharType,
    pub type_for_colon: CharType,
    pub type_for_semicolon: CharType,
}

impl Default for FontData {
    fn default() -> Self {
        Self {
            has_alternate_spacing: false,
            has_contextual_spacing: false,
            is_quote_fullwidth: false,
            type_for_dot: CharType::kOther,
            type_for_colon: CharType::kOther,
            type_for_semicolon: CharType::kOther,
        }
    }
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:20-40
fn HasOpenTypeFeature(font: &SimpleFontData, tag: u32) -> bool {
    let hb_font = font.PlatformData().RawFont().BorrowHarfBuzzFont();
    assert!(!hb_font.is_null());
    let face = unsafe { hb::hb_font_get_face(hb_font) };
    assert!(!face.is_null());
    let mut offset = 0u32;
    let mut tags = [0u32; 64];
    loop {
        let mut count = tags.len() as u32;
        let total = unsafe {
            hb_ot_layout_table_get_feature_tags(
                face,
                HB_OT_TAG_GPOS,
                offset,
                &mut count,
                tags.as_mut_ptr(),
            )
        };
        if tags[..count as usize].contains(&tag) {
            return true;
        }
        offset += count;
        if count == 0 || offset >= total {
            return false;
        }
    }
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:42-44
fn Advance(glyph: &GlyphData, is_horizontal: bool) -> f32 {
    if is_horizontal {
        glyph.advance.x()
    } else {
        glyph.advance.y()
    }
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:46-65
fn CharTypeFromSingleBounds(half_em: f32, bounds: &RectF, is_horizontal: bool) -> CharType {
    if is_horizontal {
        if bounds.right() <= half_em {
            return CharType::kClose;
        }
        if bounds.x() >= half_em {
            return CharType::kOpen;
        }
        if bounds.width() <= half_em && bounds.x() >= half_em / 2.0 {
            return CharType::kMiddle;
        }
    } else {
        if bounds.bottom() <= half_em {
            return CharType::kClose;
        }
        if bounds.y() >= half_em {
            return CharType::kOpen;
        }
        if bounds.height() <= half_em && bounds.y() >= half_em / 2.0 {
            return CharType::kMiddle;
        }
    }
    CharType::kOther
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:67-77
fn CharTypeAtIndex(
    glyphs: &GlyphDataList,
    bounds: &[RectF],
    index: usize,
    is_horizontal: bool,
) -> CharType {
    let glyph = &glyphs[index];
    if glyph.glyph == 0 {
        return CharType::kOther;
    }
    CharTypeFromSingleBounds(
        Advance(glyph, is_horizontal) / 2.0,
        &bounds[index],
        is_horizontal,
    )
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:79-111
fn CharTypeFromBounds(
    glyphs: &GlyphDataList,
    bounds: &[RectF],
    start: usize,
    count: usize,
    is_horizontal: bool,
) -> CharType {
    let mut first_advance = 0.0;
    let mut half_first_advance = 0.0;
    let mut first_type = CharType::kOther;
    let mut i = start;
    while i < start + count {
        if glyphs[i].glyph != 0 {
            first_advance = Advance(&glyphs[i], is_horizontal);
            half_first_advance = first_advance / 2.0;
            first_type = CharTypeFromSingleBounds(half_first_advance, &bounds[i], is_horizontal);
            break;
        }
        i += 1;
    }
    if i == start + count {
        return CharType::kOther;
    }
    i += 1;
    while i < start + count {
        if glyphs[i].glyph != 0
            && (Advance(&glyphs[i], is_horizontal) != first_advance
                || CharTypeFromSingleBounds(half_first_advance, &bounds[i], is_horizontal)
                    != first_type)
        {
            return CharType::kOther;
        }
        i += 1;
    }
    first_type
}

// cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:115-141
impl HanKerning {
    pub fn GetCharType(ch: u16, font_data: &FontData) -> CharType {
        match Character::GetHanKerningCharType(i32::from(ch)) {
            kind @ (CharType::kOther
            | CharType::kOpen
            | CharType::kClose
            | CharType::kMiddle
            | CharType::kOpenNarrow
            | CharType::kCloseNarrow
            | CharType::kInvalid) => kind,
            CharType::kDot => font_data.type_for_dot,
            CharType::kColon => font_data.type_for_colon,
            CharType::kSemicolon => font_data.type_for_semicolon,
            CharType::kOpenQuote => {
                if font_data.is_quote_fullwidth {
                    CharType::kOpen
                } else {
                    CharType::kOpenNarrow
                }
            }
            CharType::kCloseQuote => {
                if font_data.is_quote_fullwidth {
                    CharType::kClose
                } else {
                    CharType::kCloseNarrow
                }
            }
        }
    }
}

impl FontData {
    // cpp: font_engine/fonts/shaping/han_kerning_font_data.cc:143-202
    pub fn new(font: &SimpleFontData, locale: &LayoutLocale, is_horizontal: bool) -> Self {
        let mut data = Self::default();
        let alternate_tag = if is_horizontal {
            hb::tag(*b"halt")
        } else {
            hb::tag(*b"vhal")
        };
        data.has_alternate_spacing = HasOpenTypeFeature(font, alternate_tag);
        if !data.has_alternate_spacing {
            return data;
        }

        let contextual_tag = if is_horizontal {
            hb::tag(*b"chws")
        } else {
            hb::tag(*b"vchw")
        };
        data.has_contextual_spacing = HasOpenTypeFeature(font, contextual_tag);

        // UChar constants from foundation/blink_base/wtf/text/character_names.h.
        let characters: [u16; 10] = [
            0x3001, 0x3002, 0xFF0C, 0xFF0E, 0xFF1A, 0xFF1B, 0x201C, 0x2018, 0x201D, 0x2019,
        ];
        const DOT_COUNT: usize = 4;
        const COLON_INDEX: usize = 4;
        const SEMICOLON_INDEX: usize = 5;
        const QUOTE_INDEX: usize = 6;
        let shaper = HarfBuzzShaper::new(String::from_utf16(&characters));
        let mut glyph_data = Vec::new();
        shaper.GetGlyphData(
            font,
            locale,
            USCRIPT_HAN,
            is_horizontal,
            TextDirection::kLtr,
            &mut glyph_data,
        );
        if glyph_data.len() != characters.len() {
            data.has_alternate_spacing = false;
            return data;
        }

        let mut glyphs: Vec<Glyph> = Vec::new();
        let mut cluster = 0u32;
        for item in &glyph_data {
            if item.cluster != cluster {
                data.has_alternate_spacing = false;
                return data;
            }
            cluster += 1;
            glyphs.push(item.glyph);
        }
        let mut bounds = Vec::new();
        font.BoundsForGlyphs(&glyphs, &mut bounds);
        for (bound, item) in bounds.iter_mut().zip(&glyph_data) {
            bound.Offset(item.offset.x(), item.offset.y());
        }

        data.type_for_dot = CharTypeFromBounds(&glyph_data, &bounds, 0, DOT_COUNT, is_horizontal);
        data.type_for_colon = CharTypeAtIndex(&glyph_data, &bounds, COLON_INDEX, is_horizontal);
        data.type_for_semicolon =
            CharTypeAtIndex(&glyph_data, &bounds, SEMICOLON_INDEX, is_horizontal);
        data.is_quote_fullwidth =
            CharTypeFromBounds(&glyph_data, &bounds, QUOTE_INDEX, 2, is_horizontal)
                == CharType::kOpen
                && CharTypeFromBounds(&glyph_data, &bounds, QUOTE_INDEX + 2, 2, is_horizontal)
                    == CharType::kClose;
        data
    }
}
