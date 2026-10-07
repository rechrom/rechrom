// cpp: layoutng/internal/svg_length_adjust_type.h:25-29
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SVGLengthAdjustType {
    kSVGLengthAdjustUnknown = 0,
    kSVGLengthAdjustSpacing = 1,
    kSVGLengthAdjustSpacingAndGlyphs = 2,
}
