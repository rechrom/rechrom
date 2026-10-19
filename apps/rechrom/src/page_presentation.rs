//! Completed Page output transport. Page owns the only compositor and renderer.
use crate::{chrome::DragRegion, engine::Viewport};
use std::sync::Arc;
pub(crate) type Document = (u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContentFrameProvenance {
    pub sequence: u64,
    pub recording_revision: u64,
    /// The compositor BeginMainFrame whose effects were consumed before this
    /// Page output was produced. This is the cross-sequence equivalent of
    /// Chromium's DataForCommit ownership boundary.
    pub main_frame_epoch: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameProvenance {
    pub document: Document,
    pub content: Option<ContentFrameProvenance>,
}
/// Host-owned completed pixels, not an intermediate compositor/raster job.
pub(crate) struct PageFrame {
    pub provenance: FrameProvenance,
    pub viewport: Viewport,
    pub pixels: Arc<Vec<u32>>,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct DeviceRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl DeviceRect {
    pub(crate) fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
#[derive(Clone, PartialEq)]
pub(crate) struct Placement {
    pub document: Document,
    pub destination: DeviceRect,
}
#[derive(Clone, PartialEq)]
pub(crate) struct Scene {
    pub viewport: Viewport,
    pub placements: Vec<Placement>,
    pub live_documents: Vec<Document>,
    pub metadata: Vec<DragRegion>,
}
