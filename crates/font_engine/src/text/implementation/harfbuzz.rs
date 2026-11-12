// C ABI declarations for the HarfBuzz calls used by opentype_font.cc.
// These are glue, not a translated C++ production file.
#![allow(non_camel_case_types, dead_code)]
use std::ffi::{c_char, c_int, c_uint, c_void};

pub enum HbBlob {}
pub enum HbFace {}
pub enum HbFont {}
pub enum HbFontFuncs {}
pub enum HbBuffer {}
pub enum HbUnicodeFuncs {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbVariation {
    pub tag: u32,
    pub value: f32,
}

#[repr(C)]
#[derive(Default)]
pub struct HbVarAxisInfo {
    pub axis_index: u32,
    pub tag: u32,
    pub name_id: u32,
    pub flags: u32,
    pub min_value: f32,
    pub default_value: f32,
    pub max_value: f32,
    pub reserved: u32,
}

#[repr(C)]
pub struct HbNameEntry {
    pub name_id: u32,
    pub var: u32,
    pub language: *const c_void,
}

#[repr(C)]
#[derive(Default)]
pub struct HbGlyphExtents {
    pub x_bearing: i32,
    pub y_bearing: i32,
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[repr(C)]
pub struct HbGlyphInfo {
    pub codepoint: u32,
    pub mask: u32,
    pub cluster: u32,
    pub var1: u32,
    pub var2: u32,
}

#[repr(C)]
#[repr(C)]
pub struct HbGlyphPosition {
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub var: u32,
}

pub type HbDestroyFunc = unsafe extern "C" fn(*mut c_void);
pub type HbFontGetGlyphAdvanceFunc = unsafe extern "C" fn(
    font: *mut HbFont,
    font_data: *mut c_void,
    glyph: u32,
    user_data: *mut c_void,
) -> i32;
pub type HbFontGetGlyphAdvancesFunc = unsafe extern "C" fn(
    font: *mut HbFont,
    font_data: *mut c_void,
    count: c_uint,
    first_glyph: *const u32,
    glyph_stride: c_uint,
    first_advance: *mut i32,
    advance_stride: c_uint,
    user_data: *mut c_void,
);

pub const fn tag(bytes: [u8; 4]) -> u32 {
    u32::from_be_bytes(bytes)
}

pub const MEMORY_MODE_DUPLICATE: c_int = 0;
pub const MEMORY_MODE_READONLY: c_int = 1;
pub const DIRECTION_LTR: c_int = 4;
pub const DIRECTION_RTL: c_int = 5;
pub const DIRECTION_TTB: c_int = 6;
pub const DIRECTION_BTT: c_int = 7;
pub const NAME_ID_FONT_FAMILY: u32 = 1;
pub const NAME_ID_POSTSCRIPT_NAME: u32 = 6;
pub const NAME_ID_TYPOGRAPHIC_FAMILY: u32 = 16;

unsafe extern "C" {
    pub fn hb_set_create() -> *mut std::ffi::c_void;
    pub fn hb_set_destroy(set: *mut std::ffi::c_void);
    pub fn hb_set_clear(set: *mut std::ffi::c_void);
    pub fn hb_set_has(set: *const std::ffi::c_void, codepoint: u32) -> c_int;
    pub fn hb_ot_layout_has_positioning(face: *mut HbFace) -> c_int;
    pub fn hb_ot_layout_has_substitution(face: *mut HbFace) -> c_int;
    pub fn hb_ot_layout_table_get_lookup_count(face: *mut HbFace, tag: u32) -> c_uint;
    pub fn hb_ot_layout_lookup_collect_glyphs(
        face: *mut HbFace,
        tag: u32,
        index: c_uint,
        before: *mut std::ffi::c_void,
        input: *mut std::ffi::c_void,
        after: *mut std::ffi::c_void,
        output: *mut std::ffi::c_void,
    );

    pub fn hb_unicode_funcs_get_default() -> *mut HbUnicodeFuncs;
    pub fn hb_unicode_general_category(funcs: *mut HbUnicodeFuncs, codepoint: u32) -> c_int;
    pub fn hb_blob_create_or_fail(
        data: *const c_char,
        length: c_uint,
        mode: c_int,
        user_data: *mut c_void,
        destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> *mut HbBlob;
    pub fn hb_blob_reference(blob: *mut HbBlob) -> *mut HbBlob;
    pub fn hb_blob_destroy(blob: *mut HbBlob);
    pub fn hb_blob_get_length(blob: *mut HbBlob) -> c_uint;
    pub fn hb_blob_get_data(blob: *mut HbBlob, length: *mut c_uint) -> *const c_char;
    pub fn hb_face_count(blob: *mut HbBlob) -> c_uint;
    pub fn hb_face_create(blob: *mut HbBlob, index: c_uint) -> *mut HbFace;
    pub fn hb_face_reference(face: *mut HbFace) -> *mut HbFace;
    pub fn hb_face_destroy(face: *mut HbFace);
    pub fn hb_face_make_immutable(face: *mut HbFace);
    pub fn hb_face_get_glyph_count(face: *mut HbFace) -> c_uint;
    pub fn hb_face_get_upem(face: *mut HbFace) -> c_uint;
    pub fn hb_face_reference_table(face: *mut HbFace, tag: u32) -> *mut HbBlob;
    pub fn hb_font_create(face: *mut HbFace) -> *mut HbFont;
    pub fn hb_font_create_sub_font(parent: *mut HbFont) -> *mut HbFont;
    pub fn hb_font_get_empty() -> *mut HbFont;
    pub fn hb_font_get_face(font: *mut HbFont) -> *mut HbFace;
    pub fn hb_ot_math_has_data(face: *mut HbFace) -> c_int;
    pub fn hb_ot_math_get_constant(font: *mut HbFont, constant: c_int) -> i32;
    pub fn hb_font_destroy(font: *mut HbFont);
    pub fn hb_font_funcs_create() -> *mut HbFontFuncs;
    pub fn hb_font_funcs_destroy(funcs: *mut HbFontFuncs);
    pub fn hb_font_funcs_make_immutable(funcs: *mut HbFontFuncs);
    pub fn hb_font_funcs_set_glyph_h_advance_func(
        funcs: *mut HbFontFuncs,
        callback: Option<HbFontGetGlyphAdvanceFunc>,
        user_data: *mut c_void,
        destroy: Option<HbDestroyFunc>,
    );
    pub fn hb_font_funcs_set_glyph_h_advances_func(
        funcs: *mut HbFontFuncs,
        callback: Option<HbFontGetGlyphAdvancesFunc>,
        user_data: *mut c_void,
        destroy: Option<HbDestroyFunc>,
    );
    pub fn hb_font_set_funcs(
        font: *mut HbFont,
        funcs: *mut HbFontFuncs,
        font_data: *mut c_void,
        destroy: Option<HbDestroyFunc>,
    );
    pub fn hb_font_set_scale(font: *mut HbFont, x_scale: i32, y_scale: i32);
    pub fn hb_font_set_ptem(font: *mut HbFont, ptem: f32);
    pub fn hb_font_set_variations(font: *mut HbFont, variations: *const HbVariation, count: c_uint);
    pub fn hb_font_make_immutable(font: *mut HbFont);
    pub fn hb_ot_font_set_funcs(font: *mut HbFont);
    pub fn hb_ot_var_find_axis_info(face: *mut HbFace, tag: u32, info: *mut HbVarAxisInfo)
        -> c_int;
    pub fn hb_ot_name_get_utf8(
        face: *mut HbFace,
        name_id: u32,
        language: *const c_void,
        text_size: *mut c_uint,
        text: *mut c_char,
    ) -> c_uint;
    pub fn hb_ot_name_list_names(face: *mut HbFace, count: *mut c_uint) -> *const HbNameEntry;
    pub fn hb_font_get_nominal_glyph(font: *mut HbFont, codepoint: u32, glyph: *mut u32) -> c_int;
    pub fn hb_font_get_glyph_h_advance(font: *mut HbFont, glyph: u32) -> i32;
    pub fn hb_font_get_glyph_v_advance(font: *mut HbFont, glyph: u32) -> i32;
    pub fn hb_font_get_glyph_extents(
        font: *mut HbFont,
        glyph: u32,
        extents: *mut HbGlyphExtents,
    ) -> c_int;
    pub fn hb_ot_metrics_get_position(font: *mut HbFont, tag: u32, position: *mut i32) -> c_int;
    pub fn hb_buffer_create() -> *mut HbBuffer;
    pub fn hb_buffer_destroy(buffer: *mut HbBuffer);
    pub fn hb_buffer_set_direction(buffer: *mut HbBuffer, direction: c_int);
    pub fn hb_buffer_get_direction(buffer: *mut HbBuffer) -> c_int;
    pub fn hb_buffer_get_script(buffer: *mut HbBuffer) -> u32;
    pub fn hb_buffer_set_cluster_level(buffer: *mut HbBuffer, level: c_int);
    pub fn hb_buffer_add_utf16(
        buffer: *mut HbBuffer,
        text: *const u16,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );
    pub fn hb_buffer_set_language(buffer: *mut HbBuffer, language: *const c_void);
    pub fn hb_language_from_string(text: *const c_char, length: c_int) -> *const c_void;
    pub fn hb_buffer_set_script(buffer: *mut HbBuffer, script: u32);
    pub fn hb_script_from_string(text: *const c_char, length: c_int) -> u32;
    pub fn hb_buffer_add_utf8(
        buffer: *mut HbBuffer,
        text: *const c_char,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );
    pub fn hb_buffer_guess_segment_properties(buffer: *mut HbBuffer);
    pub fn hb_buffer_allocation_successful(buffer: *mut HbBuffer) -> c_int;
    pub fn hb_shape_full(
        font: *mut HbFont,
        buffer: *mut HbBuffer,
        features: *const c_void,
        num_features: c_uint,
        shaper_list: *const *const c_char,
    ) -> c_int;
    pub fn hb_buffer_get_glyph_infos(
        buffer: *mut HbBuffer,
        count: *mut c_uint,
    ) -> *const HbGlyphInfo;
    pub fn hb_buffer_get_glyph_positions(
        buffer: *mut HbBuffer,
        count: *mut c_uint,
    ) -> *const HbGlyphPosition;
}
