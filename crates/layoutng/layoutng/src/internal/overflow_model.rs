//! Compatibility facade for fragment-owned overflow storage.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/overflow_model.rs"]
mod layout_fragment_overflow_model;

pub use layout_fragment_overflow_model::*;
