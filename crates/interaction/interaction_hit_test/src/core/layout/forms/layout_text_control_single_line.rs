//! Chromium text-control hit testing.
//! Source: third_party/blink/renderer/core/layout/forms/layout_text_control_single_line.cc:39-75.
use super::super::{
    hit_test_location::HitTestLocation, hit_test_phase::HitTestPhase,
    hit_test_result::HitTestResult,
};
use super::layout_text_control::{HitInnerEditorElement, SameNode, TextControlHitTestSource};
use foundation::PhysicalOffset;

pub fn NodeAtPoint<'r>(
    source: &dyn TextControlHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> bool {
    let stop = source.BlockNodeAtPoint(result, location, offset, phase);
    let Some(node) = result.InnerNode() else {
        return stop;
    };
    if result
        .GetHitTestRequest()
        .GetStopNode()
        .is_some_and(|stop| SameNode(stop.NodeForHitTest().as_ref(), result.InnerNode()))
    {
        return stop;
    }
    if source.IsDescendantOfInnerEditor(node)
        || SameNode(Some(node), source.ControlNode().as_ref())
        || source
            .ContainerNode()
            .as_ref()
            .is_some_and(|container| SameNode(Some(container), Some(node)))
    {
        HitInnerEditorElement(source, result, location, offset);
    }
    stop
}
