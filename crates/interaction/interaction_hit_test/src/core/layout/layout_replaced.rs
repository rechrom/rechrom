//! Chromium LayoutReplaced hit-test execution.
//! Source: third_party/blink/renderer/core/layout/layout_replaced.cc:129-237;
//! third_party/blink/renderer/core/layout/layout_replaced.h:139-146.
use super::{
    hit_test_location::HitTestLocation, hit_test_phase::HitTestPhase,
    hit_test_result::HitTestResult, native_hit_test::NativeHitTestResult,
};
use foundation::PhysicalOffset;

use super::{
    hit_test_request::{HitTestStopNode, ListBasedHitTestBehavior},
    hit_test_result::NodeRef,
};
use crate::platform::geometry::{contoured_rect::ContouredRectHitTest, path::PathHitTest};
use foundation::{PhysicalRect, PhysicalSize};

pub trait ReplacedHitTestSource<'r> {
    // Source layout_replaced.cc:138-183 reads the LayoutBox owner, not
    // PhysicalFragment flags or fragment-local overflow geometry.
    fn HasSelfPaintingLayer(&self) -> bool;
    fn HasBorderShape(&self) -> bool;
    fn HasBorderRadius(&self) -> bool;
    fn MatchesStopNode(&self, stop: &dyn HitTestStopNode<()>) -> bool;
    fn ChildPaintBlockedByDisplayLock(&self) -> bool;
    fn ShouldClipOverflowAlongEitherAxis(&self) -> bool;
    fn OverflowClipRect(&self) -> PhysicalRect;
    fn ExpandBorderShapeRectForOverflowClipMargin(&self, rect: &mut PhysicalRect);
    fn BorderShapeOuterPath(&self, rect: PhysicalRect) -> Box<dyn PathHitTest + '_>;
    fn HitTestOverflowControl(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn MayIntersect(
        &self,
        result: &HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn VisibleToHitTestRequest(&self, ignore_pointer_events_none: bool) -> bool;
    fn IsReplacedElementRespectingCSSOverflow(&self) -> bool;
    fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        phase == HitTestPhase::kForeground
            || (phase == HitTestPhase::kSelfBlockBackground
                && self.IsReplacedElementRespectingCSSOverflow())
    }
    fn StitchedSize(&self) -> PhysicalSize;
    fn BorderBoxRect(&self) -> PhysicalRect;
    fn VisualOverflowRectIncludingFilters(&self) -> PhysicalRect;
    fn PixelSnappedBorder(
        &self,
        rect: PhysicalRect,
        inner: bool,
    ) -> Box<dyn ContouredRectHitTest + '_>;
    fn NodeForHitTest(&self) -> Option<NodeRef>;
    fn HitTestChildren(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> NativeHitTestResult;
}
pub fn NodeAtPoint<'r>(
    source: &dyn ReplacedHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    if !source.MayIntersect(result, location, offset) {
        return Ok(false);
    }
    if phase == HitTestPhase::kForeground
        && !source.HasSelfPaintingLayer()
        && source.HitTestOverflowControl(result, location, offset)
    {
        return Ok(true);
    }
    let border_shape_rect = || {
        let mut border = source.BorderBoxRect();
        border.Move(&offset);
        source.ExpandBorderShapeRectForOverflowClipMargin(&mut border);
        border
    };
    let mut skip = result
        .GetHitTestRequest()
        .GetStopNode()
        .is_some_and(|stop| source.MatchesStopNode(stop))
        || source.ChildPaintBlockedByDisplayLock();
    if !skip && source.ShouldClipOverflowAlongEitherAxis() {
        if !source.HasSelfPaintingLayer() {
            let mut clip = source.OverflowClipRect();
            clip.Move(&offset);
            if !location.Intersects(&clip) {
                skip = true;
            }
        }
        if !skip && source.HasBorderShape() {
            skip =
                !location.IntersectsPath(source.BorderShapeOuterPath(border_shape_rect()).as_ref());
        } else if !skip && source.HasBorderRadius() {
            skip = !location.IntersectsContouredRect(
                source
                    .PixelSnappedBorder(PhysicalRect::new(offset, source.StitchedSize()), true)
                    .as_ref(),
            );
        }
    }
    if !skip && source.HitTestChildren(result, location, offset, phase)? {
        return Ok(true);
    }
    if !result.GetHitTestRequest().IsHitTestVisualOverflow() {
        if source.HasBorderShape() {
            if !location.IntersectsPath(source.BorderShapeOuterPath(border_shape_rect()).as_ref()) {
                return Ok(false);
            }
        } else if source.HasBorderRadius() {
            let mut border = source.BorderBoxRect();
            border.Move(&offset);
            if !location.IntersectsContouredRect(source.PixelSnappedBorder(border, false).as_ref())
            {
                return Ok(false);
            }
        }
    }
    // Source layout_replaced.h:139-146 extends the LayoutBox foreground
    // phase for elements which respect CSS overflow.
    if source.IsInSelfHitTestingPhase(phase)
        && source.VisibleToHitTestRequest(result.GetHitTestRequest().IgnorePointerEventsNone())
    {
        let mut bounds = if result.GetHitTestRequest().IsHitTestVisualOverflow() {
            source.VisualOverflowRectIncludingFilters()
        } else {
            source.BorderBoxRect()
        };
        bounds.Move(&offset);
        if location.Intersects(&bounds) {
            let node = source.NodeForHitTest();
            if result.InnerNode().is_none() {
                result.SetNodeAndPosition(node.clone(), *location.Point() - offset);
            }
            if result.AddNodeToListBasedTestResult(node, location, &bounds)
                == ListBasedHitTestBehavior::kStopHitTesting
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
