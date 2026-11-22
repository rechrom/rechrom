//! Selection family: document-marker invalidation, editing style reads and
//! text-control hard wrapping. `range_geometry.rs` (Range client quads on
//! `LayoutNode`) is compiled as a child of `layout_output`. Exposed by
//! `layout::views::selection`; commands arrive as `LayoutMutation::Selection`.

pub(crate) mod document_markers;
pub mod editing_style;
#[cfg(feature = "translation_in_progress")]
mod text_control_hard_wrap;
