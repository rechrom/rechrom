/// Opaque Viz identity used by compositor input/browser-controls offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OffsetTag(u128);

impl OffsetTag {
    pub const fn FromToken(token: u128) -> Self {
        Self(token)
    }

    pub const fn IsEmpty(self) -> bool {
        self.0 == 0
    }

    pub const fn token(self) -> u128 {
        self.0
    }
}

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
