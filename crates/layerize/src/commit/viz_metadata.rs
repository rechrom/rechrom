//! viz surface metadata values a main Layer carries into the commit
//! (Layer::SetTrackedElementRects / capture bounds, PaintChunksToCcLayer
//! output). The values are components/viz/common (`viz_common::surfaces`);
//! the main Layer's push facet is implemented here.
pub use viz_common::surfaces::{
    FrameToken, RegionCaptureBounds, RegionCaptureCropId, TrackedElementFeature,
    TrackedElementRect, TrackedElementRects,
};

impl super::push_properties::LayerValue for TrackedElementRects {
    fn IsEmpty(&self) -> bool {
        self.0.is_empty()
    }
}
impl super::push_properties::LayerValue for RegionCaptureBounds {
    fn IsEmpty(&self) -> bool {
        self.0.is_empty()
    }
}
