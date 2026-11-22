//! Element::VisibleBoundsInLocalRoot, element.cc:3273-3311. The current source
//! RuntimeEnabledFeatures manifest disables ClipElementVisibleBoundsInLocalRoot
//! (runtime_enabled_features.json5:1182-1185); use its original LayoutView path.
use super::*;
use crate::internal::layout_view::LayoutView;
use crate::internal::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use foundation::graphics_types::graphics::visual_rect_flags::VisualRectFlags;
use foundation::{DynamicTo, LayoutUnit};
impl LayoutNode<'_> {
    pub fn ElementVisibleBoundsInLocalRoot(&self) -> gfx::Rect {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let bounding = self.object.AbsoluteBoundingBoxRectFDefault();
        // ui/gfx/geometry/rect_conversions.cc:113-121 (ClampRound endpoints).
        let left = bounding.x().round() as i32;
        let top = bounding.y().round() as i32;
        let right = bounding.right().round() as i32;
        let bottom = bounding.bottom().round() as i32;
        let mut rect = PhysicalRect::from_units(
            LayoutUnit::from_signed(left),
            LayoutUnit::from_signed(top),
            LayoutUnit::from_signed(right.saturating_sub(left)),
            LayoutUnit::from_signed(bottom.saturating_sub(top)),
        );
        let root = self
            .output
            .engine
            .tree_
            .as_ref()
            .expect("resident tree")
            .Root();
        let view = unsafe { DynamicTo::<LayoutView>(root as *const _).as_ref() }
            .expect("local root LayoutView");
        rect.Intersect(&view.ClippingRect());
        root.MapToVisualRectInAncestorSpacePhysical(
            std::ptr::null(),
            &mut rect,
            VisualRectFlags::default(),
        );
        let mut flags =
            MapCoordinatesFlags::from_mode(MapCoordinatesMode::kTraverseDocumentBoundaries);
        flags.insert(MapCoordinatesMode::kApplyRemoteMainFrameTransform);
        rect = root.AbsoluteToLocalRect(&rect, flags);
        foundation::ToPixelSnappedRect(rect)
    }
}

impl LayoutNode<'_> {
    /// Element::OutlineRectsInWidget cc3244-3268, before the existing
    /// FrameView::FrameToViewport boundary. Source layout is prepared by caller.
    pub fn ElementOutlineRectsInRootFrame(&self) -> Vec<gfx::Rect> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        if !self.object.IsBoxModelObject() {
            return Vec::new();
        }
        self.object
            .OutlineRects(
                std::ptr::null_mut(),
                &PhysicalOffset::default(),
                self.object
                    .StyleRef()
                    .OutlineRectsShouldIncludeBlockInkOverflow(),
            )
            .iter()
            .map(|rect| {
                foundation::ToPixelSnappedRect(
                    self.object
                        .LocalToAbsoluteRect(rect, MapCoordinatesFlags::default()),
                )
            })
            .collect()
    }
}

impl LayoutNode<'_> {
    /// Node::BoundingBox/PixelSnappedBoundingBox, node.cc1406-1414.
    pub fn PixelSnappedBoundingBox(&self) -> gfx::Rect {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let rect = self.object.AbsoluteBoundingBoxRectDefault();
        foundation::ToPixelSnappedRect(PhysicalRect::from_units(
            LayoutUnit::from_signed(rect.x()),
            LayoutUnit::from_signed(rect.y()),
            LayoutUnit::from_signed(rect.width()),
            LayoutUnit::from_signed(rect.height()),
        ))
    }
}
