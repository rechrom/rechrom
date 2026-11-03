//! Compatibility facade for fragment-owned CSS motion-path projection.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/motion_path_shape_projection.rs"]
mod layout_fragment_motion_path_shape_projection;

pub(crate) use layout_fragment_motion_path_shape_projection::*;
