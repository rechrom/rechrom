//! Compatibility facade for the fragment-owned caret geometry contract.

#![allow(non_snake_case, non_camel_case_types)]

#[path = "../../../layout_fragment/src/native_caret_rect.rs"]
mod layout_fragment_native_caret_rect;

pub use layout_fragment_native_caret_rect::*;
