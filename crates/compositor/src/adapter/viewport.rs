//! Borrow-only source owner connections used by cc/layers/viewport. Viewport
//! owns the scroll/pinch policy; these services forward to their native owner.
use super::input_handler::{InputLayerId, InputScrollNodeId};
use super::{
    geometry::{Point, Size, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::LayerTreeTypes,
    time::TimeDelta,
};
pub use crate::cc::input::scrollbar::ScrollbarOrientation;
use crate::cc::{input::scroll_state::ScrollState, trees::layer_tree_impl::LayerTreeImpl};
/// One call-period runtime entry. Each operation borrows only the resident
/// siblings needed by that operation; overlapping clients are never pre-borrowed.
/// No runtime method retains a tree/node/input reference after returning.
pub trait ViewportRuntime<B: LayerTreeTypes> {
    fn ActiveTree(&self) -> &LayerTreeImpl<B>;
    fn ScrollNode(&self, id: &InputScrollNodeId) -> &crate::cc::trees::scroll_node::ScrollNode;
    fn CurrentPageScaleFactor(&self) -> f32;
    fn PageScaleFactorForScroll(&self) -> f32;
    fn ScrollNodeBy(
        &mut self,
        id: i32,
        delta: &Vector2dF,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    ) -> Vector2dF;
    fn ClampScrollNodeToMaxScrollOffset(
        &mut self,
        id: i32,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    ) -> Vector2dF;
    fn SetPageScaleOnActiveTree(
        &mut self,
        scale: f32,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    );
    fn ScrollSingleNode(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        point: Point,
        direct: bool,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    ) -> Vector2dF;
    fn CanConsumeDelta(
        &mut self,
        state: &ScrollState,
        node: InputScrollNodeId,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    ) -> bool;
    fn ComputeScrollDelta(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    ) -> Vector2dF;
    fn ScrollAnimationCreate(
        &mut self,
        node: InputScrollNodeId,
        delta: Vector2dF,
        delay: TimeDelta,
        viewport: &mut crate::cc::layers::viewport::Viewport,
    );
    fn ScrollBrowserControls(&mut self, delta: &Vector2dF, inertial: bool) -> Vector2dF;
    fn ScrollbarsFor(&mut self, id: ElementId) -> Vec<InputLayerId>;
    fn ScrollbarOrientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation;
    fn ScrollbarBounds(&mut self, layer: &InputLayerId) -> Size;
    fn StartPageScaleAnimation(
        &mut self,
        target: Point,
        use_anchor: bool,
        scale: f32,
        duration: TimeDelta,
    );
}
pub struct ViewportContext<'a, B: LayerTreeTypes> {
    pub runtime: &'a mut dyn ViewportRuntime<B>,
}

// Source dependency: ui/gfx/geometry/vector2d_f.h (native vector operators).
impl Vector2dF {
    pub fn InvScale(&mut self, scale: f32) {
        self.set_x(self.x() / scale);
        self.set_y(self.y() / scale);
    }
}
impl std::ops::SubAssign for Vector2dF {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl std::ops::AddAssign<Vector2dF> for super::geometry::PointF {
    fn add_assign(&mut self, rhs: Vector2dF) {
        *self = *self + rhs;
    }
}
