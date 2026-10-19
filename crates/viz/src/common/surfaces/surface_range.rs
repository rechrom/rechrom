// Source: components/viz/common/surfaces/surface_range.h:1-75
// Source: components/viz/common/surfaces/surface_range.cc:1-87
#![allow(non_snake_case)]

use super::surface_id::SurfaceId;
use std::fmt;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SurfaceRange {
    start_: Option<SurfaceId>,
    end_: SurfaceId,
}
impl SurfaceRange {
    pub fn new(start: Option<SurfaceId>, end: SurfaceId) -> Self {
        Self {
            start_: start,
            end_: end,
        }
    }
    pub fn from_surface(surface: SurfaceId) -> Self {
        Self::new(Some(surface), surface)
    }
    pub fn start(&self) -> Option<SurfaceId> {
        self.start_
    }
    pub fn end(&self) -> SurfaceId {
        self.end_
    }
    pub fn HasDifferentFrameSinkIds(&self) -> bool {
        self.start_
            .is_some_and(|s| s.frame_sink_id() != self.end_.frame_sink_id())
    }
    pub fn HasDifferentEmbedTokens(&self) -> bool {
        self.start_
            .is_some_and(|s| !s.HasSameEmbedTokenAs(self.end_))
    }
    pub fn IsInRangeExclusive(&self, surface: SurfaceId) -> bool {
        let Some(start) = self.start_ else {
            return self.end_.IsNewerThan(surface);
        };
        if self.HasDifferentFrameSinkIds() || self.HasDifferentEmbedTokens() {
            surface.IsNewerThan(start) || self.end_.IsNewerThan(surface)
        } else {
            surface.IsNewerThan(start) && self.end_.IsNewerThan(surface)
        }
    }
    pub fn IsInRangeInclusive(&self, surface: SurfaceId) -> bool {
        self.IsInRangeExclusive(surface) || self.end_ == surface || self.start_ == Some(surface)
    }
    pub fn IsValid(&self) -> bool {
        if !self.end_.is_valid() {
            return false;
        }
        let Some(start) = self.start_ else {
            return true;
        };
        start.is_valid()
            && (self.HasDifferentFrameSinkIds()
                || self.HasDifferentEmbedTokens()
                || self.end_.IsSameOrNewerThan(start))
    }
    pub fn ToString(&self) -> String {
        format!(
            "SurfaceRange(start: {}, end: {})",
            self.start_.map_or_else(|| "none".into(), |v| v.ToString()),
            self.end_
        )
    }
}
impl fmt::Display for SurfaceRange {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.ToString())
    }
}
