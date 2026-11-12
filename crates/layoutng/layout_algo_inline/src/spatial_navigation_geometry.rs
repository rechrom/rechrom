//! spatial_navigation.cc:184-202,476-494,607-634,775-892. These queries
//! borrow the original native LayoutObject, fragments and PaintLayer identity.
use super::LayoutNode;
use crate::internal::{
    layout_box::LayoutBox,
    layout_inline::LayoutInline,
    layout_object::{BoxQuadType, IncludeDescendants},
    layout_pass_scope::LayoutPassScope,
    layout_text::LayoutText,
    map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode},
};
use foundation::graphics_types::graphics::visual_rect_flags::VisualRectFlags;
use foundation::spatial_navigation::SpatialNavigationDirection;
use foundation::{gfx, DynamicTo, LayoutUnit, PhysicalRect, Vector};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
impl LayoutNode<'_> {
    fn SpatialAbsoluteQuads(&self, traverse_frames: bool) -> Vector<gfx::QuadF> {
        let mut quads = Vector::new();
        let mut flags = MapCoordinatesFlags::default();
        if traverse_frames {
            flags.insert(MapCoordinatesMode::kTraverseDocumentBoundaries);
            flags.insert(MapCoordinatesMode::kApplyRemoteMainFrameTransform);
        }
        self.object
            .AbsoluteQuads(&mut quads, flags, BoxQuadType::kBorder);
        quads
    }
    pub fn SpatialNodeRectInRootFrame(&self) -> PhysicalRect {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        let local = if object.IsSVGText() {
            object.DecoratedBoundingBox()
        } else if let Some(box_) = unsafe { DynamicTo::<LayoutBox>(object).as_ref() } {
            box_.LocalBoundingBoxRectForAccessibility(IncludeDescendants(true))
        } else if let Some(inline) = unsafe { DynamicTo::<LayoutInline>(object).as_ref() } {
            inline.LocalBoundingBoxRectForAccessibility(IncludeDescendants(true))
        } else if let Some(text) = unsafe { DynamicTo::<LayoutText>(object).as_ref() } {
            text.LocalBoundingBoxRectForAccessibility(IncludeDescendants(true))
        } else {
            object.DecoratedBoundingBox()
        };
        let mut rect = PhysicalRect::EnclosingRect(&local);
        let style = object.StyleRef();
        rect.ContractEdges(
            LayoutUnit::from_f64(style.BorderTopWidth() as f64),
            LayoutUnit::from_f64(style.BorderRightWidth() as f64),
            LayoutUnit::from_f64(style.BorderBottomWidth() as f64),
            LayoutUnit::from_f64(style.BorderLeftWidth() as f64),
        );
        object.MapToVisualRectInAncestorSpacePhysical(
            std::ptr::null(),
            &mut rect,
            VisualRectFlags::default(),
        );
        rect
    }
    pub fn SpatialLineBoxes(&self) -> i32 {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        if !self.object.IsNonAtomicInline() {
            return 1;
        }
        let quads = self.SpatialAbsoluteQuads(false);
        if quads.iter().any(|quad| quad.BoundingBox().IsEmpty()) {
            return 1;
        }
        quads.len().try_into().expect("source vector line count")
    }
    pub fn SpatialShrinkInlineBoxToLineBox(
        &self,
        mut rect: PhysicalRect,
        line_boxes: Option<i32>,
    ) -> PhysicalRect {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let object = self.object;
        if !object.IsInline() || object.IsLayoutReplaced() || object.IsButtonOrInputButton() {
            return rect;
        }
        let horizontal = object.IsHorizontalWritingMode();
        let height = if horizontal {
            rect.Height()
        } else {
            rect.Width()
        };
        let mut line_height = object.StyleRef().ComputedLineHeightAsFixed();
        if line_height >= height {
            return rect;
        }
        if object.IsLayoutInline() {
            let mut child = object.SlowFirstChild();
            while let Some(current) = unsafe { child.as_ref() } {
                if current.IsAtomicInline() {
                    let box_ = unsafe { &*DynamicTo::<LayoutBox>(current) };
                    line_height = line_height.max(box_.LogicalHeight());
                }
                child = current.NextSibling();
            }
        }
        if line_height >= height {
            return rect;
        }
        line_height = line_height * line_boxes.unwrap_or_else(|| self.SpatialLineBoxes());
        if line_height >= height {
            return rect;
        }
        if horizontal {
            rect.size.height = line_height;
        } else {
            rect.size.width = line_height;
        }
        rect
    }
    pub fn SpatialSearchOriginFragment(
        &self,
        visible: PhysicalRect,
        direction: SpatialNavigationDirection,
    ) -> PhysicalRect {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let quads = self.SpatialAbsoluteQuads(true);
        let first = |quad: &gfx::QuadF| {
            let mut rect = PhysicalRect::EnclosingRect(&quad.BoundingBox());
            rect.Intersect(&visible);
            (!rect.IsEmpty()).then_some(rect)
        };
        match direction {
            SpatialNavigationDirection::Left | SpatialNavigationDirection::Down => {
                quads.iter().find_map(first).unwrap_or(visible)
            }
            SpatialNavigationDirection::Right | SpatialNavigationDirection::Up => {
                quads.iter().rev().find_map(first).unwrap_or(visible)
            }
            SpatialNavigationDirection::None => visible,
        }
    }
    pub fn SpatialBothOnTopmostPaintLayer(&self, other: &LayoutNode<'_>) -> bool {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let layer = self.object.PaintingLayer(-1);
        let Some(focused) = (unsafe { layer.as_ref() }) else {
            return false;
        };
        !focused.IsRootLayer()
            && layer == other.object.PaintingLayer(-1)
            && !focused.HasVisibleSelfPaintingDescendant()
    }
}
impl LayoutNode<'_> {
    /// HTMLAreaElement::ComputeAbsoluteRect cc96-108 uses the image origin
    /// with IgnoreTransforms, followed by its original cached area Path.
    pub fn SpatialImageMapOrigin(&self) -> gfx::PointF {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let point = self.object.LocalToAbsolutePointPhysical(
            &foundation::PhysicalOffset::default(),
            MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreTransforms),
        );
        gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat())
    }
}
