//! Page compositor output transported to the window-owned Viz service.
use crate::{chrome::DragRegion, engine::Viewport};
use browser::page::{PageChildCompositorFrame, PageCompositorFrame, PageReturnedResource};
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
    /// Latest browser-owned mutation applied before this Page frame was
    /// produced. The root embedder uses this only as a commit fence; the Page
    /// still owns and renders its normal live state.
    pub embedder_revision: u64,
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
    /// Delegated child surfaces referenced by `frame`. Display submits these
    /// under their producer-allocated public identities before the parent
    /// frame reaches Viz; they are never independent root-scene placements.
    pub child_frames: Vec<PageChildCompositorFrame>,
    pub frame: PageCompositorFrame,
    pub reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    /// Delegated immutable media frames are not registered in the parent cc
    /// ResourceProvider. Keep their return route separate so a coincident
    /// numeric ResourceId cannot retire a parent tile.
    pub child_reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    /// Browser-side VisualProperties ACK. Display invokes this only after the
    /// CompositorFrame has been accepted into the Page Surface.
    pub visual_properties_ack: Arc<dyn Fn() + Send + Sync>,
    pub presentation_feedback:
        Arc<dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync>,
}
impl PageFrame {
    pub(crate) fn Reclaim(&self, resources: Vec<PageReturnedResource>) {
        if !resources.is_empty() {
            (self.reclaim)(resources);
        }
    }

    /// Return every resource in a frame that never crossed the external Viz
    /// submission boundary, including delegated child frames.
    pub(crate) fn ReclaimAll(&self) {
        let resources = self
            .frame
            .frame
            .resource_list
            .iter()
            .map(|resource| resource.ToReturnedResource())
            .collect::<Vec<_>>();
        self.Reclaim(resources);
        let child_resources = self
            .child_frames
            .iter()
            .flat_map(|child| {
                child
                    .frame
                    .resource_list
                    .iter()
                    .map(|resource| resource.ToReturnedResource())
            })
            .collect::<Vec<_>>();
        if !child_resources.is_empty() {
            (self.child_reclaim)(child_resources);
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BrowserRootSolidColorQuad {
    pub destination: DeviceRect,
    pub color: [u8; 4],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrowserRootFrame {
    pub background_color: [u8; 4],
    /// Front-to-back, matching CompositorRenderPass::quad_list ordering.
    pub solid_color_quads: Vec<BrowserRootSolidColorQuad>,
}
#[derive(Clone, PartialEq)]
pub(crate) struct Scene {
    pub viewport: Viewport,
    /// BrowserView-owned pixels that exist independently of renderer-owned
    /// toolbar and WebContents Surfaces. Keeping them in the root frame makes
    /// resize fallback gutters participate in Viz damage and color handling.
    pub browser_root: BrowserRootFrame,
    pub placements: Vec<Placement>,
    pub live_documents: Vec<Document>,
    pub metadata: Vec<DragRegion>,
    /// Exact browser chrome revision belonging to this root commit.
    ///
    /// A minimum revision fence is insufficient: the toolbar Page can finish
    /// the next mutation before this Scene reaches Display, which would pair
    /// new browser chrome with the previous tab Surface for one frame.
    pub toolbar_revision: u64,
}
