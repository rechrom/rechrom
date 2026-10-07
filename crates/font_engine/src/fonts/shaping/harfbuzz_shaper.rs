#![allow(non_snake_case)]

use std::collections::BTreeMap;

use foundation::blink_geometry::geometry::TextRunLayoutUnit;
use foundation::gfx::PointF;
use foundation::{IsLtr, MakeGarbageCollected, String, TextDirection};

use super::glyph_data::{GlyphOffset, HarfBuzzRunGlyphData, SafeToBreak};
use super::run_segmenter::RunSegmenter;
use super::run_segmenter_types::RunSegmenterRange;
use super::shape_options::ShapeOptions;
use super::shape_result::ShapeResult;
use super::shape_result_run::ShapeResultRun;
use crate::fonts::canvas_rotation_in_vertical::{
    CanvasRotationInVertical, IsCanvasRotationInVerticalUpright,
};
use crate::fonts::font::Font;
use crate::fonts::font_description::FontDescription;
use crate::fonts::font_orientation::{FontOrientation, IsVerticalAnyUpright};
use crate::fonts::glyph::Glyph;
use crate::fonts::orientation_iterator_types::RenderOrientation;
use crate::fonts::simple_font_data::SimpleFontData;
use crate::text::native::layout_locale::LayoutLocale;
use crate::text::native::rustybuzz_shaper::{shape_utf16_with_specified_size, Utf16Run};
use crate::text_shaper::TextDirection as ShapingDirection;

const HB_DIRECTION_LTR: i32 = 4;
const HB_DIRECTION_TTB: i32 = 6;
const USCRIPT_HAN: i32 = 17;
const USCRIPT_SIMPLIFIED_HAN: i32 = 73;
const USCRIPT_TRADITIONAL_HAN: i32 = 74;

fn ShapingDirectionFromHarfBuzz(value: i32) -> ShapingDirection {
    match value {
        4 => ShapingDirection::kLtr,
        5 => ShapingDirection::kRtl,
        6 => ShapingDirection::kTtb,
        7 => ShapingDirection::kBtt,
        _ => panic!("invalid HarfBuzz direction"),
    }
}

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:32-47
fn CanvasRotationForRun(
    font_orientation: FontOrientation,
    render_orientation: RenderOrientation,
    description: &FontDescription,
) -> CanvasRotationInVertical {
    if font_orientation == FontOrientation::kVerticalUpright
        || (font_orientation == FontOrientation::kVerticalMixed
            && render_orientation == RenderOrientation::kOrientationKeep)
    {
        return if description.IsSyntheticOblique() {
            CanvasRotationInVertical::kRotateCanvasUprightOblique
        } else {
            CanvasRotationInVertical::kRotateCanvasUpright
        };
    }
    if font_orientation == FontOrientation::kVerticalMixed && description.IsSyntheticOblique() {
        return CanvasRotationInVertical::kOblique;
    }
    CanvasRotationInVertical::kRegular
}

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:49-61
fn HarfBuzzDirection(
    description: &FontDescription,
    text_direction: TextDirection,
    canvas_rotation: CanvasRotationInVertical,
) -> i32 {
    let direction = if IsVerticalAnyUpright(description.Orientation())
        && IsCanvasRotationInVerticalUpright(canvas_rotation)
    {
        HB_DIRECTION_TTB
    } else {
        HB_DIRECTION_LTR
    };
    if text_direction == TextDirection::kRtl {
        direction ^ 1
    } else {
        direction
    }
}

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.h:87-104
#[derive(Clone, Copy)]
pub struct GlyphData {
    pub cluster: u32,
    pub glyph: Glyph,
    pub advance: PointF,
    pub offset: PointF,
}

pub type GlyphDataList = Vec<GlyphData>;

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.h:112-125
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackFontStage {
    kIntermediate,
    kLast,
    kIntermediateWithVS,
    kLastWithVS,
    kIntermediateIgnoreVS,
    kLastIgnoreVS,
}

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.h:46-176
pub struct HarfBuzzShaper {
    text_: String,
}

impl HarfBuzzShaper {
    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.h:50-50
    pub fn new(text: String) -> Self {
        Self { text_: text }
    }

    pub fn GetText(&self) -> &String {
        &self.text_
    }

    pub fn TextLength(&self) -> u32 {
        self.text_.length()
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:330-338
    pub fn CheckTextLen(&self, start: u32, length: u32) {
        assert!(start <= self.text_.length());
        assert!(length <= self.text_.length() - start);
    }

    pub fn CheckTextEnd(&self, start: u32, end: u32) {
        assert!(start <= end);
        assert!(end <= self.text_.length());
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:89-109
    fn ShapeBuffer(
        &self,
        data: &SimpleFontData,
        units: &[u16],
        hb_direction: i32,
        range_start: u32,
        range_end: u32,
    ) -> Utf16Run {
        let font = data.PlatformData().RawFont();

        shape_utf16_with_specified_size(
            font.SourceBytes(),
            font.FaceIndex(),
            font.Size(),
            font.SpecifiedSize(),
            &font.Variations(),
            units,
            range_start,
            range_end,
            ShapingDirectionFromHarfBuzz(hb_direction),
            None,
            None,
        )
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:111-160
    fn Commit(
        &self,
        data: &SimpleFontData,
        range_start: u32,
        range_end: u32,
        buffer: Utf16Run,
        hb_direction: i32,
        canvas_rotation: CanvasRotationInVertical,
        result: *mut ShapeResult,
    ) {
        let count = buffer.glyphs.len() as u32;
        let run = MakeGarbageCollected(ShapeResultRun::new(
            data,
            buffer.direction,
            canvas_rotation,
            buffer.script,
            range_start,
            count,
            range_end - range_start,
        ));
        let run_ref = unsafe { &mut *run };
        let result_ref = unsafe { &mut *result };
        let mut total = TextRunLayoutUnit::new();
        for i in 0..count as usize {
            let glyph = &buffer.glyphs[i];
            assert!(glyph.cluster >= range_start);
            let character_index = glyph.cluster - range_start;
            assert!(character_index <= HarfBuzzRunGlyphData::kMaxCharacterIndex);
            let unsafe_to_break = glyph.unsafe_to_break;
            let vertical = (hb_direction & !1) == 6;
            let mut shaped_advance = if vertical {
                -glyph.y_advance as f32
            } else {
                glyph.x_advance as f32
            } / 65536.0;
            if !vertical
                && !data
                    .PlatformData()
                    .RawFont()
                    .UsesOpenTypeHorizontalAdvances()
            {
                shaped_advance += data.WidthForGlyph(glyph.id as Glyph)
                    - data.PlatformData().RawFont().HorizontalAdvance(glyph.id) as f32;
            }
            let advance = TextRunLayoutUnit::FromFloatRound(shaped_advance);
            run_ref.glyph_data_[i as u32] = HarfBuzzRunGlyphData::new(
                glyph.id,
                character_index,
                if unsafe_to_break {
                    SafeToBreak::kUnsafe
                } else {
                    SafeToBreak::kSafe
                },
                advance,
            );
            if glyph.x_offset != 0 || glyph.y_offset != 0 {
                run_ref.glyph_data_.SetOffsetAt(
                    i as u32,
                    GlyphOffset::new(
                        glyph.x_offset as f32 / 65536.0,
                        -glyph.y_offset as f32 / 65536.0,
                    ),
                );
                result_ref.has_vertical_offsets_ |= vertical || glyph.y_offset != 0;
            }
            total += advance;
        }
        run_ref.width_ = total.ToFloat().max(0.0);
        result_ref
            .width_
            .set(result_ref.width_.get() + run_ref.width_);
        result_ref.InsertRun(run);
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:164-226
    fn ShapeWithFallback(
        &self,
        ordered_fonts: &[foundation::Member<SimpleFontData>],
        font_index: usize,
        range_start: u32,
        range_end: u32,
        units: &[u16],
        hb_direction: i32,
        canvas_rotation: CanvasRotationInVertical,
        result: *mut ShapeResult,
    ) {
        let data = unsafe { &*ordered_fonts[font_index].Get() };
        let buffer = self.ShapeBuffer(data, units, hb_direction, range_start, range_end);
        let count = buffer.glyphs.len();
        if font_index + 1 == ordered_fonts.len() || count == 0 {
            self.Commit(
                data,
                range_start,
                range_end,
                buffer,
                hb_direction,
                canvas_rotation,
                result,
            );
            return;
        }

        let mut missing_by_cluster = BTreeMap::<u32, bool>::new();
        for info in &buffer.glyphs {
            let cluster = info.cluster.clamp(range_start, range_end - 1);
            let mut character = u32::from(units[cluster as usize]);
            if (0xD800..=0xDBFF).contains(&character) && cluster + 1 < range_end {
                let trail = u32::from(units[cluster as usize + 1]);
                if (0xDC00..=0xDFFF).contains(&trail) {
                    character = 0x10000 + ((character - 0xD800) << 10) + (trail - 0xDC00);
                }
            }
            let missing = info.id == 0 || !data.PlatformData().FontContainsCharacter(character);
            *missing_by_cluster.entry(cluster).or_default() |= missing;
        }
        let mut has_missing = false;
        for &missing in missing_by_cluster.values() {
            has_missing |= missing;
        }
        if !has_missing {
            self.Commit(
                data,
                range_start,
                range_end,
                buffer,
                hb_direction,
                canvas_rotation,
                result,
            );
            return;
        }

        let mut clusters: Vec<(u32, bool)> = missing_by_cluster.into_iter().collect();
        if clusters[0].0 != range_start {
            clusters.insert(0, (range_start, clusters[0].1));
        }
        let mut i = 0;
        while i < clusters.len() {
            let missing = clusters[i].1;
            let segment_start = clusters[i].0;
            let mut next = i + 1;
            while next < clusters.len() && clusters[next].1 == missing {
                next += 1;
            }
            let segment_end = if next < clusters.len() {
                clusters[next].0
            } else {
                range_end
            };
            if missing {
                self.ShapeWithFallback(
                    ordered_fonts,
                    font_index + 1,
                    segment_start,
                    segment_end,
                    units,
                    hb_direction,
                    canvas_rotation,
                    result,
                );
            } else {
                self.Commit(
                    data,
                    segment_start,
                    segment_end,
                    self.ShapeBuffer(data, units, hb_direction, segment_start, segment_end),
                    hb_direction,
                    canvas_rotation,
                    result,
                );
            }
            i = next;
        }
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:64-228
    fn ShapeRange(
        &self,
        font: *const Font,
        direction: TextDirection,
        start: u32,
        end: u32,
        segment: &RunSegmenterRange,
    ) -> *mut ShapeResult {
        assert!(!font.is_null());
        assert!(start <= end);
        assert!(end <= self.text_.length());
        let result = MakeGarbageCollected(ShapeResult::new(start, end - start, direction));
        if start == end {
            return result;
        }
        let font_ref = unsafe { &*font };
        let ordered_fonts = font_ref.OrderedFontData();
        assert!(!ordered_fonts.is_empty());
        let description = font_ref.GetFontDescription();
        let canvas_rotation = CanvasRotationForRun(
            description.Orientation(),
            segment.render_orientation,
            description,
        );
        let hb_direction = HarfBuzzDirection(description, direction, canvas_rotation);
        let text = self.text_.clone();
        let units = text.Span16().expect("non-null shape text");
        self.ShapeWithFallback(
            ordered_fonts,
            0,
            start,
            end,
            units,
            hb_direction,
            canvas_rotation,
            result,
        );
        result
    }

    // C++ overload Shape(font, direction, start, end).
    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:230-249
    pub fn ShapeRangeWithSegmentation(
        &self,
        font: *const Font,
        direction: TextDirection,
        start: u32,
        end: u32,
    ) -> *mut ShapeResult {
        assert!(!font.is_null());
        let text = self.text_.clone();
        let mut segmenter = RunSegmenter::new(
            text.Span16().expect("non-null shape text"),
            unsafe { &*font }.GetFontDescription().Orientation(),
        );
        let mut ranges = Vec::new();
        let mut range = RunSegmenterRange::default();
        while segmenter.Consume(&mut range) {
            if range.end <= start || range.start >= end {
                continue;
            }
            range.start = range.start.max(start);
            range.end = range.end.min(end);
            ranges.push(range);
        }
        self.ShapeWithRanges(
            font,
            direction,
            start,
            end,
            &ranges,
            ShapeOptions::default(),
        )
    }

    // C++ overload Shape(font, direction).
    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:251-254
    pub fn Shape(&self, font: *const Font, direction: TextDirection) -> *mut ShapeResult {
        self.ShapeRangeWithSegmentation(font, direction, 0, self.text_.length())
    }

    // C++ overload Shape(font, direction, start, end, ranges, options).
    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:256-275
    pub fn ShapeWithRanges(
        &self,
        font: *const Font,
        direction: TextDirection,
        start: u32,
        end: u32,
        ranges: &[RunSegmenterRange],
        _options: ShapeOptions,
    ) -> *mut ShapeResult {
        assert!(!font.is_null());
        self.CheckTextEnd(start, end);
        let result = MakeGarbageCollected(ShapeResult::new(start, 0, direction));
        for &source_range in ranges {
            let range_start = start.max(source_range.start);
            let range_end = end.min(source_range.end);
            if range_start >= range_end {
                continue;
            }
            let mut range = source_range;
            range.start = range_start;
            range.end = range_end;
            let part = self.ShapeRange(font, direction, range_start, range_end, &range);
            unsafe { &*part }.CopyRange(range_start, range_end, unsafe { &mut *result });
        }
        result
    }

    // C++ overload Shape(font, direction, start, end, range, options).
    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:277-285
    pub fn ShapeSingleRange(
        &self,
        font: *const Font,
        direction: TextDirection,
        start: u32,
        end: u32,
        range: RunSegmenterRange,
        _options: ShapeOptions,
    ) -> *mut ShapeResult {
        self.ShapeRange(font, direction, start, end, &range)
    }

    // cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:287-328
    pub fn GetGlyphData(
        &self,
        data: &SimpleFontData,
        locale: &LayoutLocale,
        script: i32,
        is_horizontal: bool,
        direction: TextDirection,
        glyphs: &mut Vec<GlyphData>,
    ) {
        glyphs.clear();
        let text = self.text_.clone();
        let units = text.Span16().expect("non-null shape text");
        let font = data.PlatformData().RawFont();
        let language = locale.Ascii();
        let buffer = shape_utf16_with_specified_size(
            font.SourceBytes(),
            font.FaceIndex(),
            font.Size(),
            font.SpecifiedSize(),
            &font.Variations(),
            units,
            0,
            units.len() as u32,
            if is_horizontal {
                if IsLtr(direction) {
                    ShapingDirection::kLtr
                } else {
                    ShapingDirection::kRtl
                }
            } else {
                ShapingDirection::kTtb
            },
            Some(&language),
            if script == USCRIPT_HAN
                || script == USCRIPT_SIMPLIFIED_HAN
                || script == USCRIPT_TRADITIONAL_HAN
            {
                Some("Hani")
            } else {
                None
            },
        );
        for info in &buffer.glyphs {
            glyphs.push(GlyphData {
                cluster: info.cluster,
                glyph: info.id as Glyph,
                advance: PointF::new(
                    info.x_advance as f32 / 65536.0,
                    -info.y_advance as f32 / 65536.0,
                ),
                offset: PointF::new(
                    info.x_offset as f32 / 65536.0,
                    -info.y_offset as f32 / 65536.0,
                ),
            });
        }
    }
}
