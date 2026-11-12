//! Canonical Element-owned DisplayLockContext.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Source: core/display_lock/display_lock_context.h:28-172,301-328,454-605;
//! display_lock_context.cc:82-159,253-328,363-410,413-512,547-629,879-933,
//! 960-1005,1102-1108,1220-1308.
#![allow(non_snake_case)]
use super::editing_node::DOMNodeClass;
use super::{DOMNamespace, DOMNodeType, PersistentDocument};
use foundation::{EContentVisibility, Persistent};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use std::cell::RefMut;
use std::collections::HashSet;

#[path = "display_lock_forced_scope.rs"]
mod forced_scope;
pub use forced_scope::{DisplayLockForcedEffect, DisplayLockForcedScopeHandle};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum DisplayLockActivationReason {
    Accessibility = 1 << 0,
    FindInPage = 1 << 1,
    FragmentNavigation = 1 << 2,
    ScriptFocus = 1 << 3,
    ScrollIntoView = 1 << 4,
    Selection = 1 << 5,
    SimulatedClick = 1 << 6,
    UserFocus = 1 << 7,
    ViewportIntersection = 1 << 8,
    Viewport = (1 << 0) | (1 << 5) | (1 << 7) | (1 << 8),
    Any = (1 << 9) - 1,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisplayLockForcedPhase {
    #[default]
    None,
    StyleAndLayoutTree,
    Layout,
    PrePaint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayLockRenderAffectingState {
    IntersectsViewport,
    SubtreeHasFocus,
    SubtreeHasSelection,
    AutoUnlockedForPrint,
    SubtreeHasTopLayerElement,
    DescendantIsViewTransitionElement,
    DescendantIsAnchorTarget,
    HasScrollerWithScrollMarkerGroup,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplayLockLifecycleInputs {
    pub has_view: bool,
    pub screen_reader_active: bool,
    pub skipped_container_recalc: bool,
}
#[derive(Clone, Debug, Default)]
pub(super) struct DisplayLockDocumentState {
    contexts: HashSet<usize>,
    forced_activatable_count: u32,
    locked_count: u32,
    blocking_all_activation_count: u32,
    inputs: DisplayLockLifecycleInputs,
    forced_scopes: Vec<DisplayLockForcedScopeHandle>,
    forced_effects: std::collections::VecDeque<DisplayLockForcedEffect>,
    forced_render_warnings: u32,
}
#[derive(Clone, Copy)]
struct Environment {
    connected_to_view: bool,
    forced_activatable: bool,
    screen_reader_active: bool,
    skipped_container_recalc: bool,
}
#[derive(Clone, Debug, Default)]
struct ForcedInfo {
    style: u32,
    layout: u32,
    prepaint: u32,
}
impl ForcedInfo {
    fn is_forced(&self, phase: DisplayLockForcedPhase) -> bool {
        match phase {
            DisplayLockForcedPhase::None => panic!("none is not a forced update phase"),
            DisplayLockForcedPhase::StyleAndLayoutTree => {
                self.style != 0 || self.layout != 0 || self.prepaint != 0
            }
            DisplayLockForcedPhase::Layout => self.layout != 0 || self.prepaint != 0,
            DisplayLockForcedPhase::PrePaint => self.prepaint != 0,
        }
    }
    fn start(&mut self, phase: DisplayLockForcedPhase) {
        let count = match phase {
            DisplayLockForcedPhase::None => return,
            DisplayLockForcedPhase::StyleAndLayoutTree => &mut self.style,
            DisplayLockForcedPhase::Layout => &mut self.layout,
            DisplayLockForcedPhase::PrePaint => &mut self.prepaint,
        };
        *count = count
            .checked_add(1)
            .expect("display-lock forced phase overflow");
    }
    fn end(&mut self, phase: DisplayLockForcedPhase) {
        let count = match phase {
            DisplayLockForcedPhase::None => return,
            DisplayLockForcedPhase::StyleAndLayoutTree => &mut self.style,
            DisplayLockForcedPhase::Layout => &mut self.layout,
            DisplayLockForcedPhase::PrePaint => &mut self.prepaint,
        };
        *count = count
            .checked_sub(1)
            .expect("display-lock forced phase underflow");
    }
}
#[derive(Clone, Debug)]
pub(super) struct DisplayLockContext {
    is_locked: bool,
    state: EContentVisibility,
    activation_mask: u16,
    forced: ForcedInfo,
    lock_requested: bool,
    set_requested_state_scope: bool,
    intersects_viewport: bool,
    subtree_has_focus: bool,
    subtree_has_selection: bool,
    auto_unlocked_until_lifecycle: bool,
    auto_unlocked_for_print: bool,
    subtree_has_top_layer: bool,
    descendant_is_view_transition: bool,
    descendant_is_anchor_target: bool,
    has_scroller_with_scroll_marker_group: bool,
    keep_unlocked_count: u32,
    needs_deferred_not_intersecting_signal: bool,
    had_viewport_notification: bool,
    had_lifecycle_since_unlock: bool,
    pending_subtree_checks: bool,
    pending_top_layer_check: bool,
    pending_clear_top_layer: bool,
    pub(super) is_details_slot_element: bool,
    is_hidden_until_found: bool,
    child_layout_was_blocked: bool,
}
impl Default for DisplayLockContext {
    fn default() -> Self {
        Self {
            is_locked: false,
            state: EContentVisibility::kVisible,
            activation_mask: DisplayLockActivationReason::Any as u16,
            forced: ForcedInfo::default(),
            lock_requested: false,
            set_requested_state_scope: false,
            intersects_viewport: false,
            subtree_has_focus: false,
            subtree_has_selection: false,
            auto_unlocked_until_lifecycle: false,
            auto_unlocked_for_print: false,
            subtree_has_top_layer: false,
            descendant_is_view_transition: false,
            descendant_is_anchor_target: false,
            has_scroller_with_scroll_marker_group: false,
            keep_unlocked_count: 0,
            needs_deferred_not_intersecting_signal: false,
            had_viewport_notification: false,
            had_lifecycle_since_unlock: false,
            pending_subtree_checks: false,
            pending_top_layer_check: false,
            pending_clear_top_layer: false,
            is_details_slot_element: false,
            is_hidden_until_found: false,
            child_layout_was_blocked: false,
        }
    }
}
impl DisplayLockContext {
    fn IsActivatable(&self, reason: DisplayLockActivationReason) -> bool {
        self.activation_mask & reason as u16 != 0
    }
    fn ShouldStyleChildren(&self, env: Environment) -> bool {
        !self.is_locked
            || self
                .forced
                .is_forced(DisplayLockForcedPhase::StyleAndLayoutTree)
            || (self.IsActivatable(DisplayLockActivationReason::Any) && env.forced_activatable)
            || (self.state == EContentVisibility::kAuto
                && self.has_scroller_with_scroll_marker_group)
            || (self.IsActivatable(DisplayLockActivationReason::Accessibility)
                && env.screen_reader_active)
    }
    fn ShouldLayoutChildren(&self, env: Environment) -> bool {
        let result = !self.is_locked
            || self.forced.is_forced(DisplayLockForcedPhase::Layout)
            || (self.IsActivatable(DisplayLockActivationReason::Any) && env.forced_activatable)
            || (self.state == EContentVisibility::kAuto
                && self.has_scroller_with_scroll_marker_group)
            || (env.skipped_container_recalc
                && self.IsActivatable(DisplayLockActivationReason::Accessibility)
                && env.screen_reader_active);
        debug_assert!(!result || self.ShouldStyleChildren(env));
        result
    }
    fn ShouldPrePaintChildren(&self, env: Environment) -> bool {
        let result = !self.is_locked
            || self.forced.is_forced(DisplayLockForcedPhase::PrePaint)
            || (self.IsActivatable(DisplayLockActivationReason::Any) && env.forced_activatable);
        debug_assert!(!result || self.ShouldLayoutChildren(env));
        result
    }
    pub(super) fn ShouldPaintChildren(&self) -> bool {
        !self.is_locked
    }
    fn RecomputeLockedState(&mut self) {
        debug_assert_eq!(
            self.state == EContentVisibility::kVisible,
            !self.lock_requested
        );
        let should_lock = self.lock_requested
            && (self.state != EContentVisibility::kAuto
                || (!self.intersects_viewport
                    && !self.subtree_has_focus
                    && !self.subtree_has_selection
                    && !self.auto_unlocked_until_lifecycle
                    && !self.auto_unlocked_for_print
                    && !self.subtree_has_top_layer
                    && !self.descendant_is_view_transition
                    && !self.descendant_is_anchor_target));
        if should_lock != self.is_locked {
            self.is_locked = should_lock;
            if !should_lock {
                self.had_lifecycle_since_unlock = false;
            }
        }
    }
    fn SetRequestedState(&mut self, state: EContentVisibility) {
        if self.state == state {
            return;
        }
        self.state = state;
        match state {
            EContentVisibility::kVisible => self.lock_requested = false,
            EContentVisibility::kAuto => {
                self.had_viewport_notification = false;
                self.activation_mask = DisplayLockActivationReason::Any as u16;
                self.lock_requested = true;
            }
            EContentVisibility::kHidden => {
                self.activation_mask = if self.is_hidden_until_found || self.is_details_slot_element
                {
                    DisplayLockActivationReason::FindInPage as u16
                } else {
                    0
                };
                self.lock_requested = true;
            }
        }
        self.needs_deferred_not_intersecting_signal = false;
        self.RecomputeLockedState();
    }
    fn KeepUnlocked(&mut self, count: u32) {
        debug_assert!(count > 0);
        self.keep_unlocked_count = self.keep_unlocked_count.max(count);
        self.auto_unlocked_until_lifecycle = true;
        self.RecomputeLockedState();
    }
    fn SetRenderState(
        &mut self,
        state: DisplayLockRenderAffectingState,
        flag: bool,
        env: Environment,
    ) {
        if (state == DisplayLockRenderAffectingState::SubtreeHasSelection
            && !flag
            && env.forced_activatable)
            || (state == DisplayLockRenderAffectingState::DescendantIsAnchorTarget && !flag)
        {
            self.KeepUnlocked(1);
        }
        match state {
            DisplayLockRenderAffectingState::IntersectsViewport => self.intersects_viewport = flag,
            DisplayLockRenderAffectingState::SubtreeHasFocus => self.subtree_has_focus = flag,
            DisplayLockRenderAffectingState::SubtreeHasSelection => {
                self.subtree_has_selection = flag
            }
            DisplayLockRenderAffectingState::AutoUnlockedForPrint => {
                self.auto_unlocked_for_print = flag
            }
            DisplayLockRenderAffectingState::SubtreeHasTopLayerElement => {
                self.subtree_has_top_layer = flag
            }
            DisplayLockRenderAffectingState::DescendantIsViewTransitionElement => {
                self.descendant_is_view_transition = flag
            }
            DisplayLockRenderAffectingState::DescendantIsAnchorTarget => {
                self.descendant_is_anchor_target = flag
            }
            DisplayLockRenderAffectingState::HasScrollerWithScrollMarkerGroup => {
                self.has_scroller_with_scroll_marker_group = flag;
                return;
            }
        }
        self.RecomputeLockedState();
    }
    fn CommitForActivation(&mut self, reason: DisplayLockActivationReason) {
        debug_assert!(self.is_locked && self.IsActivatable(DisplayLockActivationReason::Any));
        if matches!(
            reason,
            DisplayLockActivationReason::Accessibility
                | DisplayLockActivationReason::FindInPage
                | DisplayLockActivationReason::FragmentNavigation
                | DisplayLockActivationReason::ScrollIntoView
                | DisplayLockActivationReason::SimulatedClick
        ) {
            self.KeepUnlocked(2);
        }
    }

    fn ContentLockState(
        &self,
        env: Environment,
    ) -> layoutng_assembly::internal::layout_input::ContentLockState {
        layoutng_assembly::internal::layout_input::ContentLockState::new(
            self.is_locked,
            self.ShouldLayoutChildren(env),
            self.ShouldPrePaintChildren(env),
            self.ShouldPaintChildren(),
        )
    }
}

impl PersistentDocument {
    /// Source DisplayLockDocumentState::DisplayLockCount, not locked count.
    pub fn DisplayLockCount(&self) -> usize {
        self.display_lock_document_state.borrow().contexts.len()
    }
    /// Inclusive flat ancestors of a canonical DOM range node. DOM range
    /// traversal never includes the separate native pseudo-element aliases.
    pub fn DisplayLockSelectionAncestors(&self, node: usize) -> Vec<u64> {
        let mut result = Vec::new();
        let mut next = Some(node);
        while let Some(node) = next {
            if self.arena[node].display_lock_context.borrow().is_some() {
                result.push(self.arena[node].id);
            }
            next = self.FlatTreeParent(node);
        }
        result
    }
    // Source display_lock_document_state.cc:295-298.
    pub fn DisplayLockSelectionRemoved(&mut self) {
        let contexts: Vec<_> = self
            .display_lock_document_state
            .borrow()
            .contexts
            .iter()
            .copied()
            .collect();
        for context in contexts {
            self.NotifyDisplayLockSubtreeLostSelection(context);
        }
    }
    fn DisplayLockEnvironment(&self, element: usize) -> Environment {
        let state = self.display_lock_document_state.borrow();
        Environment {
            connected_to_view: state.inputs.has_view && self.IsConnectedToDocument(element),
            forced_activatable: state.forced_activatable_count != 0,
            screen_reader_active: state.inputs.screen_reader_active,
            skipped_container_recalc: state.inputs.skipped_container_recalc,
        }
    }
    fn IsInclusiveFlatAncestor(&self, ancestor: usize, node: usize) -> bool {
        let mut current = Some(node);
        while let Some(node) = current {
            if node == ancestor {
                return true;
            }
            current = self.FlatTreeParent(node);
        }
        false
    }
    fn DisplayLockSubtreeHasFocus(&self, element: usize) -> bool {
        self.DisplayLockEnvironment(element).connected_to_view
            && self
                .FocusedUserActionElements()
                .any(|focused| self.IsInclusiveFlatAncestor(element, focused))
    }
    fn DisplayLockSubtreeHasTopLayer(&self, element: usize) -> bool {
        self.DisplayLockEnvironment(element).connected_to_view
            && self
                .top_layer_elements
                .iter()
                .filter_map(|id| self.FindNodeById(*id))
                .any(|top| self.IsInclusiveFlatAncestor(element, top))
    }
    pub(super) fn EnsureDisplayLockContext(
        &self,
        element: usize,
    ) -> RefMut<'_, DisplayLockContext> {
        assert_eq!(self.arena[element].node_type, DOMNodeType::kElement);
        if self.arena[element].display_lock_context.borrow().is_none() {
            let mut context = DisplayLockContext::default();
            context.subtree_has_focus = self.DisplayLockSubtreeHasFocus(element);
            context.subtree_has_top_layer = self.DisplayLockSubtreeHasTopLayer(element);
            *self.arena[element].display_lock_context.borrow_mut() = Some(context);
            self.display_lock_document_state
                .borrow_mut()
                .contexts
                .insert(element);
        }
        RefMut::map(
            self.arena[element].display_lock_context.borrow_mut(),
            |context| context.as_mut().expect("display lock created"),
        )
    }
    /// Element::RecalcOwnStyle, element.cc:5104-5117. The unchanged style owner
    /// is retained unless the original AdjustElementStyle COW branch is needed.
    pub fn AdjustStyleForDisplayLock(
        &self,
        element: usize,
        style: Persistent<ComputedStyle>,
        has_scroller_ancestor: bool,
    ) -> Persistent<ComputedStyle> {
        let source = unsafe { &*style.Get() };
        if self.arena[element].display_lock_context.borrow().is_none()
            && source.IsContentVisibilityVisible()
        {
            return style;
        }
        let (old_state, old_scope) = {
            let mut context = self.EnsureDisplayLockContext(element);
            let values = (context.state, context.set_requested_state_scope);
            if context.state != source.ContentVisibility() {
                context.set_requested_state_scope = true;
            }
            values
        };
        let (skips_contents, _) = self.ChangeDisplayLock(element, |context, _| {
            context.SetRequestedState(source.ContentVisibility());
            context.has_scroller_with_scroll_marker_group = has_scroller_ancestor;
            context.state != EContentVisibility::kVisible && context.is_locked
        });
        if old_state != source.ContentVisibility() {
            self.ForceActiveDisplayLockScopesForElementDuringStyle(element);
            self.EnsureDisplayLockContext(element)
                .set_requested_state_scope = old_scope;
        }
        if skips_contents {
            let mut builder = ComputedStyleBuilder::from_style(source);
            builder.SetSkipsContents(true);
            return Persistent::from_ptr(builder.TakeStyle().cast_mut());
        }
        style
    }
    pub fn DisplayLockContentState(
        &self,
        element: usize,
    ) -> Option<layoutng_assembly::internal::layout_input::ContentLockState> {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .map(|context| context.ContentLockState(self.DisplayLockEnvironment(element)))
    }
    pub fn DisplayLockIsActivatable(
        &self,
        element: usize,
        reason: DisplayLockActivationReason,
    ) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_some_and(|context| context.IsActivatable(reason))
    }
    pub fn DisplayLockShouldLayoutChildren(&self, element: usize) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_none_or(|context| {
                context.ShouldLayoutChildren(self.DisplayLockEnvironment(element))
            })
    }
    pub fn DisplayLockAllowsSelectionTraversalForEditing(&self, node: usize) -> bool {
        self.arena[node]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_none_or(|context| {
                !context.is_locked || context.IsActivatable(DisplayLockActivationReason::Selection)
            })
    }
    pub fn LockedInclusiveAncestorPreventingLayoutForEditing(&self, node: usize) -> bool {
        // Source utilities.cc:205-214 checks the inclusive element before
        // the exclusive-ancestor helper's connectedness/count fast path.
        if !self.DisplayLockShouldLayoutChildren(node) {
            return true;
        }
        if !self.IsConnectedToDocument(node)
            || self.ShadowHost(node).is_some()
            || self.DisplayLockLockedCount() == 0
        {
            return false;
        }
        let mut current = self.FlatTreeParent(node);
        while let Some(element) = current {
            if !self.DisplayLockShouldLayoutChildren(element) {
                return true;
            }
            current = self.FlatTreeParent(element);
        }
        false
    }
    fn RecordDisplayLockInputChange(&mut self, element: usize, style_changed: bool) {
        if !self.DisplayLockEnvironment(element).connected_to_view {
            return;
        }
        if style_changed {
            self.RecordStyleChange(crate::style_state::StyleChange::Node(element));
        }
        let id = self.arena[element].id;
        self.style_state.geometry_input_nodes.insert(id);
        self.style_state.paint_style_nodes.insert(id);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        self.style_state.measurement_geometry_revision = self
            .style_state
            .measurement_geometry_revision
            .wrapping_add(1);
    }
    /// Completion hook for Lock/Unlock already used by embedders. It changes
    /// actual state only; it does not infer request/mask/forced phases.
    pub fn SetDisplayLockLocked(&mut self, element: usize, locked: bool) {
        let (_, change) = self.ChangeDisplayLock(element, |context, _| {
            context.is_locked = locked;
            if !locked {
                context.had_lifecycle_since_unlock = false;
            }
        });
        if change.phases_changed {
            self.RecordDisplayLockInputChange(element, change.lock_changed);
        }
    }
    pub fn SetDisplayLockRequestedState(&mut self, element: usize, state: EContentVisibility) {
        let old_state = self.EnsureDisplayLockContext(element).state;
        if old_state == state {
            return;
        }
        let previous_scope = self
            .EnsureDisplayLockContext(element)
            .set_requested_state_scope;
        self.EnsureDisplayLockContext(element)
            .set_requested_state_scope = true;
        let (_, change) =
            self.ChangeDisplayLock(element, |context, _| context.SetRequestedState(state));
        if change.phases_changed {
            self.RecordDisplayLockInputChange(element, false);
        }
        self.ForceActiveDisplayLockScopesForElement(element);
        self.EnsureDisplayLockContext(element)
            .set_requested_state_scope = previous_scope;
    }
    pub fn SetDisplayLockRenderAffectingState(
        &mut self,
        element: usize,
        state: DisplayLockRenderAffectingState,
        flag: bool,
    ) {
        let (_, change) = self.ChangeDisplayLock(element, |context, env| {
            context.SetRenderState(state, flag, env)
        });
        if change.phases_changed {
            self.RecordDisplayLockInputChange(element, change.lock_changed);
        }
    }
    pub fn SetDisplayLockLifecycleInputs(&mut self, inputs: DisplayLockLifecycleInputs) {
        let old = self.display_lock_document_state.borrow().inputs;
        if old == inputs {
            return;
        }
        self.display_lock_document_state.get_mut().inputs = inputs;
        let contexts: Vec<_> = self
            .display_lock_document_state
            .get_mut()
            .contexts
            .iter()
            .copied()
            .collect();
        for element in contexts {
            let env = self.DisplayLockEnvironment(element);
            let old_env = Environment {
                connected_to_view: old.has_view && self.IsConnectedToDocument(element),
                screen_reader_active: old.screen_reader_active,
                skipped_container_recalc: old.skipped_container_recalc,
                ..env
            };
            let changed = {
                let mut context = self.EnsureDisplayLockContext(element);
                if old.has_view != inputs.has_view && inputs.has_view {
                    context.pending_subtree_checks = true;
                }
                (
                    context.ShouldStyleChildren(old_env),
                    context.ContentLockState(old_env),
                ) != (
                    context.ShouldStyleChildren(env),
                    context.ContentLockState(env),
                )
            };
            if changed {
                self.RecordDisplayLockInputChange(element, true);
            }
        }
    }
    pub fn StartForcedActivatableDisplayLocks(&mut self) {
        let state = self.display_lock_document_state.get_mut();
        state.forced_activatable_count = state
            .forced_activatable_count
            .checked_add(1)
            .expect("forced display-lock count overflow");
        if state.forced_activatable_count != 1 {
            return;
        }
        let contexts: Vec<_> = state.contexts.iter().copied().collect();
        for element in contexts {
            if self.arena[element]
                .display_lock_context
                .borrow()
                .as_ref()
                .is_some_and(|context| {
                    context.is_locked && context.IsActivatable(DisplayLockActivationReason::Any)
                })
            {
                self.RecordDisplayLockInputChange(element, true);
            }
        }
    }
    pub fn EndForcedActivatableDisplayLocks(&mut self) {
        let state = self.display_lock_document_state.get_mut();
        state.forced_activatable_count = state
            .forced_activatable_count
            .checked_sub(1)
            .expect("forced display-lock count underflow");
        if state.forced_activatable_count != 0 {
            return;
        }
        let contexts: Vec<_> = state.contexts.iter().copied().collect();
        for element in contexts {
            if self.arena[element]
                .display_lock_context
                .borrow()
                .as_ref()
                .is_some_and(|context| {
                    context.is_locked && context.IsActivatable(DisplayLockActivationReason::Any)
                })
            {
                self.RecordDisplayLockInputChange(element, false);
            }
        }
    }
    pub fn UpgradeDisplayLockForcedScope(
        &mut self,
        element: usize,
        old: DisplayLockForcedPhase,
        new: DisplayLockForcedPhase,
    ) {
        assert!(old < new);
        let can_dirty_style = !self
            .EnsureDisplayLockContext(element)
            .set_requested_state_scope
            && self.Lifecycle().GetState()
                != crate::document_lifecycle::LifecycleState::InStyleRecalc;
        let (newly_forced_style, change) = self.ChangeDisplayLock(element, |context, _| {
            let before = context
                .forced
                .is_forced(DisplayLockForcedPhase::StyleAndLayoutTree);
            context.forced.end(old);
            context.forced.start(new);
            !before
                && context
                    .forced
                    .is_forced(DisplayLockForcedPhase::StyleAndLayoutTree)
        });
        if change.phases_changed {
            self.RecordDisplayLockInputChange(element, can_dirty_style && newly_forced_style);
        }
    }
    pub fn EndDisplayLockForcedScope(&mut self, element: usize, phase: DisplayLockForcedPhase) {
        if self.DisplayLockIsLocked(element) {
            self.display_lock_document_state
                .borrow_mut()
                .forced_effects
                .push_back(DisplayLockForcedEffect::RemoveAllPendingUpdates);
        }
        self.ChangeDisplayLock(element, |context, _| context.forced.end(phase));
    }
    pub fn CommitDisplayLockForActivation(
        &mut self,
        element: usize,
        reason: DisplayLockActivationReason,
    ) {
        let should = self.DisplayLockEnvironment(element).connected_to_view
            && self.arena[element]
                .display_lock_context
                .borrow()
                .as_ref()
                .is_some_and(|context| context.is_locked && context.IsActivatable(reason));
        if should {
            let (_, change) =
                self.ChangeDisplayLock(element, |context, _| context.CommitForActivation(reason));
            if change.phases_changed {
                self.RecordDisplayLockInputChange(element, change.lock_changed);
            }
        }
    }
    pub fn NotifyDisplayLockViewportIntersection(&mut self, element: usize, intersects: bool) {
        let nested = self.LockedAncestorPreventingPaintForEditing(element);
        let (_, change) = self.ChangeDisplayLock(element, |context, env| {
            context.had_viewport_notification = true;
            if intersects {
                context.needs_deferred_not_intersecting_signal = false;
                if env.connected_to_view {
                    context.SetRenderState(
                        DisplayLockRenderAffectingState::IntersectsViewport,
                        true,
                        env,
                    );
                }
            } else if !context.is_locked {
                context.needs_deferred_not_intersecting_signal = env.connected_to_view && nested;
                if env.connected_to_view && !nested {
                    context.SetRenderState(
                        DisplayLockRenderAffectingState::IntersectsViewport,
                        false,
                        env,
                    );
                }
            }
        });
        if change.phases_changed {
            self.RecordDisplayLockInputChange(element, change.lock_changed);
        }
    }
    pub fn WillStartDisplayLockLifecycleUpdate(&mut self) {
        let contexts: Vec<_> = self
            .display_lock_document_state
            .get_mut()
            .contexts
            .iter()
            .copied()
            .collect();
        for element in contexts {
            let work = self.arena[element]
                .display_lock_context
                .borrow()
                .as_ref()
                .map(|context| {
                    (
                        context.needs_deferred_not_intersecting_signal,
                        context.pending_subtree_checks,
                        context.pending_top_layer_check,
                        context.NeedsLifecycleNotifications(),
                    )
                });
            let Some((deferred, subtree, top, true)) = work else {
                continue;
            };
            if deferred {
                self.NotifyDisplayLockViewportIntersection(element, false);
            }
            let focus = subtree.then(|| self.DisplayLockSubtreeHasFocus(element));
            let top_layer = (subtree || top).then(|| self.DisplayLockSubtreeHasTopLayer(element));
            let (_, change) = self.ChangeDisplayLock(element, |context, env| {
                // Source counts down its keep-unlocked state before subtree checks.
                if context.keep_unlocked_count != 0 {
                    context.keep_unlocked_count -= 1;
                    if context.keep_unlocked_count == 0 {
                        context.auto_unlocked_until_lifecycle = false;
                        context.RecomputeLockedState();
                    }
                } else {
                    debug_assert!(!context.auto_unlocked_until_lifecycle);
                }
                if let Some(flag) = top_layer {
                    if env.connected_to_view {
                        context.pending_clear_top_layer = !flag;
                        if flag {
                            context.SetRenderState(
                                DisplayLockRenderAffectingState::SubtreeHasTopLayerElement,
                                true,
                                env,
                            );
                        }
                    }
                    context.pending_top_layer_check = false;
                }
                if let Some(flag) = focus {
                    context.SetRenderState(
                        DisplayLockRenderAffectingState::SubtreeHasFocus,
                        flag,
                        env,
                    );
                    // Selection notifications are delivered to the same retained
                    // context by the canonical FrameSelection owner.
                    context.pending_subtree_checks = false;
                }
                if context.pending_clear_top_layer {
                    context.SetRenderState(
                        DisplayLockRenderAffectingState::SubtreeHasTopLayerElement,
                        false,
                        env,
                    );
                    context.pending_clear_top_layer = false;
                }
            });
            if change.phases_changed {
                self.RecordDisplayLockInputChange(element, change.lock_changed);
            }
        }
    }
    pub fn DidLayoutDisplayLockChildren(&mut self, element: usize) {
        let mut context = self.EnsureDisplayLockContext(element);
        context.child_layout_was_blocked = false;
        context.had_lifecycle_since_unlock = true;
    }
    pub fn SetDisplayLockHiddenUntilFound(&mut self, element: usize, flag: bool) {
        if flag || self.arena[element].display_lock_context.borrow().is_some() {
            self.EnsureDisplayLockContext(element).is_hidden_until_found = flag;
        }
    }
    pub(super) fn DisplayLockElementDisconnected(&mut self, element: usize) {
        if self.arena[element].display_lock_context.borrow().is_some() {
            self.SetDisplayLockRequestedState(element, EContentVisibility::kVisible);
        }
    }
}

impl PersistentDocument {
    // Source: display_lock_context.cc:1021-1095. Dynamic class identity comes
    // from Node's allocation owner; this never guesses lock state from a tag.
    fn DisplayLockStyleRequiresForceUnlock(&self, element: usize, style: &ComputedStyle) -> bool {
        use foundation::EDisplay;
        if style.Display() == EDisplay::kContents
            || !style.ContainsStyle()
            || !style.ContainsLayout()
        {
            return true;
        }
        match &self.arena[element].class {
            DOMNodeClass::HTMLObjectElement(owner) => {
                if !owner.UseFallbackContent() {
                    return false;
                }
            }
            DOMNodeClass::HTMLImageElement
            | DOMNodeClass::HTMLCanvasElement
            | DOMNodeClass::HTMLInputElement
            | DOMNodeClass::MediaControlInput { .. }
            | DOMNodeClass::HTMLSelectElement(_)
            | DOMNodeClass::HTMLTextAreaElement
            | DOMNodeClass::HTMLButtonElement
            | DOMNodeClass::HTMLFieldSetElement
            | DOMNodeClass::HTMLProgressElement
            | DOMNodeClass::HTMLMeterElement
            | DOMNodeClass::HTMLMediaElement { .. }
            | DOMNodeClass::HTMLFrameElementBase => return false,
            _ if self.arena[element].namespace == DOMNamespace::kSVG => return false,
            _ => {}
        }
        (style.IsDisplayTableType() && style.Display() != EDisplay::kTableCell)
            || style.Display() == EDisplay::kRubyText
            || style.IsNonAtomicInlineDisplayType()
    }
    pub fn DisplayLockDidStyleSelf(&self, element: usize) {
        if self.arena[element].display_lock_context.borrow().is_none() {
            return;
        }
        let style = self
            .ResolvedStyleFor(element)
            .and_then(|style| unsafe { style.computed_style_owner.Get().as_ref() });
        let force_unlock =
            style.is_some_and(|style| self.DisplayLockStyleRequiresForceUnlock(element, style));
        self.ChangeDisplayLock(element, |context, _| {
            if style.is_none() {
                context.SetRequestedState(EContentVisibility::kVisible);
                return;
            }
            if force_unlock {
                if context.is_locked {
                    context.is_locked = false;
                    context.had_lifecycle_since_unlock = false;
                    context.SetRequestedState(EContentVisibility::kVisible);
                }
                return;
            }
            if !context.is_locked && context.state != EContentVisibility::kVisible {
                context.RecomputeLockedState();
            }
        });
    }
    pub(super) fn RefreshDisplayLockFocusAncestors(&mut self, node: usize) {
        let mut current = Some(node);
        while let Some(element) = current {
            current = self.FlatTreeParent(element);
            if self.arena[element].display_lock_context.borrow().is_none() {
                continue;
            }
            let flag = self.DisplayLockSubtreeHasFocus(element);
            self.SetDisplayLockRenderAffectingState(
                element,
                DisplayLockRenderAffectingState::SubtreeHasFocus,
                flag,
            );
        }
    }
    pub(super) fn NotifyDisplayLockTopLayerAdded(&mut self, node: usize) {
        let mut current = Some(node);
        while let Some(element) = current {
            current = self.FlatTreeParent(element);
            if self.arena[element].display_lock_context.borrow().is_none() {
                continue;
            }
            {
                let mut context = self.EnsureDisplayLockContext(element);
                context.pending_clear_top_layer = false;
            }
            self.SetDisplayLockRenderAffectingState(
                element,
                DisplayLockRenderAffectingState::SubtreeHasTopLayerElement,
                true,
            );
        }
    }
    pub(super) fn ScheduleDisplayLockTopLayerCheck(&mut self, node: usize) {
        let mut current = Some(node);
        while let Some(element) = current {
            current = self.FlatTreeParent(element);
            if let Some(context) = self.arena[element]
                .display_lock_context
                .borrow_mut()
                .as_mut()
            {
                context.pending_top_layer_check = true;
            }
        }
    }
    pub(super) fn DisplayLockElementConnected(&self, element: usize) {
        // Called within the existing shadow-including node insertion walk.
        // Source ElementConnected defers focus/selection/top-layer checks.
        if let Some(context) = self.arena[element]
            .display_lock_context
            .borrow_mut()
            .as_mut()
        {
            context.pending_subtree_checks = true;
        }
    }
}

#[derive(Clone, Copy)]
struct DisplayLockChange {
    phases_changed: bool,
    lock_changed: bool,
}
impl DisplayLockContext {
    fn NeedsLifecycleNotifications(&self) -> bool {
        self.needs_deferred_not_intersecting_signal
            || self.auto_unlocked_until_lifecycle
            || self.pending_subtree_checks
            || self.pending_clear_top_layer
            || self.pending_top_layer_check
    }
}
impl PersistentDocument {
    fn ChangeDisplayLock<R>(
        &self,
        element: usize,
        update: impl FnOnce(&mut DisplayLockContext, Environment) -> R,
    ) -> (R, DisplayLockChange) {
        let env = self.DisplayLockEnvironment(element);
        let (result, old_locked, old_blocking, new_locked, new_blocking, phases_changed) = {
            let mut context = self.EnsureDisplayLockContext(element);
            let old_locked = context.is_locked;
            let old_blocking = old_locked && context.activation_mask == 0;
            let before = (
                context.ShouldStyleChildren(env),
                context.ContentLockState(env),
            );
            let result = update(&mut context, env);
            let after = (
                context.ShouldStyleChildren(env),
                context.ContentLockState(env),
            );
            (
                result,
                old_locked,
                old_blocking,
                context.is_locked,
                context.is_locked && context.activation_mask == 0,
                before != after,
            )
        };
        let mut state = self.display_lock_document_state.borrow_mut();
        fn adjust(count: &mut u32, was: bool, now: bool) {
            if was != now {
                if now {
                    *count = count.checked_add(1).expect("display-lock count overflow");
                } else {
                    *count = count.checked_sub(1).expect("display-lock count underflow");
                }
            }
        }
        adjust(&mut state.locked_count, old_locked, new_locked);
        adjust(
            &mut state.blocking_all_activation_count,
            old_blocking,
            new_blocking,
        );
        (
            result,
            DisplayLockChange {
                phases_changed,
                lock_changed: old_locked != new_locked,
            },
        )
    }
    pub fn DisplayLockLockedCount(&self) -> u32 {
        self.display_lock_document_state.borrow().locked_count
    }
    pub fn DisplayLockNeedsLifecycleUpdate(&self) -> bool {
        self.display_lock_document_state
            .borrow()
            .contexts
            .iter()
            .any(|&element| {
                self.arena[element]
                    .display_lock_context
                    .borrow()
                    .as_ref()
                    .is_some_and(DisplayLockContext::NeedsLifecycleNotifications)
            })
    }
    pub fn DisplayLockRequestedState(&self, element: usize) -> Option<EContentVisibility> {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .map(|context| context.state)
    }
    pub fn DisplayLockIsLocked(&self, element: usize) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_some_and(|context| context.is_locked)
    }
    pub fn DisplayLockShouldStyleChildren(&self, element: usize) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_none_or(|context| context.ShouldStyleChildren(self.DisplayLockEnvironment(element)))
    }
    pub fn DisplayLockShouldPrePaintChildren(&self, element: usize) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_none_or(|context| {
                context.ShouldPrePaintChildren(self.DisplayLockEnvironment(element))
            })
    }
}

impl PersistentDocument {
    pub fn DisplayLockShouldCommitForActivation(
        &self,
        element: usize,
        reason: DisplayLockActivationReason,
    ) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_some_and(|context| context.is_locked && context.IsActivatable(reason))
    }
    pub fn DisplayLockShouldPaintChildren(&self, element: usize) -> bool {
        self.arena[element]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_none_or(DisplayLockContext::ShouldPaintChildren)
    }
    pub fn DisplayLockBlockingAllActivationCount(&self) -> u32 {
        self.display_lock_document_state
            .borrow()
            .blocking_all_activation_count
    }
}

impl PersistentDocument {
    pub fn NotifyDisplayLockSubtreeGainedSelection(&mut self, element: usize) {
        if self.arena[element].display_lock_context.borrow().is_some() {
            self.SetDisplayLockRenderAffectingState(
                element,
                DisplayLockRenderAffectingState::SubtreeHasSelection,
                true,
            );
        }
    }
    pub fn NotifyDisplayLockSubtreeLostSelection(&mut self, element: usize) {
        if self.arena[element].display_lock_context.borrow().is_some() {
            self.SetDisplayLockRenderAffectingState(
                element,
                DisplayLockRenderAffectingState::SubtreeHasSelection,
                false,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_activation_and_nested_forcing_keep_paint_locked() {
        // Source hidden details locks permit find-in-page, not selection.
        let mut context = DisplayLockContext {
            is_details_slot_element: true,
            ..Default::default()
        };
        context.SetRequestedState(EContentVisibility::kHidden);
        let env = Environment {
            connected_to_view: true,
            forced_activatable: false,
            screen_reader_active: false,
            skipped_container_recalc: false,
        };
        assert!(context.IsActivatable(DisplayLockActivationReason::FindInPage));
        assert!(!context.IsActivatable(DisplayLockActivationReason::Selection));
        assert!(!context.ShouldLayoutChildren(env));
        context.forced.start(DisplayLockForcedPhase::Layout);
        context.forced.start(DisplayLockForcedPhase::PrePaint);
        assert!(context.ShouldPrePaintChildren(env));
        context.forced.end(DisplayLockForcedPhase::PrePaint);
        assert!(context.ShouldLayoutChildren(env));
        assert!(!context.ShouldPrePaintChildren(env));
        assert!(!context.ShouldPaintChildren());
        context.forced.end(DisplayLockForcedPhase::Layout);
        assert!(!context.ShouldLayoutChildren(env));
    }
    #[test]
    fn disconnected_inclusive_query_checks_own_lock_first() {
        let mut document = PersistentDocument::new();
        let element = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.SetDisplayLockRequestedState(element, EContentVisibility::kHidden);
        assert!(document.LockedInclusiveAncestorPreventingLayoutForEditing(element));
        assert!(!document.LockedAncestorPreventingPaintForEditing(element));
        assert_eq!(document.DisplayLockLockedCount(), 1);
        document.SetDisplayLockRequestedState(element, EContentVisibility::kVisible);
        assert_eq!(document.DisplayLockLockedCount(), 0);
    }
}

impl PersistentDocument {
    // DisplayLockUtilities::LockedInclusiveAncestorPreventingPaint,
    // display_lock_utilities.cc:523-529. Same context and flat ancestry owner.
    pub fn LockedInclusiveAncestorPreventingPaintForEditing(&self, node: usize) -> bool {
        if !self.IsConnectedToDocument(node)
            || self.ShadowHost(node).is_some()
            || self.DisplayLockLockedCount() == 0
        {
            return false;
        }
        !self.DisplayLockShouldPaintChildren(node)
            || self.LockedAncestorPreventingPaintForEditing(node)
    }
}

impl PersistentDocument {
    /// DisplayLockUtilities::ElementLostFocus/ElementGainedFocus, cc:692-726;
    /// DisplayLockContext::NotifySubtreeLost/GainedFocus, cc:1102-1108.
    /// Canonical DOM nodes currently represent ordinary Elements and UA shadow
    /// descendants. Generated scroll-marker pseudo nodes have no DOM arena
    /// identity and cannot enter this source path through a synthetic alias.
    pub fn NotifyDisplayLockFocusChanged(&mut self, element: usize, focused: bool) {
        if self
            .display_lock_document_state
            .borrow()
            .contexts
            .is_empty()
        {
            return;
        }
        let mut ancestor = Some(element);
        while let Some(node) = ancestor {
            ancestor = self.FlatTreeParent(node);
            if self.arena[node].display_lock_context.borrow().is_some() {
                self.SetDisplayLockRenderAffectingState(
                    node,
                    DisplayLockRenderAffectingState::SubtreeHasFocus,
                    focused,
                );
            }
        }
    }
    /// Element::ActivateDisplayLockIfNeeded, element.cc:9205-9236. Collect the
    /// native contexts before committing any, so a non-activatable ancestor
    /// prevents every activation in this transaction.
    pub fn ActivateDisplayLockForFocus(&mut self, element: usize, script_focus: bool) -> bool {
        self.ActivateDisplayLockIfNeeded(
            element,
            if script_focus {
                DisplayLockActivationReason::ScriptFocus
            } else {
                DisplayLockActivationReason::UserFocus
            },
        )
    }
    pub fn ActivateDisplayLockIfNeeded(
        &mut self,
        element: usize,
        reason: DisplayLockActivationReason,
    ) -> bool {
        {
            let document = self.display_lock_document_state.borrow();
            if document.locked_count == document.blocking_all_activation_count {
                return false;
            }
        }
        let mut targets = Vec::new();
        let mut ancestor = Some(element);
        while let Some(node) = ancestor {
            ancestor = self.FlatTreeParent(node);
            if let Some(context) = self.arena[node].display_lock_context.borrow().as_ref() {
                if context.is_locked && !context.IsActivatable(reason) {
                    return false;
                }
                targets.push(node);
            }
        }
        let mut activated = false;
        for target in targets {
            if self.DisplayLockShouldCommitForActivation(target, reason) {
                activated = true;
                self.CommitDisplayLockForActivation(target, reason);
            }
        }
        activated
    }
}

impl PersistentDocument {
    /// DisplayLockDocumentState.cc:191-193.
    pub fn DisplayLockHasActivatableLocks(&self) -> bool {
        let state = self.display_lock_document_state.borrow();
        state.locked_count != state.blocking_all_activation_count
    }
    /// DisplayLockUtilities.cc:268-295. The editing owner supplies the exact
    /// EnclosingBlock(kCanCrossEditingBoundary), and DOM owns this ancestor walk.
    pub fn DisplayLockNeedsActivationForFindInPage(&self, enclosing_block: usize) -> bool {
        if !self.DisplayLockHasActivatableLocks() {
            return false;
        }
        let mut ancestor = Some(enclosing_block);
        while let Some(node) = ancestor {
            if self
                .DisplayLockShouldCommitForActivation(node, DisplayLockActivationReason::FindInPage)
            {
                return true;
            }
            ancestor = self.Node(node).Parent();
        }
        false
    }
}
