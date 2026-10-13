#![allow(non_snake_case)]

use std::collections::BTreeMap;

use foundation::blink_geometry::geometry::TextRunLayoutUnit;
use foundation::gfx::PointF;
use foundation::{IsLtr, MakeGarbageCollected, String, TextDirection};

use super::font_features::{FontFeatureRange, FontFeatureTag};
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
use crate::fonts::font_fallback_iterator::FontFallbackIterator;
use crate::fonts::font_fallback_priority::{FontFallbackPriority, HasVSFallbackPriority};
use crate::fonts::font_orientation::{FontOrientation, IsVerticalAnyUpright};
use crate::fonts::font_variant_emoji::FontVariantEmoji;
use crate::fonts::glyph::Glyph;
use crate::fonts::orientation_iterator_types::RenderOrientation;
use crate::fonts::script_run_iterator::ScriptShortName;
use crate::fonts::simple_font_data::SimpleFontData;
use crate::text::native::character::Character;
use crate::text::native::character_break_iterator::CharacterBreakIterator;
use crate::text::native::layout_locale::LayoutLocale;
use crate::text::native::rustybuzz_shaper::{
    shape_utf16_with_specified_size, shape_utf16_with_specified_size_and_features, Utf16Run,
};
use crate::text_shaper::TextDirection as ShapingDirection;

const HB_DIRECTION_LTR: i32 = 4;
const HB_DIRECTION_TTB: i32 = 6;
const USCRIPT_HAN: i32 = 17;
const USCRIPT_SIMPLIFIED_HAN: i32 = 73;
const USCRIPT_TRADITIONAL_HAN: i32 = 74;

// cpp: font_engine/fonts/shaping/harfbuzz_shaper.cc:185-201
fn ApplyFontVariantEmojiOnFallbackPriority(
    priority: FontFallbackPriority,
    variant: FontVariantEmoji,
) -> FontFallbackPriority {
    if !HasVSFallbackPriority(priority) {
        if variant == FontVariantEmoji::kEmojiVariantEmoji {
            return FontFallbackPriority::kEmojiEmoji;
        }
        if variant == FontVariantEmoji::kTextVariantEmoji {
            return FontFallbackPriority::kText;
        }
    }
    priority
}

fn DecodeCodepoint(units: &[u16], index: u32, end: u32) -> (u32, u32) {
    let lead = u32::from(units[index as usize]);
    if (0xd800..=0xdbff).contains(&lead) && index + 1 < end {
        let trail = u32::from(units[index as usize + 1]);
        if (0xdc00..=0xdfff).contains(&trail) {
            return (
                0x10000 + ((lead - 0xd800) << 10) + (trail - 0xdc00),
                index + 2,
            );
        }
    }
    (lead, index + 1)
}

// Chromium revision 78ef49f74feba3d0dde852881feef92bbf86aad7,
// harfbuzz_shaper.cc CollectFallbackHintChars and ShapeSegment. Hints are
// collected from the outstanding reshape ranges, never from covered text.
fn CollectFallbackHints(units: &[u16], ranges: &[(u32, u32)], all: bool) -> Vec<u32> {
    let mut hints = Vec::new();
    for &(start, end) in ranges {
        let mut index = start;
        while index < end {
            let (character, next) = DecodeCodepoint(units, index, end);
            hints.push(character);
            if !all && FontFallbackIterator::HasLikelyScript(character) {
                return hints;
            }
            index = next;
        }
    }
    hints
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VariationSelectorMode {
    Normal,
    Text,
    Emoji,
    Unicode,
    Ignore,
}

impl VariationSelectorMode {
    fn FromVariant(variant: FontVariantEmoji) -> Self {
        match variant {
            FontVariantEmoji::kNormalVariantEmoji => Self::Normal,
            FontVariantEmoji::kTextVariantEmoji => Self::Text,
            FontVariantEmoji::kEmojiVariantEmoji => Self::Emoji,
            FontVariantEmoji::kUnicodeVariantEmoji => Self::Unicode,
        }
    }

    fn Selector(self, base: u32, explicit: Option<u32>) -> Option<u32> {
        if self == Self::Ignore {
            return None;
        }
        if let Some(selector) = explicit
            .filter(|&selector| Character::IsVariationSequence(base as i32, selector as i32))
        {
            return Some(selector);
        }
        if !Character::IsEmoji(base as i32) {
            return None;
        }
        match self {
            Self::Text => Some(0xfe0e),
            Self::Emoji => Some(0xfe0f),
            Self::Unicode => Some(if Character::IsEmojiEmojiDefault(base as i32) {
                0xfe0f
            } else {
                0xfe0e
            }),
            _ => None,
        }
    }
}

fn UnmatchedVariationSequence(
    data: &SimpleFontData,
    units: &[u16],
    start: u32,
    end: u32,
    mode: VariationSelectorMode,
) -> bool {
    if mode == VariationSelectorMode::Ignore {
        return false;
    }
    let font = data.PlatformData().RawFont();
    let mut face = None;
    let mut index = start;
    while index < end {
        let (base, next) = DecodeCodepoint(units, index, end);
        let explicit = (next < end).then(|| DecodeCodepoint(units, next, end).0);
        if let Some(selector) = mode.Selector(base, explicit) {
            // Blink's font callback emits kUnmatchedVSGlyphId only if the
            // base is present but its requested variation is unsupported.
            let face = face.get_or_insert_with(|| {
                ttf_parser::Face::parse(font.SourceBytes(), font.FaceIndex()).expect("valid font")
            });
            if let (Some(base), Some(selector)) = (char::from_u32(base), char::from_u32(selector)) {
                if face.glyph_index(base).is_some()
                    && face.glyph_variation_index(base, selector).is_none()
                {
                    let has_color = [*b"COLR", *b"CBDT", *b"sbix"].iter().any(|tag| {
                        face.raw_face()
                            .table(ttf_parser::Tag::from_bytes(tag))
                            .is_some()
                    });
                    let wrong_presentation = match selector as u32 {
                        0xfe0e => has_color,
                        0xfe0f => !has_color,
                        _ => true,
                    };
                    if wrong_presentation {
                        return true;
                    }
                }
            }
        }
        index = next;
    }
    false
}

#[derive(Clone, Copy)]
struct ClusterResult {
    start: u32,
    end: u32,
    missing: bool,
    unmatched_variation: bool,
}

fn AnalyzeClusters(
    data: &SimpleFontData,
    buffer: &Utf16Run,
    units: &[u16],
    start: u32,
    end: u32,
    mode: VariationSelectorMode,
) -> Vec<ClusterResult> {
    let mut missing_by_cluster = BTreeMap::<u32, bool>::new();
    for glyph in &buffer.glyphs {
        assert!((start..end).contains(&glyph.cluster));
        // Nominal cmap membership is not a shaping result: normalization,
        // composition, and default-ignorable handling may legitimately shape
        // a character for which nominal lookup fails.
        let missing = glyph.id == 0
            || (glyph.id == u32::from(data.SpaceGlyph())
                && units[glyph.cluster as usize] == 0x3000);
        *missing_by_cluster.entry(glyph.cluster).or_default() |= missing;
    }
    let entries: Vec<_> = missing_by_cluster.into_iter().collect();
    let clusters = entries
        .iter()
        .enumerate()
        .map(|(index, &(cluster, mut missing))| {
            let cluster_start = if index == 0 { start } else { cluster };
            let cluster_end = entries.get(index + 1).map_or(end, |entry| entry.0);
            let ranges = &data.PlatformData().unicode_ranges_;
            if !ranges.is_empty() {
                let mut character_index = cluster_start;
                while character_index < cluster_end {
                    let (character, next) = DecodeCodepoint(units, character_index, cluster_end);
                    if !Character::IsDefaultIgnorable(character as i32)
                        && !ranges
                            .iter()
                            .any(|&(start, end)| (start..=end).contains(&character))
                    {
                        missing = true;
                    }
                    character_index = next;
                }
            }
            let unmatched_variation =
                UnmatchedVariationSequence(data, units, cluster_start, cluster_end, mode);
            ClusterResult {
                start: cluster_start,
                end: cluster_end,
                missing: missing || unmatched_variation,
                unmatched_variation,
            }
        })
        .collect::<Vec<_>>();
    // HarfBuzz can give the primary font's invisible ZWJ/selector glyphs
    // separate clusters between missing bases. Keep the entire Unicode
    // grapheme for fallback; committing those invisibles now would destroy
    // emoji ligatures and base/mark shaping in the next face.
    let graphemes = CharacterBreakIterator::from_utf16(&units[start as usize..end as usize]);
    let mut merged: Vec<ClusterResult> = Vec::with_capacity(clusters.len());
    for cluster in clusters {
        if !graphemes.IsBreak((cluster.start - start) as i32) {
            if let Some(previous) = merged.last_mut() {
                previous.end = cluster.end;
                previous.missing |= cluster.missing;
                previous.unmatched_variation |= cluster.unmatched_variation;
                continue;
            }
        }
        merged.push(cluster);
    }
    merged
}

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
        language: &str,
        script: Option<&str>,
        font_features: &[FontFeatureRange],
        variation_mode: VariationSelectorMode,
    ) -> Utf16Run {
        let font = data.PlatformData().RawFont();
        // Chromium prepends face-resolved variant features, then appends the
        // FontDescription features so explicit CSS settings retain priority.
        let mut features =
            Vec::with_capacity(data.PlatformData().ResolvedFeatures().len() + font_features.len());
        features.extend(
            data.PlatformData()
                .ResolvedFeatures()
                .iter()
                .map(|feature| {
                    FontFeatureRange::new(FontFeatureTag::new(feature.tag), feature.value)
                }),
        );
        features.extend_from_slice(font_features);

        // The backend accepts scalar buffers rather than custom cmap callbacks.
        // Materialize forced/ignored selectors only when requested, preserving
        // original UTF-16 cluster indices and CSS feature ranges at the edge.
        let transformed = variation_mode != VariationSelectorMode::Normal;
        let mut shaping_units = Vec::new();
        let mut source_indices = Vec::new();
        let mut boundaries = Vec::new();
        if transformed {
            boundaries.resize(units.len() + 1, 0);
            let mut index = 0;
            while index < units.len() as u32 {
                let (character, next) = DecodeCodepoint(units, index, units.len() as u32);
                for offset in index..next {
                    boundaries[offset as usize] = shaping_units.len() as u32;
                }
                if variation_mode != VariationSelectorMode::Ignore
                    || !Character::IsUnicodeVariationSelector(character as i32)
                {
                    for offset in index..next {
                        shaping_units.push(units[offset as usize]);
                        source_indices.push(index);
                    }
                    let explicit = (next < units.len() as u32)
                        .then(|| DecodeCodepoint(units, next, units.len() as u32).0);
                    if !explicit.is_some_and(|selector| {
                        Character::IsVariationSequence(character as i32, selector as i32)
                    }) {
                        if let Some(selector) = variation_mode.Selector(character, None) {
                            shaping_units.push(selector as u16);
                            source_indices.push(index);
                        }
                    }
                }
                index = next;
            }
            boundaries[units.len()] = shaping_units.len() as u32;
            for feature in &mut features {
                if (feature.start as usize) < boundaries.len() {
                    feature.start = boundaries[feature.start as usize];
                }
                if (feature.end as usize) < boundaries.len() {
                    feature.end = boundaries[feature.end as usize];
                }
            }
        }
        let mut result = shape_utf16_with_specified_size_and_features(
            font.SourceBytes(),
            font.FaceIndex(),
            font.Size(),
            font.SpecifiedSize(),
            &font.Variations(),
            if transformed { &shaping_units } else { units },
            if transformed {
                boundaries[range_start as usize]
            } else {
                range_start
            },
            if transformed {
                boundaries[range_end as usize]
            } else {
                range_end
            },
            ShapingDirectionFromHarfBuzz(hb_direction),
            Some(language),
            script,
            &features,
        );
        if transformed {
            for glyph in &mut result.glyphs {
                glyph.cluster = source_indices[glyph.cluster as usize];
            }
        }
        // rustybuzz can leave an unsupported selector as a separate invisible
        // glyph at MonotoneCharacters. Blink's variation callback instead
        // marks the base glyph unmatched. Keep the base and selector together
        // before extracting failure slices so neither pass splits the pair.
        if units[range_start as usize..range_end as usize]
            .iter()
            .any(|unit| matches!(*unit, 0x180b..=0x180d | 0xfe00..=0xfe0f | 0xdb40))
        {
            let cluster_starts: std::collections::BTreeSet<_> =
                result.glyphs.iter().map(|glyph| glyph.cluster).collect();
            let mut selector_bases = BTreeMap::new();
            let mut index = range_start;
            let mut previous_base = None;
            while index < range_end {
                let (character, next) = DecodeCodepoint(units, index, range_end);
                if Character::IsUnicodeVariationSelector(character as i32) {
                    if let Some(base) = previous_base {
                        if let Some(&cluster) = cluster_starts.range(..=base).next_back() {
                            selector_bases.insert(index, cluster);
                        }
                    }
                } else {
                    previous_base = Some(index);
                }
                index = next;
            }
            for glyph in &mut result.glyphs {
                if let Some(&base) = selector_bases.get(&glyph.cluster) {
                    glyph.cluster = base;
                }
            }
        }
        result
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

    // Chromium ShapeSegment/ExtractShapeResults use one iterator and queue per
    // script/presentation segment. Only missing clusters enter the next cycle.
    fn ShapeWithFallback(
        &self,
        font: &Font,
        priority: FontFallbackPriority,
        range_start: u32,
        range_end: u32,
        units: &[u16],
        hb_direction: i32,
        canvas_rotation: CanvasRotationInVertical,
        language: &str,
        script: Option<&str>,
        font_features: &[FontFeatureRange],
        result: *mut ShapeResult,
    ) {
        let iterator = font.CreateFontFallbackIterator(priority);
        let mode = VariationSelectorMode::FromVariant(font.GetFontDescription().VariantEmoji());
        let mut work = vec![(iterator, vec![(range_start, range_end)], mode, false)];
        while let Some((mut iterator, mut pending, mut mode, mut saw_unmatched_variation)) =
            work.pop()
        {
            while !pending.is_empty() {
                // Explicit families share the ordinary reshape queue. Once the
                // queue reaches contextual system fallback, each disjoint missing
                // slice needs its own provider answer and candidate history.
                if iterator.NeedsSystemFallback() && pending.len() > 1 {
                    for range in pending.into_iter().rev() {
                        work.push((iterator.clone(), vec![range], mode, saw_unmatched_variation));
                    }
                    break;
                }
                let hints = CollectFallbackHints(units, &pending, iterator.NeedsHintList());
                let (query_start, query_end) = pending[0];
                let query_text = &units[query_start as usize..query_end as usize];
                let previous_iterator = (pending.len() > 1).then(|| iterator.clone());
                let Some(candidate) = iterator.NextForText(&hints, query_text) else {
                    break;
                };
                // Next may skip loading/range-inapplicable family entries and
                // enter the system stage in this call. Isolate remaining
                // slices before the selected contextual face can shape them.
                if iterator.NeedsSystemFallback() && pending.len() > 1 {
                    for range in pending.drain(1..).rev() {
                        work.push((
                            previous_iterator.as_ref().unwrap().clone(),
                            vec![range],
                            mode,
                            saw_unmatched_variation,
                        ));
                    }
                }
                let data = unsafe { &*candidate };
                let last = !iterator.HasNext();
                let mut next_pending = Vec::new();
                for (start, end) in pending {
                    let buffer = self.ShapeBuffer(
                        data,
                        units,
                        hb_direction,
                        start,
                        end,
                        language,
                        script,
                        font_features,
                        mode,
                    );
                    if buffer.glyphs.is_empty() {
                        self.Commit(
                            data,
                            start,
                            end,
                            buffer,
                            hb_direction,
                            canvas_rotation,
                            result,
                        );
                        continue;
                    }
                    let clusters = AnalyzeClusters(data, &buffer, units, start, end, mode);
                    saw_unmatched_variation |=
                        clusters.iter().any(|cluster| cluster.unmatched_variation);
                    let terminal =
                        last && (!saw_unmatched_variation || mode == VariationSelectorMode::Ignore);
                    if last && clusters.iter().any(|cluster| cluster.missing) {
                        let text = &units[start as usize..end as usize];
                        let scalars = CollectFallbackHints(text, &[(0, text.len() as u32)], true);
                        if scalars
                            .iter()
                            .any(|&ch| !Character::IsDefaultIgnorable(ch as i32))
                            && !scalars.iter().any(|&ch| {
                                Character::IsPrivateUse(ch as i32)
                                    || Character::IsNonCharacter(ch as i32)
                            })
                        {
                            font.ReportMissing(text, priority);
                        }
                    }
                    if terminal {
                        self.Commit(
                            data,
                            start,
                            end,
                            buffer,
                            hb_direction,
                            canvas_rotation,
                            result,
                        );
                        continue;
                    }
                    let mut index = 0;
                    while index < clusters.len() {
                        let first = clusters[index];
                        let mut limit = index + 1;
                        while limit < clusters.len() && clusters[limit].missing == first.missing {
                            limit += 1;
                        }
                        let segment_end = clusters[limit - 1].end;
                        if first.missing {
                            next_pending.push((first.start, segment_end));
                        } else {
                            // Preserve the original shaped glyphs, positions and
                            // unsafe-to-break flags. Re-shaping this substring
                            // would discard its kerning/ligature/joining context.
                            let slice = Utf16Run {
                                glyphs: buffer
                                    .glyphs
                                    .iter()
                                    .filter(|glyph| {
                                        glyph.cluster >= first.start && glyph.cluster < segment_end
                                    })
                                    .copied()
                                    .collect(),
                                direction: buffer.direction,
                                script: buffer.script,
                            };
                            self.Commit(
                                data,
                                first.start,
                                segment_end,
                                slice,
                                hb_direction,
                                canvas_rotation,
                                result,
                            );
                        }
                        index = limit;
                    }
                }
                pending = next_pending;
                if last
                    && !pending.is_empty()
                    && saw_unmatched_variation
                    && mode != VariationSelectorMode::Ignore
                {
                    iterator.Reset();
                    mode = VariationSelectorMode::Ignore;
                    saw_unmatched_variation = false;
                }
            }
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
        let description = font_ref.GetFontDescription();
        let text = self.text_.clone();
        let units = text.Span16().expect("non-null shape text");
        font_ref.WillUseFontData(&units[start as usize..end as usize]);
        let fallback_priority = ApplyFontVariantEmojiOnFallbackPriority(
            segment.font_fallback_priority,
            description.VariantEmoji(),
        );
        let canvas_rotation = CanvasRotationForRun(
            description.Orientation(),
            segment.render_orientation,
            description,
        );
        let hb_direction = HarfBuzzDirection(description, direction, canvas_rotation);
        let language = description.LocaleOrDefault().Ascii();
        let script = ScriptShortName(segment.script);
        let font_features = font_ref.GetFontFeatures();
        self.ShapeWithFallback(
            font_ref,
            fallback_priority,
            start,
            end,
            units,
            hb_direction,
            canvas_rotation,
            &language,
            script,
            &font_features,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::font::FontFallbackSource;
    use crate::fonts::font_fallback_iterator::tests::face;
    use crate::fonts::font_fallback_metadata::{FontFallbackMetadata, FontFallbackRole};
    use crate::text::native::rustybuzz_shaper::Utf16Glyph;
    use foundation::LayoutHeapScope;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn glyph(id: u32, cluster: u32) -> Utf16Glyph {
        Utf16Glyph {
            id,
            cluster,
            unsafe_to_break: true,
            x_advance: 65536,
            y_advance: 0,
            x_offset: 0,
            y_offset: 0,
        }
    }

    fn shape(font: &Font, text: &str, direction: TextDirection) -> *mut ShapeResult {
        let units: Vec<_> = text.encode_utf16().collect();
        HarfBuzzShaper::new(String::from_utf16(&units)).Shape(font, direction)
    }

    #[test]
    fn invisible_joiners_remain_in_the_missing_emoji_grapheme() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let data = unsafe { &*primary };
        let text: Vec<_> = "👩‍👩‍👧‍👦".encode_utf16().collect();
        let buffer = Utf16Run {
            glyphs: vec![
                glyph(0, 0),
                glyph(u32::from(data.SpaceGlyph()), 2),
                glyph(0, 3),
                glyph(u32::from(data.SpaceGlyph()), 5),
                glyph(0, 6),
                glyph(u32::from(data.SpaceGlyph()), 8),
                glyph(0, 9),
            ],
            direction: HB_DIRECTION_LTR as u32,
            script: 0,
        };
        let clusters = AnalyzeClusters(data, &buffer, &text, 0, 11, VariationSelectorMode::Normal);
        assert_eq!(clusters.len(), 1);
        assert_eq!(
            (clusters[0].start, clusters[0].end, clusters[0].missing),
            (0, 11, true)
        );
        let source = Rc::new(RecordingSource {
            data: primary,
            missing: RefCell::new(vec![]),
        });
        let font = Font::new_with_lazy_fallbacks(
            FontDescription::default(),
            source.clone(),
            primary,
            &[FontFallbackMetadata::default()],
            1,
        );
        shape(&font, "👩‍👩‍👧‍👦", TextDirection::kLtr);
        assert_eq!(source.missing.borrow().as_slice(), &[text]);
    }

    #[test]
    #[ignore = "requires RECHROM_TEST_COLOR_EMOJI_FONT pointing to NotoColorEmoji.ttf"]
    fn installed_color_emoji_keeps_family_zwj_as_one_ligature() {
        use crate::fonts::font_platform_data::FontPlatformData;
        use crate::fonts::text_rendering_mode::TextRenderingMode;
        use crate::text::native::opentype_font::OpenTypeFont;
        use std::sync::Arc;
        let _heap = LayoutHeapScope::new();
        let bytes = std::fs::read(
            std::env::var("RECHROM_TEST_COLOR_EMOJI_FONT").expect("set emoji fixture path"),
        )
        .unwrap();
        let fallback = SimpleFontData::Create(MakeGarbageCollected(FontPlatformData::new(
            Arc::new(OpenTypeFont::new(&bytes, 0, 16.0, &[])),
            None,
            16.0,
            false,
            false,
            TextRenderingMode::kAutoTextRendering,
            vec![],
            FontOrientation::kHorizontal,
            vec![],
        )));
        let primary = face('A' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[primary, fallback],
            primary,
            &[
                FontFallbackMetadata::default(),
                FontFallbackMetadata {
                    role: FontFallbackRole::kEmoji,
                    ..Default::default()
                },
            ],
            1,
        );
        let result = unsafe { &*shape(&font, "👩‍👩‍👧‍👦", TextDirection::kLtr) };
        assert_eq!(result.NumCharacters(), 11);
        assert_eq!(
            result.NumGlyphs(),
            1,
            "complete family must reach emoji GSUB together"
        );
        assert_eq!(
            unsafe { &*result.runs_[0].Get() }.font_data_.Get(),
            fallback
        );
    }

    #[test]
    fn only_missing_clusters_fall_back_in_both_directions() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let fallback = face('B' as u32, vec![]);
        let font = Font::new_with_ordered_fonts(FontDescription::default(), &[primary, fallback]);
        for direction in [TextDirection::kLtr, TextDirection::kRtl] {
            let result = unsafe { &*shape(&font, "ABA", direction) };
            assert_eq!(result.NumCharacters(), 3);
            assert_eq!(result.runs_.len(), 3);
            for run in result.runs_.iter() {
                let run = unsafe { &*run.Get() };
                assert_eq!(
                    run.font_data_.Get().cast_const(),
                    if run.start_index_ == 1 {
                        fallback
                    } else {
                        primary
                    }
                );
                assert!(run.glyph_data_.data_.iter().all(|glyph| glyph.glyph() != 0));
            }
        }
    }

    #[test]
    fn cluster_failure_includes_every_glyph_and_keeps_normalized_success() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let data = unsafe { &*primary };
        let buffer = Utf16Run {
            glyphs: vec![glyph(3, 0), glyph(0, 0), glyph(3, 2)],
            direction: 4,
            script: 0,
        };
        let clusters = AnalyzeClusters(
            data,
            &buffer,
            &[0x41, 0x301, 0x42],
            0,
            3,
            VariationSelectorMode::Normal,
        );
        assert_eq!(
            (clusters[0].start, clusters[0].end, clusters[0].missing),
            (0, 2, true)
        );
        // B has no nominal mapping, but a nonzero shaped output is authoritative.
        assert!(!clusters[1].missing);
    }

    #[test]
    fn subset_coverage_checks_nonleading_characters_in_a_cluster() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![(0x41, 0x41)]);
        let buffer = Utf16Run {
            glyphs: vec![glyph(3, 0)],
            direction: 4,
            script: 0,
        };
        let clusters = AnalyzeClusters(
            unsafe { &*primary },
            &buffer,
            &[0x41, 0x301],
            0,
            2,
            VariationSelectorMode::Normal,
        );
        assert!(clusters[0].missing);
    }

    struct RecordingSource {
        data: *const SimpleFontData,
        missing: RefCell<Vec<Vec<u16>>>,
    }
    impl FontFallbackSource for RecordingSource {
        fn FontDataAt(&self, _: usize) -> *const SimpleFontData {
            self.data
        }
        fn ReportMissing(&self, text: &[u16], _: FontFallbackPriority) {
            self.missing.borrow_mut().push(text.to_vec());
        }
    }

    #[test]
    fn unmatched_variation_restarts_with_base_and_preserves_utf16_indices() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let source = Rc::new(RecordingSource {
            data: primary,
            missing: RefCell::new(vec![]),
        });
        let font = Font::new_with_lazy_fallbacks(
            FontDescription::default(),
            source.clone(),
            primary,
            &[FontFallbackMetadata::default()],
            1,
        );
        let result = unsafe { &*shape(&font, "0\u{fe00}A", TextDirection::kLtr) };
        assert_eq!(result.NumCharacters(), 3);
        assert!(result.runs_.iter().all(|run| unsafe { &*run.Get() }
            .glyph_data_
            .data_
            .iter()
            .all(|glyph| glyph.glyph() != 0)));
        assert_eq!(source.missing.borrow().as_slice(), &[vec![0x30, 0xfe00]]);
    }

    #[test]
    fn supported_variation_glyph_is_kept_on_the_first_pass() {
        let _heap = LayoutHeapScope::new();
        use crate::fonts::font_platform_data::FontPlatformData;
        use crate::fonts::text_rendering_mode::TextRenderingMode;
        use crate::minimal_font::{Build, Put16, Put32};
        use crate::text::native::opentype_font::OpenTypeFont;
        use std::sync::Arc;
        let mut bytes = Build(false, 0, 0x41, false, false);
        let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let old_cmap = parsed
            .raw_face()
            .table(ttf_parser::Tag::from_bytes(b"cmap"))
            .unwrap();
        let base_cmap = old_cmap[12..].to_vec();
        let variation_offset = 20 + base_cmap.len();
        let mut cmap = vec![0; variation_offset + 30];
        Put16(&mut cmap, 2, 2);
        Put16(&mut cmap, 4, 3);
        Put16(&mut cmap, 6, 10);
        Put32(&mut cmap, 8, 20);
        Put16(&mut cmap, 12, 0);
        Put16(&mut cmap, 14, 5);
        Put32(&mut cmap, 16, variation_offset as u32);
        cmap[20..variation_offset].copy_from_slice(&base_cmap);
        Put16(&mut cmap, variation_offset, 14);
        Put32(&mut cmap, variation_offset + 2, 30);
        Put32(&mut cmap, variation_offset + 6, 1);
        cmap[variation_offset + 10..variation_offset + 13].copy_from_slice(&[0, 0xfe, 0]);
        Put32(&mut cmap, variation_offset + 17, 21);
        Put32(&mut cmap, variation_offset + 21, 1);
        cmap[variation_offset + 25..variation_offset + 28].copy_from_slice(&[0, 0, 0x30]);
        Put16(&mut cmap, variation_offset + 28, 3);
        let directory = bytes[12..]
            .chunks_exact(16)
            .position(|entry| &entry[..4] == b"cmap")
            .unwrap()
            * 16
            + 12;
        let new_offset = bytes.len() as u32;
        Put32(&mut bytes, directory + 8, new_offset);
        Put32(&mut bytes, directory + 12, cmap.len() as u32);
        bytes.extend_from_slice(&cmap);
        let raw = Arc::new(OpenTypeFont::new(&bytes, 0, 16.0, &[]));
        let platform = MakeGarbageCollected(FontPlatformData::new(
            raw,
            None,
            16.0,
            false,
            false,
            TextRenderingMode::kAutoTextRendering,
            vec![],
            FontOrientation::kHorizontal,
            vec![],
        ));
        let primary = SimpleFontData::Create(platform);
        let source = Rc::new(RecordingSource {
            data: primary,
            missing: RefCell::new(vec![]),
        });
        let font = Font::new_with_lazy_fallbacks(
            FontDescription::default(),
            source.clone(),
            primary,
            &[FontFallbackMetadata::default()],
            1,
        );
        let result = unsafe { &*shape(&font, "0\u{fe00}", TextDirection::kLtr) };
        assert_eq!(result.NumCharacters(), 2);
        assert_eq!(result.runs_.len(), 1);
        let run = unsafe { &*result.runs_[0].Get() };
        assert_eq!(run.glyph_data_[0].glyph(), 3);
        assert!(source.missing.borrow().is_empty());
    }

    #[test]
    fn explicit_emoji_selector_overrides_font_variant_emoji() {
        assert_eq!(
            VariationSelectorMode::Text.Selector(0x2764, Some(0xfe0f)),
            Some(0xfe0f)
        );
        assert_eq!(
            VariationSelectorMode::Emoji.Selector(0x2764, Some(0xfe0e)),
            Some(0xfe0e)
        );
        assert_eq!(
            VariationSelectorMode::Ignore.Selector(0x2764, Some(0xfe0f)),
            None
        );
        assert_eq!(
            VariationSelectorMode::Normal.Selector(0x41, Some(0xfe0f)),
            None
        );
    }

    #[test]
    fn private_use_and_ignorables_do_not_request_platform_fallback() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let source = Rc::new(RecordingSource {
            data: primary,
            missing: RefCell::new(vec![]),
        });
        let font = Font::new_with_lazy_fallbacks(
            FontDescription::default(),
            source.clone(),
            primary,
            &[FontFallbackMetadata::default()],
            1,
        );
        let _ = shape(&font, "\u{e000}\u{200d}\u{10ffff}", TextDirection::kLtr);
        assert!(source.missing.borrow().is_empty());
    }

    #[test]
    fn supplementary_missing_cluster_is_reported_as_a_surrogate_pair() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let source = Rc::new(RecordingSource {
            data: primary,
            missing: RefCell::new(vec![]),
        });
        let font = Font::new_with_lazy_fallbacks(
            FontDescription::default(),
            source.clone(),
            primary,
            &[FontFallbackMetadata {
                role: FontFallbackRole::kFamilyOnly,
                ..Default::default()
            }],
            1,
        );
        let result = unsafe { &*shape(&font, "\u{1f600}", TextDirection::kLtr) };
        assert_eq!(result.NumCharacters(), 2);
        assert_eq!(source.missing.borrow().as_slice(), &[vec![0xd83d, 0xde00]]);
    }
}
