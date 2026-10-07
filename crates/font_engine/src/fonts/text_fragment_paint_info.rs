#![allow(non_snake_case)]

use super::shaping::forward::ShapeResultView;
use foundation::StringView;

// The source keeps text and shape context while painting only [from, to).
// StringView is owned at the Rust boundary, preserving UTF-16 code-unit offsets.
// cpp: font_engine/fonts/text_fragment_paint_info.h:16-37
#[derive(Clone, Debug)]
pub struct TextFragmentPaintInfo {
    pub text: StringView,
    pub from: u32,
    pub to: u32,
    pub shape_result: *const ShapeResultView,
    pub text_fit_scaling_factor: f32,
}

impl TextFragmentPaintInfo {
    // cpp: font_engine/fonts/text_fragment_paint_info.cc:9-18
    pub fn Slice(&self, slice_from: u32, slice_to: u32) -> Self {
        debug_assert!(self.from <= slice_from);
        debug_assert!(slice_from <= slice_to);
        debug_assert!(slice_to <= self.to);
        let mut result = self.clone();
        result.from = slice_from;
        result.to = slice_to;
        result
    }

    // cpp: font_engine/fonts/text_fragment_paint_info.cc:20-23
    pub fn WithStartOffset(&self, start_from: u32) -> Self {
        self.Slice(start_from, self.to)
    }

    // cpp: font_engine/fonts/text_fragment_paint_info.cc:25-28
    pub fn WithEndOffset(&self, end_to: u32) -> Self {
        self.Slice(self.from, end_to)
    }

    // cpp: font_engine/fonts/text_fragment_paint_info.h:23-23
    pub fn Length(&self) -> u32 {
        self.to - self.from
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_keep_full_text_and_shape_context() {
        let original = TextFragmentPaintInfo {
            text: StringView::from("ab😀cd"),
            from: 1,
            to: 5,
            shape_result: std::ptr::null(),
            text_fit_scaling_factor: 0.75,
        };
        let slice = original.Slice(2, 4);
        assert_eq!(slice.Length(), 2);
        assert_eq!(slice.text.length(), 6);
        assert_eq!(slice.text_fit_scaling_factor, 0.75);
        assert_eq!(original.WithStartOffset(3).from, 3);
        assert_eq!(original.WithEndOffset(3).to, 3);
    }
}
