//! Chromium LayoutObject hit-test phase order.
//! Source: third_party/blink/renderer/core/layout/layout_object.cc:4524-4544,4590-4600.
use super::{
    hit_test_location::HitTestLocation,
    hit_test_phase::HitTestPhase,
    hit_test_result::{HitTestResult, NodeRef},
    native_hit_test::{NativeHitTestResult, NativeLayoutHitTestSource},
};
use foundation::PhysicalOffset;

// Source: layout_object.cc:4590-4600. The caller supplies the same live
// NodeForHitTest result; this helper owns the conditional result update.
pub fn UpdateHitTestResult(
    result: &mut HitTestResult<'_, ()>,
    node: Option<NodeRef>,
    point: PhysicalOffset,
) {
    if result.InnerNode().is_none() && node.is_some() {
        result.SetNodeAndPosition(node, point);
    }
}

pub fn HitTestAllPhases<'r>(
    source: &dyn NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> NativeHitTestResult {
    for phase in [
        HitTestPhase::kForeground,
        HitTestPhase::kFloat,
        HitTestPhase::kDescendantBlockBackgrounds,
        HitTestPhase::kSelfBlockBackground,
    ] {
        if super::native_hit_test::NodeAtPoint(source, result, location, offset, phase)? {
            return Ok(true);
        }
    }
    Ok(false)
}
