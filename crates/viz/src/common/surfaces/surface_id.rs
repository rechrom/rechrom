// Source: components/viz/common/surfaces/surface_id.h:1-99
// Source: components/viz/common/surfaces/surface_id.cc:1-52
#![allow(non_snake_case)]

use super::{frame_sink_id::FrameSinkId, local_surface_id::LocalSurfaceId};
use std::fmt;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId {
    frame_sink_id_: FrameSinkId,
    local_surface_id_: LocalSurfaceId,
}
impl SurfaceId {
    pub const fn new(frame_sink_id: FrameSinkId, local_surface_id: LocalSurfaceId) -> Self {
        Self {
            frame_sink_id_: frame_sink_id,
            local_surface_id_: local_surface_id,
        }
    }
    pub const fn MaxSequenceId(frame_sink_id: FrameSinkId) -> Self {
        Self::new(frame_sink_id, LocalSurfaceId::MaxSequenceId())
    }
    pub fn is_valid(self) -> bool {
        self.frame_sink_id_.is_valid() && self.local_surface_id_.is_valid()
    }
    pub const fn frame_sink_id(self) -> FrameSinkId {
        self.frame_sink_id_
    }
    pub const fn local_surface_id(self) -> LocalSurfaceId {
        self.local_surface_id_
    }
    pub fn IsNewerThan(self, other: Self) -> bool {
        self.frame_sink_id_ == other.frame_sink_id_
            && self.local_surface_id_.IsNewerThan(other.local_surface_id_)
    }
    pub fn IsSameOrNewerThan(self, other: Self) -> bool {
        self == other || self.IsNewerThan(other)
    }
    pub fn ToSmallestId(self) -> Self {
        Self::new(self.frame_sink_id_, self.local_surface_id_.ToSmallestId())
    }
    pub fn HasSameEmbedTokenAs(self, other: Self) -> bool {
        self.local_surface_id_.embed_token() == other.local_surface_id_.embed_token()
    }
    pub fn ToString(self) -> String {
        format!(
            "SurfaceId({}, {})",
            self.frame_sink_id_, self.local_surface_id_
        )
    }
}
impl fmt::Display for SurfaceId {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.ToString())
    }
}
