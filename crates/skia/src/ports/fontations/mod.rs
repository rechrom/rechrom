// Mapped from Skia src/ports/fontations/src/ffi.rs (Skia cacf77bdba7b).
//
// Upstream exposes these Rust functions to C++ through a `cxx::bridge`. In
// Rechrom the C++ side (SkTypeface_fontations.cpp) is itself translated to
// Rust, so the bridge's shared structs and enums are declared here as plain
// Rust types with the same names and fields, the `extern "Rust"` functions are
// called directly, and the `extern "C++"` callback classes become traits.
// Phase 1 covers outline glyphs; colr.rs and bitmap.rs (COLR and bitmap
// glyphs) are not mapped yet.
#![allow(non_snake_case, dead_code)]

pub mod base;
pub mod hinting;
pub mod names;
pub mod verbs_points_pen;

pub use base::{
    coordinates_for_shifted_named_instance_index, fill_glyph_to_unicode_map, font_or_collection,
    font_ref_is_valid, get_font_style, get_outline_collection, get_skia_metrics,
    get_unscaled_metrics, has_any_color_table, is_embeddable, is_fixed_pitch, is_script_style,
    is_serif_style, is_subsettable, italic_angle, lookup_glyph_or_zero, make_font_ref,
    make_mapping_index, normalized_coords_equal, num_axes, num_glyphs, num_named_instances,
    outline_format, populate_axes, resolve_into_normalized_coords, table_data, table_tags,
    unhinted_advance_width_or_zero, units_per_em_or_zero, variation_position, BridgeFontRef,
    BridgeMappingIndex, BridgeNormalizedCoords, BridgeOutlineCollection,
};
pub use hinting::{
    get_bridge_glyph_styles, hinting_reliant, make_hinting_instance, make_mono_hinting_instance,
    no_hinting_instance, BridgeGlyphStyles, BridgeHintingInstance,
};
pub use names::{
    family_name, get_localized_strings, localized_name_next, postscript_name,
    BridgeLocalizedStrings,
};
pub use verbs_points_pen::{get_path_verbs_points, shrink_verbs_points_if_needed};

// Source: src/ports/fontations/src/ffi.rs:44-181 (#[cxx::bridge] shared types)
pub mod ffi {
    pub struct ColorStop {
        pub stop: f32,
        pub palette_index: u16,
        pub alpha: f32,
    }

    #[derive(Default)]
    pub struct Metrics {
        pub top: f32,
        pub ascent: f32,
        pub descent: f32,
        pub bottom: f32,
        pub leading: f32,
        pub avg_char_width: f32,
        pub max_char_width: f32,
        pub x_min: f32,
        pub x_max: f32,
        pub x_height: f32,
        pub cap_height: f32,
        pub underline_position: f32,
        pub underline_thickness: f32,
        pub strikeout_position: f32,
        pub strikeout_thickness: f32,
    }

    #[derive(Clone, Copy, Default, PartialEq)]
    #[repr(C)]
    pub struct FfiPoint {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Default)]
    pub struct BridgeLocalizedName {
        pub string: String,
        pub language: String,
    }

    #[derive(PartialEq, Debug, Default, Clone, Copy)]
    #[repr(C)]
    pub struct SkiaDesignCoordinate {
        pub axis: u32,
        pub value: f32,
    }

    #[derive(Default)]
    pub struct BridgeScalerMetrics {
        pub has_overlaps: bool,
        pub has_adjusted_advance: bool,
        pub adjusted_advance: f32,
    }

    pub struct PaletteOverride {
        pub index: u16,
        pub color_8888: u32,
    }

    #[derive(Default)]
    pub struct ClipBox {
        pub x_min: f32,
        pub y_min: f32,
        pub x_max: f32,
        pub y_max: f32,
    }

    // This type is used to mirror SkFontStyle values for Weight, Slant and Width
    #[derive(Default)]
    pub struct BridgeFontStyle {
        pub weight: i32,
        pub slant: i32,
        pub width: i32,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum AutoHintingControl {
        ForceForGlyf,
        ForceForGlyfAndCff,
        ForceOff,
        Fallback,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum OutlineFormat {
        NoOutlines,
        Glyf,
        Cff,
        Cff2,
    }

    // Source: src/ports/fontations/src/ffi.rs:387-398 (extern "C++" AxisWrapper),
    // implemented by sk_fontations::AxisWrapper (SkTypeface_fontations.cpp:239-258).
    pub trait AxisWrapper {
        fn populate_axis(
            &mut self,
            i: usize,
            axis: u32,
            min: f32,
            def: f32,
            max: f32,
            hidden: bool,
        ) -> bool;
        fn size(&self) -> usize;
    }
}
