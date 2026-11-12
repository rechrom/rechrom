//! Native leaves for WebFrameWidgetImpl::ComputeBlockBound and WebViewImpl's
//! document/viewport geometry. No zoom algorithm or retained tree is owned here.
use crate::{
    internal::{layout_box::LayoutBox, layout_view::LayoutView},
    layout_engine::LayoutEngine,
};
use foundation::{gfx, DynamicTo};
#[derive(Clone, Copy, Debug)]
pub struct NativePageScaleGeometry {
    pub document_size: gfx::Size,
    pub layout_viewport_size: gfx::Size,
    pub layout_scroll_offset: gfx::Vector2dF,
    pub minimum_scroll_offset: gfx::Vector2d,
    pub maximum_scroll_offset: gfx::Vector2d,
}
pub(crate) fn BlockZoomBoundsForNode(
    engine: &LayoutEngine,
    node: u64,
) -> Option<(bool, gfx::Rect)> {
    let _scope = engine.NativeScrollPassScope();
    let object = engine.GetLayoutTree()?.OutputNode(node)?;
    Some((object.IsInline(), object.AbsoluteBoundingBoxRectDefault()))
}
impl LayoutEngine {
    pub fn PageScaleGeometry(&self) -> Option<NativePageScaleGeometry> {
        let _scope = self.NativeScrollPassScope();
        let root = self.GetLayoutTree()?.Root();
        let view = unsafe { DynamicTo::<LayoutView>(root as *const _).as_ref() }?;
        let box_ = unsafe { DynamicTo::<LayoutBox>(root as *const _).as_ref() }?;
        let area = unsafe { box_.GetScrollableArea().as_ref() }?;
        let visible = area.VisibleContentRect(
            crate::internal::scroll_types::IncludeScrollbarsInRect::kExcludeScrollbars,
        );
        Some(NativePageScaleGeometry {
            document_size: gfx::ToEnclosingRect(gfx::RectF::from(view.DocumentRect())).size(),
            layout_viewport_size: visible.size(),
            layout_scroll_offset: area.GetScrollOffset(),
            minimum_scroll_offset: area.MinimumScrollOffsetInt(),
            maximum_scroll_offset: area.MaximumScrollOffsetInt(),
        })
    }
}
