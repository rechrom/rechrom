//! Shared source cc scroll snapping, geometry and opaque element identity.
//! No Page, DOM, Layout, Paint or Compositor engine dependency is admitted.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
mod element_id;
pub mod geometry;
pub mod scroll_snap_data;
pub mod scroll_source_type;
pub mod scroll_utils;
pub mod snap_geometry;
pub mod snap_selection_strategy;
pub mod source_geometry;
pub use element_id::{ElementId, RemapElementIdToCcNamespace};
pub mod input_types {
    pub use crate::ElementId;
}
