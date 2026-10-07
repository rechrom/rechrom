// cpp: font_engine/fonts/shaping/glyph_index_result.h:12-25
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GlyphIndexResult {
    pub left_character_index: u32,
    pub right_character_index: u32,
    pub origin_x: f32,
    pub advance: f32,
}
