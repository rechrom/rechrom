#![allow(non_snake_case)]

use crate::fonts::glyph::Glyph;
use foundation::Vector;

// cpp: font_engine/fonts/opentype/open_type_math_stretch_data.h:14-19
pub struct OpenTypeMathStretchData;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchAxis {
    Horizontal = 0,
    Vertical = 1,
}

// cpp: font_engine/fonts/opentype/open_type_math_stretch_data.h:23-33
pub type GlyphVariantRecord = Glyph;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GlyphPartRecord {
    pub glyph: Glyph,
    pub start_connector_length: f32,
    pub end_connector_length: f32,
    pub full_advance: f32,
    pub is_extender: bool,
}

// cpp: font_engine/fonts/opentype/open_type_math_stretch_data.h:35-42
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AssemblyParameters {
    pub connector_overlap: f32,
    pub repetition_count: u32,
    pub glyph_count: u32,
    pub stretch_size: f32,
    pub parts: Vector<GlyphPartRecord>,
}
