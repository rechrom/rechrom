//! Compatibility facade for fragment-owned ink-overflow storage.

#![allow(non_snake_case, non_camel_case_types)]

#[path = "../../../layout_fragment/src/ink_overflow.rs"]
mod layout_fragment_ink_overflow;

pub use layout_fragment_ink_overflow::*;
