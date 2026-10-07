// cpp: font_engine/fonts/shaping/shape_options.h:15-19
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShapeOptions {
    pub is_line_start: bool,
    pub han_kerning_start: bool,
    pub han_kerning_end: bool,
}
