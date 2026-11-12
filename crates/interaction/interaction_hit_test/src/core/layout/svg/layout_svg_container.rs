//! Source: core/layout/svg/layout_svg_container.cc:227-277 and
//! SVGContentContainer::HitTest. All child identities remain native owners.
use super::transformed_hit_test_location::TransformedHitTestLocation;
use crate::core::layout::hit_test_location::HitTestLocation;
use foundation::{blink_geometry::transforms::affine_transform::AffineTransform, gfx::RectF};

pub trait SVGClipNode {
    fn NodeAtPointForSvgClipContent(&self, location: &HitTestLocation) -> bool;
}
pub trait SVGClipContainer {
    fn LocalToSVGParentTransform(&self) -> AffineTransform;
    fn HasClipPath(&self) -> bool;
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool;
    fn ChildPaintBlockedByDisplayLock(&self) -> bool;
    fn ChildrenInReverseOrder(&self) -> Box<dyn Iterator<Item = Box<dyn SVGClipNode + '_>> + '_>;
    fn PointerEventsBoundingBox(&self) -> bool;
    fn ObjectBoundingBox(&self) -> RectF;
}
pub fn NodeAtPointForSvgClipContent(
    container: &dyn SVGClipContainer,
    location: &HitTestLocation,
) -> bool {
    let local = TransformedHitTestLocation::new(location, &container.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if container.HasClipPath() && !container.ClipPathHitTest(local) {
        return false;
    }
    if !container.ChildPaintBlockedByDisplayLock() {
        for child in container.ChildrenInReverseOrder() {
            if child.NodeAtPointForSvgClipContent(local) {
                return true;
            }
        }
    }
    container.PointerEventsBoundingBox() && local.Intersects(&container.ObjectBoundingBox())
}

/// The general container branch delegates children to the same native cursor
/// and original request. ForeignObject children retain their source layer port.
pub trait SVGContainer {
    fn LocalToSVGParentTransform(&self) -> AffineTransform;
    fn HasClipPath(&self) -> bool;
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool;
    fn ChildPaintBlockedByDisplayLock(&self) -> bool;
    fn PointerEventsBoundingBox(&self) -> bool;
    fn ObjectBoundingBox(&self) -> RectF;
    fn HitTestChildren<'r>(
        &self,
        result: &mut super::super::hit_test_result::HitTestResult<'r, ()>,
        location: &HitTestLocation,
        phase: super::super::hit_test_phase::HitTestPhase,
    ) -> bool;
    fn IsObjectBoundingBoxValid(&self) -> bool;
    fn FilteredDecoratedBoundingBox(&self) -> RectF;
    fn Node(&self) -> Option<super::super::hit_test_result::NodeRef>;
}
pub fn NodeAtPoint<'r>(
    container: &dyn SVGContainer,
    result: &mut super::super::hit_test_result::HitTestResult<'r, ()>,
    location: &HitTestLocation,
    phase: super::super::hit_test_phase::HitTestPhase,
) -> bool {
    let local = TransformedHitTestLocation::new(location, &container.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if container.HasClipPath() && !container.ClipPathHitTest(local) {
        return false;
    }
    if !container.ChildPaintBlockedByDisplayLock()
        && container.HitTestChildren(result, local, phase)
    {
        return true;
    }
    if !container.IsObjectBoundingBoxValid() {
        return false;
    }
    let visual = result.GetHitTestRequest().IsHitTestVisualOverflow();
    let bounds = if visual {
        container.FilteredDecoratedBoundingBox()
    } else {
        container.ObjectBoundingBox()
    };
    if (visual || container.PointerEventsBoundingBox()) && local.Intersects(&bounds) {
        if result.InnerNode().is_none() {
            result.SetNodeAndPosition(
                container.Node(),
                foundation::PhysicalOffset::FromPointFRound(&local.TransformedPoint()),
            );
        }
        return result.AddNodeToListBasedTestResult(
            container.Node(),
            local,
            &foundation::PhysicalRect::default(),
        ) == super::super::hit_test_request::ListBasedHitTestBehavior::kStopHitTesting;
    }
    false
}
