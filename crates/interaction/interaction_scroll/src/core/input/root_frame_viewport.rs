// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/frame/root_frame_viewport.h:22-40,55-58,102-110,166-172,192-218
// Source: third_party/blink/renderer/core/frame/root_frame_viewport.cc:78-102,547-661
//
// Rechrom currently has no resident VisualViewport ScrollableArea. This owner
// is nevertheless distinct from the layout viewport, just as Chromium's
// RootFrameViewport is. Its typed port is the layout-viewport half of the
// source decorator; adding a VisualViewport later extends the distribution
// owner rather than changing the Page/ScrollManager contract.

use std::{cell::Cell, rc::Rc};

use super::scroll_manager::{
    ScrollCompletion, ScrollEffectEmitter, ScrollableArea, ScrollableAreaFacts, ScrollableAreaRef,
    SnapScrollKind, UserScrollRequest,
};
use crate::core::scroll::scroll_types::{ScrollConsumption, ScrollDirectionPhysical};

/// The live layout-viewport half of Chromium's RootFrameViewport decorator.
///
/// `WillConsumeUserScroll` is the source `ScrollAnimator::ComputeDeltaToConsume`
/// admission from root_frame_viewport.cc:561-583. It must inspect the same
/// desired/current native scroll owner used by `UserScroll`; returning true
/// only for a non-zero layout consumption lets the root sequence flag become
/// visible before a synchronous completion callback runs.
pub trait RootFrameLayoutViewport {
    fn Facts(&self) -> ScrollableAreaFacts;
    fn HasRunningUserScrollAnimation(&self) -> bool;
    fn WillConsumeUserScroll(&self, request: &UserScrollRequest) -> bool;
    fn SnapScroll(
        &self,
        kind: SnapScrollKind,
        direction: ScrollDirectionPhysical,
        emit: &ScrollEffectEmitter,
    ) -> bool;
    fn UserScroll(
        &self,
        request: UserScrollRequest,
        completion: ScrollCompletion,
        emit: &ScrollEffectEmitter,
    ) -> ScrollConsumption;
    fn OnScrollFinished(&self, enqueue_scrollend: bool, emit: &ScrollEffectEmitter);
}

/// Source RootFrameViewport identity for a Page without a resident visual
/// viewport. The wrapper owns only Chromium's sequence bit; offsets, animator,
/// snap state and completion stay on the live layout-viewport port.
pub struct RootFrameViewport {
    layout_viewport: Rc<dyn RootFrameLayoutViewport>,
    user_scroll_sequence_affects_layout_viewport_: Cell<bool>,
}

impl RootFrameViewport {
    // Source: root_frame_viewport.cc:78-84.
    pub fn new(layout_viewport: Rc<dyn RootFrameLayoutViewport>) -> Rc<Self> {
        Rc::new(Self {
            layout_viewport,
            user_scroll_sequence_affects_layout_viewport_: Cell::new(false),
        })
    }

    pub fn AsScrollableArea(self: &Rc<Self>) -> ScrollableAreaRef {
        self.clone()
    }

    pub fn LayoutViewport(&self) -> Rc<dyn RootFrameLayoutViewport> {
        self.layout_viewport.clone()
    }

    // Source: root_frame_viewport.h:166-172.
    pub fn ScrollAffectsLayoutViewport(&self) -> bool {
        self.user_scroll_sequence_affects_layout_viewport_.get()
    }
}

impl ScrollableArea for RootFrameViewport {
    fn Facts(&self) -> ScrollableAreaFacts {
        let mut facts = self.layout_viewport.Facts();
        // Source: root_frame_viewport.h:58; cc:655-661. Element identity and
        // animator enablement delegate to LayoutViewport, while the decorator
        // retains its own dynamic sequence admission.
        facts.is_root_frame_viewport = true;
        facts.scroll_affects_layout_viewport = self.ScrollAffectsLayoutViewport();
        facts
    }

    // Source: root_frame_viewport.h:125-150. Scroll-snap state belongs to the
    // layout viewport even when the compound viewport owns user scrolling.
    fn SnapScroll(
        &self,
        kind: SnapScrollKind,
        direction: ScrollDirectionPhysical,
        emit: &ScrollEffectEmitter,
    ) -> bool {
        self.layout_viewport.SnapScroll(kind, direction, emit)
    }

    // Source: root_frame_viewport.cc:547-653. With no resident visual
    // viewport, visual_consumed_delta is exactly zero, so the complete pixel
    // delta and completion owner are delegated to LayoutViewport. Preserve the
    // source sequence reset/set order around that call.
    fn UserScroll(
        &self,
        request: UserScrollRequest,
        completion: ScrollCompletion,
        emit: &ScrollEffectEmitter,
    ) -> ScrollConsumption {
        let facts = self.layout_viewport.Facts();
        if facts.scroll_animator_enabled && !self.layout_viewport.HasRunningUserScrollAnimation() {
            self.user_scroll_sequence_affects_layout_viewport_
                .set(false);
        }
        if self.layout_viewport.WillConsumeUserScroll(&request) {
            // Source cc:622-624. Set before UserScroll because its completion
            // may run synchronously on the immediate-scroll path.
            self.user_scroll_sequence_affects_layout_viewport_.set(true);
        }
        self.layout_viewport.UserScroll(request, completion, emit)
    }

    // ScrollableArea::OnScrollFinished acts on the layout viewport's native
    // LayoutBox/snap/event owners. RootFrameViewport's only special completion
    // admission is the sequence bit exposed through Facts above.
    fn OnScrollFinished(&self, enqueue_scrollend: bool, emit: &ScrollEffectEmitter) {
        self.layout_viewport
            .OnScrollFinished(enqueue_scrollend, emit)
    }
}
