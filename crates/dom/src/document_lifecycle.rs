//! Document's original rendering lifecycle, independent of engine placement.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! core/dom/document_lifecycle.h:60-93,191-218; .cc:65-190,228-255.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleState {
    Uninitialized,
    Inactive,
    VisualUpdatePending,
    InStyleRecalc,
    StyleClean,
    InPerformLayout,
    AfterPerformLayout,
    LayoutClean,
    InCompositingInputsUpdate,
    CompositingInputsClean,
    InPrePaint,
    PrePaintClean,
    InPaint,
    PaintClean,
    Stopping,
    Stopped,
}

pub struct DocumentLifecycle {
    state: Cell<LifecycleState>,
    no_transition_count: Cell<u32>,
    style_version: Cell<u64>,
}
impl DocumentLifecycle {
    /// Document's constructor advances its initially uninitialized lifecycle
    /// to inactive (document.cc:1128), before attaching a LocalFrame.
    pub fn Inactive() -> Self {
        let lifecycle = Self {
            state: Cell::new(LifecycleState::Uninitialized),
            no_transition_count: Cell::new(0),
            style_version: Cell::new(0),
        };
        lifecycle.AdvanceTo(LifecycleState::Inactive);
        lifecycle
    }
    pub fn GetState(&self) -> LifecycleState {
        self.state.get()
    }
    pub fn IsActive(&self) -> bool {
        self.GetState() > LifecycleState::Inactive && self.GetState() < LifecycleState::Stopping
    }
    pub fn AdvanceTo(&self, next: LifecycleState) {
        debug_assert_eq!(
            self.no_transition_count.get(),
            0,
            "DocumentLifecycle transition disallowed by source hit-test scope"
        );
        debug_assert!(
            self.CanAdvanceTo(next),
            "invalid document lifecycle {:?} -> {:?}",
            self.GetState(),
            next
        );
        self.state.set(next);
    }
    pub fn EnsureStateAtMost(&self, next: LifecycleState) {
        debug_assert_eq!(
            self.no_transition_count.get(),
            0,
            "DocumentLifecycle rewind disallowed by source hit-test scope"
        );
        use LifecycleState::*;
        debug_assert!(matches!(
            next,
            VisualUpdatePending | StyleClean | LayoutClean
        ));
        if self.GetState() <= next {
            return;
        }
        debug_assert!(
            self.CanRewind(),
            "cannot rewind document lifecycle {:?} -> {:?}",
            self.GetState(),
            next
        );
        self.state.set(next);
    }
    /// ScheduleLayoutTreeUpdate rewinds completed rendering state. Changes
    /// during an owning lifecycle pass are consumed by that pass's dirty bits.
    pub fn ScheduleVisualUpdate(&self) {
        // Document::ScheduleLayoutTreeUpdate: one source scheduling transition,
        // not one increment for each invalidation while an update is pending.
        if self.CanRewind() && self.GetState() > LifecycleState::VisualUpdatePending {
            self.EnsureStateAtMost(LifecycleState::VisualUpdatePending);
            self.style_version
                .set(self.style_version.get().wrapping_add(1));
        }
    }
    pub fn StyleVersion(&self) -> u64 {
        self.style_version.get()
    }
    pub fn ScheduleLayout(&self) {
        if self.CanRewind() {
            self.EnsureStateAtMost(LifecycleState::StyleClean);
        }
    }
    fn CanRewind(&self) -> bool {
        use LifecycleState::*;
        matches!(
            self.GetState(),
            StyleClean
                | AfterPerformLayout
                | LayoutClean
                | CompositingInputsClean
                | PrePaintClean
                | PaintClean
        )
    }
    fn CanAdvanceTo(&self, next: LifecycleState) -> bool {
        use LifecycleState::*;
        if next == Stopping {
            return true;
        }
        match self.GetState() {
            Uninitialized => next == Inactive,
            Inactive => next == StyleClean,
            VisualUpdatePending => matches!(
                next,
                InStyleRecalc | InPerformLayout | InCompositingInputsUpdate
            ),
            InStyleRecalc => next == StyleClean,
            StyleClean => matches!(
                next,
                InStyleRecalc
                    | InPerformLayout
                    | StyleClean
                    | LayoutClean
                    | InCompositingInputsUpdate
            ),
            InPerformLayout => next == AfterPerformLayout,
            AfterPerformLayout => matches!(next, InPerformLayout | LayoutClean),
            LayoutClean => matches!(
                next,
                InStyleRecalc
                    | InPerformLayout
                    | LayoutClean
                    | StyleClean
                    | InCompositingInputsUpdate
                    | InPrePaint
            ),
            InCompositingInputsUpdate => next == CompositingInputsClean,
            CompositingInputsClean => {
                matches!(next, InStyleRecalc | InCompositingInputsUpdate | InPrePaint)
            }
            InPrePaint => next == PrePaintClean,
            PrePaintClean => matches!(
                next,
                InPaint | InStyleRecalc | InCompositingInputsUpdate | InPrePaint
            ),
            InPaint => next == PaintClean,
            PaintClean => matches!(
                next,
                InStyleRecalc | CompositingInputsClean | InPrePaint | InPaint
            ),
            Stopping => next == Stopped,
            Stopped => false,
        }
    }
}

/// DocumentLifecycle::DisallowTransitionScope, h:117-136. The same Rc owner
/// permits a synchronous query guard without retaining a RefCell document borrow.
pub struct DisallowTransitionScope {
    owner: std::rc::Rc<DocumentLifecycle>,
}
impl DocumentLifecycle {
    pub fn DisallowTransition(self: &std::rc::Rc<Self>) -> DisallowTransitionScope {
        self.no_transition_count.set(
            self.no_transition_count
                .get()
                .checked_add(1)
                .expect("lifecycle scope count overflow"),
        );
        DisallowTransitionScope {
            owner: self.clone(),
        }
    }
}
impl Drop for DisallowTransitionScope {
    fn drop(&mut self) {
        self.owner.no_transition_count.set(
            self.owner
                .no_transition_count
                .get()
                .checked_sub(1)
                .expect("balanced lifecycle scope"),
        );
    }
}
