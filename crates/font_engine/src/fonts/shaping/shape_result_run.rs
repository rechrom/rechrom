#![allow(non_snake_case, non_upper_case_globals)]

use super::glyph_data::{GlyphOffset, HarfBuzzRunGlyphData};
use super::glyph_data_range::GlyphDataRange;
use super::glyph_index_result::GlyphIndexResult;
use super::glyph_offset_iterator::GlyphOffsetIterator;
use super::shape_result_types::{AdjustMidCluster, BreakGlyphsOption};
use crate::fonts::canvas_rotation_in_vertical::CanvasRotationInVertical;
use crate::fonts::simple_font_data::SimpleFontData;
use crate::text::native::harfbuzz::HbGlyphInfo;
use foundation::blink_geometry::geometry::InlineLayoutUnit;
use foundation::{GCedHeapVector, HeapVector, MakeGarbageCollected, Member, Traceable, Visitor};
use std::ops::{Index, IndexMut};

// cpp: font_engine/fonts/shaping/shape_result_run.h:257-271
#[derive(Default)]
struct RareData {
    offsets_: Member<GCedHeapVector<GlyphOffset>>,
    graphemes_: Member<GCedHeapVector<u32>>,
}

impl Traceable for RareData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.offsets_);
        visitor.Trace(&self.graphemes_);
    }
}

// cpp: font_engine/fonts/shaping/shape_result_run.h:252-454
pub struct GlyphDataCollection {
    pub(crate) data_: HeapVector<HarfBuzzRunGlyphData>,
    rare_data_: Member<RareData>,
}

impl GlyphDataCollection {
    pub fn new(num_glyphs: u32) -> Self {
        Self {
            data_: HeapVector::with_size(num_glyphs),
            rare_data_: Member::default(),
        }
    }

    pub fn size(&self) -> u32 {
        self.data_.size()
    }

    pub fn IsEmpty(&self) -> bool {
        self.size() == 0
    }

    pub fn front(&self) -> &HarfBuzzRunGlyphData {
        &self.data_[0]
    }

    pub fn back(&self) -> &HarfBuzzRunGlyphData {
        self.data_.last().expect("empty glyph collection")
    }

    pub fn back_mut(&mut self) -> &mut HarfBuzzRunGlyphData {
        self.data_.last_mut().expect("empty glyph collection")
    }

    pub fn HasNonZeroOffsets(&self) -> bool {
        self.OffsetsVector().is_some()
    }

    pub fn HasGraphemes(&self) -> bool {
        self.Graphemes().is_some()
    }

    pub fn Graphemes(&self) -> Option<&GCedHeapVector<u32>> {
        let rare = unsafe { self.rare_data_.Get().as_ref()? };
        unsafe { rare.graphemes_.Get().as_ref() }
    }

    pub fn GraphemesMut(&mut self) -> Option<&mut GCedHeapVector<u32>> {
        let rare = unsafe { self.rare_data_.Get().as_mut()? };
        unsafe { rare.graphemes_.Get().as_mut() }
    }

    pub fn SetGraphemes(&mut self, graphemes: *mut GCedHeapVector<u32>) {
        assert!(!graphemes.is_null());
        self.EnsureRareData().graphemes_ = Member::from_ptr(graphemes);
    }

    pub fn ByteSize(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.size() as usize * std::mem::size_of::<HarfBuzzRunGlyphData>()
            + self.OffsetsVector().map_or(0, |offsets| offsets.len())
                * std::mem::size_of::<GlyphOffset>()
    }

    pub fn Offsets(&self) -> &[GlyphOffset] {
        self.OffsetsVector()
            .map_or(&[], |offsets| offsets.as_slice())
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:339-343
    pub fn GetOffsets<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
    ) -> GlyphOffsetIterator<'_, HAS_NON_ZERO_GLYPH_OFFSETS> {
        GlyphOffsetIterator::new(self.Offsets())
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:345-369
    pub fn CopyFrom(&mut self, first: &Self, second: &Self) {
        let first_size = first.size() as usize;
        let second_size = second.size() as usize;
        assert_eq!(self.size() as usize, first_size + second_size);
        debug_assert!(!first.IsEmpty() && !second.IsEmpty());
        self.data_[..first_size].copy_from_slice(&first.data_);
        self.data_[first_size..].copy_from_slice(&second.data_);
        if first.HasNonZeroOffsets() {
            self.AllocateOffsetsIfNeeded();
            self.OffsetsVectorMut().unwrap()[..first_size].copy_from_slice(first.Offsets());
        }
        if second.HasNonZeroOffsets() {
            self.AllocateOffsetsIfNeeded();
            self.OffsetsVectorMut().unwrap()[first_size..].copy_from_slice(second.Offsets());
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:371-386
    pub fn CopyFromRange(&mut self, range: &GlyphDataRange) {
        assert_eq!(range.size(), self.size());
        self.data_.copy_from_slice(range.Glyphs());
        if !range.HasOffsets() || range.IsEmpty() {
            self.ClearOffsets();
        } else {
            self.AllocateOffsets();
            self.OffsetsVectorMut()
                .unwrap()
                .copy_from_slice(range.Offsets());
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:388-410
    pub fn AddOffsetHeightAt(&mut self, index: u32, delta: f32) {
        debug_assert_ne!(delta, 0.0);
        self.AllocateOffsetsIfNeeded();
        let offset = &mut self.OffsetsVectorMut().unwrap()[index as usize];
        offset.set_y(offset.y() + delta);
    }

    pub fn AddOffsetWidthAt(&mut self, index: u32, delta: f32) {
        debug_assert_ne!(delta, 0.0);
        self.AllocateOffsetsIfNeeded();
        let offset = &mut self.OffsetsVectorMut().unwrap()[index as usize];
        offset.set_x(offset.x() + delta);
    }

    pub fn SetOffsetAt(&mut self, index: u32, offset: GlyphOffset) {
        if !self.HasNonZeroOffsets() {
            if offset.x() == 0.0 && offset.y() == 0.0 {
                return;
            }
            self.AllocateOffsets();
        }
        self.OffsetsVectorMut().unwrap()[index as usize] = offset;
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:399-418
    pub fn begin(&self) -> *const HarfBuzzRunGlyphData {
        self.data_.as_ptr()
    }

    pub fn end(&self) -> *const HarfBuzzRunGlyphData {
        self.data_.as_ptr().wrapping_add(self.data_.len())
    }

    pub fn begin_mut(&mut self) -> *mut HarfBuzzRunGlyphData {
        self.data_.as_mut_ptr()
    }

    pub fn end_mut(&mut self) -> *mut HarfBuzzRunGlyphData {
        self.data_.as_mut_ptr().wrapping_add(self.data_.len())
    }

    pub fn rbegin(&self) -> std::iter::Rev<std::slice::Iter<'_, HarfBuzzRunGlyphData>> {
        self.data_.iter().rev()
    }

    pub fn rend(&self) -> std::iter::Rev<std::slice::Iter<'_, HarfBuzzRunGlyphData>> {
        self.data_[0..0].iter().rev()
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:437-454
    pub fn Reverse(&mut self) {
        self.data_.reverse();
        if let Some(offsets) = self.OffsetsVectorMut() {
            offsets.reverse();
        }
    }

    pub fn Shrink(&mut self, new_size: u32) {
        debug_assert!(new_size >= 1);
        if new_size == self.size() {
            return;
        }
        debug_assert!(new_size < self.size());
        self.data_.Shrink(new_size);
        if let Some(offsets) = self.OffsetsVectorMut() {
            offsets.Shrink(new_size);
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:468-516
    fn AllocateOffsets(&mut self) {
        debug_assert!(self.size() >= 1 && !self.HasNonZeroOffsets());
        let offsets = MakeGarbageCollected(GCedHeapVector::with_size(self.size()));
        self.EnsureRareData().offsets_ = Member::from_ptr(offsets);
    }

    fn AllocateOffsetsIfNeeded(&mut self) {
        if !self.HasNonZeroOffsets() {
            self.AllocateOffsets();
        }
    }

    fn OffsetsVector(&self) -> Option<&GCedHeapVector<GlyphOffset>> {
        let rare = unsafe { self.rare_data_.Get().as_ref()? };
        unsafe { rare.offsets_.Get().as_ref() }
    }

    fn OffsetsVectorMut(&mut self) -> Option<&mut GCedHeapVector<GlyphOffset>> {
        let rare = unsafe { self.rare_data_.Get().as_mut()? };
        unsafe { rare.offsets_.Get().as_mut() }
    }

    pub fn ClearOffsets(&mut self) {
        if self.rare_data_.Get().is_null() {
            return;
        }
        self.EnsureRareData().offsets_.Clear();
        self.ClearRareDataIfEmpty();
    }

    fn EnsureRareData(&mut self) -> &mut RareData {
        if self.rare_data_.Get().is_null() {
            self.rare_data_ = Member::from_ptr(MakeGarbageCollected(RareData::default()));
        }
        unsafe { &mut *self.rare_data_.Get() }
    }

    fn ClearRareDataIfEmpty(&mut self) {
        let Some(rare) = (unsafe { self.rare_data_.Get().as_ref() }) else {
            return;
        };
        if rare.offsets_.Get().is_null() && rare.graphemes_.Get().is_null() {
            self.rare_data_.Clear();
        }
    }
}

impl Clone for GlyphDataCollection {
    // cpp: font_engine/fonts/shaping/shape_result_run.h:276-289
    fn clone(&self) -> Self {
        let mut copy = Self {
            data_: self.data_.clone(),
            rare_data_: Member::default(),
        };
        if let Some(offsets) = self.OffsetsVector() {
            copy.AllocateOffsets();
            copy.OffsetsVectorMut().unwrap().copy_from_slice(offsets);
        }
        if let Some(graphemes) = self.Graphemes() {
            copy.SetGraphemes(graphemes as *const _ as *mut _);
        }
        copy
    }
}

impl Traceable for GlyphDataCollection {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.data_);
        visitor.Trace(&self.rare_data_);
    }
}

// cpp: font_engine/fonts/shaping/shape_result_run.h:293-297,440-446
impl Index<u32> for GlyphDataCollection {
    type Output = HarfBuzzRunGlyphData;

    fn index(&self, index: u32) -> &Self::Output {
        &self.data_[index as usize]
    }
}

impl IndexMut<u32> for GlyphDataCollection {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.data_[index as usize]
    }
}

impl PartialEq for GlyphDataCollection {
    fn eq(&self, other: &Self) -> bool {
        self.data_ == other.data_
            && self.OffsetsVector() == other.OffsetsVector()
            && self.Graphemes() == other.Graphemes()
    }
}

impl<'a> IntoIterator for &'a GlyphDataCollection {
    type Item = &'a HarfBuzzRunGlyphData;
    type IntoIter = std::slice::Iter<'a, HarfBuzzRunGlyphData>;

    fn into_iter(self) -> Self::IntoIter {
        self.data_.iter()
    }
}

impl<'a> IntoIterator for &'a mut GlyphDataCollection {
    type Item = &'a mut HarfBuzzRunGlyphData;
    type IntoIter = std::slice::IterMut<'a, HarfBuzzRunGlyphData>;

    fn into_iter(self) -> Self::IntoIter {
        self.data_.iter_mut()
    }
}

// cpp: font_engine/fonts/shaping/shape_result_run.h:57-106,531-553
pub struct ShapeResultRun {
    pub(crate) glyph_data_: GlyphDataCollection,
    pub(crate) font_data_: Member<SimpleFontData>,
    pub(crate) start_index_: u32,
    pub(crate) num_characters_: u32,
    pub(crate) width_: f32,
    pub(crate) script_: u32,
    pub(crate) hb_direction_: u8,
    pub(crate) canvas_rotation_: CanvasRotationInVertical,
}

impl ShapeResultRun {
    pub fn new(
        font: *const SimpleFontData,
        direction: u32,
        canvas_rotation: CanvasRotationInVertical,
        script: u32,
        start_index: u32,
        num_glyphs: u32,
        num_characters: u32,
    ) -> Self {
        Self {
            glyph_data_: GlyphDataCollection::new(
                num_glyphs.min(HarfBuzzRunGlyphData::kMaxCharacterIndex + 1),
            ),
            font_data_: Member::from_ptr(font.cast_mut()),
            start_index_: start_index,
            num_characters_: num_characters,
            width_: 0.0,
            script_: script,
            hb_direction_: direction as u8,
            canvas_rotation_: canvas_rotation,
        }
    }

    pub fn NumCharacters(&self) -> u32 {
        self.num_characters_
    }

    pub fn Width(&self) -> f32 {
        self.width_
    }

    pub fn NumGlyphs(&self) -> u32 {
        self.glyph_data_.size()
    }

    pub fn HasLigatures(&self) -> bool {
        self.NumGlyphs() < self.num_characters_
    }

    pub fn HbDirection(&self) -> u32 {
        u32::from(self.hb_direction_)
    }

    pub fn IsLtr(&self) -> bool {
        self.HbDirection() & !2 == 4
    }

    pub fn IsRtl(&self) -> bool {
        self.HbDirection() & !2 == 5
    }

    pub fn IsHorizontal(&self) -> bool {
        self.HbDirection() & !1 == 4
    }

    pub fn CanvasRotation(&self) -> CanvasRotationInVertical {
        self.canvas_rotation_
    }

    pub fn StartIndex(&self) -> u32 {
        self.start_index_
    }

    pub fn GlyphToCharacterIndex(&self, index: u32) -> u32 {
        self.start_index_ + self.glyph_data_.data_[index as usize].character_index()
    }

    pub fn ByteSize(&self) -> usize {
        std::mem::size_of::<Self>() + self.glyph_data_.ByteSize()
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:132-140,242-249
    pub fn FindGlyphDataRange(
        &self,
        start_character_index: u32,
        end_character_index: u32,
    ) -> GlyphDataRange {
        self.GetGlyphDataRange().FindGlyphDataRange(
            self.IsRtl(),
            start_character_index,
            end_character_index,
        )
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:136-163
    pub fn CreateSubRun(&mut self, start: u32, end: u32) -> *mut Self {
        debug_assert!(end > start);
        let number_of_characters = (end - start).min(self.num_characters_);
        let glyphs = self.FindGlyphDataRange(start, end);
        let number_of_glyphs = glyphs.size();
        if number_of_glyphs == 0 {
            return std::ptr::null_mut();
        }
        let run = MakeGarbageCollected(Self::new(
            self.font_data_.Get(),
            self.HbDirection(),
            self.canvas_rotation_,
            self.script_,
            self.start_index_ + start,
            number_of_glyphs,
            number_of_characters,
        ));
        let run = unsafe { &mut *run };
        run.glyph_data_.CopyFromRange(&glyphs);
        let mut total_advance = InlineLayoutUnit::new();
        for glyph in &mut run.glyph_data_.data_ {
            glyph.SetCharacterIndex(glyph.character_index().wrapping_sub(start));
            total_advance += glyph.advance.To::<16, i64>();
        }
        run.width_ = total_advance.ToFloat();
        run.num_characters_ = number_of_characters;
        run
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:165-193
    pub fn MergeIfPossible(&self, other: &Self) -> *mut Self {
        if !self.CanMerge(other) {
            return std::ptr::null_mut();
        }
        debug_assert!(self.start_index_ < other.start_index_);
        let run = MakeGarbageCollected(Self::new(
            self.font_data_.Get(),
            self.HbDirection(),
            self.canvas_rotation_,
            self.script_,
            self.start_index_,
            self.glyph_data_.size() + other.glyph_data_.size(),
            self.num_characters_ + other.num_characters_,
        ));
        let run = unsafe { &mut *run };
        let index_adjust = other.start_index_ - self.start_index_;
        if self.IsRtl() {
            run.glyph_data_
                .CopyFrom(&other.glyph_data_, &self.glyph_data_);
            let count = other.glyph_data_.size() as usize;
            for glyph in &mut run.glyph_data_.data_[..count] {
                glyph.SetCharacterIndex(glyph.character_index().wrapping_add(index_adjust));
            }
        } else {
            run.glyph_data_
                .CopyFrom(&self.glyph_data_, &other.glyph_data_);
            let first = self.glyph_data_.size() as usize;
            for glyph in &mut run.glyph_data_.data_[first..] {
                glyph.SetCharacterIndex(glyph.character_index().wrapping_add(index_adjust));
            }
        }
        run.width_ = self.width_ + other.width_;
        run
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:195-203
    pub fn CanMerge(&self, other: &Self) -> bool {
        self.start_index_.wrapping_add(self.num_characters_) == other.start_index_
            && self.canvas_rotation_ == other.canvas_rotation_
            && self.font_data_ == other.font_data_
            && self.hb_direction_ == other.hb_direction_
            && self.script_ == other.script_
            && self.glyph_data_.size() + other.glyph_data_.size()
                < HarfBuzzRunGlyphData::kMaxCharacterIndex + 1
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:205-244
    pub fn ExpandRangeToIncludePartialGlyphs(&self, offset: i32, from: &mut i32, to: &mut i32) {
        let mut end = offset + self.num_characters_ as i32;
        let mut start;
        if self.IsLtr() {
            start = offset + self.num_characters_ as i32;
            for glyph in &self.glyph_data_.data_ {
                let index = offset + glyph.character_index() as i32;
                if start == index {
                    continue;
                }
                end = index;
                if end > *from && start < *to {
                    *from = (*from).min(start);
                    *to = (*to).max(end);
                }
                end = offset + self.num_characters_ as i32;
                start = index;
            }
        } else {
            start = offset + self.num_characters_ as i32;
            for glyph in &self.glyph_data_.data_ {
                let index = offset + glyph.character_index() as i32;
                if start == index {
                    continue;
                }
                if end > *from && start < *to {
                    *from = (*from).min(start);
                    *to = (*to).max(end);
                }
                end = start;
                start = index;
            }
        }
        if end > *from && start < *to {
            *from = (*from).min(start);
            *to = (*to).max(end);
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.h:522-526
    pub fn CheckConsistency(&self) {
        for glyph in &self.glyph_data_ {
            debug_assert!(glyph.character_index() < self.num_characters_);
        }
    }

    pub fn GetRunInfo(&self) -> &Self {
        self
    }

    pub fn GetGlyphDataRange(&self) -> GlyphDataRange {
        GlyphDataRange::new(self)
    }

    pub fn OffsetToRunStartIndex(&self) -> u32 {
        0
    }
}

impl Clone for ShapeResultRun {
    // cpp: font_engine/fonts/shaping/shape_result_run.h:77-86
    fn clone(&self) -> Self {
        Self {
            glyph_data_: self.glyph_data_.clone(),
            font_data_: self.font_data_,
            start_index_: self.start_index_,
            num_characters_: self.num_characters_,
            width_: self.width_,
            script_: self.script_,
            hb_direction_: self.hb_direction_,
            canvas_rotation_: self.canvas_rotation_,
        }
    }
}

// cpp: font_engine/fonts/shaping/shape_result_run.h:497-520
impl PartialEq for ShapeResultRun {
    fn eq(&self, other: &Self) -> bool {
        const HB_SCRIPT_COMMON: u32 = u32::from_be_bytes(*b"Zyyy");
        const HB_SCRIPT_LATIN: u32 = u32::from_be_bytes(*b"Latn");
        let script_equivalent = self.script_ == other.script_
            || (self.script_ == HB_SCRIPT_COMMON && other.script_ == HB_SCRIPT_LATIN)
            || (self.script_ == HB_SCRIPT_LATIN && other.script_ == HB_SCRIPT_COMMON);
        self.glyph_data_ == other.glyph_data_
            && self.font_data_ == other.font_data_
            && self.start_index_ == other.start_index_
            && self.num_characters_ == other.num_characters_
            && self.width_ == other.width_
            && script_equivalent
            && self.hb_direction_ == other.hb_direction_
            && self.canvas_rotation_ == other.canvas_rotation_
    }
}

impl Traceable for ShapeResultRun {
    // cpp: font_engine/fonts/shaping/shape_result_run.h:88-91
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.glyph_data_);
        visitor.Trace(&self.font_data_);
    }
}

#[allow(non_snake_case)]
impl ShapeResultRun {
    // cpp: font_engine/fonts/shaping/shape_result_run.cc:39-59
    pub fn NextSafeToBreakOffset(&self, offset: u32) -> u32 {
        debug_assert!(offset <= self.num_characters_);
        if self.IsLtr() {
            for glyph in &self.glyph_data_.data_ {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() >= offset {
                    return glyph.character_index();
                }
            }
        } else {
            for glyph in self.glyph_data_.data_.iter().rev() {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() >= offset {
                    return glyph.character_index();
                }
            }
        }
        self.num_characters_
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:61-82
    pub fn PreviousSafeToBreakOffset(&self, offset: u32) -> u32 {
        if offset >= self.num_characters_ {
            return self.num_characters_;
        }
        if self.IsLtr() {
            for glyph in self.glyph_data_.data_.iter().rev() {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() <= offset {
                    return glyph.character_index();
                }
            }
        } else {
            for glyph in &self.glyph_data_.data_ {
                if glyph.IsSafeToBreakBefore() && glyph.character_index() <= offset {
                    return glyph.character_index();
                }
            }
        }
        0
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:84-91
    pub fn XPositionForVisualOffset(&self, mut offset: u32, adjust: AdjustMidCluster) -> f32 {
        debug_assert!(offset < self.num_characters_);
        if self.IsRtl() {
            offset = self.num_characters_ - offset - 1;
        }
        self.XPositionForOffset(offset, adjust)
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:93-102
    pub fn NumGraphemes(&self, start: u32, end: u32) -> u32 {
        let Some(graphemes) = self.glyph_data_.Graphemes() else {
            return 0;
        };
        if start >= self.num_characters_ {
            return 0;
        }
        assert!(start < end && end <= self.num_characters_);
        assert_eq!(self.num_characters_ as usize, graphemes.len());
        graphemes[end as usize - 1] - graphemes[start as usize] + 1
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:104-225
    pub fn XPositionForOffset(&self, offset: u32, adjust: AdjustMidCluster) -> f32 {
        debug_assert!(offset <= self.num_characters_);
        let num_glyphs = self.glyph_data_.size();
        let mut glyph_sequence_start = 0;
        let mut glyph_sequence_end = self.num_characters_;
        let mut glyph_sequence_advance = InlineLayoutUnit::new();
        let mut accumulated_position = InlineLayoutUnit::new();

        if self.IsLtr() {
            for index in 0..num_glyphs {
                let glyph = &self.glyph_data_.data_[index as usize];
                let current_character_index = glyph.character_index();
                if glyph_sequence_start == current_character_index {
                    glyph_sequence_advance += glyph.advance.To::<16, i64>();
                    continue;
                }
                if glyph_sequence_start <= offset && offset < current_character_index {
                    glyph_sequence_end = current_character_index;
                    break;
                }
                glyph_sequence_start = current_character_index;
                glyph_sequence_end = self.num_characters_;
                accumulated_position += glyph_sequence_advance;
                glyph_sequence_advance = glyph.advance.To::<16, i64>();
            }
        } else {
            glyph_sequence_start = self.num_characters_;
            glyph_sequence_end = self.num_characters_;
            for index in 0..num_glyphs {
                let glyph = &self.glyph_data_.data_[index as usize];
                let current_character_index = glyph.character_index();
                if glyph_sequence_start == current_character_index {
                    glyph_sequence_advance += glyph.advance.To::<16, i64>();
                    continue;
                }
                if glyph_sequence_start <= offset && offset < glyph_sequence_end {
                    break;
                }
                glyph_sequence_end = glyph_sequence_start;
                glyph_sequence_start = current_character_index;
                accumulated_position += glyph_sequence_advance;
                glyph_sequence_advance = glyph.advance.To::<16, i64>();
            }
        }

        let mut is_at_sequence_start = offset == glyph_sequence_start;
        let graphemes = self.NumGraphemes(glyph_sequence_start, glyph_sequence_end);
        if graphemes > 1 {
            debug_assert!(glyph_sequence_end >= glyph_sequence_start);
            let next_offset = offset + u32::from(offset != self.num_characters_);
            let graphemes_to_offset = self.NumGraphemes(glyph_sequence_start, next_offset) - 1;
            if offset > 0 {
                is_at_sequence_start = self.NumGraphemes(offset - 1, next_offset) != 1;
            }
            glyph_sequence_advance = glyph_sequence_advance / graphemes;
            let graphemes_from_left = if self.IsLtr() {
                graphemes_to_offset
            } else {
                graphemes - graphemes_to_offset - 1
            };
            accumulated_position += glyph_sequence_advance * graphemes_from_left;
        }

        if self.IsLtr() && adjust == AdjustMidCluster::kToEnd && !is_at_sequence_start {
            accumulated_position += glyph_sequence_advance;
        } else if self.IsRtl() && adjust == AdjustMidCluster::kToEnd && !is_at_sequence_start {
            accumulated_position -= glyph_sequence_advance;
        }
        if self.IsRtl() {
            accumulated_position += glyph_sequence_advance;
        }
        accumulated_position.ToFloat()
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:227-303
    pub fn CharacterIndexForXPosition(
        &self,
        target_x: f32,
        break_glyphs: BreakGlyphsOption,
        result: &mut GlyphIndexResult,
    ) {
        debug_assert!(target_x >= 0.0 && target_x <= self.width_);
        result.origin_x = 0.0;
        let mut glyph_sequence_start = 0;
        let mut glyph_sequence_end = self.num_characters_;
        result.advance = 0.0;
        if self.IsRtl() {
            glyph_sequence_start = self.num_characters_;
            glyph_sequence_end = self.num_characters_;
        }
        for glyph in &self.glyph_data_.data_ {
            let current_character_index = glyph.character_index();
            if glyph_sequence_start == current_character_index {
                result.advance += glyph.advance.ToFloat();
                continue;
            }
            if result.origin_x + result.advance > target_x {
                if self.IsLtr() {
                    glyph_sequence_end = current_character_index;
                }
                break;
            }
            if self.IsRtl() {
                glyph_sequence_end = glyph_sequence_start;
            }
            glyph_sequence_start = current_character_index;
            result.origin_x += result.advance;
            result.advance = glyph.advance.ToFloat();
        }
        if break_glyphs.0 && glyph_sequence_end > glyph_sequence_start {
            let graphemes = self.NumGraphemes(glyph_sequence_start, glyph_sequence_end);
            if graphemes > 1 {
                let unit_size = result.advance / graphemes as f32;
                let step = ((target_x - result.origin_x) / unit_size).floor() as u32;
                let glyph_length = glyph_sequence_end - glyph_sequence_start;
                let final_size = glyph_length / graphemes;
                result.origin_x += unit_size * step as f32;
                if self.IsLtr() {
                    glyph_sequence_start += step;
                    glyph_sequence_end = glyph_sequence_start + final_size;
                } else {
                    glyph_sequence_end -= step;
                    glyph_sequence_start = glyph_sequence_end - final_size;
                }
                result.advance = unit_size;
            }
        }
        if self.IsLtr() {
            result.left_character_index = glyph_sequence_start;
            result.right_character_index = glyph_sequence_end;
        } else {
            result.left_character_index = glyph_sequence_end;
            result.right_character_index = glyph_sequence_start;
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_run.cc:305-422
    pub fn LimitNumGlyphs(
        &mut self,
        start_glyph: u32,
        num_glyphs_in_out: &mut u32,
        num_glyphs_removed_out: &mut u32,
        is_ltr: bool,
        glyph_infos: &[HbGlyphInfo],
    ) {
        let mut num_glyphs = *num_glyphs_in_out;
        assert!(num_glyphs > 0);
        let first = start_glyph as usize;
        let mut right = first + num_glyphs as usize - 1;
        let mut start_cluster;
        if is_ltr {
            start_cluster = glyph_infos[first].cluster;
            let last_cluster = glyph_infos[right].cluster;
            let max_cluster = start_cluster + HarfBuzzRunGlyphData::kMaxCharacterIndex;
            if last_cluster > max_cluster {
                let window = &glyph_infos[first..=right];
                let limit = window.partition_point(|info| info.cluster <= max_cluster) - 1;
                assert!(limit > 0 && limit < window.len() - 1);
                debug_assert!(window[limit].cluster <= max_cluster);
                right = first + limit;
                num_glyphs = (right - first + 1) as u32;
                self.num_characters_ = glyph_infos[right + 1].cluster - start_cluster;
            }
        } else {
            start_cluster = glyph_infos[right].cluster;
            let last_cluster = glyph_infos[first].cluster;
            let max_cluster = start_cluster + HarfBuzzRunGlyphData::kMaxCharacterIndex;
            if last_cluster > max_cluster {
                let min_cluster = last_cluster - HarfBuzzRunGlyphData::kMaxCharacterIndex;
                debug_assert!(start_cluster < min_cluster);
                let window = &glyph_infos[first..=right];
                let limit = window.partition_point(|info| info.cluster >= min_cluster) - 1;
                assert!(limit > 0 && limit < window.len() - 1);
                debug_assert!(window[limit].cluster >= min_cluster);
                right = first + limit;
                start_cluster = glyph_infos[right].cluster;
                num_glyphs = (right - first + 1) as u32;
                self.start_index_ = start_cluster;
                self.num_characters_ = last_cluster - glyph_infos[right + 1].cluster;
            }
        }

        *num_glyphs_removed_out = 0;
        if num_glyphs > HarfBuzzRunGlyphData::kMaxGlyphs {
            let old_num_glyphs = num_glyphs;
            num_glyphs = HarfBuzzRunGlyphData::kMaxGlyphs;
            let end_cluster = glyph_infos[first + num_glyphs as usize].cluster;
            while num_glyphs > 0 {
                if glyph_infos[first + num_glyphs as usize - 1].cluster != end_cluster {
                    break;
                }
                num_glyphs -= 1;
            }
            if num_glyphs == 0 {
                num_glyphs = HarfBuzzRunGlyphData::kMaxGlyphs;
                *num_glyphs_removed_out = old_num_glyphs - num_glyphs;
            } else if is_ltr {
                self.num_characters_ = end_cluster - start_cluster;
                debug_assert!(self.num_characters_ > 0);
            } else {
                self.num_characters_ = glyph_infos[first].cluster - end_cluster;
                self.start_index_ = glyph_infos[first + num_glyphs as usize - 1].cluster;
                debug_assert!(self.num_characters_ > 0);
            }
        }
        debug_assert!(num_glyphs <= HarfBuzzRunGlyphData::kMaxGlyphs);
        if num_glyphs == *num_glyphs_in_out {
            return;
        }
        self.glyph_data_.Shrink(num_glyphs);
        *num_glyphs_in_out = num_glyphs;
    }
}
