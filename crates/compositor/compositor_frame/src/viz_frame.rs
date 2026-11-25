//! The frame-output facets of the viz frame values (viz_common): trace
//! serialization (cc/trees/frame_data.cc) and the unbounded frame-sink
//! transport (cc/trees/unbounded_frame_sink_handler.cc).
use crate::frame_data_boundary::{FrameDataRenderPass, FrameDataTraceValue};
use crate::unbounded::CompositorFrame as UnboundedFrame;
use compositor_tree::geometry::Size;
use std::collections::HashMap;
use viz_common::compositor_render_pass::{CompositorFrame, CompositorRenderPass, FrameValueTypes};
use viz_common::returned_resource::RasterReturnedResource;
use viz_common::transferable_resource::RasterTransferableResource;
use viz_common::ResourceId;

impl<V: FrameValueTypes> FrameDataRenderPass for CompositorRenderPass<V> {
    type ResourceId = ResourceId;
    fn AsValueInto<T: FrameDataTraceValue>(
        &self,
        value: &mut T,
        resources: &mut HashMap<ResourceId, usize>,
    ) {
        value.BeginDictionary();
        value.SetBoolean(
            "has_transparent_background",
            self.has_transparent_background,
        );
        value.SetBoolean(
            "has_damage_from_contributing_content",
            self.has_damage_from_contributing_content,
        );
        value.BeginArray("quad_list");
        for quad in &self.quad_list {
            value.BeginDictionary();
            let id = quad.resource_id();
            if id != ResourceId::default() {
                let next = resources.len();
                resources.entry(id).or_insert(next);
            }
            value.SetBoolean("needs_blending", quad.geometry.needs_blending);
            value.EndDictionary();
        }
        value.EndArray();
    }
}

impl<M, V: FrameValueTypes> UnboundedFrame<RasterReturnedResource>
    for CompositorFrame<M, CompositorRenderPass<V>, RasterTransferableResource>
{
    fn render_pass_list_is_empty(&self) -> bool {
        self.render_pass_list.is_empty()
    }
    fn SetLastRenderPassOutputSize(&mut self, size: Size) {
        let last = self
            .render_pass_list
            .last_mut()
            .expect("native nonempty render pass list");
        last.output_rect.width = size.width();
        last.output_rect.height = size.height();
    }
    fn size_in_pixels(&self) -> Size {
        self.render_pass_list
            .last()
            .expect("native nonempty render pass list")
            .output_rect
            .size()
    }
    // Source dependency: cc/trees/unbounded_frame_sink_handler.cc:71-76;
    // components/viz/common/resources/transferable_resource.cc:44-50. Each actual transfer's EndImport is used.
    fn ReturnResources(&self) -> Vec<RasterReturnedResource> {
        self.resource_list
            .iter()
            .map(RasterTransferableResource::ToReturnedResource)
            .collect()
    }
}
