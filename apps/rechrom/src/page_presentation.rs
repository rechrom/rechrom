//! Page compositor output transported to the window-owned Viz service.
use crate::{chrome::DragRegion, engine::Viewport};
use browser::page::{PageCompositorFrame, PageReturnedResource};
use std::{sync::Arc, time::Instant};
pub(crate) type Document = (u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResizeProvenance {
    pub id: u64,
    pub queued_at: Instant,
    pub samples: usize,
}
impl ResizeProvenance {
    pub(crate) fn merge(&mut self, next: Self) {
        if next.queued_at < self.queued_at {
            self.id = next.id;
            self.queued_at = next.queued_at;
        }
        self.samples = self.samples.saturating_add(next.samples);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContentFrameProvenance {
    pub sequence: u64,
    pub recording_revision: u64,
    /// The compositor BeginMainFrame whose effects were consumed before this
    /// Page output was produced. This is the cross-sequence equivalent of
    /// Chromium's DataForCommit ownership boundary.
    pub main_frame_epoch: u64,
    /// Last impl scroll callback incorporated by this Page output.
    pub compositor_scroll_sequence: u64,
    /// App-owned live-resize transaction represented by this content frame.
    pub resize: Option<ResizeProvenance>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameProvenance {
    pub document: Document,
    pub content: Option<ContentFrameProvenance>,
}
/// A Page's move-only FrameSink submission. The window display owns the sole
/// Viz/Renderer service and returns resources to this Page's compositor.
pub(crate) struct PageFrame {
    pub provenance: FrameProvenance,
    pub viewport: Viewport,
    /// Chromium CompositorFrameMetadata::is_handling_interaction. Viz carries
    /// this through aggregation so the platform presenter can apply its
    /// native interaction-frame presentation policy.
    pub is_handling_interaction: bool,
    pub frame: PageCompositorFrame,
    pub reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
}
impl PageFrame {
    pub(crate) fn Reclaim(&self, resources: Vec<PageReturnedResource>) {
        if !resources.is_empty() {
            (self.reclaim)(resources);
        }
    }
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
