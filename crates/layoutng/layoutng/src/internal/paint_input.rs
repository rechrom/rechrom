//! Compatibility facade for the fragment-owned paint input schema.

#![allow(non_camel_case_types)]

#[path = "../../../layout_fragment/src/paint_input.rs"]
mod layout_fragment_paint_input;

pub use layout_fragment_paint_input::*;
