//! Scroll family: native ScrollableArea sources, user and programmatic
//! scrolling, overscroll, autoscroll, wheel scrollable-area traversal, scroll
//! anchoring suppression and page-scale geometry. Exposed by
//! `layout::views::scroll`; commands arrive as `LayoutMutation::Scroll`.

pub mod autoscroll_source;
pub mod page_scale_geometry;
pub(crate) mod scroll_anchor_suppression;
pub mod scroll_source;
pub mod user_scroll_source;
pub mod wheel_scrollable_traversal;
