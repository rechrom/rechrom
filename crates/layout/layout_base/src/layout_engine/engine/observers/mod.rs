//! Observer family: IntersectionObserver and anchor-element-metrics geometry.
//! `resize_observer_geometry.rs` (ResizeObserver sizes on `LayoutNode`) is
//! compiled as a child of `layout_output`. Not a `layout::views` view: the
//! facade forwards these queries from `LayoutEngine` and `layout::output`.

mod anchor_metrics;
pub(crate) mod intersection_observer_geometry;
