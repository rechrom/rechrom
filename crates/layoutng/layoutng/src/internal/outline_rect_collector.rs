//! Compatibility facade for fragment-owned outline collection.

#![allow(non_snake_case, non_camel_case_types)]

#[path = "../../../layout_fragment/src/native_outline_rect_collector.rs"]
mod layout_fragment_native_outline_rect_collector;

pub use layout_fragment_native_outline_rect_collector::*;
