//! Native anchor geometry from anchor_element_metrics.cc:146-154,201-280.
//! The original LayoutObject/outline/coordinate owners stay inside Layout.
use crate::{
    internal::{
        layout_box::LayoutBox, map_coordinates_flags::MapCoordinatesFlags,
        scroll_types::IncludeScrollbarsInRect,
    },
    layout_engine::LayoutEngine,
};
use foundation::{gfx, DynamicTo, PhysicalOffset};
use layoutng_style::style::outline_type::OutlineType;
impl LayoutEngine {
    pub fn AnchorMetricsBoundingBox(
        &self,
        node: u64,
        include_block_ink_overflow: bool,
    ) -> Option<gfx::Rect> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.OutputNode(node)?;
        if !include_block_ink_overflow {
            return Some(object.AbsoluteBoundingBoxRectDefault());
        }
        let rects = object.OutlineRects(
            std::ptr::null_mut(),
            &PhysicalOffset::default(),
            OutlineType::kIncludeBlockInkOverflow,
        );
        let rect = layoutng_geometry::geometry::physical_rect::UnionRect(&rects);
        let rect = object.LocalToAbsoluteRect(&rect, MapCoordinatesFlags::default());
        Some(gfx::ToEnclosingRect(gfx::RectF::new(
            gfx::PointF::new(rect.X().ToFloat(), rect.Y().ToFloat()),
            gfx::SizeF::new(rect.Width().ToFloat(), rect.Height().ToFloat()),
        )))
    }
    pub fn AnchorMetricsRootLayoutViewport(&self) -> Option<(gfx::Rect, i32)> {
        let _scope = self.NativeScrollPassScope();
        let root = self.GetLayoutTree()?.Root();
        let root = unsafe { DynamicTo::<LayoutBox>(root as *const _).as_ref() }?;
        let area = unsafe { root.GetScrollableArea().as_ref() }?;
        Some((
            area.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars),
            area.ScrollOffsetInt().y(),
        ))
    }
}
