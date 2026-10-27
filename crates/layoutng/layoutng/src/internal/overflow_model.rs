//! Compatibility facade for fragment-owned overflow storage.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_overflow_model.rs"]
mod layout_fragment_native_overflow_model;

pub use layout_fragment_native_overflow_model::*;
