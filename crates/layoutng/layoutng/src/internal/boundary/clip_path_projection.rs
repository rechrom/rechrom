//! Compatibility facade for fragment-owned clip-path projection.

#[path = "../../../../layout_fragment/src/clip_path_operation_projection.rs"]
mod layout_fragment_clip_path_operation_projection;

pub(super) use layout_fragment_clip_path_operation_projection::ProjectClipPathOperation;
