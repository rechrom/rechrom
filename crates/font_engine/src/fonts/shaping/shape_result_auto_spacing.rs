#![allow(non_snake_case)]

use super::glyph_data::GlyphOffset;
use super::shape_result::{OffsetWithSpacing, ShapeResult};
use foundation::TextDirection;

impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:11-19
    pub fn HasAutoSpacingAfter(&self, offset: u32) -> bool {
        let character_position = self.character_position_.borrow();
        if !character_position.empty() && offset >= self.StartIndex() && offset < self.EndIndex() {
            if character_position.size() == 1 && self.NumCharacters() > 1 {
                return false;
            }
            return character_position[(offset - self.StartIndex()) as usize]
                .has_auto_spacing_after;
        }
        false
    }

    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:21-23
    pub fn HasAutoSpacingBefore(&self, offset: u32) -> bool {
        self.HasAutoSpacingAfter(offset.wrapping_sub(1))
    }

    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:25-35
    pub fn ApplyTextAutoSpacing(&mut self, offsets: &[OffsetWithSpacing]) {
        debug_assert!(!offsets.is_empty());
        self.EnsurePositionData(false);
        if self.IsLtr() {
            self.ApplyTextAutoSpacingCore::<true, _>(offsets.iter().copied());
        } else {
            self.ApplyTextAutoSpacingCore::<false, _>(offsets.iter().rev().copied());
        }
        self.RecalcCharacterPositions(false);
    }

    // `TextDirection` template parameter selects the same glyph-side rule;
    // Rust's const bool keeps the C++ instantiations separate.
    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:37-111
    fn ApplyTextAutoSpacingCore<const LTR: bool, I>(&mut self, offsets: I)
    where
        I: Iterator<Item = OffsetWithSpacing>,
    {
        let mut current = offsets.peekable();
        debug_assert!(current.peek().is_some());
        if current
            .peek()
            .is_some_and(|item| item.offset == self.StartIndex())
        {
            if self.Direction() == TextDirection::kRtl {
                current.next();
            } else {
                for member in &self.runs_ {
                    let Some(run) = (unsafe { member.Get().as_mut() }) else {
                        continue;
                    };
                    debug_assert_eq!(run.start_index_, current.peek().unwrap().offset);
                    let mut last_glyph = 0;
                    let amount = current.peek().unwrap().spacing;
                    let glyph_offset = if run.IsHorizontal() {
                        GlyphOffset::new(amount, 0.0)
                    } else {
                        GlyphOffset::new(0.0, amount)
                    };
                    for i in 0..run.NumGlyphs() {
                        if run.glyph_data_[i].character_index() != 0 {
                            break;
                        }
                        run.glyph_data_.SetOffsetAt(i, glyph_offset);
                        last_glyph = i;
                    }
                    run.glyph_data_[last_glyph].AddAdvanceFloat(amount);
                    self.has_vertical_offsets_ |= glyph_offset.y() != 0.0;
                    run.width_ += amount;
                    current.next();
                    break;
                }
            }
        }

        for member in &self.runs_ {
            let Some(run) = (unsafe { member.Get().as_mut() }) else {
                continue;
            };
            let Some(item) = current.peek() else {
                break;
            };
            let mut offset = item.offset;
            debug_assert!(offset >= run.start_index_);
            let mut offset_in_run = offset - run.start_index_;
            if offset_in_run > run.num_characters_ {
                continue;
            }
            let mut total_space = 0.0;
            for i in 0..run.NumGlyphs() {
                let next_index = if i + 1 < run.glyph_data_.size() {
                    run.glyph_data_[i + 1].character_index()
                } else {
                    run.num_characters_
                };
                let add = if LTR {
                    next_index >= offset_in_run
                } else if offset_in_run == run.num_characters_ {
                    i == run.NumGlyphs() - 1
                } else {
                    next_index < offset_in_run
                };
                if !add {
                    continue;
                }
                let glyph = &mut run.glyph_data_[i];
                let spacing = current.peek().unwrap().spacing;
                glyph.AddAdvanceFloat(spacing);
                total_space += spacing;
                let mut data = self.CharacterDataMut(offset.wrapping_sub(1));
                debug_assert!(!data.has_auto_spacing_after);
                data.has_auto_spacing_after = true;
                drop(data);
                current.next();
                let Some(item) = current.peek() else {
                    break;
                };
                offset = item.offset;
                debug_assert!(offset >= run.start_index_);
                offset_in_run = offset - run.start_index_;
            }
            run.width_ += total_space;
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:113-136
    pub fn UnapplyAutoSpacing(
        &self,
        spacing_width: f32,
        start_offset: u32,
        break_offset: u32,
    ) -> *const Self {
        debug_assert!(start_offset >= self.StartIndex());
        debug_assert!(break_offset > start_offset);
        debug_assert!(break_offset <= self.EndIndex());
        debug_assert!(self.HasAutoSpacingBefore(break_offset));
        let sub_range = self.SubRange(start_offset, break_offset);
        let result = unsafe { &mut *sub_range };
        for member in result.runs_.iter().rev() {
            let run = unsafe { &mut *member.Get() };
            if run.NumGlyphs() == 0 {
                continue;
            }
            let last = run.glyph_data_.back_mut();
            debug_assert!(last.advance.ToFloat() >= spacing_width);
            last.AddAdvanceFloat(-spacing_width);
            run.width_ -= spacing_width;
            result.width_.set(result.width_.get() - spacing_width);
            break;
        }
        sub_range
    }

    // cpp: font_engine/fonts/shaping/shape_result_auto_spacing.cc:130-156
    pub fn AdjustOffsetForAutoSpacing(
        &self,
        spacing_width: f32,
        mut offset: u32,
        mut position: f32,
    ) -> u32 {
        debug_assert!(!self.character_position_.borrow().empty());
        debug_assert!(self.HasAutoSpacingAfter(offset));
        debug_assert!(offset >= self.StartIndex());
        offset -= self.StartIndex();
        debug_assert!(offset < self.NumCharacters());
        if self.IsLtr() {
            position += spacing_width;
            if offset + 1 < self.NumCharacters() {
                if self.character_position_.borrow()[(offset + 1) as usize]
                    .x_position()
                    .ToFloat()
                    <= position
                {
                    offset += 1;
                }
            } else if self.Width() <= position {
                offset = self.NumCharacters();
            }
        } else {
            position -= spacing_width;
            if offset + 1 < self.NumCharacters() {
                if self.character_position_.borrow()[(offset + 1) as usize]
                    .x_position()
                    .ToFloat()
                    >= position
                {
                    offset += 1;
                }
            } else if self.Width() <= -position {
                offset = self.NumCharacters();
            }
        }
        offset + self.StartIndex()
    }
}
