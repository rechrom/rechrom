#![allow(non_snake_case)]

use super::glyph_data::{GlyphOffset, HarfBuzzRunGlyphData};
use super::shape_result_run::ShapeResultRun;
use foundation::{Member, Traceable, Visitor};

// A GC edge retains the run, while index and size select a contiguous range
// in its glyph allocation. C++ iterator pointers become offsets here.
// cpp: font_engine/fonts/shaping/glyph_data_range.h:14-60
#[derive(Clone, Copy, Default)]
pub struct GlyphDataRange {
    run_: Member<ShapeResultRun>,
    index_: u32,
    size_: u32,
}

impl GlyphDataRange {
    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:13-14
    pub fn new(run: &ShapeResultRun) -> Self {
        Self {
            run_: Member::from_ptr(run as *const _ as *mut _),
            index_: 0,
            size_: run.NumGlyphs(),
        }
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:16-28
    fn subset(&self, begin: u32, end: u32) -> Self {
        let run = unsafe { self.run_.Get().as_ref() }.expect("range has no run");
        assert!(begin >= self.index_ && begin <= run.NumGlyphs());
        assert!(end >= begin && end <= run.NumGlyphs());
        Self {
            run_: self.run_,
            index_: begin,
            size_: end - begin,
        }
    }

    pub fn size(&self) -> u32 {
        self.size_
    }

    pub fn IsEmpty(&self) -> bool {
        self.size_ == 0
    }

    pub fn GetRun(&self) -> *const ShapeResultRun {
        self.run_.Get()
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:30-33
    pub fn Glyphs(&self) -> &[HarfBuzzRunGlyphData] {
        let Some(run) = (unsafe { self.run_.Get().as_ref() }) else {
            return &[];
        };
        let start = self.index_ as usize;
        &run.glyph_data_.data_[start..start + self.size_ as usize]
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:35-41
    pub fn begin(&self) -> *const HarfBuzzRunGlyphData {
        if self.run_.Get().is_null() {
            std::ptr::null()
        } else {
            self.Glyphs().as_ptr()
        }
    }

    pub fn end(&self) -> *const HarfBuzzRunGlyphData {
        self.begin().wrapping_add(self.size_ as usize)
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:43-52
    pub fn HasOffsets(&self) -> bool {
        unsafe { self.run_.Get().as_ref() }.is_some_and(|run| run.glyph_data_.HasNonZeroOffsets())
    }

    pub fn Offsets(&self) -> &[GlyphOffset] {
        if !self.HasOffsets() {
            return &[];
        }
        let run = unsafe { &*self.run_.Get() };
        let start = self.index_ as usize;
        &run.glyph_data_.Offsets()[start..start + self.size_ as usize]
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:56-93
    pub fn FindGlyphDataRange(
        &self,
        is_rtl: bool,
        start_character_index: u32,
        end_character_index: u32,
    ) -> Self {
        let glyphs = self.Glyphs();
        let count = glyphs.len();
        if !is_rtl {
            let start =
                glyphs.partition_point(|glyph| glyph.character_index() < start_character_index);
            if start == count {
                let end = self.index_ + count as u32;
                return self.subset(end, end);
            }
            let end = start
                + glyphs[start..]
                    .partition_point(|glyph| glyph.character_index() < end_character_index);
            return self.subset(self.index_ + start as u32, self.index_ + end as u32);
        }

        // Reverse iterators traverse RTL glyphs in increasing logical index.
        let reverse_lower_bound = |from: usize, target: u32| -> usize {
            let (mut low, mut high) = (from, count);
            while low < high {
                let middle = low + (high - low) / 2;
                if glyphs[count - 1 - middle].character_index() < target {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            low
        };
        let start = reverse_lower_bound(0, start_character_index);
        if start == count {
            return self.subset(self.index_, self.index_);
        }
        let end = reverse_lower_bound(start, end_character_index);
        self.subset(
            self.index_ + (count - end) as u32,
            self.index_ + (count - start) as u32,
        )
    }
}

// cpp: font_engine/fonts/shaping/glyph_data_range.cc:95-97
impl Traceable for GlyphDataRange {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.run_);
    }
}
