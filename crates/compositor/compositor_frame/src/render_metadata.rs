//! Native Viz metadata values whose concrete representation belongs to Viz.
pub trait RenderFrameMetadataTypes {
    type Selection: Clone + Default + PartialEq;
    type LocalSurfaceId: Clone + PartialEq;
    type TrackedElementRects: Clone + Default + PartialEq;
}
// Source dependency: components/viz/common/vertical_scroll_direction.h:13.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalScrollDirection {
    #[default]
    kNull,
    kDown,
    kUp,
}
