#![allow(non_snake_case)]

use crate::fonts::font_fallback_priority::FontFallbackPriority;
use crate::fonts::orientation_iterator_types::RenderOrientation;

pub use super::run_segmenter::RunSegmenter;

// cpp: font_engine/fonts/shaping/run_segmenter.h:32-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunSegmenterRange {
    pub start: u32,
    pub end: u32,
    // ICU's UScriptCode is an int32 enum. Preserve the invalid -1 value.
    pub script: i32,
    pub render_orientation: RenderOrientation,
    pub font_fallback_priority: FontFallbackPriority,
}

impl Default for RunSegmenterRange {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            script: -1, // USCRIPT_INVALID_CODE
            render_orientation: RenderOrientation::kOrientationKeep,
            font_fallback_priority: FontFallbackPriority::kText,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_defaults_match_source() {
        let range = RunSegmenterRange::default();
        assert_eq!((range.start, range.end, range.script), (0, 0, -1));
        assert_eq!(
            range.render_orientation,
            RenderOrientation::kOrientationKeep
        );
        assert_eq!(range.font_fallback_priority, FontFallbackPriority::kText);
    }
}
