//! Chromium text-control hit testing.
//! Source: third_party/blink/renderer/core/layout/forms/layout_text_control.cc:79-93.
use super::super::{
    hit_test_location::HitTestLocation,
    hit_test_phase::HitTestPhase,
    hit_test_result::{HitTestResult, NodeRef},
};
use foundation::PhysicalOffset;
pub trait TextControlHitTestSource<'r> {
    fn BlockNodeAtPoint(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn ControlNode(&self) -> Option<NodeRef>;
    fn InnerEditorNode(&self) -> NodeRef;
    fn ContainerNode(&self) -> Option<NodeRef>;
    fn IsDescendantOfInnerEditor(&self, node: &NodeRef) -> bool;
    fn InnerEditorOffset(&self) -> Option<PhysicalOffset>;
}
pub(super) fn SameNode(a: Option<&NodeRef>, b: Option<&NodeRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => match (a.NativeNodeOwner(), b.NativeNodeOwner()) {
            // Chromium compares Node*. Anonymous control shadow nodes may
            // share a projected DOM host while retaining distinct owners.
            (Some(a), Some(b)) => a.Get() == b.Get(),
            _ => a.DOMIdentity() == b.DOMIdentity(),
        },
        (None, None) => true,
        _ => false,
    }
}
pub(super) fn HitInnerEditorElement<'r>(
    source: &dyn TextControlHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) {
    let Some(editor_offset) = source.InnerEditorOffset() else {
        return;
    };
    result.OverrideNodeAndPosition(
        Some(source.InnerEditorNode()),
        *location.Point() - offset - editor_offset,
    );
}
