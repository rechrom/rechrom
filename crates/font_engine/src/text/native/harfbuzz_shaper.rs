#![allow(non_snake_case)]

use super::rustybuzz_shaper::shape_run;
use crate::text_shaper::{ShapeRequest, ShapedRun, TextShaper};

// The C++ class is local to this implementation file and implements the
// package's TextShaper interface by virtual dispatch.
// cpp: font_engine/text/native/harfbuzz_shaper.cc:10-25
struct HarfBuzzShaper;

impl TextShaper for HarfBuzzShaper {
    // cpp: font_engine/text/native/harfbuzz_shaper.cc:13-24
    fn Shape(&self, input: &ShapeRequest<'_>) -> ShapedRun {
        assert!(
            !input.font_bytes.is_empty()
                && input.font_bytes.len() <= u32::MAX as usize
                && input.utf8.len() <= i32::MAX as usize
                && input.language.len() <= i32::MAX as usize
                && input.script.len() <= i32::MAX as usize
                && input.font_size.is_finite()
                && input.font_size >= 1.0 / 64.0
                && input.font_size <= 16_000_000.0,
            "invalid font or text shaping request"
        );
        shape_run(
            input.font_bytes,
            input.face_index,
            input.font_size,
            &[],
            input.utf8,
            input.direction,
            input.language,
            input.script,
        )
    }
}

// C++ unique_ptr<TextShaper> maps to Box<dyn TextShaper>.
// cpp: font_engine/text/native/harfbuzz_shaper.h:5-9
// cpp: font_engine/text/native/harfbuzz_shaper.cc:27-30
pub fn CreateHarfBuzzShaper() -> Box<dyn TextShaper> {
    Box::new(HarfBuzzShaper)
}
