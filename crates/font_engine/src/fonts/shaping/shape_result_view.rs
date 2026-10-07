#![allow(non_snake_case)]

use super::forward::GlyphCallback;
use super::glyph_bounds_accumulator::GlyphBoundsAccumulator;
use super::glyph_data::HarfBuzzRunGlyphData;
use super::glyph_data_range::GlyphDataRange;
use super::glyph_offset_iterator::GlyphOffsetIterator;
use super::shape_result::ShapeResult;
use super::shape_result_run::ShapeResultRun;
use crate::fonts::glyph::Glyph;
use crate::fonts::simple_font_data::SimpleFontData;
use foundation::blink_geometry::geometry::InlineLayoutUnit;
use foundation::gfx::RectF;
use foundation::{
    HeapHashSet, HeapVector, IsLtr, IsRtl, LayoutUnit, MakeGarbageCollected, Member, TextDirection,
    Traceable, Visitor,
};
use std::ffi::c_void;

// cpp: font_engine/fonts/shaping/shape_result_view.h:77-107
#[derive(Clone, Copy)]
pub struct Segment {
    pub result: *const ShapeResult,
    pub view: *const ShapeResultView,
    pub start_index: u32,
    pub end_index: u32,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            result: std::ptr::null(),
            view: std::ptr::null(),
            start_index: 0,
            end_index: 0,
        }
    }
}

impl Segment {
    pub fn from_result(result: *const ShapeResult, start_index: u32, end_index: u32) -> Self {
        Self {
            result,
            view: std::ptr::null(),
            start_index,
            end_index,
        }
    }
    pub fn from_view(view: *const ShapeResultView, start_index: u32, end_index: u32) -> Self {
        Self {
            result: std::ptr::null(),
            view,
            start_index,
            end_index,
        }
    }
}

// cpp: font_engine/fonts/shaping/shape_result_view.h:161-263
#[derive(Clone, Copy)]
pub struct RunInfoPart {
    pub(crate) range_: GlyphDataRange,
    pub(crate) start_index_: u32,
    pub(crate) offset_: u32,
    pub(crate) num_characters_: u32,
    pub(crate) width_: f32,
}

impl RunInfoPart {
    // cpp: font_engine/fonts/shaping/shape_result_view.cc:16-25
    pub fn new(
        range: GlyphDataRange,
        start_index: u32,
        offset: u32,
        num_characters: u32,
        width: f32,
    ) -> Self {
        Self {
            range_: range,
            start_index_: start_index,
            offset_: offset,
            num_characters_: num_characters,
            width_: width,
        }
    }

    pub fn Glyphs(&self) -> &[HarfBuzzRunGlyphData] {
        self.range_.Glyphs()
    }
    pub fn begin(&self) -> *const HarfBuzzRunGlyphData {
        self.range_.begin()
    }
    pub fn end(&self) -> *const HarfBuzzRunGlyphData {
        self.range_.end()
    }
    pub fn rbegin(&self) -> std::iter::Rev<std::slice::Iter<'_, HarfBuzzRunGlyphData>> {
        self.Glyphs().iter().rev()
    }
    pub fn rend(&self) -> std::iter::Rev<std::slice::Iter<'_, HarfBuzzRunGlyphData>> {
        self.Glyphs()[0..0].iter().rev()
    }
    pub fn GlyphAt(&self, index: u32) -> &HarfBuzzRunGlyphData {
        &self.Glyphs()[index as usize]
    }
    pub fn GetGlyphOffsets<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
    ) -> GlyphOffsetIterator<'_, HAS_NON_ZERO_GLYPH_OFFSETS> {
        GlyphOffsetIterator::from_range(&self.range_)
    }
    pub fn HasGlyphOffsets(&self) -> bool {
        self.range_.HasOffsets()
    }
    pub fn CharacterIndexOfEndGlyph(&self) -> u32 {
        self.num_characters_.wrapping_add(self.offset_)
    }
    pub fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }
    pub fn NumGlyphs(&self) -> u32 {
        self.range_.size()
    }
    pub fn Width(&self) -> f32 {
        self.width_
    }
    pub fn GetRunInfo(&self) -> *const ShapeResultRun {
        self.range_.GetRun()
    }
    pub fn GetGlyphDataRange(&self) -> &GlyphDataRange {
        &self.range_
    }
    pub fn OffsetToRunStartIndex(&self) -> u32 {
        self.offset_
    }
    pub fn Get(&self) -> &Self {
        self
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:31-47
    pub fn PreviousSafeToBreakOffset(&self, mut offset: u32) -> u32 {
        if offset >= self.NumCharacters() {
            return self.NumCharacters();
        }
        offset = offset.wrapping_add(self.offset_);
        let run = unsafe { &*self.GetRunInfo() };
        if run.IsLtr() {
            for glyph in self.Glyphs().iter().rev() {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() <= offset {
                    return glyph.character_index().wrapping_sub(self.offset_);
                }
            }
        } else {
            for glyph in self.Glyphs() {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() <= offset {
                    return glyph.character_index().wrapping_sub(self.offset_);
                }
            }
        }
        0
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:49-52
    pub fn FindGlyphDataRange(&self, start: u32, end: u32) -> GlyphDataRange {
        self.range_
            .FindGlyphDataRange(unsafe { &*self.GetRunInfo() }.IsRtl(), start, end)
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.h:218-249
    fn ComputeStart<R: ViewRun>(run: &R, start_index_offset_for_run: u32, is_ltr: bool) -> u32 {
        let part_start = run.start_index().wrapping_add(start_index_offset_for_run);
        if is_ltr {
            part_start
        } else {
            part_start.max(run.OffsetToRunStartIndex())
        }
    }

    fn ComputeStartEnd<R: ViewRun>(
        run: &R,
        start_index_offset_for_run: u32,
        is_ltr: bool,
        segment: &Segment,
    ) -> Option<(u32, u32)> {
        if run.GetRunInfo().is_null() {
            return None;
        }
        let part_start = Self::ComputeStart(run, start_index_offset_for_run, is_ltr);
        if segment.end_index <= part_start {
            return None;
        }
        if run.NumCharacters() == 0 {
            return Some((part_start, part_start));
        }
        let part_end = part_start.wrapping_add(run.NumCharacters());
        if segment.start_index >= part_end {
            return None;
        }
        Some((part_start, part_end))
    }
}

// cpp: font_engine/fonts/shaping/shape_result_view.cc:27-29
impl Traceable for RunInfoPart {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.range_);
    }
}

trait ViewRun {
    fn start_index(&self) -> u32;
    fn NumCharacters(&self) -> u32;
    fn Width(&self) -> f32;
    fn GetRunInfo(&self) -> *const ShapeResultRun;
    fn GetGlyphDataRange(&self) -> GlyphDataRange;
    fn FindGlyphDataRange(&self, start: u32, end: u32) -> GlyphDataRange;
    fn OffsetToRunStartIndex(&self) -> u32;
}

impl ViewRun for ShapeResultRun {
    fn start_index(&self) -> u32 {
        self.start_index_
    }
    fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }
    fn Width(&self) -> f32 {
        self.width_
    }
    fn GetRunInfo(&self) -> *const ShapeResultRun {
        self
    }
    fn GetGlyphDataRange(&self) -> GlyphDataRange {
        self.GetGlyphDataRange()
    }
    fn FindGlyphDataRange(&self, start: u32, end: u32) -> GlyphDataRange {
        self.FindGlyphDataRange(start, end)
    }
    fn OffsetToRunStartIndex(&self) -> u32 {
        0
    }
}

impl ViewRun for RunInfoPart {
    fn start_index(&self) -> u32 {
        self.start_index_
    }
    fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }
    fn Width(&self) -> f32 {
        self.width_
    }
    fn GetRunInfo(&self) -> *const ShapeResultRun {
        self.GetRunInfo()
    }
    fn GetGlyphDataRange(&self) -> GlyphDataRange {
        self.range_
    }
    fn FindGlyphDataRange(&self, start: u32, end: u32) -> GlyphDataRange {
        self.FindGlyphDataRange(start, end)
    }
    fn OffsetToRunStartIndex(&self) -> u32 {
        self.offset_
    }
}

// cpp: font_engine/fonts/shaping/shape_result_view.h:83-85,303-324
pub struct ShapeResultView {
    pub(crate) parts_: HeapVector<RunInfoPart, 1>,
    start_index_: u32,
    width_: f32,
    num_characters_: u32,
    direction_: TextDirection,
    has_vertical_offsets_: bool,
    char_index_offset_: u32,
}

impl Traceable for ShapeResultView {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.parts_);
    }
}

// cpp: font_engine/fonts/shaping/shape_result_view.cc:59-118
#[derive(Clone, Copy)]
struct InitData {
    start_index: u32,
    char_index_offset: u32,
    direction: TextDirection,
    has_vertical_offsets: bool,
    num_parts: u32,
}

impl Default for InitData {
    fn default() -> Self {
        Self {
            start_index: 0,
            char_index_offset: 0,
            direction: TextDirection::kLtr,
            has_vertical_offsets: false,
            num_parts: 0,
        }
    }
}

impl InitData {
    fn PopulateResult(&mut self, result: &ShapeResult) {
        self.direction = result.Direction();
        if self.IsLtr() {
            self.char_index_offset = result.StartIndex();
        } else {
            self.start_index = result.StartIndex();
        }
        self.has_vertical_offsets = result.HasVerticalOffsets();
        self.num_parts = result.RunsOrParts().size();
    }

    fn InitializeDirection(&mut self, direction: TextDirection, start_index: u32) {
        self.direction = direction;
        if self.IsLtr() {
            self.char_index_offset = start_index;
        } else {
            self.start_index = start_index;
        }
    }

    fn CountResult(result: &ShapeResult, segment: &Segment) -> u32 {
        let mut count = 0;
        for entry in result.RunsOrParts() {
            let run = unsafe { &*entry.Get() };
            if RunInfoPart::ComputeStartEnd(
                run,
                result.StartIndexOffsetForRun(),
                result.IsLtr(),
                segment,
            )
            .is_some()
            {
                count += 1;
            }
        }
        count
    }

    fn CountView(view: &ShapeResultView, segment: &Segment) -> u32 {
        let mut count = 0;
        for run in &view.parts_ {
            if RunInfoPart::ComputeStartEnd(
                run,
                view.StartIndexOffsetForRun(),
                view.IsLtr(),
                segment,
            )
            .is_some()
            {
                count += 1;
            }
        }
        count
    }

    fn PopulateSegments(&mut self, segments: &[Segment]) {
        assert!(!segments.is_empty());
        let first = &segments[0];
        if !first.result.is_null() {
            let result = unsafe { &*first.result };
            self.InitializeDirection(result.Direction(), result.StartIndex());
        } else {
            let view = unsafe { &*first.view };
            self.InitializeDirection(view.Direction(), view.StartIndex());
        }
        if self.IsLtr() {
            self.char_index_offset = self.char_index_offset.max(first.start_index);
        } else {
            self.start_index = self.start_index.max(first.start_index);
        }
        for segment in segments {
            if !segment.result.is_null() {
                let result = unsafe { &*segment.result };
                assert_eq!(result.Direction(), self.direction);
                self.has_vertical_offsets |= result.HasVerticalOffsets();
                self.num_parts += Self::CountResult(result, segment);
            } else {
                assert!(!segment.view.is_null());
                let view = unsafe { &*segment.view };
                assert_eq!(view.Direction(), self.direction);
                self.has_vertical_offsets |= view.HasVerticalOffsets();
                self.num_parts += Self::CountView(view, segment);
            }
        }
    }

    fn IsLtr(&self) -> bool {
        IsLtr(self.direction)
    }
}

#[allow(non_snake_case)]
impl ShapeResultView {
    // cpp: font_engine/fonts/shaping/shape_result_view.cc:54-57
    fn CharacterIndexOffsetForGlyphData(&self, part: &RunInfoPart) -> u32 {
        part.start_index_
            .wrapping_add(self.char_index_offset_)
            .wrapping_sub(part.offset_)
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:120-124
    fn new(data: &InitData) -> Self {
        Self {
            parts_: HeapVector::new(),
            start_index_: data.start_index,
            width_: 0.0,
            num_characters_: 0,
            direction_: data.direction,
            has_vertical_offsets_: data.has_vertical_offsets,
            char_index_offset_: data.char_index_offset,
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.h:124-148
    pub fn StartIndex(&self) -> u32 {
        self.start_index_.wrapping_add(self.char_index_offset_)
    }
    pub fn EndIndex(&self) -> u32 {
        self.StartIndex().wrapping_add(self.num_characters_)
    }
    pub fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }
    pub fn Width(&self) -> f32 {
        self.width_
    }
    pub fn SnappedWidth(&self) -> LayoutUnit {
        LayoutUnit::FromFloatCeil(self.width_)
    }
    pub fn Direction(&self) -> TextDirection {
        self.direction_
    }
    pub fn IsLtr(&self) -> bool {
        IsLtr(self.direction_)
    }
    pub fn IsRtl(&self) -> bool {
        IsRtl(self.direction_)
    }
    pub fn HasVerticalOffsets(&self) -> bool {
        self.has_vertical_offsets_
    }
    pub fn RunsOrParts(&self) -> &[RunInfoPart] {
        &self.parts_
    }
    pub fn StartIndexOffsetForRun(&self) -> u32 {
        self.char_index_offset_
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:126-163
    fn PopulateRun<R: ViewRun>(
        &mut self,
        run: &R,
        source_offset: u32,
        source_ltr: bool,
        segment: &Segment,
        index_diff: u32,
    ) {
        let Some((part_start, part_end)) =
            RunInfoPart::ComputeStartEnd(run, source_offset, source_ltr, segment)
        else {
            return;
        };
        let run_start = part_start.wrapping_sub(run.OffsetToRunStartIndex());
        let range_start = if segment.start_index > run_start {
            segment.start_index.max(part_start).wrapping_sub(run_start)
        } else {
            0
        };
        let range_end = segment.end_index.min(part_end).wrapping_sub(run_start);
        assert!(range_end > range_start);
        let (range, part_width) =
            if part_start >= segment.start_index && part_end <= segment.end_index {
                (run.GetGlyphDataRange(), run.Width())
            } else {
                let range = run.FindGlyphDataRange(range_start, range_end);
                let mut width = InlineLayoutUnit::new();
                for glyph in range.Glyphs() {
                    width += glyph.advance.To::<16, i64>();
                }
                (range, width.ToFloat())
            };
        self.width_ += part_width;
        self.parts_.emplace_back(RunInfoPart::new(
            range,
            run_start.wrapping_add(range_start).wrapping_add(index_diff),
            range_start,
            range_end - range_start,
            part_width,
        ));
    }

    fn PopulateRunInfoPartsResult(&mut self, result: &ShapeResult, segment: &Segment) {
        let index_diff = self
            .start_index_
            .wrapping_add(self.num_characters_)
            .wrapping_sub(segment.start_index.max(result.StartIndex()));
        let clipped_start = segment.start_index.max(result.StartIndex());
        let clipped_end = segment.end_index.min(result.EndIndex());
        if clipped_end <= clipped_start {
            return;
        }
        self.num_characters_ = self
            .num_characters_
            .wrapping_add(clipped_end - clipped_start)
            & ((1 << 30) - 1);
        for entry in result.RunsOrParts() {
            self.PopulateRun(
                unsafe { &*entry.Get() },
                result.StartIndexOffsetForRun(),
                result.IsLtr(),
                segment,
                index_diff,
            );
        }
    }

    fn PopulateRunInfoPartsView(&mut self, view: &ShapeResultView, segment: &Segment) {
        let index_diff = self
            .start_index_
            .wrapping_add(self.num_characters_)
            .wrapping_sub(segment.start_index.max(view.StartIndex()));
        let clipped_start = segment.start_index.max(view.StartIndex());
        let clipped_end = segment.end_index.min(view.EndIndex());
        if clipped_end <= clipped_start {
            return;
        }
        self.num_characters_ = self
            .num_characters_
            .wrapping_add(clipped_end - clipped_start)
            & ((1 << 30) - 1);
        for run in &view.parts_ {
            self.PopulateRun(
                run,
                view.StartIndexOffsetForRun(),
                view.IsLtr(),
                segment,
                index_diff,
            );
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:165-170
    fn PopulateRunInfoParts(&mut self, segment: &Segment) {
        if !segment.result.is_null() {
            self.PopulateRunInfoPartsResult(unsafe { &*segment.result }, segment);
        } else {
            self.PopulateRunInfoPartsView(unsafe { &*segment.view }, segment);
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:172-204
    pub fn Create(segments: &[Segment]) -> *mut Self {
        assert!(!segments.is_empty());
        let mut data = InitData::default();
        data.PopulateSegments(segments);
        let output = MakeGarbageCollected(Self::new(&data));
        let output_ref = unsafe { &mut *output };
        output_ref.parts_.ReserveInitialCapacity(data.num_parts);
        if output_ref.IsLtr() {
            for segment in segments {
                output_ref.PopulateRunInfoParts(segment);
            }
        } else {
            for segment in segments.iter().rev() {
                output_ref.PopulateRunInfoParts(segment);
            }
        }
        output
    }

    pub fn CreateFromResultRange(result: *const ShapeResult, start: u32, end: u32) -> *mut Self {
        Self::Create(&[Segment::from_result(result, start, end)])
    }
    pub fn CreateFromViewRange(view: *const ShapeResultView, start: u32, end: u32) -> *mut Self {
        Self::Create(&[Segment::from_view(view, start, end)])
    }
    pub fn CreateFromResult(result: *const ShapeResult) -> *mut Self {
        assert!(!result.is_null());
        let result_ref = unsafe { &*result };
        Self::CreateFromResultRange(result, result_ref.StartIndex(), result_ref.EndIndex())
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:206-225
    pub fn CreateShapeResult(&self) -> *mut ShapeResult {
        let result = MakeGarbageCollected(ShapeResult::new(
            self.StartIndex(),
            self.num_characters_,
            self.Direction(),
        ));
        let result_ref = unsafe { &mut *result };
        result_ref.runs_.ReserveInitialCapacity(self.parts_.size());
        for part in &self.parts_ {
            let source = unsafe { &*part.GetRunInfo() };
            let run = MakeGarbageCollected(ShapeResultRun::new(
                source.font_data_.Get(),
                source.HbDirection(),
                source.canvas_rotation_,
                source.script_,
                part.start_index_,
                part.NumGlyphs(),
                part.num_characters_,
            ));
            let run_ref = unsafe { &mut *run };
            run_ref.glyph_data_.CopyFromRange(&part.range_);
            for glyph in &mut run_ref.glyph_data_.data_ {
                glyph.SetCharacterIndex(glyph.character_index().wrapping_sub(part.offset_));
            }
            run_ref.start_index_ = run_ref.start_index_.wrapping_add(self.char_index_offset_);
            run_ref.width_ = part.width_;
            result_ref.runs_.push_back(Member::from_ptr(run));
        }
        result_ref.has_vertical_offsets_ = self.has_vertical_offsets_;
        result_ref.width_.set(self.width_);
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:227-239
    pub fn PreviousSafeToBreakOffset(&self, index: u32) -> u32 {
        for part in self.parts_.iter().rev() {
            let run_start = part.start_index_.wrapping_add(self.char_index_offset_);
            if index >= run_start {
                let offset = index - run_start;
                if offset <= part.num_characters_ {
                    return part
                        .PreviousSafeToBreakOffset(offset)
                        .wrapping_add(run_start);
                }
                if self.IsLtr() {
                    return run_start.wrapping_add(part.num_characters_);
                }
            }
        }
        self.StartIndex()
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:241-253
    pub fn NumGlyphs(&self) -> u32 {
        let mut count = 0u32;
        for part in &self.parts_ {
            count = count.wrapping_add(part.NumGlyphs());
        }
        count
    }

    pub fn UsedFonts(&self) -> HeapHashSet<Member<SimpleFontData>> {
        let mut fonts = HeapHashSet::new();
        for part in &self.parts_ {
            let font = unsafe { &*part.GetRunInfo() }.font_data_.Get();
            if !font.is_null() {
                fonts.insert(Member::from_ptr(font));
            }
        }
        fonts
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:255-261
    fn ComputePartInkBounds<const IS_HORIZONTAL_RUN: bool, const HAS_GLYPH_OFFSETS: bool>(
        &self,
        part: &RunInfoPart,
        run_advance: f32,
        ink_bounds: &mut RectF,
    ) {
        self.ComputePartInkBoundsScalar::<IS_HORIZONTAL_RUN, HAS_GLYPH_OFFSETS>(
            part,
            run_advance,
            ink_bounds,
        );
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:263-279
    fn ComputePartInkBoundsScalar<const IS_HORIZONTAL_RUN: bool, const HAS_GLYPH_OFFSETS: bool>(
        &self,
        part: &RunInfoPart,
        run_advance: f32,
        ink_bounds: &mut RectF,
    ) {
        let mut offsets = part.GetGlyphOffsets::<HAS_GLYPH_OFFSETS>();
        let font_data = unsafe { &*(*part.GetRunInfo()).font_data_.Get() };
        let mut bounds = GlyphBoundsAccumulator::<IS_HORIZONTAL_RUN>::default();
        let mut origin = InlineLayoutUnit::FromFloatCeil(run_advance);
        for glyph in part.Glyphs() {
            bounds.Unite(
                font_data.BoundsForGlyph(glyph.glyph() as Glyph),
                origin.ToFloat(),
                offsets.get(),
            );
            offsets.advance();
            origin += glyph.advance.To::<16, i64>();
        }
        ink_bounds.Union(bounds.BuildBounds(font_data.GetFontMetrics()));
    }

    // cpp: font_engine/fonts/shaping/shape_result_view.cc:281-299
    pub fn ComputeInkBounds(&self) -> RectF {
        let mut ink_bounds = RectF::default();
        let mut run_advance = 0.0;
        for part in &self.parts_ {
            if part.HasGlyphOffsets() {
                if unsafe { &*part.GetRunInfo() }.IsHorizontal() {
                    self.ComputePartInkBounds::<true, true>(part, run_advance, &mut ink_bounds);
                } else {
                    self.ComputePartInkBounds::<false, true>(part, run_advance, &mut ink_bounds);
                }
            } else if unsafe { &*part.GetRunInfo() }.IsHorizontal() {
                self.ComputePartInkBounds::<true, false>(part, run_advance, &mut ink_bounds);
            } else {
                self.ComputePartInkBounds::<false, false>(part, run_advance, &mut ink_bounds);
            }
            run_advance += part.Width();
        }
        ink_bounds
    }

    // cpp: font_engine/fonts/shaping/shape_result_view_glyphs.cc:10-32
    fn ForEachGlyphImpl<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
        initial_advance: f32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
        part: &RunInfoPart,
    ) -> f32 {
        let mut glyph_offsets = part.GetGlyphOffsets::<HAS_NON_ZERO_GLYPH_OFFSETS>();
        let run = unsafe { &*part.GetRunInfo() };
        let mut total_advance = InlineLayoutUnit::FromFloatRound(initial_advance);
        let is_horizontal = run.IsHorizontal();
        let font_data = run.font_data_.Get();
        let character_index_offset_for_glyph_data = self.CharacterIndexOffsetForGlyphData(part);
        for glyph_data in part.Glyphs() {
            let character_index = glyph_data
                .character_index()
                .wrapping_add(character_index_offset_for_glyph_data);
            glyph_callback(
                context,
                character_index,
                glyph_data.glyph() as Glyph,
                glyph_offsets.get(),
                total_advance.ToFloat(),
                is_horizontal,
                run.canvas_rotation_,
                font_data,
            );
            total_advance += glyph_data.advance.To::<16, i64>();
            glyph_offsets.advance();
        }
        total_advance.ToFloat()
    }

    // cpp: font_engine/fonts/shaping/shape_result_view_glyphs.cc:34-48
    pub fn ForEachGlyph(
        &self,
        initial_advance: f32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
    ) -> f32 {
        let mut total_advance = initial_advance;
        for part in self.RunsOrParts() {
            if part.HasGlyphOffsets() {
                total_advance =
                    self.ForEachGlyphImpl::<true>(total_advance, glyph_callback, context, part);
            } else {
                total_advance =
                    self.ForEachGlyphImpl::<false>(total_advance, glyph_callback, context, part);
            }
        }
        total_advance
    }

    // cpp: font_engine/fonts/shaping/shape_result_view_glyphs.cc:50-95
    fn ForEachGlyphImplRange<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
        initial_advance: f32,
        from: u32,
        to: u32,
        _index_offset: u32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
        part: &RunInfoPart,
    ) -> f32 {
        let mut glyph_offsets = part.GetGlyphOffsets::<HAS_NON_ZERO_GLYPH_OFFSETS>();
        let mut total_advance = InlineLayoutUnit::FromFloatRound(initial_advance);
        let run = unsafe { &*part.GetRunInfo() };
        let is_horizontal = run.IsHorizontal();
        let font_data = run.font_data_.Get();
        let character_index_offset_for_glyph_data = self.CharacterIndexOffsetForGlyphData(part);
        if run.IsLtr() {
            for glyph_data in part.Glyphs() {
                let character_index = glyph_data
                    .character_index()
                    .wrapping_add(character_index_offset_for_glyph_data);
                if character_index >= to {
                    break;
                }
                if character_index >= from {
                    glyph_callback(
                        context,
                        character_index,
                        glyph_data.glyph() as Glyph,
                        glyph_offsets.get(),
                        total_advance.ToFloat(),
                        is_horizontal,
                        run.canvas_rotation_,
                        font_data,
                    );
                }
                total_advance += glyph_data.advance.To::<16, i64>();
                glyph_offsets.advance();
            }
        } else {
            for glyph_data in part.Glyphs() {
                let character_index = glyph_data
                    .character_index()
                    .wrapping_add(character_index_offset_for_glyph_data);
                if character_index < from {
                    break;
                }
                if character_index < to {
                    glyph_callback(
                        context,
                        character_index,
                        glyph_data.glyph() as Glyph,
                        glyph_offsets.get(),
                        total_advance.ToFloat(),
                        is_horizontal,
                        run.canvas_rotation_,
                        font_data,
                    );
                }
                total_advance += glyph_data.advance.To::<16, i64>();
                glyph_offsets.advance();
            }
        }
        total_advance.ToFloat()
    }

    // cpp: font_engine/fonts/shaping/shape_result_view_glyphs.cc:97-114
    pub fn ForEachGlyphRange(
        &self,
        initial_advance: f32,
        from: u32,
        to: u32,
        index_offset: u32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
    ) -> f32 {
        let mut total_advance = initial_advance;
        for part in &self.parts_ {
            if part.HasGlyphOffsets() {
                total_advance = self.ForEachGlyphImplRange::<true>(
                    total_advance,
                    from,
                    to,
                    index_offset,
                    glyph_callback,
                    context,
                    part,
                );
            } else {
                total_advance = self.ForEachGlyphImplRange::<false>(
                    total_advance,
                    from,
                    to,
                    index_offset,
                    glyph_callback,
                    context,
                    part,
                );
            }
        }
        total_advance
    }
}
