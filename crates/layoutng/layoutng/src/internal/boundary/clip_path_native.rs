//! Compatibility facade for fragment-owned clip-path projection.

#[path = "../../../../layout_fragment/src/native_clip_path_projection.rs"]
mod layout_fragment_native_clip_path_projection;

pub(super) use layout_fragment_native_clip_path_projection::NativeClipPath;
