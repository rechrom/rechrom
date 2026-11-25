// components/viz/common/quads/offset_tag.h (a viz value).
pub use viz_common::offset_tag::OffsetTag;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserControlsOffsetTags {
    pub top_controls_offset_tag: OffsetTag,
    pub content_offset_tag: OffsetTag,
    pub bottom_controls_offset_tag: OffsetTag,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserControlsOffsetTagModifications {
    pub tags: BrowserControlsOffsetTags,
    pub top_controls_additional_height: i32,
    pub bottom_controls_additional_height: i32,
}
