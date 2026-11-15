// An OpenType face at one size and variation location. Blink holds this as
// an hb_font_t over an immutable hb_face_t; here the face is a read-fonts
// `Font` instance (shared across sized copies) that HarfRust shapes with.
use super::harfrust_shaper::shape_run;
use crate::text_shaper::{ShapedRun, TextDirection};
use crate::SharedFontBytes;
use harfrust::font::{Blob, Font, Variation};
use harfrust::{GlyphId, ShaperFont, Tag};
use read_fonts::types::NameId;
use read_fonts::TableProvider;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetric {
    kHorizontalAscender,
    kHorizontalDescender,
    kHorizontalLineGap,
    kVerticalAscender,
    kVerticalDescender,
    kVerticalLineGap,
    kXHeight,
    kCapHeight,
    kUnderlineOffset,
    kUnderlineSize,
    kStrikeoutOffset,
    kStrikeoutSize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphExtents {
    pub x_bearing: f64,
    pub y_bearing: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenTypeVariation {
    pub tag: u32,
    pub value: f32,
}

static NEXT_FACE_IDENTITY: AtomicU64 = AtomicU64::new(1);

fn next_face_identity() -> u32 {
    let id = NEXT_FACE_IDENTITY.fetch_add(1, Ordering::Relaxed);
    u32::try_from(id).expect("OpenType face identity exhausted")
}

fn check_size(size: f64) {
    assert!(
        size.is_finite() && (0.0..=16_000_000.0).contains(&size),
        "invalid OpenType font size"
    );
}

/// The 16.16 pixel scale HarfBuzz receives through `hb_font_set_scale`.
fn scale_for_size(size: f64) -> i32 {
    (size * 65536.0).round().clamp(0.0, i32::MAX as f64) as i32
}

fn check_variations(variations: &[OpenTypeVariation]) -> Vec<OpenTypeVariation> {
    for variation in variations {
        assert!(
            variation.tag != 0 && variation.value.is_finite(),
            "invalid OpenType variation"
        );
    }
    variations.to_vec()
}

/// `hb_font_create` + `hb_font_set_variations`: the face's named instance
/// (upper 16 bits of the face index, as in `hb_face_create`) with explicit
/// axis values applied on top.
fn create_instance(face: &Font, face_index: u32, variations: &[OpenTypeVariation]) -> Font {
    let values = variations
        .iter()
        .map(|variation| Variation::from((Tag::from_u32(variation.tag), variation.value)));
    let builder = face.instance_builder();
    match face_index >> 16 {
        0 => builder.variations(values),
        instance => builder.named_instance_with_overrides(instance - 1, values),
    }
    .build()
}

pub struct OpenTypeFont {
    source_bytes: SharedFontBytes,
    face_index: u32,
    size: f64,
    specified_size: f64,
    font: Font,
    variations: Vec<OpenTypeVariation>,
    uses_open_type_horizontal_advances: bool,
    identity: u32,
}

impl OpenTypeFont {
    pub fn new(bytes: &[u8], face_index: u32, size: f64, variations: &[OpenTypeVariation]) -> Self {
        Self::new_shared(SharedFontBytes::from(bytes), face_index, size, variations)
    }

    /// Retain immutable font data without copying it.
    pub fn new_shared(
        bytes: impl Into<SharedFontBytes>,
        face_index: u32,
        size: f64,
        variations: &[OpenTypeVariation],
    ) -> Self {
        let bytes = bytes.into();
        assert!(
            !bytes.is_empty() && bytes.len() <= u32::MAX as usize,
            "invalid OpenType font request"
        );
        check_size(size);
        let blob = Blob::Shared(Arc::new(bytes.clone()));
        let face = Font::new(blob, face_index & 0xffff).expect("invalid OpenType font or face index");
        assert!(face.num_glyphs() != 0, "font contains no glyphs");
        let has_table = |tag: &[u8; 4]| {
            face.tables()
                .data_for_tag(Tag::new(tag))
                .is_some_and(|data| !data.is_empty())
        };
        let uses_open_type_horizontal_advances = has_table(b"trak") && !has_table(b"sbix");
        let variations = check_variations(variations);
        let font = create_instance(&face, face_index, &variations);
        Self {
            source_bytes: bytes,
            face_index,
            size,
            specified_size: size,
            font,
            variations,
            uses_open_type_horizontal_advances,
            identity: next_face_identity(),
        }
    }

    pub fn FaceIdentity(&self) -> u32 {
        self.identity
    }

    pub fn WithSize(&self, size: f64) -> Arc<Self> {
        let variations = self.variations.clone();
        self.WithSizeAndVariations(size, &variations)
    }

    pub fn WithSizeAndVariations(&self, size: f64, variations: &[OpenTypeVariation]) -> Arc<Self> {
        let specified_size = if self.size > 0.0 {
            self.specified_size * size / self.size
        } else {
            size
        };
        self.WithSizeVariationsAndSpecifiedSize(size, variations, specified_size)
    }

    // Blink HarfBuzzFace::GetScaledFont uses the unzoomed specified CSS size
    // for AAT tracking (ptem), while the glyph scale uses the computed size.
    pub fn WithSizeVariationsAndSpecifiedSize(
        &self,
        size: f64,
        variations: &[OpenTypeVariation],
        specified_size: f64,
    ) -> Arc<Self> {
        assert!(
            specified_size.is_finite() && specified_size >= 0.0 && specified_size <= 16_000_000.0
        );
        check_size(size);
        let variations = check_variations(variations);
        // Instances share the parsed face; only the location differs.
        let font = create_instance(&self.font, self.face_index, &variations);
        Arc::new(Self {
            source_bytes: self.source_bytes.clone(),
            face_index: self.face_index,
            size,
            specified_size,
            font,
            variations,
            uses_open_type_horizontal_advances: self.uses_open_type_horizontal_advances,
            identity: self.identity,
        })
    }

    pub fn HasVariationAxis(&self, tag: u32) -> bool {
        if tag == 0 {
            return false;
        }
        let tag = Tag::from_u32(tag);
        self.font
            .tables()
            .fvar()
            .and_then(|fvar| fvar.axes())
            .is_ok_and(|axes| axes.iter().any(|axis| axis.axis_tag() == tag))
    }

    pub fn UsesOpenTypeHorizontalAdvances(&self) -> bool {
        self.uses_open_type_horizontal_advances
    }

    /// The face at this instance's location, for OpenType table queries.
    pub(crate) fn HarfRustFont(&self) -> &Font {
        &self.font
    }

    /// The scaled font HarfBuzz shapes with (`hb_font_t` with
    /// `hb_ot_font_set_funcs` at this size).
    pub(crate) fn ShaperFont<'f>(&self) -> ShaperFont<'_, 'f> {
        ShaperFont::new(&self.font).with_scale(scale_for_size(self.size))
    }

    pub(crate) fn Scale(&self) -> i32 {
        scale_for_size(self.size)
    }

    pub fn SourceBytes(&self) -> &[u8] {
        &self.source_bytes
    }

    pub fn FaceIndex(&self) -> u32 {
        self.face_index
    }

    pub fn SpecifiedSize(&self) -> f64 {
        self.specified_size
    }

    pub fn Size(&self) -> f64 {
        self.size
    }

    pub fn Variations(&self) -> Vec<OpenTypeVariation> {
        self.variations.clone()
    }

    /// `hb_ot_name_get_utf8` with HB_LANGUAGE_INVALID: English, otherwise the
    /// first record with that name id.
    fn Name(&self, id: u16) -> String {
        self.font
            .names()
            .get(NameId::new(id))
            .and_then(|name| name.english_or_first())
            .map(|localized| localized.name().to_string())
            .unwrap_or_default()
    }

    pub fn FamilyName(&self) -> String {
        let family = self.Name(16);
        if family.is_empty() {
            self.Name(1)
        } else {
            family
        }
    }

    pub fn PostScriptName(&self) -> String {
        self.Name(6)
    }

    pub fn UnitsPerEm(&self) -> u32 {
        u32::from(self.font.units_per_em())
    }

    pub fn GlyphCount(&self) -> u32 {
        self.font.num_glyphs()
    }

    pub fn GlyphForCharacter(&self, character: u32) -> Option<u32> {
        assert!(
            character <= 0x10ffff && !(0xd800..=0xdfff).contains(&character),
            "invalid Unicode scalar"
        );
        self.ShaperFont()
            .nominal_glyph(character)
            .map(|glyph| glyph.to_u32())
    }

    fn CheckGlyph(&self, glyph: u32) {
        assert!(glyph < self.GlyphCount(), "glyph index outside font");
    }

    pub fn HorizontalAdvance(&self, glyph: u32) -> f64 {
        self.CheckGlyph(glyph);
        f64::from(self.ShaperFont().glyph_h_advance(GlyphId::new(glyph))) / 65536.0
    }

    pub fn VerticalAdvance(&self, glyph: u32) -> f64 {
        self.CheckGlyph(glyph);
        f64::from(self.ShaperFont().glyph_v_advance(GlyphId::new(glyph))) / 65536.0
    }

    pub fn Extents(&self, glyph: u32) -> Option<GlyphExtents> {
        self.CheckGlyph(glyph);
        let extents = self.ShaperFont().glyph_extents(GlyphId::new(glyph))?;
        Some(GlyphExtents {
            x_bearing: f64::from(extents.x_bearing) / 65536.0,
            y_bearing: f64::from(extents.y_bearing) / 65536.0,
            width: f64::from(extents.width) / 65536.0,
            height: f64::from(extents.height) / 65536.0,
        })
    }

    /// `hb_ot_metrics_get_position` (harfbuzz/src/hb-ot-metrics.cc), with the
    /// location's MVAR deltas applied by read-fonts.
    pub fn Metric(&self, metric: FontMetric) -> Option<f64> {
        use FontMetric::*;
        let metrics = self.font.metrics();
        let ascender = |value: f64| value.abs();
        let descender = |value: f64| -value.abs();
        let design = match metric {
            kHorizontalAscender | kHorizontalDescender | kHorizontalLineGap => {
                let line = metrics
                    .typo_line
                    .filter(|_| metrics.use_typo_metrics)
                    .or(metrics.hhea_line)?;
                match metric {
                    kHorizontalAscender => ascender(line.ascender.to_f64()),
                    kHorizontalDescender => descender(line.descender.to_f64()),
                    _ => line.line_gap.to_f64(),
                }
            }
            kVerticalAscender | kVerticalDescender | kVerticalLineGap => {
                let line = metrics.vhea_line?;
                match metric {
                    kVerticalAscender => ascender(line.ascender.to_f64()),
                    kVerticalDescender => descender(line.descender.to_f64()),
                    _ => line.line_gap.to_f64(),
                }
            }
            kXHeight => metrics.x_height?.to_f64(),
            kCapHeight => metrics.cap_height?.to_f64(),
            kUnderlineOffset | kUnderlineSize => {
                let underline = self.font.style_metrics().underline?;
                if metric == kUnderlineOffset {
                    underline.position.to_f64()
                } else {
                    underline.thickness.to_f64()
                }
            }
            kStrikeoutOffset | kStrikeoutSize => {
                let strike = self.font.style_metrics().strike_through?;
                if metric == kStrikeoutOffset {
                    strike.position.to_f64()
                } else {
                    strike.thickness.to_f64()
                }
            }
        };
        // hb_font_t::em_scalef_y, then the integer position HarfBuzz returns.
        let upem = f64::from(self.font.units_per_em());
        let position = (design * f64::from(self.Scale()) / upem).round();
        Some(position / 65536.0)
    }

    pub fn CopyTable(&self, tag: u32) -> Vec<u8> {
        self.font
            .tables()
            .data_for_tag(Tag::from_u32(tag))
            .map(|data| data.as_bytes().to_vec())
            .unwrap_or_default()
    }

    pub fn ShapeRun(
        &self,
        utf8: &str,
        direction_value: TextDirection,
        language: &str,
        script: &str,
    ) -> ShapedRun {
        shape_run(self, utf8, direction_value, language, script)
    }
}

#[cfg(test)]
mod shared_bytes_tests {
    use super::*;

    #[test]
    fn instances_retain_shared_backing_until_last_owner() {
        let bytes: Arc<[u8]> = include_bytes!("../../../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into();
        let weak = Arc::downgrade(&bytes);
        let original_pointer = bytes.as_ptr();
        let font = OpenTypeFont::new_shared(bytes, 0, 16.0, &[]);
        let sized = font.WithSize(24.0);
        assert_eq!(font.SourceBytes().as_ptr(), original_pointer);
        assert_eq!(sized.SourceBytes().as_ptr(), original_pointer);
        drop(font);
        let glyph = sized.GlyphForCharacter('A' as u32).unwrap();
        assert!(sized.HorizontalAdvance(glyph) > 0.0);
        assert!(weak.upgrade().is_some());
        drop(sized);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn roboto_metrics_and_names() {
        let font = OpenTypeFont::new(
            include_bytes!("../../../../../assets/fonts/Roboto-Regular.ttf"),
            0,
            16.0,
            &[],
        );
        assert_eq!(font.FamilyName(), "Roboto");
        assert!(font.Metric(FontMetric::kHorizontalAscender).unwrap() > 0.0);
        assert!(font.Metric(FontMetric::kHorizontalDescender).unwrap() < 0.0);
        let run = font.ShapeRun("office", TextDirection::kLtr, "en", "Latn");
        assert!(!run.glyphs.is_empty() && run.advance > 0.0);
    }
}
