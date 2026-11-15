//! Text shaping through HarfRust, the HarfBuzz project's Rust port.
//!
//! This is the single shaping engine for `font_engine`. It receives the same
//! inputs Blink gives `hb_shape_full`: a font scaled to 16.16 pixels, the
//! specified CSS size as ptem (for AAT `trak`), an optional platform
//! font-functions override, and a UTF-16 range with its surrounding context.

use harfrust::{
    Buffer, ClusterLevel, Direction, Feature, FontFuncs, Language, Script, ShapeOptions, Tag,
};

use crate::fonts::shaping::font_features::FontFeatureRange;
use crate::text_shaper::{ShapedGlyph, ShapedRun, TextDirection};

use super::opentype_font::OpenTypeFont;

pub(crate) fn direction(value: TextDirection) -> Direction {
    match value {
        TextDirection::kLtr => Direction::LeftToRight,
        TextDirection::kRtl => Direction::RightToLeft,
        TextDirection::kTtb => Direction::TopToBottom,
        TextDirection::kBtt => Direction::BottomToTop,
    }
}

// hb_direction_t values, which the translated Blink shaper stores.
fn hb_direction(value: Direction) -> u32 {
    match value {
        Direction::LeftToRight => 4,
        Direction::RightToLeft => 5,
        Direction::TopToBottom => 6,
        Direction::BottomToTop => 7,
        Direction::Invalid => 0,
    }
}

fn language(value: &str) -> Language {
    value.parse().expect("invalid HarfBuzz language")
}

fn script(value: &str) -> Script {
    value.parse().expect("invalid ISO 15924 script")
}

/// One shaped glyph in HarfBuzz's 16.16 `hb_glyph_position_t` units, with
/// the UTF-16 code-unit cluster of `hb_buffer_add_utf16`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf16Glyph {
    pub id: u32,
    pub cluster: u32,
    pub unsafe_to_break: bool,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Utf16Run {
    pub glyphs: Vec<Utf16Glyph>,
    /// The resolved `hb_direction_t`.
    pub direction: u32,
    /// The resolved `hb_script_t` tag; zero (HB_SCRIPT_INVALID) when the range
    /// holds only Common, Inherited or Unknown characters.
    pub script: u32,
}

fn to_feature(feature: &FontFeatureRange) -> Feature {
    let tag = Tag::from_u32(feature.tag);
    if feature.end == u32::MAX {
        Feature::new(tag, feature.value, feature.start as usize..)
    } else {
        Feature::new(tag, feature.value, feature.start as usize..feature.end as usize)
    }
}

/// Decodes `units[start..end]` as `hb_buffer_add_utf16` does: clusters are
/// code-unit indices into `units`, unpaired surrogates become U+FFFD, and the
/// text outside the range is the pre/post context.
fn add_utf16(buffer: &mut Buffer, units: &[u16], start: usize, end: usize) {
    let decode = |range: &[u16]| -> Vec<u32> {
        char::decode_utf16(range.iter().copied())
            .map(|value| value.unwrap_or(char::REPLACEMENT_CHARACTER) as u32)
            .collect()
    };
    buffer.set_pre_context_codepoints(&decode(&units[..start]));
    let mut index = start;
    while index < end {
        let first = units[index];
        let (codepoint, length) = if (0xD800..=0xDBFF).contains(&first)
            && index + 1 < end
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            (
                0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(units[index + 1]) - 0xDC00),
                2,
            )
        } else if (0xD800..=0xDFFF).contains(&first) {
            (0xFFFD, 1)
        } else {
            (u32::from(first), 1)
        };
        buffer.push(codepoint, index as u32);
        index += length;
    }
    buffer.set_post_context_codepoints(&decode(&units[end..]));
}

/// `hb_shape_full` over a UTF-16 range, with HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS
/// as Blink's HarfBuzzShaper sets it.
///
/// `scale` is the 16.16 pixel scale and `funcs` the optional platform
/// font-functions override (Blink's HarfBuzzFace sub-font).
#[allow(clippy::too_many_arguments)]
pub fn shape_utf16(
    font: &OpenTypeFont,
    scale: i32,
    funcs: Option<&dyn FontFuncs>,
    units: &[u16],
    start: u32,
    end: u32,
    text_direction: TextDirection,
    language_tag: Option<&str>,
    script_tag: Option<&str>,
    features: &[FontFeatureRange],
) -> Utf16Run {
    assert!(start <= end && end as usize <= units.len());
    let mut buffer = Buffer::new();
    buffer.set_direction(direction(text_direction));
    buffer.set_cluster_level(ClusterLevel::MonotoneCharacters);
    if let Some(value) = language_tag.filter(|value| !value.is_empty()) {
        buffer.set_language(Some(language(value)));
    }
    if let Some(value) = script_tag.filter(|value| !value.is_empty()) {
        buffer.set_script(Some(script(value)));
    }
    add_utf16(&mut buffer, units, start as usize, end as usize);
    buffer.guess_segment_properties();
    let resolved_direction = hb_direction(buffer.direction());
    let resolved_script = match buffer.script() {
        Some(value) if value != Script::UNKNOWN => u32::from_be_bytes(value.tag().to_be_bytes()),
        _ => 0,
    };
    let features: Vec<Feature> = features.iter().map(to_feature).collect();
    let mut shaper = font.ShaperFont();
    shaper.set_scale(scale);
    shaper.set_font_funcs(funcs);
    harfrust::shape(
        &shaper,
        &mut buffer,
        ShapeOptions::new()
            .features(&features)
            .point_size(Some(font.SpecifiedSize() as f32)),
    )
    .expect("HarfBuzz shaping failed");
    let glyphs = buffer
        .glyph_infos()
        .iter()
        .zip(buffer.glyph_positions())
        .map(|(info, position)| Utf16Glyph {
            id: info.glyph_id,
            cluster: info.cluster,
            unsafe_to_break: info.unsafe_to_break(),
            x_advance: position.x_advance,
            y_advance: position.y_advance,
            x_offset: position.x_offset,
            y_offset: position.y_offset,
        })
        .collect();
    Utf16Run {
        glyphs,
        direction: resolved_direction,
        script: resolved_script,
    }
}

/// `OpenTypeFont::ShapeRun`: shapes UTF-8 text with the font's own metrics.
pub fn shape_run(
    font: &OpenTypeFont,
    utf8: &str,
    text_direction: TextDirection,
    language_tag: &str,
    script_tag: &str,
) -> ShapedRun {
    let mut buffer = Buffer::new();
    buffer.set_direction(direction(text_direction));
    if !language_tag.is_empty() {
        buffer.set_language(Some(language(language_tag)));
    }
    if !script_tag.is_empty() {
        buffer.set_script(Some(script(script_tag)));
    }
    buffer.push_str(utf8);
    buffer.guess_segment_properties();
    let shaper = font.ShaperFont();
    harfrust::shape(
        &shaper,
        &mut buffer,
        ShapeOptions::new().point_size(Some(font.SpecifiedSize() as f32)),
    )
    .expect("HarfBuzz shaping failed");
    let horizontal = matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl);
    let mut result = ShapedRun {
        glyphs: Vec::with_capacity(buffer.len()),
        advance: 0.0,
    };
    for (info, position) in buffer.glyph_infos().iter().zip(buffer.glyph_positions()) {
        let glyph = ShapedGlyph {
            id: info.glyph_id,
            cluster: info.cluster,
            x_advance: f64::from(position.x_advance) / 65536.0,
            y_advance: f64::from(position.y_advance) / 65536.0,
            x_offset: f64::from(position.x_offset) / 65536.0,
            y_offset: f64::from(position.y_offset) / 65536.0,
        };
        result.advance += if horizontal {
            glyph.x_advance
        } else {
            glyph.y_advance
        };
        result.glyphs.push(glyph);
    }
    result
}
