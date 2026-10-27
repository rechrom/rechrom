//! Compatibility facade for fragment-owned CSS motion-path projection.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_motion_path_projection.rs"]
mod layout_fragment_native_motion_path_projection;

pub(crate) use layout_fragment_native_motion_path_projection::*;
