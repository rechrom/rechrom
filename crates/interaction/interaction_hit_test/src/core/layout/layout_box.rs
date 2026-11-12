//! Native LayoutBox hit-test bounds and overflow controls.
//! Source: core/layout/layout_box.cc:1988-2026.

#![allow(non_snake_case)]
use super::{
    hit_test_location::HitTestLocation,
    hit_test_request::ListBasedHitTestBehavior,
    hit_test_result::{HitTestResult, NodeRef},
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, ToRoundedPoint};
pub trait OverflowControlBox {
    fn VisibleToHitTestRequest(&self, ignore_pointer_events_none: bool) -> bool;
    fn HitTestOverflowControls(
        &self,
        result: &mut HitTestResult<'_, ()>,
        point: &gfx::Point,
    ) -> bool;
    fn NodeForHitTest(&self) -> Option<NodeRef>;
}
pub fn HitTestOverflowControl(
    owner: &dyn OverflowControlBox,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> bool {
    if !owner.VisibleToHitTestRequest(result.GetHitTestRequest().IgnorePointerEventsNone()) {
        return false;
    }
    let local = *location.Point() - offset;
    if !owner.HitTestOverflowControls(result, &ToRoundedPoint(&local)) {
        return false;
    }
    let node = owner.NodeForHitTest();
    super::layout_object::UpdateHitTestResult(result, node.clone(), local);
    result.AddNodeToListBasedTestResult(node, location, &PhysicalRect::default())
        == ListBasedHitTestBehavior::kStopHitTesting
}

// Source: layout_box.cc:1988-1997. LayoutBox rejects non-intersecting content
// before entering LayoutObject\'s normal foreground-to-background phase order.
pub fn HitTestAllPhases<'r>(
    source: &dyn super::native_hit_test::NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> super::native_hit_test::NativeHitTestResult {
    if !source.MayIntersect(result, location, offset) {
        return Ok(false);
    }
    super::layout_object::HitTestAllPhases(source, result, location, offset)
}
