// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/document.cc:5407-5435

use crate::core::{
    layout::{
        hit_test_location::HitTestLocation,
        hit_test_result::{HitTestResult, HitTestResultRequest, NodeRef},
    },
    page::event_with_hit_test_results::MouseEventWithHitTestResults,
};
use foundation::PhysicalOffset;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;

/// Synchronous source Document/LayoutView operations. Readiness is checked
/// before entering Layout so early mouse motion cannot force first layout.
pub trait DocumentMouseHitTest<'a, L: ?Sized> {
    fn HasLayoutView(&self) -> bool;
    fn HasView(&self) -> bool;
    fn DidFirstLayout(&self) -> bool;
    fn HitTest(&self, location: &HitTestLocation, result: &mut HitTestResult<'a, L>);
    fn UpdateHoverActiveState(
        &self,
        active: bool,
        update_active_chain: bool,
        node: Option<NodeRef>,
    );
}

pub fn PerformMouseEventHitTest<'a, L: ?Sized>(
    document: &dyn DocumentMouseHitTest<'a, L>,
    request: &HitTestResultRequest<'a, L>,
    point: PhysicalOffset,
    event: &WebMouseEvent,
) -> MouseEventWithHitTestResults<'a, L> {
    if !document.HasLayoutView() || !document.HasView() || !document.DidFirstLayout() {
        let location = HitTestLocation::FromPhysicalOffset(PhysicalOffset::default());
        return MouseEventWithHitTestResults::new(
            event,
            &location,
            &HitTestResult::new(request, &location),
        );
    }
    let location = HitTestLocation::FromPhysicalOffset(point);
    let mut result = HitTestResult::new(request, &location);
    document.HitTest(&location, &mut result);
    if !request.ReadOnly() {
        document.UpdateHoverActiveState(
            request.Active(),
            !request.Move(),
            result.InnerPossiblyPseudoElement().cloned(),
        );
    }
    MouseEventWithHitTestResults::new(event, &location, &result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Document {
        ready: [bool; 3],
        queries: Cell<usize>,
        hover_updates: Cell<usize>,
    }
    impl DocumentMouseHitTest<'static, ()> for Document {
        fn HasLayoutView(&self) -> bool {
            self.ready[0]
        }
        fn HasView(&self) -> bool {
            self.ready[1]
        }
        fn DidFirstLayout(&self) -> bool {
            self.ready[2]
        }
        fn HitTest(&self, _: &HitTestLocation, _: &mut HitTestResult<'static, ()>) {
            self.queries.set(self.queries.get() + 1);
        }
        fn UpdateHoverActiveState(&self, _: bool, _: bool, _: Option<NodeRef>) {
            self.hover_updates.set(self.hover_updates.get() + 1);
        }
    }

    #[test]
    fn mouse_motion_before_first_layout_never_enters_layout_or_hover_updates() {
        for ready in [
            [false, true, true],
            [true, false, true],
            [true, true, false],
        ] {
            let document = Document {
                ready,
                queries: Cell::new(0),
                hover_updates: Cell::new(0),
            };
            let request =
                HitTestResultRequest::<()>::new(HitTestResultRequest::<()>::kMove, None, None);
            let result = PerformMouseEventHitTest(
                &document,
                &request,
                PhysicalOffset::FromPoint(&foundation::gfx::Point::new(43, 27)),
                &WebMouseEvent::default(),
            );
            assert_eq!(document.queries.get(), 0);
            assert_eq!(document.hover_updates.get(), 0);
            assert_eq!(
                *result.GetHitTestLocation().Point(),
                PhysicalOffset::default()
            );
            assert!(result.InnerNode().is_none());
        }
    }
}
