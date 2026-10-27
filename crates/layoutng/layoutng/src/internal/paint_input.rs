//! Compatibility facade for the fragment-owned paint input schema.

#![allow(non_camel_case_types)]

#[path = "../../../layout_fragment/src/native_paint_input.rs"]
mod layout_fragment_native_paint_input;

pub use layout_fragment_native_paint_input::*;
