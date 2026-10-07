#![allow(non_snake_case)]

use super::shape_result::ShapeResult;
use super::shape_result_spacing::ShapeResultSpacing;
use foundation::blink_geometry::geometry::{InlineLayoutUnit, TextRunLayoutUnit};
use foundation::{LayoutUnit, TextJustify};

// The C++ hb_script_t values are ISO 15924 four-byte tags in
// harfbuzz/hb-script-list.h; retain those exact values at this boundary.
// cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:11-26
fn IsCursiveScript(script: u32) -> bool {
    matches!(
        script,
        s if s == u32::from_be_bytes(*b"Arab")
            || s == u32::from_be_bytes(*b"Rohg")
            || s == u32::from_be_bytes(*b"Mand")
            || s == u32::from_be_bytes(*b"Mong")
            || s == u32::from_be_bytes(*b"Nkoo")
            || s == u32::from_be_bytes(*b"Phag")
            || s == u32::from_be_bytes(*b"Syrc")
    )
}

impl ShapeResult {
    // cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:29-66
    fn ApplySpacingOrExpansion(
        &mut self,
        spacing: &mut ShapeResultSpacing,
        method: Option<TextJustify>,
        text_start_offset: i32,
    ) -> TextRunLayoutUnit {
        let mut total_advance = 0.0;
        let mut spacing_after = TextRunLayoutUnit::new();
        for member in &self.runs_ {
            let Some(run) = (unsafe { member.Get().as_mut() }) else {
                continue;
            };
            let run_start = run.start_index_.wrapping_add_signed(text_start_offset);
            let mut run_advance = InlineLayoutUnit::new();
            for i in 0..run.glyph_data_.size() {
                if i + 1 < run.glyph_data_.size()
                    && run.glyph_data_[i].character_index()
                        == run.glyph_data_[i + 1].character_index()
                {
                    run_advance += run.glyph_data_[i].advance.To::<16, i64>();
                    continue;
                }
                let index = run_start.wrapping_add(run.glyph_data_[i].character_index());
                let mut spacing_before = TextRunLayoutUnit::new();
                if let Some(method) = method {
                    (spacing_before, spacing_after) =
                        spacing.ComputeExpansion(method, index, IsCursiveScript(run.script_));
                } else {
                    spacing_after = spacing.ComputeSpacing(index, IsCursiveScript(run.script_));
                }
                run.glyph_data_[i].AddAdvance(spacing_before + spacing_after);
                run_advance += run.glyph_data_[i].advance.To::<16, i64>();
                if spacing_before.RawValue() != 0 {
                    let offset = spacing_before.ToFloat();
                    if run.IsHorizontal() {
                        run.glyph_data_.AddOffsetWidthAt(i, offset);
                    } else {
                        run.glyph_data_.AddOffsetHeightAt(i, offset);
                        self.has_vertical_offsets_ = true;
                    }
                }
            }
            run.width_ = run_advance.ToFloat();
            total_advance += run.width_;
        }
        self.width_.set(total_advance);
        spacing_after
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:68-73
    pub fn ApplySpacing(&mut self, spacing: &mut ShapeResultSpacing, offset: i32) {
        assert!(!self.is_applied_spacing_);
        self.is_applied_spacing_ = true;
        self.ApplySpacingOrExpansion(spacing, None, offset);
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:75-81
    pub fn ApplyExpansion(
        &mut self,
        method: TextJustify,
        spacing: &mut ShapeResultSpacing,
        offset: i32,
    ) -> TextRunLayoutUnit {
        assert!(!self.is_applied_spacing_);
        self.is_applied_spacing_ = true;
        self.ApplySpacingOrExpansion(spacing, Some(method), offset)
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:83-107
    pub fn ApplyLeadingExpansion(&mut self, expansion: LayoutUnit) {
        if expansion <= LayoutUnit::new() {
            return;
        }
        for member in &self.runs_ {
            let Some(run) = (unsafe { member.Get().as_mut() }) else {
                continue;
            };
            for i in 0..run.glyph_data_.size() {
                if i + 1 < run.glyph_data_.size()
                    && run.glyph_data_[i].character_index()
                        == run.glyph_data_[i + 1].character_index()
                {
                    continue;
                }
                let amount = expansion.To::<16, i32>();
                run.glyph_data_[i].AddAdvance(amount);
                let value = amount.ToFloat();
                run.width_ += value;
                self.width_.set(self.width_.get() + value);
                if run.IsHorizontal() {
                    run.glyph_data_.AddOffsetWidthAt(i, value);
                } else {
                    run.glyph_data_.AddOffsetHeightAt(i, value);
                    self.has_vertical_offsets_ = true;
                }
                return;
            }
        }
        panic!("ApplyLeadingExpansion requires a glyph");
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing_apply.cc:109-122
    pub fn ApplyTrailingExpansion(&mut self, expansion: LayoutUnit) {
        if expansion <= LayoutUnit::new() {
            return;
        }
        for member in self.runs_.iter().rev() {
            let Some(run) = (unsafe { member.Get().as_mut() }) else {
                continue;
            };
            if run.glyph_data_.IsEmpty() {
                continue;
            }
            let glyph = run.glyph_data_.back_mut();
            let amount = expansion.To::<16, i32>();
            glyph.AddAdvance(amount);
            let value = amount.ToFloat();
            run.width_ += value;
            self.width_.set(self.width_.get() + value);
            return;
        }
        panic!("ApplyTrailingExpansion requires a glyph");
    }
}
