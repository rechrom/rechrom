#![allow(non_snake_case)]

use super::glyph_data::{GlyphOffset, HarfBuzzRunGlyphData};
use super::shape_result_run::ShapeResultRun;
use foundation::{Member, Traceable, Visitor};

#[inline(always)]
fn lower_bound_character_index<const REVERSED: bool>(
    glyphs: &[HarfBuzzRunGlyphData],
    mut low: usize,
    mut high: usize,
    target: u32,
) -> usize {
    debug_assert!(low <= high && high <= glyphs.len());
    while low < high {
        let middle = low + (high - low) / 2;
        let index = if REVERSED {
            glyphs.len() - 1 - middle
        } else {
            middle
        };
        // Chromium's std::lower_bound walks raw contiguous iterators. The
        // search interval proves index < glyphs.len() in both directions.
        let character_index = unsafe { glyphs.get_unchecked(index) }.character_index();
        if character_index < target {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    low
}

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

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size_
    }

    #[inline(always)]
    pub fn IsEmpty(&self) -> bool {
        self.size_ == 0
    }

    #[inline(always)]
    pub fn GetRun(&self) -> *const ShapeResultRun {
        self.run_.Get()
    }

    // cpp: font_engine/fonts/shaping/glyph_data_range.cc:30-33
    #[inline]
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
    #[inline]
    pub fn FindGlyphDataRange(
        &self,
        is_rtl: bool,
        start_character_index: u32,
        end_character_index: u32,
    ) -> Self {
        // cpp: glyph_data_range.cc:83-89. The private C++ range constructor
        // stores these three fields directly; bounds were established by the
        // search below and are not revalidated for every resulting subrange.
        if self.run_.Get().is_null() {
            return Self::default();
        }
        if self.size_ == 0 {
            return Self {
                run_: self.run_,
                index_: self.index_,
                size_: 0,
            };
        }

        let glyphs = self.Glyphs();
        let count = glyphs.len();
        if !is_rtl {
            let start =
                lower_bound_character_index::<false>(glyphs, 0, count, start_character_index);
            if start == count {
                return Self {
                    run_: self.run_,
                    index_: self.index_ + self.size_,
                    size_: 0,
                };
            }
            let end =
                lower_bound_character_index::<false>(glyphs, start, count, end_character_index);
            return Self {
                run_: self.run_,
                index_: self.index_ + start as u32,
                size_: (end - start) as u32,
            };
        }

        // Reverse iterators traverse RTL glyphs in increasing logical index.
        let start = lower_bound_character_index::<true>(glyphs, 0, count, start_character_index);
        if start == count {
            return Self {
                run_: self.run_,
                index_: self.index_,
                size_: 0,
            };
        }
        let end = lower_bound_character_index::<true>(glyphs, start, count, end_character_index);
        Self {
            run_: self.run_,
            index_: self.index_ + (count - end) as u32,
            size_: (end - start) as u32,
        }
    }
}

// cpp: font_engine/fonts/shaping/glyph_data_range.cc:95-97
impl Traceable for GlyphDataRange {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.run_);
    }
}
