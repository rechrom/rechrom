//! Pure Rust counterpart of `OpenTypeFont::ShapeRun`.
//!
//! HarfBuzz in the C++ reference receives 16.16 pixel scales. rustybuzz
//! shapes in font units, so positions are converted at that same boundary.

use rustybuzz::ttf_parser::trak::{Track, TrackData};
use rustybuzz::ttf_parser::Tag;
use rustybuzz::{BufferClusterLevel, Direction, Face, Feature, UnicodeBuffer, Variation};

use crate::text_shaper::{ShapedGlyph, ShapedRun, TextDirection};

use super::opentype_font::OpenTypeVariation;

fn direction(value: TextDirection) -> Direction {
    match value {
        TextDirection::kLtr => Direction::LeftToRight,
        TextDirection::kRtl => Direction::RightToLeft,
        TextDirection::kTtb => Direction::TopToBottom,
        TextDirection::kBtt => Direction::BottomToTop,
    }
}

fn pixel_position(font_units: i32, em_scale: i32, units_per_em: i32) -> f64 {
    // hb_ot_font scales each font-unit position to the configured 16.16
    // hb_font_t scale before exposing hb_glyph_position_t.
    (f64::from(font_units) * f64::from(em_scale) / f64::from(units_per_em)).round() / 65536.0
}

// C++ harfbuzz/hb-aat-layout-trak-table.hh:49-179. In particular, a table
// with one track uses that track regardless of its nominal track value.
fn track_value(
    track: &Track<'_>,
    sizes: &rustybuzz::ttf_parser::LazyArray16<'_, rustybuzz::ttf_parser::Fixed>,
    ptem: f32,
) -> f32 {
    let count = sizes.len();
    if count == 0 {
        return 0.0;
    }
    if count == 1 {
        return f32::from(track.values.get(0).unwrap_or(0));
    }
    let mut next = 0;
    while next < count && sizes.get(next).is_some_and(|size| size.0 < ptem) {
        next += 1;
    }
    if next == 0 {
        return f32::from(track.values.get(0).unwrap_or(0));
    }
    if next == count {
        return f32::from(track.values.get(count - 1).unwrap_or(0));
    }
    let mut size1 = sizes.get(next).unwrap().0;
    let mut value1 = f32::from(track.values.get(next).unwrap_or(0));
    if size1 == ptem {
        return value1;
    }
    let mut size0 = sizes.get(next - 1).unwrap().0;
    let mut value0 = f32::from(track.values.get(next - 1).unwrap_or(0));
    if size1 < size0 {
        std::mem::swap(&mut size0, &mut size1);
        std::mem::swap(&mut value0, &mut value1);
    }
    if ptem < size0 {
        return value0;
    }
    if ptem > size1 {
        return value1;
    }
    if size1 == size0 {
        return (value0 + value1) * 0.5;
    }
    value0 + ((ptem - size0) / (size1 - size0)) * (value1 - value0)
}

fn tracking_font_units(data: TrackData<'_>, ptem: f32) -> f32 {
    let count = data.tracks.len();
    if count == 0 {
        return 0.0;
    }
    if count == 1 {
        return track_value(&data.tracks.get(0).unwrap(), &data.sizes, ptem);
    }
    // The source asks for track 0. Find the two nearest track values,
    // retaining HarfBuzz's edge behavior when all values lie on one side.
    let mut low = 0;
    let mut high = count - 1;
    while low + 1 < count
        && data
            .tracks
            .get(low + 1)
            .is_some_and(|track| track.value <= 0.0)
    {
        low += 1;
    }
    while high > 0
        && data
            .tracks
            .get(high - 1)
            .is_some_and(|track| track.value >= 0.0)
    {
        high -= 1;
    }
    let first = data.tracks.get(low).unwrap();
    if low == high {
        return track_value(&first, &data.sizes, ptem);
    }
    let last = data.tracks.get(high).unwrap();
    let value0 = track_value(&first, &data.sizes, ptem);
    let value1 = track_value(&last, &data.sizes, ptem);
    let factor = -first.value / (last.value - first.value);
    value0 + factor * (value1 - value0)
}

// The Blink shaping path passes UTF-16 ranges to hb_buffer_add_utf16. Preserve
// its original code-unit clusters and the pre/post context from the full text.
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
    pub direction: u32,
    pub script: u32,
}

fn scaled_position(font_units: i32, em_scale: i32, units_per_em: i32) -> i32 {
    (f64::from(font_units) * f64::from(em_scale) / f64::from(units_per_em)).round() as i32
}

pub fn shape_utf16(
    bytes: &[u8],
    face_index: u32,
    size: f64,
    variations: &[OpenTypeVariation],
    units: &[u16],
    start: u32,
    end: u32,
    text_direction: TextDirection,
    language: Option<&str>,
    script: Option<&str>,
) -> Utf16Run {
    shape_utf16_with_specified_size(
        bytes,
        face_index,
        size,
        size,
        variations,
        units,
        start,
        end,
        text_direction,
        language,
        script,
    )
}

pub fn shape_utf16_with_specified_size(
    bytes: &[u8],
    face_index: u32,
    size: f64,
    specified_size: f64,
    variations: &[OpenTypeVariation],
    units: &[u16],
    start: u32,
    end: u32,
    text_direction: TextDirection,
    language: Option<&str>,
    script: Option<&str>,
) -> Utf16Run {
    assert!(start <= end && end as usize <= units.len());
    assert!(
        face_index >> 16 == 0,
        "named TTC instances need explicit fvar coordinates"
    );
    let mut face = Face::from_slice(bytes, face_index).expect("invalid OpenType font face");
    let values: Vec<_> = variations
        .iter()
        .map(|axis| Variation {
            tag: Tag(axis.tag),
            value: axis.value,
        })
        .collect();
    face.set_variations(&values);
    face.set_points_per_em(Some(specified_size as f32));

    let mut buffer = UnicodeBuffer::new();
    buffer.set_direction(direction(text_direction));
    buffer.set_cluster_level(BufferClusterLevel::MonotoneCharacters);
    if let Some(language) = language.filter(|value| !value.is_empty()) {
        buffer.set_language(language.parse().expect("invalid HarfBuzz language"));
    }
    if let Some(script) = script.filter(|value| !value.is_empty()) {
        buffer.set_script(script.parse().expect("invalid ISO 15924 script"));
    }
    if start != 0 {
        buffer.set_pre_context(&String::from_utf16_lossy(&units[..start as usize]));
    }
    let mut index = start as usize;
    while index < end as usize {
        let first = units[index];
        let (character, length) = if (0xD800..=0xDBFF).contains(&first)
            && index + 1 < end as usize
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let value = 0x10000
                + ((u32::from(first) - 0xD800) << 10)
                + (u32::from(units[index + 1]) - 0xDC00);
            (char::from_u32(value).unwrap(), 2)
        } else {
            (
                char::from_u32(u32::from(first)).unwrap_or(char::REPLACEMENT_CHARACTER),
                1,
            )
        };
        buffer.add(character, index as u32);
        index += length;
    }
    if (end as usize) < units.len() {
        buffer.set_post_context(&String::from_utf16_lossy(&units[end as usize..]));
    }
    buffer.guess_segment_properties();
    let resolved_direction = buffer.direction();
    // C++ leaves HB_SCRIPT_INVALID (zero) when the range contains only
    // Common/Inherited/Unknown characters. rustybuzz reports UNKNOWN here.
    let resolved_script = if script.is_none() && buffer.script() == rustybuzz::script::UNKNOWN {
        0
    } else {
        buffer.script().tag().0
    };
    let disable_internal_tracking = Feature::new(Tag::from_bytes(b"trak"), 0, ..);
    let shaped = rustybuzz::shape(&face, &[disable_internal_tracking], buffer);
    let scale = (size * 65536.0).round().clamp(0.0, f64::from(i32::MAX)) as i32;
    let upem = face.units_per_em();
    let track_units = face
        .tables()
        .trak
        .filter(|_| face.tables().stat.is_some())
        .map_or(0.0, |trak| {
            if matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl) {
                tracking_font_units(trak.horizontal, specified_size as f32)
            } else {
                tracking_font_units(trak.vertical, specified_size as f32)
            }
        });
    let tracking = (track_units * (scale as f32 / upem as f32)).round() as i32;
    let mut previous_cluster = None;
    let mut glyphs = Vec::with_capacity(shaped.len());
    for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
        let first_in_cluster = previous_cluster != Some(info.cluster);
        let horizontal = matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl);
        glyphs.push(Utf16Glyph {
            id: info.glyph_id,
            cluster: info.cluster,
            unsafe_to_break: info.unsafe_to_break(),
            x_advance: scaled_position(position.x_advance, scale, upem)
                + if horizontal && first_in_cluster {
                    tracking
                } else {
                    0
                },
            y_advance: scaled_position(position.y_advance, scale, upem)
                + if !horizontal && first_in_cluster {
                    tracking
                } else {
                    0
                },
            x_offset: scaled_position(position.x_offset, scale, upem),
            y_offset: scaled_position(position.y_offset, scale, upem),
        });
        previous_cluster = Some(info.cluster);
    }
    Utf16Run {
        glyphs,
        direction: match resolved_direction {
            Direction::LeftToRight => 4,
            Direction::RightToLeft => 5,
            Direction::TopToBottom => 6,
            Direction::BottomToTop => 7,
            _ => unreachable!("resolved HarfBuzz direction"),
        },
        script: resolved_script,
    }
}

pub fn shape_run(
    bytes: &[u8],
    face_index: u32,
    size: f64,
    variations: &[OpenTypeVariation],
    utf8: &str,
    text_direction: TextDirection,
    language: &str,
    script: &str,
) -> ShapedRun {
    shape_run_with_specified_size(
        bytes,
        face_index,
        size,
        size,
        variations,
        utf8,
        text_direction,
        language,
        script,
    )
}

pub fn shape_run_with_specified_size(
    bytes: &[u8],
    face_index: u32,
    size: f64,
    specified_size: f64,
    variations: &[OpenTypeVariation],
    utf8: &str,
    text_direction: TextDirection,
    language: &str,
    script: &str,
) -> ShapedRun {
    assert!(!bytes.is_empty() && size.is_finite() && (0.0..=16_000_000.0).contains(&size));
    assert!(
        face_index >> 16 == 0,
        "named TTC instances need explicit fvar coordinates"
    );
    let mut face = Face::from_slice(bytes, face_index).expect("invalid OpenType font face");
    assert!(face.number_of_glyphs() != 0, "font contains no glyphs");
    let values: Vec<Variation> = variations
        .iter()
        .map(|axis| {
            assert!(
                axis.tag != 0 && axis.value.is_finite(),
                "invalid OpenType variation"
            );
            Variation {
                tag: Tag(axis.tag),
                value: axis.value,
            }
        })
        .collect();
    face.set_variations(&values);
    face.set_points_per_em(Some(specified_size as f32));

    let mut buffer = UnicodeBuffer::new();
    buffer.set_direction(direction(text_direction));
    if !language.is_empty() {
        buffer.set_language(language.parse().expect("invalid HarfBuzz language"));
    }
    if !script.is_empty() {
        buffer.set_script(script.parse().expect("invalid ISO 15924 script"));
    }
    buffer.push_str(utf8);
    buffer.guess_segment_properties();
    // rustybuzz currently chooses only a track with nominal value 0 and also
    // changes glyph offsets. The C++ HarfBuzz source selects/interpolates
    // tracks separately and changes the first glyph's advance per grapheme.
    let disable_internal_tracking = Feature::new(Tag::from_bytes(b"trak"), 0, ..);
    let glyphs = rustybuzz::shape(&face, &[disable_internal_tracking], buffer);
    let scale = (size * 65536.0).round().clamp(0.0, f64::from(i32::MAX)) as i32;
    let upem = face.units_per_em();
    let track_units = face
        .tables()
        .trak
        .filter(|_| face.tables().stat.is_some())
        .map_or(0.0, |trak| {
            if matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl) {
                tracking_font_units(trak.horizontal, specified_size as f32)
            } else {
                tracking_font_units(trak.vertical, specified_size as f32)
            }
        });
    let tracking = ((track_units * (scale as f32 / upem as f32)).round() as i32) as f64 / 65536.0;
    let mut result = ShapedRun {
        glyphs: Vec::with_capacity(glyphs.len()),
        advance: 0.0,
    };
    let mut previous_cluster = None;
    for (info, position) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
        let mut glyph = ShapedGlyph {
            id: info.glyph_id,
            cluster: info.cluster,
            x_advance: pixel_position(position.x_advance, scale, upem),
            y_advance: pixel_position(position.y_advance, scale, upem),
            x_offset: pixel_position(position.x_offset, scale, upem),
            y_offset: pixel_position(position.y_offset, scale, upem),
        };
        if previous_cluster != Some(info.cluster) {
            if matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl) {
                glyph.x_advance += tracking;
            } else {
                glyph.y_advance += tracking;
            }
        }
        previous_cluster = Some(info.cluster);
        result.advance += if matches!(text_direction, TextDirection::kLtr | TextDirection::kRtl) {
            glyph.x_advance
        } else {
            glyph.y_advance
        };
        result.glyphs.push(glyph);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::native::harfbuzz as hb;
    use crate::text::native::harfbuzz_shaper::CreateHarfBuzzShaper;
    use crate::text::native::opentype_font::OpenTypeFont;
    use crate::text_shaper::{ShapeRequest, TextShaper};

    fn compare_with_native(
        path: &str,
        face_index: u32,
        text: &str,
        size: f64,
        script: &str,
        language: &str,
        direction: TextDirection,
        variations: &[OpenTypeVariation],
    ) {
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let native_font = OpenTypeFont::new(&bytes, face_index, size, variations);
        let native = native_font.ShapeRunNativeForTesting(text, direction, language, script);
        let rust = if variations.is_empty() {
            let shaper: Box<dyn TextShaper> = CreateHarfBuzzShaper();
            shaper.Shape(&ShapeRequest {
                font_bytes: &bytes,
                utf8: text,
                font_size: size,
                face_index,
                direction,
                language,
                script,
            })
        } else {
            shape_run(
                &bytes, face_index, size, variations, text, direction, language, script,
            )
        };
        assert_eq!(native.glyphs.len(), rust.glyphs.len());
        for (index, (a, b)) in native.glyphs.iter().zip(&rust.glyphs).enumerate() {
            assert_eq!((a.id, a.cluster), (b.id, b.cluster), "glyph {index}");
            for (name, x, y) in [
                ("x_advance", a.x_advance, b.x_advance),
                ("y_advance", a.y_advance, b.y_advance),
                ("x_offset", a.x_offset, b.x_offset),
                ("y_offset", a.y_offset, b.y_offset),
            ] {
                assert_eq!(x, y, "glyph {index} {name}: native={x}, rust={y}");
            }
        }
        assert_eq!(native.advance, rust.advance, "run advance");
    }

    fn compare_utf16_with_native(
        path: &str,
        face_index: u32,
        text: &str,
        start: u32,
        end: u32,
        direction: TextDirection,
    ) {
        let bytes = std::fs::read(path).expect("reference font must be installed");
        let units: Vec<u16> = text.encode_utf16().collect();
        let font = OpenTypeFont::new(&bytes, face_index, 16.0, &[]);
        let rust = shape_utf16(
            &bytes,
            face_index,
            16.0,
            &[],
            &units,
            start,
            end,
            direction,
            None,
            None,
        );
        let buffer = unsafe { hb::hb_buffer_create() };
        assert!(!buffer.is_null());
        unsafe {
            hb::hb_buffer_set_direction(buffer, direction as i32 + hb::DIRECTION_LTR);
            hb::hb_buffer_set_cluster_level(buffer, 1);
            hb::hb_buffer_add_utf16(
                buffer,
                units.as_ptr(),
                units.len() as i32,
                start,
                (end - start) as i32,
            );
            hb::hb_buffer_guess_segment_properties(buffer);
            assert_ne!(
                hb::hb_shape_full(
                    font.BorrowHarfBuzzFont(),
                    buffer,
                    std::ptr::null(),
                    0,
                    std::ptr::null()
                ),
                0
            );
            let mut count = 0;
            let infos = hb::hb_buffer_get_glyph_infos(buffer, &mut count);
            let positions = hb::hb_buffer_get_glyph_positions(buffer, std::ptr::null_mut());
            let infos = std::slice::from_raw_parts(infos, count as usize);
            let positions = std::slice::from_raw_parts(positions, count as usize);
            assert_eq!(rust.direction, hb::hb_buffer_get_direction(buffer) as u32);
            assert_eq!(rust.script, hb::hb_buffer_get_script(buffer));
            assert_eq!(rust.glyphs.len(), count as usize);
            for (index, ((info, position), glyph)) in
                infos.iter().zip(positions).zip(&rust.glyphs).enumerate()
            {
                assert_eq!(glyph.id, info.codepoint, "glyph {index} id");
                assert_eq!(glyph.cluster, info.cluster, "glyph {index} cluster");
                assert_eq!(
                    glyph.unsafe_to_break,
                    info.mask & 1 != 0,
                    "glyph {index} break flag"
                );
                assert_eq!(
                    (
                        glyph.x_advance,
                        glyph.y_advance,
                        glyph.x_offset,
                        glyph.y_offset
                    ),
                    (
                        position.x_advance,
                        position.y_advance,
                        position.x_offset,
                        position.y_offset
                    ),
                    "glyph {index} position"
                );
            }
            hb::hb_buffer_destroy(buffer);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn utf16_cjk_slice_matches_native_harfbuzz() {
        compare_utf16_with_native(
            "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc",
            16,
            "甲三张照片见证中美关系的刻度乙",
            1,
            15,
            TextDirection::kLtr,
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn utf16_emoji_surrogate_clusters_match_native_harfbuzz() {
        compare_utf16_with_native(
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            0,
            "a👩‍💻b",
            1,
            6,
            TextDirection::kLtr,
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn baidu_cjk_matches_native_harfbuzz() {
        compare_with_native(
            "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc",
            16,
            "三张照片见证中美关系的刻度",
            16.0,
            "Hani",
            "zh",
            TextDirection::kLtr,
            &[],
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn system_font_optical_size_kerning_matches_native_harfbuzz() {
        for text in ["26", "30.6", "256", "23.6", "29.6", "11:35"] {
            compare_with_native(
                "/System/Library/Fonts/SFNS.ttf",
                0,
                text,
                13.0,
                "Latn",
                "en",
                TextDirection::kLtr,
                &[
                    OpenTypeVariation {
                        tag: u32::from_be_bytes(*b"wght"),
                        value: 400.0,
                    },
                    OpenTypeVariation {
                        tag: u32::from_be_bytes(*b"opsz"),
                        value: 13.0,
                    },
                ],
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn latin_matches_native_harfbuzz() {
        compare_with_native(
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            0,
            "About Baidu",
            12.0,
            "Latn",
            "en",
            TextDirection::kLtr,
            &[],
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn arabic_rtl_matches_native_harfbuzz() {
        compare_with_native(
            "/System/Library/Fonts/SFArabic.ttf",
            0,
            "مرحبا بالعالم",
            18.0,
            "Arab",
            "ar",
            TextDirection::kRtl,
            &[],
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn combining_marks_and_ligatures_match_native_harfbuzz() {
        compare_with_native(
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            0,
            "office cafe\u{301}",
            17.5,
            "Latn",
            "en",
            TextDirection::kLtr,
            &[],
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn vertical_cjk_matches_native_harfbuzz() {
        compare_with_native(
            "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc",
            16,
            "三张照片见证中美关系的刻度",
            16.0,
            "Hani",
            "zh",
            TextDirection::kTtb,
            &[],
        );
    }
}
