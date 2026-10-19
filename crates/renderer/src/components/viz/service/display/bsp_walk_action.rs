use super::draw_polygon::DrawPolygon;
use compositor_core::adapter::{geometry::Rect, property_tree::PropertyTreeTransform as _};
use viz::common::quads::aggregated_render_pass::NativeTransform;
// Source: components/viz/service/display/bsp_walk_action.h:20-23
// Rust capability dispatch replaces C++ virtual operator() inheritance.
pub trait BspWalkAction {
    fn call(&mut self, item: &mut DrawPolygon);
}
// Source: components/viz/service/display/bsp_walk_action.h:25-39
// Private renderer capability keeps canonical quads in their owning pass; the
// index handle resolves the original shared-state transform without copying it.
pub(crate) trait PolygonDrawBackend {
    fn polygon_transform(&self, original_ref: usize) -> &NativeTransform;
    fn do_draw_polygon(
        &mut self,
        item: &DrawPolygon,
        scissor: Rect,
        using_scissor_as_optimization: bool,
    );
}
pub(crate) struct BspWalkActionDrawPolygon<'a, B: PolygonDrawBackend> {
    renderer: &'a mut B,
    render_pass_scissor: &'a Rect,
    using_scissor_as_optimization: bool,
}
impl<'a, B: PolygonDrawBackend> BspWalkActionDrawPolygon<'a, B> {
    // Source: components/viz/service/display/bsp_walk_action.cc:17-23
    pub fn new(
        renderer: &'a mut B,
        render_pass_scissor: &'a Rect,
        using_scissor_as_optimization: bool,
    ) -> Self {
        Self {
            renderer,
            render_pass_scissor,
            using_scissor_as_optimization,
        }
    }
}
impl<B: PolygonDrawBackend> BspWalkAction for BspWalkActionDrawPolygon<'_, B> {
    // Source: components/viz/service/display/bsp_walk_action.cc:25-40
    fn call(&mut self, item: &mut DrawPolygon) {
        let mut inverse = NativeTransform::default();
        if !self
            .renderer
            .polygon_transform(item.original_ref)
            .GetInverse(&mut inverse)
        {
            return;
        }
        item.transform_to_layer_space(&inverse);
        self.renderer.do_draw_polygon(
            item,
            *self.render_pass_scissor,
            self.using_scissor_as_optimization,
        );
    }
}
// Source: components/viz/service/display/bsp_walk_action.h:41-49
// A borrowed collection has the same tree-lifetime requirement as C++ raw_ptr.
pub struct BspWalkActionToVector<'a, 'tree> {
    list: &'a mut Vec<&'tree DrawPolygon>,
}
impl<'a, 'tree> BspWalkActionToVector<'a, 'tree> {
    // Source: components/viz/service/display/bsp_walk_action.cc:42-44
    pub fn new(list: &'a mut Vec<&'tree DrawPolygon>) -> Self {
        Self { list }
    }
    // Source: components/viz/service/display/bsp_walk_action.cc:46-48
    pub fn call(&mut self, item: &'tree DrawPolygon) {
        self.list.push(item);
    }
}
