//! Compatibility facade for fragment-owned clip-path projection.

#[path = "fragment/clip_path_operation_projection.rs"]
mod layout_fragment_clip_path_operation_projection;

pub(super) use layout_fragment_clip_path_operation_projection::ProjectClipPathOperation;
