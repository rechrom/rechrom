// C++: font_engine/fonts/shaping/shape_result.h and shape_result_data.cc.
// ShapeResult owns ordered runs and the result-level algorithms; per-run
// glyph calculations remain with ShapeResultRun.
use super::glyph_bounds_accumulator::GlyphBoundsAccumulator;
use super::glyph_data::{GlyphOffset, HarfBuzzRunGlyphData, SafeToBreak};
use super::shape_result_run::ShapeResultRun;
use super::shape_result_types::AdjustMidCluster;
use crate::fonts::canvas_rotation_in_vertical::CanvasRotationInVertical;
use crate::fonts::font::Font;
use crate::fonts::glyph::Glyph;
use crate::fonts::opentype::open_type_math_stretch_data::{AssemblyParameters, StretchAxis};
use crate::fonts::simple_font_data::SimpleFontData;
use foundation::blink_geometry::geometry::{InlineLayoutUnit, TextRunLayoutUnit};
use foundation::gfx::RectF;
use foundation::{
    HeapVector, IsLtr, IsRtl, LayoutUnit, MakeGarbageCollected, Member, TabSize, TextDirection,
    Traceable, Visitor,
};
use std::cell::{Cell, RefCell};

const HB_DIRECTION_LTR: u32 = 4;
const HB_DIRECTION_RTL: u32 = 5;
const HB_DIRECTION_TTB: u32 = 6;
const HB_SCRIPT_COMMON: u32 = u32::from_be_bytes(*b"Zyyy");

// The C++ union stores two 32-bit fixed-point
// interpretations of the same bits: LayoutUnit (6 fractional bits) or
// TextRunLayoutUnit (16 fractional bits). Cache construction writes the first.
// cpp: font_engine/fonts/shaping/shape_result.h:86-105
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeResultCharacterData {
    position_or_advance_raw: i32,
    pub is_cluster_base: bool,
    pub safe_to_break_before: bool,
    pub has_auto_spacing_after: bool,
}

// cpp: font_engine/fonts/shaping/shape_result.h:109-113
#[derive(Clone, Copy, Debug)]
pub struct OffsetWithSpacing {
    pub offset: u32,
    pub spacing: f32,
}

impl ShapeResultCharacterData {
    pub fn SetCachedData(
        &mut self,
        new_x_position: LayoutUnit,
        new_is_cluster_base: bool,
        new_safe_to_break_before: bool,
    ) {
        self.position_or_advance_raw = new_x_position.RawValue();
        self.is_cluster_base = new_is_cluster_base;
        self.safe_to_break_before = new_safe_to_break_before;
    }

    pub fn x_position(&self) -> LayoutUnit {
        LayoutUnit::FromRawValue(self.position_or_advance_raw)
    }

    pub fn advance(&self) -> TextRunLayoutUnit {
        TextRunLayoutUnit::FromRawValue(self.position_or_advance_raw)
    }

    pub fn set_advance(&mut self, advance: TextRunLayoutUnit) {
        self.position_or_advance_raw = advance.RawValue();
    }
}

impl Traceable for ShapeResultCharacterData {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// C++: shape_result.h:130-551; shape_result_data.cc:37-61.
// cpp: font_engine/fonts/shaping/shape_result.h:130-133,481-512
pub struct ShapeResult {
    pub(crate) character_position_: RefCell<HeapVector<ShapeResultCharacterData>>,
    pub(crate) runs_: HeapVector<Member<ShapeResultRun>, 1>,
    pub(crate) width_: Cell<f32>,
    pub(crate) start_index_: u32,
    pub(crate) num_characters_: u32,
    direction_: TextDirection,
    pub(crate) has_vertical_offsets_: bool,
    pub(crate) is_applied_spacing_: bool,
}

// cpp: font_engine/fonts/shaping/shape_result.h:328-337
#[derive(Clone, Copy)]
pub struct ShapeRange {
    pub start: u32,
    pub end: u32,
    pub target: Member<ShapeResult>,
}

impl ShapeRange {
    pub fn new(start: u32, end: u32, target: *mut ShapeResult) -> Self {
        Self {
            start,
            end,
            target: Member::from_ptr(target),
        }
    }
}

impl Traceable for ShapeRange {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.target);
    }
}

// cpp: font_engine/fonts/shaping/shape_result_data.cc:44-54
impl Clone for ShapeResult {
    fn clone(&self) -> Self {
        let mut runs = HeapVector::new();
        runs.ReserveInitialCapacity(self.runs_.size());
        for run in &self.runs_ {
            let run = unsafe { run.Get().as_ref() }.expect("ShapeResult run is null");
            runs.push_back(Member::from_ptr(MakeGarbageCollected(run.clone())));
        }
        Self {
            character_position_: RefCell::new(HeapVector::new()),
            runs_: runs,
            width_: Cell::new(self.width_.get()),
            start_index_: self.start_index_,
            num_characters_: self.num_characters_,
            direction_: self.direction_,
            has_vertical_offsets_: self.has_vertical_offsets_,
            is_applied_spacing_: self.is_applied_spacing_,
        }
    }
}

#[allow(non_snake_case)]
impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_creation.cc:12-49
    pub fn CreateForTabulationCharacters(
        font: &Font,
        direction: TextDirection,
        tab_size: &TabSize,
        position: f32,
        mut start_index: u32,
        length: u32,
    ) -> *const Self {
        assert!(length > 0);
        let data = font.PrimaryFontForTabSize();
        assert!(!data.is_null());
        let result = MakeGarbageCollected(Self::new(start_index, length, direction));
        let result_ref = unsafe { &mut *result };
        result_ref.has_vertical_offsets_ = unsafe { &*data }.PlatformData().IsVerticalAnyUpright();
        let hb_direction = if IsLtr(direction) {
            HB_DIRECTION_LTR
        } else {
            HB_DIRECTION_RTL
        };
        let mut advance =
            TextRunLayoutUnit::FromFloatRound(font.TabWidthAt(data, tab_size, position));
        let mut remaining = length;
        loop {
            let run_length = remaining.min(HarfBuzzRunGlyphData::kMaxGlyphs);
            let run = MakeGarbageCollected(ShapeResultRun::new(
                data,
                hb_direction,
                CanvasRotationInVertical::kRegular,
                HB_SCRIPT_COMMON,
                start_index,
                run_length,
                run_length,
            ));
            let run_ref = unsafe { &mut *run };
            let mut run_width = InlineLayoutUnit::new();
            for i in 0..run_length {
                if i == 1 {
                    advance = TextRunLayoutUnit::FromFloatRound(font.TabWidth(data, tab_size));
                }
                let index = if IsLtr(direction) {
                    i
                } else {
                    remaining - 1 - i
                };
                run_ref.glyph_data_[i] = HarfBuzzRunGlyphData::new(
                    u32::from(unsafe { &*data }.SpaceGlyph()),
                    index,
                    SafeToBreak::kSafe,
                    advance,
                );
                run_width += advance.To::<16, i64>();
            }
            run_ref.width_ = run_width.ToFloat();
            result_ref
                .width_
                .set(result_ref.width_.get() + run_ref.width_);
            result_ref.runs_.push_back(Member::from_ptr(run));
            remaining -= run_length;
            start_index = start_index.wrapping_add(run_length);
            if remaining == 0 {
                break;
            }
        }
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_creation.cc:51-75
    pub fn CreateForSpaces(
        font: &Font,
        direction: TextDirection,
        start_index: u32,
        length: u32,
        width: f32,
    ) -> *const Self {
        assert!(length > 0);
        let data = font.PrimaryFont();
        assert!(!data.is_null());
        let result = MakeGarbageCollected(Self::new(start_index, length, direction));
        let result_ref = unsafe { &mut *result };
        result_ref.has_vertical_offsets_ = unsafe { &*data }.PlatformData().IsVerticalAnyUpright();
        let run = MakeGarbageCollected(ShapeResultRun::new(
            data,
            if IsLtr(direction) {
                HB_DIRECTION_LTR
            } else {
                HB_DIRECTION_RTL
            },
            CanvasRotationInVertical::kRegular,
            HB_SCRIPT_COMMON,
            start_index,
            length,
            length,
        ));
        let run_ref = unsafe { &mut *run };
        run_ref.width_ = width;
        result_ref.width_.set(width);
        let mut glyph_width = TextRunLayoutUnit::FromFloatRound(width);
        for i in 0..run_ref.NumGlyphs() {
            let index = if IsLtr(direction) { i } else { length - 1 - i };
            run_ref.glyph_data_[i] = HarfBuzzRunGlyphData::new(
                u32::from(unsafe { &*data }.SpaceGlyph()),
                index,
                SafeToBreak::kSafe,
                glyph_width,
            );
            glyph_width = TextRunLayoutUnit::new();
        }
        result_ref.runs_.push_back(Member::from_ptr(run));
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_creation.cc:77-88
    pub fn CreateForStretchyMathOperatorGlyph(
        font: &Font,
        direction: TextDirection,
        glyph: Glyph,
        size: f32,
    ) -> *const Self {
        let result = MakeGarbageCollected(Self::new(0, 1, direction));
        let result_ref = unsafe { &mut *result };
        let run = MakeGarbageCollected(ShapeResultRun::new(
            font.PrimaryFont(),
            HB_DIRECTION_LTR,
            CanvasRotationInVertical::kRegular,
            HB_SCRIPT_COMMON,
            0,
            1,
            1,
        ));
        let run_ref = unsafe { &mut *run };
        run_ref.glyph_data_[0] = HarfBuzzRunGlyphData::new(
            u32::from(glyph),
            0,
            SafeToBreak::kSafe,
            TextRunLayoutUnit::FromFloatRound(size),
        );
        run_ref.width_ = size.max(0.0);
        result_ref.width_.set(run_ref.width_);
        result_ref.runs_.push_back(Member::from_ptr(run));
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_creation.cc:90-129
    pub fn CreateForStretchyMathOperatorAssembly(
        font: &Font,
        direction: TextDirection,
        axis: StretchAxis,
        assembly: &AssemblyParameters,
    ) -> *const Self {
        assert!(!assembly.parts.is_empty());
        assert!(assembly.glyph_count <= HarfBuzzRunGlyphData::kMaxGlyphs);
        let horizontal = axis == StretchAxis::Horizontal;
        let result = MakeGarbageCollected(Self::new(0, 1, direction));
        let result_ref = unsafe { &mut *result };
        let run = MakeGarbageCollected(ShapeResultRun::new(
            font.PrimaryFont(),
            if horizontal {
                HB_DIRECTION_LTR
            } else {
                HB_DIRECTION_TTB
            },
            CanvasRotationInVertical::kRegular,
            HB_SCRIPT_COMMON,
            0,
            assembly.glyph_count,
            1,
        ));
        let run_ref = unsafe { &mut *run };
        let overlap = assembly.connector_overlap;
        let mut part_index = 0u32;
        for part in &assembly.parts {
            let repetitions = if part.is_extender {
                assembly.repetition_count
            } else {
                1
            };
            for _ in 0..repetitions {
                let glyph_index = if horizontal {
                    part_index
                } else {
                    assembly.glyph_count - 1 - part_index
                };
                let advance = if glyph_index == assembly.glyph_count - 1 {
                    part.full_advance
                } else {
                    part.full_advance - overlap
                };
                run_ref.glyph_data_[glyph_index] = HarfBuzzRunGlyphData::new(
                    u32::from(part.glyph),
                    0,
                    if glyph_index != 0 {
                        SafeToBreak::kUnsafe
                    } else {
                        SafeToBreak::kSafe
                    },
                    TextRunLayoutUnit::FromFloatRound(advance),
                );
                if !horizontal {
                    let ascent = -unsafe { &*font.PrimaryFont() }
                        .BoundsForGlyph(part.glyph)
                        .y();
                    let offset = GlyphOffset::new(0.0, -assembly.stretch_size + ascent);
                    run_ref.glyph_data_.SetOffsetAt(glyph_index, offset);
                    result_ref.has_vertical_offsets_ |= offset.y() != 0.0;
                }
                part_index += 1;
            }
        }
        run_ref.width_ = assembly.stretch_size.max(0.0);
        result_ref.width_.set(run_ref.width_);
        result_ref.runs_.push_back(Member::from_ptr(run));
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:44-54
    pub fn DeepCopy(&self) -> *mut Self {
        MakeGarbageCollected(self.clone())
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:63-74
    pub fn HasLigatures(&self) -> bool {
        for run in &self.runs_ {
            if unsafe { &*run.Get() }.HasLigatures() {
                return true;
            }
        }
        false
    }

    pub fn NumGlyphs(&self) -> u32 {
        let mut count = 0u32;
        for run in &self.runs_ {
            count = count.wrapping_add(unsafe { &*run.Get() }.NumGlyphs());
        }
        count
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:76-83
    pub fn IsStartSafeToBreak(&self) -> bool {
        if self.runs_.empty() {
            return false;
        }
        let run = unsafe {
            &*(if self.IsLtr() {
                self.runs_.first().unwrap()
            } else {
                self.runs_.last().unwrap()
            })
            .Get()
        };
        let glyph = if self.IsLtr() {
            run.glyph_data_.front()
        } else {
            run.glyph_data_.back()
        };
        glyph.IsSafeToBreakBefore()
            && self.StartIndex() == run.start_index_.wrapping_add(glyph.character_index())
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:85-98
    pub fn ReorderRtlRuns(&mut self, before: u32) {
        let before = before as usize;
        if self.runs_.len() == before + 1 {
            if before == 0 {
                return;
            }
            let run = self.runs_.pop().expect("last run");
            self.runs_.insert(0, run);
            return;
        }
        let mut reordered = HeapVector::new();
        reordered.ReserveInitialCapacity(self.runs_.size());
        for i in before..self.runs_.len() {
            reordered.push_back(self.runs_[i]);
        }
        for i in 0..before {
            reordered.push_back(self.runs_[i]);
        }
        std::mem::swap(&mut self.runs_, &mut reordered);
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:100-110
    pub fn InsertRun(&mut self, run: *mut ShapeResultRun) {
        let incoming = unsafe { &*run };
        let position = self.runs_.iter().position(|current| {
            let current = unsafe { &*current.Get() };
            if incoming.IsLtr() {
                current.start_index_ >= incoming.start_index_
            } else {
                current.start_index_ <= incoming.start_index_
            }
        });
        if let Some(position) = position {
            self.runs_.insert(position, Member::from_ptr(run));
        } else {
            self.runs_.push_back(Member::from_ptr(run));
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:112-159
    pub fn CopyRangeInternal(
        &self,
        mut run_index: u32,
        start_offset: u32,
        end_offset: u32,
        target: &mut Self,
    ) -> u32 {
        target.is_applied_spacing_ |= self.is_applied_spacing_;
        let clipped_start = start_offset.max(self.StartIndex());
        let clipped_end = end_offset.min(self.EndIndex());
        if clipped_start >= clipped_end {
            return run_index;
        }
        let index_diff = if target.num_characters_ == 0 {
            target.start_index_ = start_offset;
            0u32
        } else {
            target.EndIndex().wrapping_sub(clipped_start)
        };
        target.num_characters_ = target
            .num_characters_
            .wrapping_add(clipped_end - clipped_start)
            & ((1 << 29) - 1);
        let run_size_before = target.runs_.size();
        let mut should_merge = !target.runs_.empty();
        let mut has_glyphs = false;
        while (run_index as usize) < self.runs_.len() {
            let run = unsafe { &mut *self.runs_[run_index as usize].Get() };
            let run_start = run.start_index_;
            let run_end = run_start.wrapping_add(run.num_characters_);
            if start_offset < run_end && end_offset > run_start {
                let start = if start_offset > run_start {
                    start_offset - run_start
                } else {
                    0
                };
                let end = end_offset.min(run_end) - run_start;
                let sub_run = run.CreateSubRun(start, end);
                if !sub_run.is_null() {
                    let sub_run_ref = unsafe { &mut *sub_run };
                    sub_run_ref.start_index_ = sub_run_ref.start_index_.wrapping_add(index_diff);
                    target.width_.set(target.width_.get() + sub_run_ref.width_);
                    has_glyphs |= sub_run_ref.glyph_data_.size() != 0;
                    let merged = if should_merge {
                        unsafe { &*target.runs_.last().unwrap().Get() }.MergeIfPossible(sub_run_ref)
                    } else {
                        std::ptr::null_mut()
                    };
                    if !merged.is_null() {
                        *target.runs_.last_mut().unwrap() = Member::from_ptr(merged);
                    } else {
                        target.runs_.push_back(Member::from_ptr(sub_run));
                    }
                }
                should_merge = false;
                if (self.IsLtr() && end_offset <= run_end)
                    || (self.IsRtl() && start_offset >= run_start)
                {
                    break;
                }
            }
            run_index += 1;
        }
        if has_glyphs && self.IsRtl() && target.runs_.size() != run_size_before {
            target.ReorderRtlRuns(run_size_before);
        }
        target.has_vertical_offsets_ |= self.has_vertical_offsets_;
        run_index
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:161-165
    pub fn CopyRange(&self, start_offset: u32, end_offset: u32, target: &mut Self) {
        self.CopyRangeInternal(0, start_offset, end_offset, target);
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:167-184
    pub fn CopyRanges(&self, ranges: &[ShapeRange]) {
        assert!(!ranges.is_empty());
        let mut run_index = 0;
        if self.IsRtl() {
            for range in ranges.iter().rev() {
                run_index = self.CopyRangeInternal(run_index, range.start, range.end, unsafe {
                    &mut *range.target.Get()
                });
            }
        } else {
            for range in ranges {
                run_index = self.CopyRangeInternal(run_index, range.start, range.end, unsafe {
                    &mut *range.target.Get()
                });
            }
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:186-201
    pub fn SubRange(&self, start_offset: u32, end_offset: u32) -> *mut Self {
        let result = MakeGarbageCollected(Self::new(0, 0, self.Direction()));
        self.CopyRange(start_offset, end_offset, unsafe { &mut *result });
        result
    }

    pub fn CopyAdjustedOffset(&self, start_index: u32) -> *const Self {
        let result = MakeGarbageCollected(self.clone());
        let result_ref = unsafe { &mut *result };
        let delta = start_index.wrapping_sub(result_ref.StartIndex());
        for run in &result_ref.runs_ {
            let run = unsafe { &mut *run.Get() };
            run.start_index_ = run.start_index_.wrapping_add(delta);
        }
        result_ref.start_index_ = start_index;
        result
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:204-241
    pub fn PositionForOffset(&self, absolute_offset: u32, adjust: AdjustMidCluster) -> f32 {
        let mut x = 0.0;
        let mut offset = absolute_offset;
        if self.IsRtl() && offset < self.NumCharacters() {
            offset = self.NumCharacters() - offset - 1;
        }
        for run in &self.runs_ {
            let Some(run) = (unsafe { run.Get().as_ref() }) else {
                continue;
            };
            debug_assert_eq!(self.IsRtl(), run.IsRtl());
            if offset < run.num_characters_ {
                return run.XPositionForVisualOffset(offset, adjust) + x;
            }
            offset -= run.num_characters_;
            x += run.width_;
        }
        if absolute_offset == self.NumCharacters() {
            return if self.IsRtl() { 0.0 } else { self.width_.get() };
        }
        0.0
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:243-250
    pub fn HasFallbackFonts(&self, primary_font: *const SimpleFontData) -> bool {
        for run in &self.runs_ {
            if unsafe { &*run.Get() }.font_data_.Get() != primary_font.cast_mut() {
                return true;
            }
        }
        false
    }
    // cpp: font_engine/fonts/shaping/shape_result_data.cc:37-42
    pub fn new(start_index: u32, num_characters: u32, direction: TextDirection) -> Self {
        Self {
            character_position_: RefCell::new(HeapVector::new()),
            runs_: HeapVector::new(),
            width_: Cell::new(0.0),
            start_index_: start_index,
            num_characters_: num_characters & ((1 << 29) - 1),
            direction_: direction,
            has_vertical_offsets_: false,
            is_applied_spacing_: false,
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result.h:136-138
    pub fn CreateEmpty(other: &Self) -> *mut Self {
        MakeGarbageCollected(Self::new(0, 0, other.Direction()))
    }

    // cpp: font_engine/fonts/shaping/shape_result.h:159-162,171-180,185,188
    pub fn Width(&self) -> f32 {
        self.width_.get()
    }
    pub fn SnappedWidth(&self) -> LayoutUnit {
        LayoutUnit::FromFloatCeil(self.width_.get())
    }
    pub fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }
    pub fn StartIndex(&self) -> u32 {
        self.start_index_
    }
    pub fn EndIndex(&self) -> u32 {
        self.start_index_.wrapping_add(self.num_characters_)
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
    pub fn IsAppliedSpacing(&self) -> bool {
        self.is_applied_spacing_
    }

    // cpp: font_engine/fonts/shaping/shape_result.h:475-479
    pub fn RunsOrParts(&self) -> &HeapVector<Member<ShapeResultRun>, 1> {
        &self.runs_
    }
    pub fn StartIndexOffsetForRun(&self) -> u32 {
        0
    }

    // cpp: font_engine/fonts/shaping/shape_result_data.cc:58-61
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.runs_);
        visitor.Trace(&*self.character_position_.borrow());
    }
}

#[allow(non_snake_case)]
impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_ink_bounds.cc:40-47
    pub fn ComputeRunInkBounds<
        const IS_HORIZONTAL_RUN: bool,
        const HAS_NON_ZERO_GLYPH_OFFSETS: bool,
    >(
        &self,
        run: &ShapeResultRun,
        run_advance: f32,
        ink_bounds: &mut RectF,
    ) {
        self.ComputeRunInkBoundsScalar::<IS_HORIZONTAL_RUN, HAS_NON_ZERO_GLYPH_OFFSETS>(
            run,
            run_advance,
            ink_bounds,
        );
    }

    // cpp: font_engine/fonts/shaping/shape_result_ink_bounds.cc:49-65
    pub fn ComputeRunInkBoundsScalar<
        const IS_HORIZONTAL_RUN: bool,
        const HAS_NON_ZERO_GLYPH_OFFSETS: bool,
    >(
        &self,
        run: &ShapeResultRun,
        run_advance: f32,
        ink_bounds: &mut RectF,
    ) {
        let mut offsets = run.glyph_data_.GetOffsets::<HAS_NON_ZERO_GLYPH_OFFSETS>();
        let font_data = unsafe { &*run.font_data_.Get() };
        let mut bounds = GlyphBoundsAccumulator::<IS_HORIZONTAL_RUN>::default();
        let mut origin = InlineLayoutUnit::FromFloatCeil(run_advance);
        for glyph in &run.glyph_data_ {
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

    // cpp: font_engine/fonts/shaping/shape_result_ink_bounds.cc:67-85
    pub fn ComputeInkBounds(&self) -> RectF {
        let mut ink_bounds = RectF::default();
        let mut run_advance = 0.0;
        for run in &self.runs_ {
            let run = unsafe { &*run.Get() };
            if run.glyph_data_.HasNonZeroOffsets() {
                if run.IsHorizontal() {
                    self.ComputeRunInkBounds::<true, true>(run, run_advance, &mut ink_bounds);
                } else {
                    self.ComputeRunInkBounds::<false, true>(run, run_advance, &mut ink_bounds);
                }
            } else if run.IsHorizontal() {
                self.ComputeRunInkBounds::<true, false>(run, run_advance, &mut ink_bounds);
            } else {
                self.ComputeRunInkBounds::<false, false>(run, run_advance, &mut ink_bounds);
            }
            run_advance += run.width_;
        }
        ink_bounds
    }
}

impl Traceable for ShapeResult {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ShapeResult::Trace(self, visitor);
    }
}

// cpp: font_engine/fonts/shaping/shape_result_data.cc:252-272
impl PartialEq for ShapeResult {
    fn eq(&self, other: &Self) -> bool {
        if self.runs_.size() != other.runs_.size() {
            return false;
        }
        for (left, right) in self.runs_.iter().zip(other.runs_.iter()) {
            let left = unsafe { left.Get().as_ref() };
            let right = unsafe { right.Get().as_ref() };
            if left != right {
                return false;
            }
        }
        self.start_index_ == other.start_index_
            && self.num_characters_ == other.num_characters_
            && self.direction_ == other.direction_
            && self.has_vertical_offsets_ == other.has_vertical_offsets_
            && self.is_applied_spacing_ == other.is_applied_spacing_
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_result_preserves_source_indices_direction_and_ceil_width() {
        let result = ShapeResult::new(7, 3, TextDirection::kRtl);
        assert_eq!(result.StartIndex(), 7);
        assert_eq!(result.EndIndex(), 10);
        assert_eq!(result.NumCharacters(), 3);
        assert!(result.IsRtl());
        assert_eq!(result.SnappedWidth(), LayoutUnit::FromFloatCeil(0.0));
        assert!(result.RunsOrParts().empty());
    }

    #[test]
    fn cached_character_data_keeps_layout_unit_bits_and_flags() {
        let mut data = ShapeResultCharacterData::default();
        let x = LayoutUnit::FromFloatRound(1.25);
        data.SetCachedData(x, true, false);
        assert_eq!(data.x_position(), x);
        assert!(data.is_cluster_base);
        assert!(!data.safe_to_break_before);
    }
}
