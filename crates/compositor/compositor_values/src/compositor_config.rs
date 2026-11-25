//! The Page compositor configuration and widget viewport values, named by the
//! main half (LayerTreeView::Initialize, LayerTreeHost::SetViewportRectAndScale)
//! and the impl half (LayerTreeHostImpl construction) alike. Moved from
//! compositor/src/engine.rs.
//!
//! Source: third_party/blink/renderer/platform/widget/compositing/
//!         layer_tree_settings.cc GenerateLayerTreeSettings (the initial
//!         screen size and device scale factor it reads);
//!         third_party/blink/public/common/widget/visual_properties.h
//!         (new_size, screen_infos device_scale_factor, page scale limits,
//!         visibility).
use compositor_tree::geometry::Size;

#[derive(Clone, Debug)]
pub struct CompositorConfig {
    pub thread: bool,
    /// Whether the root render pass preserves alpha instead of filling the
    /// viewport with the default opaque background.
    pub transparent_background: bool,
    /// Initial widget viewport used by Blink's compositor settings policy.
    /// The compositor still receives every later viewport through Commit.
    pub initial_viewport_size: Size,
    pub initial_device_scale_factor: f32,
    pub raster_task_limit: usize,
    /// Maximum number of independent task-graph drains offered to the
    /// embedding's raster executor. The executor remains authoritative for
    /// the actual worker count and sequencing policy.
    pub raster_worker_limit: usize,
}
impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            thread: false,
            transparent_background: false,
            initial_viewport_size: Size::default(),
            initial_device_scale_factor: 1.0,
            raster_task_limit: 32,
            raster_worker_limit: 8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompositorViewportState {
    pub size: Size,
    pub device_scale_factor: f32,
    pub page_scale_factor: f32,
    pub min_page_scale_factor: f32,
    pub max_page_scale_factor: f32,
    pub visible: bool,
}
impl CompositorViewportState {
    /// Whether the size, device scale factor and page scale limits are a
    /// valid widget viewport.
    pub fn IsValid(&self) -> bool {
        !(self.size.width() < 0
            || self.size.height() < 0
            || !self.device_scale_factor.is_finite()
            || self.device_scale_factor <= 0.0
            || !self.page_scale_factor.is_finite()
            || self.page_scale_factor <= 0.0
            || !self.min_page_scale_factor.is_finite()
            || self.min_page_scale_factor <= 0.0
            || !self.max_page_scale_factor.is_finite()
            || self.max_page_scale_factor < self.min_page_scale_factor
            || self.page_scale_factor < self.min_page_scale_factor
            || self.page_scale_factor > self.max_page_scale_factor)
    }
}
