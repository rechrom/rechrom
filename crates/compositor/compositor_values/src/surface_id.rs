//! Native SurfaceId values (components/viz/common/surfaces). Routing and
//! aggregation are Viz work; these are the identities frames carry.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameSinkId {
    pub client_id: u32,
    pub sink_id: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSurfaceId {
    pub parent_sequence_number: u32,
    pub child_sequence_number: u32,
    pub embed_token: [u64; 2],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId {
    pub frame_sink_id: FrameSinkId,
    pub local_surface_id: LocalSurfaceId,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceRange {
    pub start: Option<SurfaceId>,
    pub end: SurfaceId,
}
impl crate::local_surface_id::LocalSurfaceId for LocalSurfaceId {
    fn is_valid(&self) -> bool {
        self.parent_sequence_number != 0
            && self.child_sequence_number != 0
            && self.embed_token != [0, 0]
    }
}
impl LocalSurfaceId {
    // Source: components/viz/common/surfaces/local_surface_id.cc:47-61.
    pub fn IsNewerThan(&self, other: &Self) -> bool {
        self.embed_token == other.embed_token
            && self
                .child_sequence_number
                .wrapping_sub(other.child_sequence_number)
                < (1u32 << 31)
            && self
                .parent_sequence_number
                .wrapping_sub(other.parent_sequence_number)
                < (1u32 << 31)
            && (self.child_sequence_number != other.child_sequence_number
                || self.parent_sequence_number != other.parent_sequence_number)
    }
    // Source: components/viz/common/surfaces/local_surface_id.cc:68-70.
    pub fn IsSameOrNewerThan(&self, other: &Self) -> bool {
        self == other || self.IsNewerThan(other)
    }
    // Source: components/viz/common/surfaces/local_surface_id.cc:63-66.
    pub fn IsNewerThanOrEmbeddingChanged(&self, other: &Self) -> bool {
        self.IsNewerThan(other) || self.embed_token != other.embed_token
    }
}

// Source dependency: ui/gfx/display_color_spaces.h:36-45.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ContentColorUsage {
    #[default]
    kSRGB = 0,
    kWideColorGamut = 1,
    kHDR = 2,
}
impl ContentColorUsage {
    pub const kMaxValue: Self = Self::kHDR;
}
