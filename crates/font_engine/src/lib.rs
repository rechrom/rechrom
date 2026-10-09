#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod font_family_names;
pub mod fonts;
pub mod minimal_font;
pub mod shared_font_bytes;
pub use shared_font_bytes::SharedFontBytes;
pub mod text;
pub mod text_shaper;

pub use text::native::hyphenation::{Hyphenation, HyphenationBackend};
pub use text::native::hyphenation_services::{
    CurrentNativeHyphenationResolver, NativeHyphenationResolver, NativeHyphenationResolverScope,
};
pub use text::native::layout_locale::LayoutLocale;
pub use text::native::opentype_font::{FontMetric, GlyphExtents, OpenTypeFont, OpenTypeVariation};
pub use text_shaper::{ShapeRequest, ShapedGlyph, ShapedRun, TextDirection, TextShaper};

pub use fonts::canvas_rotation_in_vertical::{
    CanvasRotationInVertical, IsCanvasRotationInVerticalUpright, IsCanvasRotationOblque,
};
pub use fonts::custom_font_data::{BasicCustomFontData, CustomFontData};
pub use fonts::font::{Font, FontSelector};
pub use fonts::font_backend::{
    FontBackend, FontBackendFactory, FontBackendGlyphMetrics, FontBackendMetrics, FontVariation,
};
pub use fonts::font_baseline::FontBaseline;
pub use fonts::font_data::FontData;
pub use fonts::font_description::FontDescription;
pub use fonts::font_fallback_priority::FontFallbackPriority;
pub use fonts::font_family::{FontFamily, FontFamilyType, SharedFontFamily};
pub use fonts::font_height::FontHeight;
pub use fonts::font_metrics::{ApplyBaselineTable, FontMetrics};
pub use fonts::font_metrics_override::FontMetricsOverride;
pub use fonts::font_optical_sizing::OpticalSizing;
pub use fonts::font_orientation::FontOrientation;
pub use fonts::font_palette::{
    BasePaletteValue, BasePaletteValueType, FontPalette, FontPaletteOverride, KeywordPaletteName,
    NonNormalizedPercentages,
};
pub use fonts::font_platform_data::FontPlatformData;
pub use fonts::font_selection_types::{
    FontSelectionCapabilities, FontSelectionRange, FontSelectionRequest, FontSelectionRequestKey,
    FontSelectionValue,
};
pub use fonts::font_size_adjust::FontSizeAdjust;
pub use fonts::font_smoothing_mode::FontSmoothingMode;
pub use fonts::font_variant_alternates::FontVariantAlternates;
pub use fonts::font_variant_east_asian::{EastAsianForm, EastAsianWidth, FontVariantEastAsian};
pub use fonts::font_variant_emoji::FontVariantEmoji;
pub use fonts::font_variant_numeric::{
    FontVariantNumeric, NumericFigure, NumericFraction, NumericSpacing, Ordinal, SlashedZero,
};
pub use fonts::font_vertical_position_type::FontVerticalPositionType;
pub use fonts::font_width_variant::{kFontWidthVariantWidth, FontWidthVariant};
pub use fonts::glyph::Glyph;
pub use fonts::glyph_metrics_map::{GlyphMetricsMap, GlyphMetricsValue};
pub use fonts::opentype::font_settings::{
    CreateFontFeatureSettings, CreateFontVariationSettings, FontFeature, FontFeatureSettings,
    FontSettings, FontTagValuePair, FontVariationAxis, FontVariationSettings,
};
pub use fonts::orientation_iterator_types::RenderOrientation;
pub use fonts::resolved_font_features::ResolvedFontFeatures;
pub use fonts::shaping::font_features::{
    FontFeatureRange, FontFeatureRanges, FontFeatureRangesSaver, FontFeatureTag, FontFeatureValue,
    HarfBuzzFeature,
};
pub use fonts::shaping::forward::ShapeResultView;
pub use fonts::shaping::forward::{HarfBuzzShaper, ShapeResultSpacing};
pub use fonts::shaping::glyph_bounds_accumulator::GlyphBoundsAccumulator;
pub use fonts::shaping::glyph_data::{
    GlyphOffset, HarfBuzzRunGlyphData, IsSafeToBreak, SafeToBreak,
};
pub use fonts::shaping::glyph_data_range::GlyphDataRange;
pub use fonts::shaping::glyph_index_result::GlyphIndexResult;
pub use fonts::shaping::glyph_offset_iterator::GlyphOffsetIterator;
pub use fonts::shaping::harfbuzz_face::HarfBuzzFace;
pub use fonts::shaping::run_segmenter_types::{RunSegmenter, RunSegmenterRange};
pub use fonts::shaping::shape_options::ShapeOptions;
pub use fonts::shaping::shape_result::{ShapeResult, ShapeResultCharacterData};
pub use fonts::shaping::shape_result_run::ShapeResultRun;
pub use fonts::shaping::shape_result_types::{AdjustMidCluster, BreakGlyphsOption};
pub use fonts::shaping::text_spacing_trim::{
    kTextSpacingTrimBitCount, ShouldTrimAdjacent, ShouldTrimEnd, ShouldTrimStartOfParagraph,
    ShouldTrimStartOfWrappedLine, TextSpacingTrim,
};
pub use fonts::simple_font_data::{GlyphData, SimpleFontData};
pub use fonts::text_fragment_paint_info::TextFragmentPaintInfo;
pub use fonts::text_rendering_mode::TextRenderingMode;
pub use fonts::typesetting_features::{
    kCaps, kKerning, kLigatures, kMaxTypesettingFeatureIndex, TypesettingFeature,
    TypesettingFeatures,
};
pub use fonts::utf16_text_iterator::UTF16TextIterator;

/// Validate decoded bytes entering the font subsystem.
///
/// WOFF/WOFF2 decoding and OTS sanitization happen at DecodeEngine's
/// WebFontDecoder boundary. FontEngine only accepts the resulting OpenType
/// data and verifies that its shaping implementation can open it.
pub fn AcceptDecodedWebFont(bytes: Vec<u8>) -> std::io::Result<std::sync::Arc<[u8]>> {
    if bytes.is_empty() {
        return Err(std::io::Error::other("empty font resource"));
    }
    let validated = std::panic::catch_unwind(|| OpenTypeFont::new(&bytes, 0, 1.0, &[]));
    match validated {
        Ok(_) => Ok(bytes.into()),
        Err(error) => {
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied());
            match message {
                Some(
                    message @ ("invalid OpenType font request"
                    | "invalid OpenType font or face index"
                    | "font contains no glyphs"),
                ) => Err(std::io::Error::other(message.to_owned())),
                Some("HarfBuzz allocation failed") => Err(std::io::Error::other("std::bad_alloc")),
                _ => std::panic::resume_unwind(error),
            }
        }
    }
}
