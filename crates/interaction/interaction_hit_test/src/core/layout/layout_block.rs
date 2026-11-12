//! Chromium LayoutBlock hit-test execution.
//! Source: third_party/blink/renderer/core/layout/layout_block.cc:378-403.
use super::{
    hit_test_location::HitTestLocation, hit_test_phase::HitTestPhase,
    hit_test_result::HitTestResult, native_hit_test::NativeLayoutHitTestSource,
};
use foundation::PhysicalOffset;

use crate::core::paint::box_fragment_painter::BoxFragmentPainter;

pub fn NodeAtPoint<'r>(
    source: &dyn NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> bool {
    if !source.MayIntersect(result, location, offset) {
        return false;
    }
    source.Fragment().is_some_and(|fragment| {
        BoxFragmentPainter::new(fragment).NodeAtPoint(result, location, offset, phase)
    })
}
