// The C++ text paint header forward-declares this font-engine-owned class.
// Keep the established import path while the real class lives beside its
// shape_result_view.h/.cc translation.
// cpp: font_engine/fonts/text_fragment_paint_info.h:14-14
pub use super::shape_result_view::ShapeResultView;

// Callback signature shared by ShapeResultView glyph traversal and paint.
use crate::fonts::canvas_rotation_in_vertical::CanvasRotationInVertical;
use crate::fonts::simple_font_data::SimpleFontData;
use foundation::gfx;
use std::ffi::c_void;

pub type GlyphCallback = fn(
    *mut c_void,
    u32,
    u16,
    gfx::Vector2dF,
    f32,
    bool,
    CanvasRotationInVertical,
    *const SimpleFontData,
);

// These classes cross the layout package boundary by pointer/reference only.
// Their data and executable methods remain in their respective source files.
// cpp: font_engine/fonts/shaping/harfbuzz_shaper.h:46-46
pub use super::harfbuzz_shaper::HarfBuzzShaper;
// cpp: font_engine/fonts/shaping/shape_result_spacing.h:22-22
pub use super::shape_result_spacing::ShapeResultSpacing;
