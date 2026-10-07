#![allow(non_snake_case)]

use super::forward::GlyphCallback;
use super::glyph_index_result::GlyphIndexResult;
use super::shape_result::{ShapeResult, ShapeResultCharacterData};
use super::shape_result_run::ShapeResultRun;
use super::shape_result_types::{AdjustMidCluster, BreakGlyphsOption};
use crate::fonts::glyph::Glyph;
use crate::text::native::character_break_iterator::GraphemesClusterList;
use foundation::blink_geometry::geometry::{InlineLayoutUnit, TextRunLayoutUnit};
use foundation::{
    GCedHeapVector, HeapVector, IsLtr, LayoutUnit, MakeGarbageCollected, String, StringView,
    TextDirection,
};
use std::cell::{Ref, RefMut};
use std::ffi::c_void;

impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_position.cc:39-58
    pub fn EnsureGraphemes(&self, text: &StringView) {
        assert_eq!(self.NumCharacters(), text.length());
        if self.runs_.empty() {
            return;
        }
        if unsafe { &*self.runs_.first().unwrap().Get() }
            .glyph_data_
            .HasGraphemes()
        {
            return;
        }
        let result_start_index = self.StartIndex();
        for member in &self.runs_ {
            let Some(run) = (unsafe { member.Get().as_mut() }) else {
                continue;
            };
            debug_assert!(run.start_index_ >= result_start_index);
            let graphemes =
                MakeGarbageCollected(GCedHeapVector::<u32>::with_size(run.num_characters_));
            run.glyph_data_.SetGraphemes(graphemes);
            let start = (run.start_index_ - result_start_index) as usize;
            let end = start + run.num_characters_ as usize;
            let run_text = String::from_utf16(&text.Span16()[start..end]);
            GraphemesClusterList(&StringView::from(&run_text), unsafe { &mut *graphemes });
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:60-73
    pub fn CharacterData(&self, offset: u32) -> Ref<'_, ShapeResultCharacterData> {
        debug_assert!(offset >= self.StartIndex() && offset < self.EndIndex());
        debug_assert!(!self.character_position_.borrow().empty());
        Ref::map(self.character_position_.borrow(), |data| {
            &data[(offset - self.StartIndex()) as usize]
        })
    }

    pub fn CharacterDataMut(&self, offset: u32) -> RefMut<'_, ShapeResultCharacterData> {
        debug_assert!(offset >= self.StartIndex() && offset < self.EndIndex());
        debug_assert!(!self.character_position_.borrow().empty());
        let index = (offset - self.StartIndex()) as usize;
        RefMut::map(self.character_position_.borrow_mut(), |data| {
            &mut data[index]
        })
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:75-95
    pub fn NextSafeToBreakOffset(&self, index: u32) -> u32 {
        for (run_index, member) in self.runs_.iter().enumerate() {
            let Some(run) = (unsafe { member.Get().as_ref() }) else {
                continue;
            };
            let run_start = run.start_index_;
            if index >= run_start {
                let offset = index - run_start;
                if offset < run.num_characters_ {
                    return run.NextSafeToBreakOffset(offset).wrapping_add(run_start);
                }
                if self.IsRtl() {
                    if run_index == 0 {
                        return run_start.wrapping_add(run.num_characters_);
                    }
                    return unsafe { &*self.runs_[run_index - 1].Get() }.start_index_;
                }
            } else if self.IsLtr() {
                return run_start;
            }
        }
        self.EndIndex()
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:97-117
    pub fn PreviousSafeToBreakOffset(&self, index: u32) -> u32 {
        for run_index in (0..self.runs_.len()).rev() {
            let Some(run) = (unsafe { self.runs_[run_index].Get().as_ref() }) else {
                continue;
            };
            let run_start = run.start_index_;
            if index >= run_start {
                let offset = index - run_start;
                if offset <= run.num_characters_ {
                    return run
                        .PreviousSafeToBreakOffset(offset)
                        .wrapping_add(run_start);
                }
                if self.IsLtr() {
                    return run_start.wrapping_add(run.num_characters_);
                }
            } else if self.IsRtl() {
                if run_index + 1 == self.runs_.len() {
                    return run.start_index_;
                }
                let previous_run = unsafe { &*self.runs_[run_index + 1].Get() };
                return previous_run
                    .start_index_
                    .wrapping_add(previous_run.num_characters_);
            }
        }
        self.StartIndex()
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:119-162
    pub fn OffsetForPositionInternal(
        &self,
        target_x: f32,
        break_glyphs: BreakGlyphsOption,
        result: &mut GlyphIndexResult,
    ) {
        if target_x <= 0.0 {
            if self.IsRtl() {
                result.left_character_index = self.NumCharacters();
                result.right_character_index = self.NumCharacters();
            }
            return;
        }
        let mut characters_so_far = if self.IsRtl() {
            self.NumCharacters()
        } else {
            0
        };
        let mut current_x = 0.0;
        for member in &self.runs_ {
            let Some(run) = (unsafe { member.Get().as_ref() }) else {
                continue;
            };
            if self.IsRtl() {
                characters_so_far = characters_so_far.wrapping_sub(run.num_characters_);
            }
            let next_x = current_x + run.width_;
            let offset_for_run = target_x - current_x;
            if offset_for_run >= 0.0 && offset_for_run < run.width_ {
                run.CharacterIndexForXPosition(offset_for_run, break_glyphs, result);
                result.left_character_index =
                    result.left_character_index.wrapping_add(characters_so_far);
                result.right_character_index =
                    result.right_character_index.wrapping_add(characters_so_far);
                result.origin_x += current_x;
                return;
            }
            if self.IsLtr() {
                characters_so_far = characters_so_far.wrapping_add(run.num_characters_);
            }
            current_x = next_x;
        }
        if self.IsRtl() {
            result.left_character_index = 0;
            result.right_character_index = 0;
        } else {
            result.left_character_index =
                result.left_character_index.wrapping_add(characters_so_far);
            result.right_character_index =
                result.right_character_index.wrapping_add(characters_so_far);
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:164-171
    pub fn OffsetForPosition(&self, x: f32) -> u32 {
        let mut result = GlyphIndexResult::default();
        self.OffsetForPositionInternal(x, BreakGlyphsOption(true), &mut result);
        if self.IsLtr() {
            return result.left_character_index;
        }
        if x == result.origin_x {
            result.left_character_index
        } else {
            result.right_character_index
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:173-189
    pub fn CaretOffsetForHitTest(&self, x: f32, text: &StringView) -> u32 {
        self.EnsureGraphemes(text);
        let mut result = GlyphIndexResult::default();
        self.OffsetForPositionInternal(x, BreakGlyphsOption(true), &mut result);
        if x - result.origin_x <= result.advance / 2.0 {
            result.left_character_index
        } else {
            result.right_character_index
        }
    }

    pub fn CaretPositionForOffset(
        &self,
        offset: u32,
        text: &StringView,
        adjust: AdjustMidCluster,
    ) -> f32 {
        self.EnsureGraphemes(text);
        self.PositionForOffset(offset, adjust)
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:191-199
    pub fn OffsetToFit(&self, x: f32, line_direction: TextDirection) -> u32 {
        let mut result = GlyphIndexResult::default();
        self.OffsetForPositionInternal(x, BreakGlyphsOption(false), &mut result);
        if IsLtr(line_direction) {
            return result.left_character_index;
        }
        if x == result.origin_x {
            result.left_character_index
        } else {
            result.right_character_index
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:201-217
    fn ForEachGlyphImpl<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
        initial_advance: f32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
        run: &ShapeResultRun,
    ) -> f32 {
        let mut glyph_offsets = run.glyph_data_.GetOffsets::<HAS_NON_ZERO_GLYPH_OFFSETS>();
        let mut total_advance = InlineLayoutUnit::FromFloatRound(initial_advance);
        let is_horizontal = run.IsHorizontal();
        for glyph_data in &run.glyph_data_ {
            glyph_callback(
                context,
                run.start_index_.wrapping_add(glyph_data.character_index()),
                glyph_data.glyph() as Glyph,
                glyph_offsets.get(),
                total_advance.ToFloat(),
                is_horizontal,
                run.canvas_rotation_,
                run.font_data_.Get(),
            );
            total_advance += glyph_data.advance.To::<16, i64>();
            glyph_offsets.advance();
        }
        total_advance.ToFloat()
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:219-232
    pub fn ForEachGlyph(
        &self,
        initial_advance: f32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
    ) -> f32 {
        let mut total_advance = initial_advance;
        for member in &self.runs_ {
            let run = unsafe { &*member.Get() };
            total_advance = if run.glyph_data_.HasNonZeroOffsets() {
                self.ForEachGlyphImpl::<true>(total_advance, glyph_callback, context, run)
            } else {
                self.ForEachGlyphImpl::<false>(total_advance, glyph_callback, context, run)
            };
        }
        total_advance
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:234-275
    fn ForEachGlyphImplRange<const HAS_NON_ZERO_GLYPH_OFFSETS: bool>(
        &self,
        initial_advance: f32,
        from: u32,
        to: u32,
        index_offset: u32,
        glyph_callback: GlyphCallback,
        context: *mut c_void,
        run: &ShapeResultRun,
    ) -> f32 {
        let mut glyph_offsets = run.glyph_data_.GetOffsets::<HAS_NON_ZERO_GLYPH_OFFSETS>();
        let mut total_advance = InlineLayoutUnit::FromFloatRound(initial_advance);
        let run_start = run.start_index_.wrapping_add(index_offset);
        let is_horizontal = run.IsHorizontal();
        let font_data = run.font_data_.Get();
        if run.IsLtr() {
            for glyph_data in &run.glyph_data_ {
                let character_index = run_start.wrapping_add(glyph_data.character_index());
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
            for glyph_data in &run.glyph_data_ {
                let character_index = run_start.wrapping_add(glyph_data.character_index());
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

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:277-295
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
        for member in &self.runs_ {
            let run = unsafe { &*member.Get() };
            total_advance = if run.glyph_data_.HasNonZeroOffsets() {
                self.ForEachGlyphImplRange::<true>(
                    total_advance,
                    from,
                    to,
                    index_offset,
                    glyph_callback,
                    context,
                    run,
                )
            } else {
                self.ForEachGlyphImplRange::<false>(
                    total_advance,
                    from,
                    to,
                    index_offset,
                    glyph_callback,
                    context,
                    run,
                )
            };
        }
        total_advance
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdvanceType {
    kUnknown,
    kVariable,
    kMono,
}

impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_position.cc:297-368
    pub fn ComputePositionData<const RTL: bool>(&self, allow_compaction: bool) {
        let mut next_character_index = 0u32;
        let mut total_advance = InlineLayoutUnit::new();
        let mut last_x_position = LayoutUnit::new();
        let mut advance_type = if RTL {
            AdvanceType::kVariable
        } else {
            AdvanceType::kUnknown
        };
        let mut mono_advance = TextRunLayoutUnit::new();
        let mut mono_glyph_count = 0u32;
        let mut character_position = self.character_position_.borrow_mut();
        for run_ptr in &self.runs_ {
            let Some(run) = (unsafe { run_ptr.Get().as_ref() }) else {
                continue;
            };
            debug_assert_eq!(self.IsRtl(), run.IsRtl());
            for glyph_data in &run.glyph_data_ {
                debug_assert!(run.start_index_ >= self.start_index_);
                let logical_index = run
                    .start_index_
                    .wrapping_add(glyph_data.character_index())
                    .wrapping_sub(self.start_index_);
                debug_assert!(logical_index < self.num_characters_);
                let character_index = if RTL {
                    self.num_characters_ - logical_index - 1
                } else {
                    logical_index
                };
                if next_character_index <= character_index {
                    if next_character_index < character_index {
                        let x_position = if !RTL {
                            last_x_position
                        } else {
                            total_advance.ToCeil::<6, i32>()
                        };
                        for i in next_character_index..character_index {
                            character_position[i as usize].SetCachedData(x_position, false, false);
                        }
                    }
                    last_x_position = total_advance.ToCeil::<6, i32>();
                    character_position[character_index as usize].SetCachedData(
                        last_x_position,
                        true,
                        glyph_data.IsSafeToBreakBefore(),
                    );
                }
                match advance_type {
                    AdvanceType::kUnknown => {
                        if character_index == mono_glyph_count {
                            mono_advance = glyph_data.advance;
                            advance_type = AdvanceType::kMono;
                        } else {
                            advance_type = AdvanceType::kVariable;
                        }
                    }
                    AdvanceType::kMono => {
                        if mono_advance != glyph_data.advance || character_index != mono_glyph_count
                        {
                            advance_type = AdvanceType::kVariable;
                        }
                    }
                    AdvanceType::kVariable => {}
                }
                mono_glyph_count = mono_glyph_count.wrapping_add(1);
                total_advance += glyph_data.advance.To::<16, i64>();
                next_character_index = character_index.wrapping_add(1);
            }
        }
        if next_character_index < self.num_characters_ {
            let x_position = if !RTL {
                last_x_position
            } else {
                total_advance.ToCeil::<6, i32>()
            };
            for i in next_character_index..self.num_characters_ {
                character_position[i as usize].SetCachedData(x_position, false, false);
            }
        }
        if allow_compaction
            && advance_type == AdvanceType::kMono
            && !RTL
            && self.NumCharacters() > 1
            && mono_glyph_count == self.num_characters_
        {
            character_position
                .first_mut()
                .unwrap()
                .set_advance(mono_advance);
            character_position.Shrink(1);
            character_position.shrink_to_fit();
        }
        self.width_.set(total_advance.ToFloat());
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:370-385
    pub fn EnsurePositionData(&self, allow_compaction: bool) {
        {
            let data = self.character_position_.borrow();
            if !data.empty() && (allow_compaction || data.size() == self.num_characters_) {
                return;
            }
        }
        *self.character_position_.borrow_mut() = HeapVector::with_size(self.num_characters_);
        self.RecalcCharacterPositions(allow_compaction);
    }

    pub fn RecalcCharacterPositions(&self, allow_compaction: bool) {
        debug_assert!(!self.character_position_.borrow().empty());
        if self.IsLtr() {
            self.ComputePositionData::<false>(allow_compaction);
        } else {
            self.ComputePositionData::<true>(allow_compaction);
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:387-435
    pub fn CachedOffsetForPosition(&self, x: LayoutUnit) -> u32 {
        debug_assert!(!self.character_position_.borrow().empty());
        let rtl = self.IsRtl();
        let length = self.NumCharacters();
        if x <= 0 {
            return if !rtl { 0 } else { length };
        }
        if x >= self.width_.get() {
            return if !rtl { length } else { 0 };
        }
        let data = self.character_position_.borrow();
        if data.size() == 1 && self.NumCharacters() > 1 {
            let advance_raw = i64::from(data.first().unwrap().advance().RawValue());
            if advance_raw <= 0 {
                return 0;
            }
            let x_raw = i64::from(x.RawValue())
                << (InlineLayoutUnit::kFractionalBits - LayoutUnit::kFractionalBits);
            let mut offset = (x_raw / advance_raw) as u32;
            if offset >= length {
                offset = length - 1;
            }
            drop(data);
            while offset + 1 < length && self.CachedPositionForOffset(offset + 1) <= x {
                offset += 1;
            }
            while offset > 0 && self.CachedPositionForOffset(offset) > x {
                offset -= 1;
            }
            return offset;
        }
        debug_assert_eq!(data.size(), length);
        let mut low = 0u32;
        let mut high = length - 1;
        while low <= high {
            let midpoint = low + (high - low) / 2;
            let x_position = data[midpoint as usize].x_position();
            if x_position <= x
                && (midpoint + 1 == length || data[midpoint as usize + 1].x_position() > x)
            {
                if !rtl {
                    return midpoint;
                }
                return if x_position == x {
                    length - midpoint
                } else {
                    length - midpoint - 1
                };
            }
            if x < x_position {
                high = midpoint.wrapping_sub(1);
            } else {
                low = midpoint + 1;
            }
        }
        0
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:437-464
    pub fn CachedPositionForOffset(&self, offset: u32) -> LayoutUnit {
        debug_assert!(offset <= self.num_characters_);
        let data = self.character_position_.borrow();
        debug_assert!(!data.empty());
        let rtl = self.IsRtl();
        let length = self.NumCharacters();
        if !rtl {
            if offset < length {
                if data.size() == 1 && self.NumCharacters() > 1 {
                    let raw =
                        i64::from(data.first().unwrap().advance().RawValue()) * i64::from(offset);
                    let position = InlineLayoutUnit::FromRawValue(raw);
                    return position.ToCeil::<6, i32>();
                }
                return data[offset as usize].x_position();
            }
        } else {
            if offset >= length {
                return LayoutUnit::new();
            }
            for visual_offset in (length - offset - 1)..length {
                if data[visual_offset as usize].is_cluster_base {
                    return if visual_offset + 1 < length {
                        data[visual_offset as usize + 1].x_position()
                    } else {
                        LayoutUnit::FromFloatCeil(self.width_.get())
                    };
                }
            }
        }
        LayoutUnit::FromFloatCeil(self.width_.get())
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:466-472
    pub fn CachedWidth(&self, start_offset: u32, end_offset: u32) -> LayoutUnit {
        let adjust = self.StartIndex();
        let start = self.CachedPositionForOffset(start_offset.wrapping_sub(adjust));
        let end = self.CachedPositionForOffset(end_offset.wrapping_sub(adjust));
        if self.IsLtr() {
            end - start
        } else {
            start - end
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:474-488
    pub fn CachedNextSafeToBreakOffset(&self, offset: u32) -> u32 {
        if self.IsRtl() {
            return self.NextSafeToBreakOffset(offset);
        }
        let data = self.character_position_.borrow();
        debug_assert!(!data.empty());
        let adjusted = offset.wrapping_sub(self.start_index_);
        let length = self.NumCharacters();
        debug_assert!(adjusted < length);
        if data.size() == 1 && self.NumCharacters() > 1 {
            return self.start_index_.wrapping_add(adjusted);
        }
        for i in adjusted..length {
            if data[i as usize].safe_to_break_before {
                return self.start_index_.wrapping_add(i);
            }
        }
        self.start_index_.wrapping_add(length)
    }

    // cpp: font_engine/fonts/shaping/shape_result_position.cc:490-506
    pub fn CachedPreviousSafeToBreakOffset(&self, offset: u32) -> u32 {
        if self.IsRtl() {
            return self.PreviousSafeToBreakOffset(offset);
        }
        let data = self.character_position_.borrow();
        debug_assert!(!data.empty());
        let adjusted = offset.wrapping_sub(self.start_index_);
        let length = self.NumCharacters();
        debug_assert!(adjusted <= length);
        if adjusted >= length {
            return self.start_index_.wrapping_add(length);
        }
        if data.size() == 1 && self.NumCharacters() > 1 {
            return self.start_index_.wrapping_add(adjusted);
        }
        for i in (0..=adjusted).rev() {
            if data[i as usize].safe_to_break_before {
                return self.start_index_.wrapping_add(i);
            }
        }
        self.start_index_
    }
}
