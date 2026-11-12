//! Chromium LayoutImage temporary hit-result ownership.
//! Source: third_party/blink/renderer/core/layout/layout_image.cc:377-392.
use super::{
    hit_test_location::HitTestLocation,
    hit_test_phase::HitTestPhase,
    hit_test_result::HitTestResult,
    native_hit_test::{NativeHitTestResult, NativeLayoutHitTestSource},
};
use foundation::PhysicalOffset;

pub fn NodeAtPoint<'r>(
    source: &dyn NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let mut temporary = result.clone();
    let inside = super::layout_replaced::NodeAtPoint(
        source.Replaced(),
        &mut temporary,
        location,
        offset,
        phase,
    )?;
    if !inside && result.GetHitTestRequest().ListBased() {
        result.Append(&temporary);
    }
    if inside {
        result.Assign(&temporary);
    }
    Ok(inside)
}
