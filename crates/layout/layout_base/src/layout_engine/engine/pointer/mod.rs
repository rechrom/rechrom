//! Pointer family: mouse-event coordinates, drag images, registered
//! scrollable areas, and pointer scrollbar/resizer controls. Exposed by
//! `layout::views::pointer`; commands arrive as `LayoutMutation::Pointer`.

#[cfg(feature = "translation_in_progress")]
pub mod mouse_coordinates;
pub mod pointer_scroll_controls;
