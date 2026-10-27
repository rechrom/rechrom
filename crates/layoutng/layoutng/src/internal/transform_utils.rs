//! Compatibility facade for fragment-owned transform geometry.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_transform_utils.rs"]
mod layout_fragment_native_transform_utils;

pub use layout_fragment_native_transform_utils::*;
