// Source: components/viz/service/display/aggregated_frame.h:1-75
// Source: components/viz/service/display/aggregated_frame.cc:1-46
#![allow(non_snake_case)]

use crate::common::quads::aggregated_render_pass::{
    AggregatedRenderPass, AggregatedRenderPassList,
};
use compositor_core::{
    adapter::{geometry::Rect, layer_tree_host_impl::ContentColorUsage},
    runtime::{
        input::LatencyInfo,
        metadata::{DelegatedInkMetadata, TrackedElementRects},
    },
};

pub type SurfaceDamageRectList = Vec<Rect>;

/// Viz-owned display data after all producer Surfaces have been resolved.
/// Surface lifetime, resource-provider state and draw acknowledgement belong
/// to the outer Viz Engine output and are intentionally absent here.
#[derive(Default)]
pub struct AggregatedFrame {
    pub latency_info: Vec<LatencyInfo>,
    pub content_color_usage: ContentColorUsage,
    pub has_copy_requests: bool,
    pub page_fullscreen_mode: bool,
    pub surface_damage_rect_list_: SurfaceDamageRectList,
    pub delegated_ink_metadata: Option<Box<DelegatedInkMetadata>>,
    pub render_pass_list: AggregatedRenderPassList,
    pub tracked_element_rects: TrackedElementRects,
}
impl AggregatedFrame {
    pub fn RenderPasses(&self) -> &[AggregatedRenderPass] {
        &self.render_pass_list
    }
    pub fn RootDamageRect(&self) -> Rect {
        self.render_pass_list
            .last()
            .map_or(Rect::default(), |pass| pass.damage_rect)
    }
    pub fn SurfaceDamageRects(&self) -> &[Rect] {
        &self.surface_damage_rect_list_
    }
}
