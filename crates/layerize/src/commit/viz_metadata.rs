//! viz surface metadata values a main Layer carries into the commit
//! (Layer::SetTrackedElementRects / capture bounds, PaintChunksToCcLayer
//! output) and the impl half reads into CompositorFrameMetadata.
// Source dependency: components/viz/common/surfaces/tracked_element_rects.h:22-34.
// Browser-maintained feature numbers intentionally remain accepted values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackedElementFeature(pub i32);
impl TrackedElementFeature {
    pub const kIframeTracking: Self = Self(0);
    pub const kPasswordTracking: Self = Self(1);
    pub const kTrackedElementFeatureEnd: Self = Self(2);
    pub const kTrackedElementFeatureMax: Self = Self(4);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameToken {
    Local([u64; 2]),
    Remote([u64; 2]),
}
// Source dependency: components/viz/common/surfaces/tracked_element_rects.h:37-85.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackedElementRect {
    pub id: [u64; 2],
    pub visible_bounds: super::geometry::Rect,
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub frame_token: Option<FrameToken>,
    pub parent_frame_token: Option<[u64; 2]>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackedElementRects(
    pub std::collections::BTreeMap<TrackedElementFeature, Vec<TrackedElementRect>>,
);
// Source dependency: components/viz/common/surfaces/region_capture_bounds.h:20-62.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionCaptureCropId(pub [u64; 2]);
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegionCaptureBounds(
    pub std::collections::BTreeMap<RegionCaptureCropId, super::geometry::Rect>,
);
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
