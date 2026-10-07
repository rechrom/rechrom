// C++: font_engine/fonts/text_rendering_mode.h. ToString and ToStringForIdl
// have no definitions in the supplied source tree.
// cpp: font_engine/fonts/text_rendering_mode.h:35-40
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextRenderingMode {
    kAutoTextRendering,
    kOptimizeSpeed,
    kOptimizeLegibility,
    kGeometricPrecision,
}
