// Copyright (C) 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
// This library is free software under the GNU Library General Public License,
// version 2 or (at your option) any later version; see Chromium COPYING.LIB.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/frame_selection.h:155,166-182,380
// Source: third_party/blink/renderer/core/editing/frame_selection.cc:231-298,313-465,554-556,718-725

use super::{
    set_selection_options::{CursorAlignOnScroll, SetSelectionBy, SetSelectionOptions},
    text_granularity::TextGranularity,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub trait SelectionValue: Clone + PartialEq {
    fn IsNone(&self) -> bool;
}

/// Native owner leaves for frame_selection.cc:976-1039. Handles retain the
/// original Frame/Element identities through synchronous focus listener reentry.
pub trait FrameOwnerSelectionSource {
    type Frame;
    type Owner;
    fn Parent(&self) -> Option<Self::Frame>;
    fn HasPage(&self) -> bool;
    fn SelectionIsRange(&self) -> bool;
    fn UpdateStyleAndLayoutForSelection(&self);
    fn VisibleSelectionCoversDocument(&self) -> bool;
    fn LocalParentOwner(&self, parent: &Self::Frame) -> Option<Self::Owner>;
    fn UpdateOwnerDocumentStyleAndLayout(&self, owner: &Self::Owner) -> bool;
    fn OwnerParentIsEditable(&self, owner: &Self::Owner) -> bool;
    fn FocusParent(&self, parent: &Self::Frame) -> bool;
    fn OwnerIsConnectedToParentDocument(&self, owner: &Self::Owner, parent: &Self::Frame) -> bool;
    fn SelectOwnerInParent(&self, owner: &Self::Owner, parent: &Self::Frame);
}
pub fn SelectFrameElementInParentIfFullySelected<S: FrameOwnerSelectionSource>(source: &S) {
    let Some(parent) = source.Parent() else {
        return;
    };
    if !source.HasPage() || !source.SelectionIsRange() {
        return;
    }
    source.UpdateStyleAndLayoutForSelection();
    if !source.VisibleSelectionCoversDocument() {
        return;
    }
    let Some(owner) = source.LocalParentOwner(&parent) else {
        return;
    };
    if !source.UpdateOwnerDocumentStyleAndLayout(&owner) || !source.OwnerParentIsEditable(&owner) {
        return;
    }
    if !source.FocusParent(&parent) {
        return;
    }
    if source.OwnerIsConnectedToParentDocument(&owner, &parent) {
        source.SelectOwnerInParent(&owner, &parent);
    }
}
/// The original optional granularity strategy owns its drag-extension state.
pub trait GranularityStrategy {
    fn Clear(&self);
}
/// The canonical SelectionEditor owns S and live DOM positions. The interaction
/// wrapper stores only Chromium's derived flags, never another selection tree.
pub trait SelectionOwner<S: SelectionValue> {
    fn Current(&self) -> S;
    fn SetSelectionAndEndTyping(&self, selection: &S);
    fn EmptySelection(&self) -> S;
    fn HasEditableRoot(&self) -> bool;
    fn CaretSelectionForContentsPoint(&self, point: (i32, i32)) -> S;
    fn DocumentIdentity(&self) -> u128;
    fn IsAvailable(&self) -> bool;
}
pub trait SelectionEffects<S: SelectionValue> {
    fn AlwaysUseDirectionalSelection(&self) -> bool;
    fn ClearGranularityStrategy(&self);
    fn CloseTyping(&self);
    fn ClearTypingStyle(&self);
    fn NotifyDisplayLock(&self, old: &S, new: &S);
    fn ScheduleVisualUpdateForVisualOverflow(&self);
    fn SetFocusedNodeIfNeeded(&self);
    fn StopCaretBlinkTimer(&self);
    fn UpdateAppearance(&self);
    fn SelectFrameElementInParentIfFullySelected(&self);
    fn NotifyTextControlOfSelectionChange(&self, by: SetSelectionBy);
    fn CenterAlignRevealedSelection(&self) -> bool;
    fn RevealSelection(&self, alignment: SelectionAlignment, reveal_extent: RevealExtentOption);
    fn NotifyAccessibilityForSelectionChange(&self);
    fn NotifyCompositorForSelectionChange(&self);
    fn NotifyEventHandlerForSelectionChange(&self);
    fn EnclosingTextControlIsOutsideShadowTree(&self) -> bool;
    fn ScheduleTextControlSelectionchange(&self);
    fn ScheduleDocumentSelectionchange(&self);
    fn EnqueueLegacyDocumentSelectionchange(&self);
    fn RespondToChangedSelection(&self);
    fn DidChangeSelectionAfterCommand(&self, is_caret: bool);
    fn GetEditorFrameClient(&self) -> Option<std::rc::Rc<dyn super::editor::EditorFrameClient>>;
    fn HandleAXEditableTextContentChanged(&self, anchor: Option<&S>);
    fn SpellCheckerRespondToChangedContents(&self);
    fn SpellCheckerRespondToChangedSelection(&self);
    fn FrameClientDidChangeContents(&self);
    fn CurrentEventType(&self) -> Option<String>;
}
#[derive(Clone, Copy)]
pub enum SelectionAlignment {
    CenterAlways,
    CenterIfNeeded,
    TopAlways,
    EdgeIfNeeded,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RevealExtentOption {
    #[default]
    kDoNotRevealExtent,
    kRevealExtent,
}

/// Borrowed native geometry and lifecycle leaves for RevealSelection. The
/// canonical FrameSelection and LayoutSelection remain their original owners.
pub trait SelectionRevealSource<S: SelectionValue> {
    type Node: Copy;
    fn UpdateStyleAndLayoutForSelection(&self);
    fn ComputeVisibleSelectionInDOMTree(&self, selection: &S) -> S;
    fn SelectionIsCaret(&self, selection: &S) -> bool;
    fn VisibleSelectionStartAnchor(&self, selection: &S) -> Option<Self::Node>;
    fn MarkInitialScrollWasScrolledByUser(&self);
    fn HasLayoutObject(&self, node: Self::Node) -> bool;
    fn UpdateStyleAndLayoutForNode(&self, node: Self::Node);
    fn AbsoluteCaretBounds(&self) -> foundation::gfx::Rect;
    /// AbsoluteCaretBoundsOf(CreateVisiblePosition(selection.Focus())).
    fn AbsoluteCaretBoundsOfVisibleFocus(&self, selection: &S) -> foundation::gfx::Rect;
    fn SetLayoutSelectionHasPendingSelection(&self);
    fn AbsoluteSelectionBounds(&self) -> foundation::gfx::Rect;
    fn ScrollRectToVisible(
        &self,
        node: Self::Node,
        rect: foundation::gfx::Rect,
        alignment: SelectionAlignment,
    );
}
#[derive(Clone, Copy, Default)]
pub struct FrameSelectionFeatures {
    pub bidi_visual_order_caret_movement: bool,
    pub notify_unchanged: bool,
    pub per_element_selectionchange: bool,
}

// Source: frame_selection.cc:1206-1248,1444-1470. These leaves refer to
// existing Document/LayoutSelection/FrameCaret/EventHandler owners.
pub trait SelectionFocusEffects<S: SelectionValue> {
    fn PageIsActive(&self) -> bool;
    /// C++ SelectorChecker reads these live frame/page flags. Carry only the
    /// same values across the independent Style package's environment input.
    fn PublishFocusState(&self, frame_focused_and_active: bool, page_is_active: bool);
    fn FocusedElementFocusStateChanged(&self);
    fn HasLayoutView(&self) -> bool;
    fn InvalidateStyleAndPaintForSelection(&self);
    fn UpdateStyleAndLayoutTree(&self);
    fn UpdateStyleAndLayoutForSelection(&self);
    fn DocumentIsEditable(&self) -> bool;
    fn SelectionAtBodyStart(&self) -> Option<S>;
    fn SetCaretEnabled(&self, enabled: bool);
    fn CapsLockStateMayHaveChanged(&self, frame_focused_and_active: bool);
}
/// Source frame_selection.cc931-967. These are transient visible positions,
/// sourced from the same SelectionEditor and native read-only/active hit test.
pub trait SelectionContainmentSource<S: SelectionValue> {
    type Position: Clone + PartialOrd;
    fn HasLayoutView(&self) -> bool;
    fn ComputeVisibleSelectionInDOMTreeIsNonNull(&self, selection: &S) -> bool;
    fn VisibleFlatSelectionEndpoints(
        &self,
        selection: &S,
    ) -> Option<(Self::Position, Self::Position)>;
    fn HitTestPositionReadOnlyActive(
        &self,
        point: &foundation::PhysicalOffset,
    ) -> Option<Self::Position>;
}
pub trait SelectionBoundsSource {
    fn HasView(&self) -> bool;
    fn HasLayoutView(&self) -> bool;
    fn AbsoluteSelectionBounds(&self) -> foundation::gfx::Rect;
}
pub struct FrameSelection<S: SelectionValue> {
    granularity_strategy: RefCell<Option<Rc<dyn GranularityStrategy>>>,
    absolute_bounds: super::selection_editor::SelectionEditorAbsoluteBounds,
    focused_: Cell<bool>,
    focus_effects_: std::cell::OnceCell<Rc<dyn SelectionFocusEffects<S>>>,
    pub owner: Rc<dyn SelectionOwner<S>>,
    pub effects: Rc<dyn SelectionEffects<S>>,
    pub features: FrameSelectionFeatures,
    pub granularity: Cell<TextGranularity>,
    pub is_handle_visible: Cell<bool>,
    pub is_directional: Cell<bool>,
    pub should_shrink_next_tap: Cell<bool>,
    pub is_being_modified: Cell<bool>,
    pub caret_bidi_level: Cell<Option<u8>>,
    pub entered_bidi_run: Cell<bool>,
    pub x_pos_for_vertical_arrow_navigation: Cell<foundation::LayoutUnit>,
}
impl<S: SelectionValue> FrameSelection<S> {
    pub fn new(
        owner: Rc<dyn SelectionOwner<S>>,
        effects: Rc<dyn SelectionEffects<S>>,
        features: FrameSelectionFeatures,
    ) -> Self {
        let is_directional = effects.AlwaysUseDirectionalSelection();
        Self {
            granularity_strategy: RefCell::new(None),
            absolute_bounds: Default::default(),
            focused_: Cell::new(false),
            focus_effects_: std::cell::OnceCell::new(),
            owner,
            effects,
            features,
            granularity: Cell::new(TextGranularity::kCharacter),
            is_handle_visible: Cell::new(false),
            is_directional: Cell::new(is_directional),
            should_shrink_next_tap: Cell::new(false),
            is_being_modified: Cell::new(false),
            caret_bidi_level: Cell::new(None),
            entered_bidi_run: Cell::new(false),
            x_pos_for_vertical_arrow_navigation: Cell::new(
                super::selection_modifier::NoXPosForVerticalArrowNavigation(),
            ),
        }
    }
    // Source: frame_selection.cc:1390-1402. The second visible-selection read
    // is intentional: the preceding node lifecycle update can change it.
    pub fn ComputeRectToScroll<R: SelectionRevealSource<S>>(
        &self,
        source: &R,
        reveal_extent: RevealExtentOption,
    ) -> foundation::gfx::Rect {
        let selection = source.ComputeVisibleSelectionInDOMTree(&self.owner.Current());
        if source.SelectionIsCaret(&selection) {
            return source.AbsoluteCaretBounds();
        }
        debug_assert!(!selection.IsNone());
        if reveal_extent == RevealExtentOption::kRevealExtent {
            return source.AbsoluteCaretBoundsOfVisibleFocus(&selection);
        }
        source.SetLayoutSelectionHasPendingSelection();
        source.AbsoluteSelectionBounds()
    }
    // Source: frame_selection.cc:1405-1442; frame_selection.h:317-319.
    pub fn RevealSelection<R: SelectionRevealSource<S>>(
        &self,
        source: &R,
        alignment: SelectionAlignment,
        reveal_extent: RevealExtentOption,
    ) {
        debug_assert!(self.owner.IsAvailable());
        source.UpdateStyleAndLayoutForSelection();
        let selection = source.ComputeVisibleSelectionInDOMTree(&self.owner.Current());
        if selection.IsNone() {
            return;
        }
        source.MarkInitialScrollWasScrolledByUser();
        let start = source
            .VisibleSelectionStartAnchor(&selection)
            .expect("visible selection has a start anchor");
        if !source.HasLayoutObject(start) {
            return;
        }
        source.UpdateStyleAndLayoutForNode(start);
        let rect = self.ComputeRectToScroll(source, reveal_extent);
        if rect == foundation::gfx::Rect::default() {
            return;
        }
        source.ScrollRectToVisible(start, rect, alignment);
        self.effects.UpdateAppearance();
    }
    pub fn Contains<C: SelectionContainmentSource<S>>(
        &self,
        source: &C,
        point: &foundation::PhysicalOffset,
    ) -> bool {
        ContainsSelection(source, || self.owner.Current(), point)
    }
}
/// Read-only Contains body over the existing SelectionEditor value. This lets
/// a native child Frame borrow its canonical selection without constructing a
/// second editing controller or borrowing mutation-only Editor services.
pub fn ContainsSelection<S: SelectionValue, C: SelectionContainmentSource<S>>(
    source: &C,
    current: impl FnOnce() -> S,
    point: &foundation::PhysicalOffset,
) -> bool {
    if !source.HasLayoutView() {
        return false;
    }
    let current = current();
    // Source null-DOM cache workaround also invalidates the flat result.
    // No second cache is introduced at the independent package boundary.
    if !source.ComputeVisibleSelectionInDOMTreeIsNonNull(&current) {
        return false;
    }
    let Some((start, end)) = source.VisibleFlatSelectionEndpoints(&current) else {
        return false;
    };
    if start.partial_cmp(&end) != Some(std::cmp::Ordering::Less) {
        return false;
    }
    let Some(position) = source.HitTestPositionReadOnlyActive(point) else {
        return false;
    };
    SelectionRangeContainsPosition(&start, &end, &position)
}
impl<S: SelectionValue> FrameSelection<S> {
    /// frame_selection.cc1380-1389. This is the committed range union, not a
    /// bounding rectangle of endpoint carets.
    pub fn AbsoluteUnclippedBounds(
        &self,
        source: &dyn SelectionBoundsSource,
    ) -> foundation::PhysicalRect {
        if !source.HasView() || !source.HasLayoutView() {
            return foundation::PhysicalRect::default();
        }
        let rect = source.AbsoluteSelectionBounds();
        foundation::PhysicalRect::from_units(
            foundation::LayoutUnit::from_signed(rect.x()),
            foundation::LayoutUnit::from_signed(rect.y()),
            foundation::LayoutUnit::from_signed(rect.width()),
            foundation::LayoutUnit::from_signed(rect.height()),
        )
    }
    // Source frame_selection.cc:323-324. Retain the strategy identity and clear
    // its source state; clearing selection does not replace that owner.
    pub fn ClearGranularityStrategy(&self) {
        let strategy = self.granularity_strategy.borrow().clone();
        if let Some(strategy) = strategy {
            strategy.Clear();
        }
    }
    pub fn SetGranularityStrategy(&self, strategy: Option<Rc<dyn GranularityStrategy>>) {
        *self.granularity_strategy.borrow_mut() = strategy;
    }

    /// Source frame_selection.cc:1717-1719, same SelectionEditor cache owner.
    pub fn MarkCacheDirty(&self) {
        self.absolute_bounds.MarkCacheDirty();
    }
    /// Source frame_selection.cc:905-919.
    pub fn ComputeAbsoluteBounds(
        &self,
        source: &dyn super::selection_editor::SelectionAbsoluteBoundsSource<S>,
    ) -> Option<(foundation::gfx::Rect, foundation::gfx::Rect)> {
        if !self.owner.IsAvailable() || self.owner.Current().IsNone() {
            return None;
        }
        source.UpdateStyleAndLayoutForSelection();
        let visible = source.ComputeVisibleSelectionInDOMTree(&self.owner.Current());
        if visible.IsNone() {
            return None;
        }
        self.absolute_bounds.ComputeAbsoluteBounds(&visible, source)
    }
    pub fn BindFocusEffects(&self, effects: Rc<dyn SelectionFocusEffects<S>>) {
        assert!(
            self.focus_effects_.set(effects).is_ok(),
            "one FrameSelection focus owner"
        );
    }
    fn FocusEffects(&self) -> &dyn SelectionFocusEffects<S> {
        self.focus_effects_
            .get()
            .expect("FrameSelection source focus owners are bound before input")
            .as_ref()
    }
    // Source: frame_selection.cc:727-815. The adapter supplies the same
    // resident visible selection and canonical flat-tree/focus owners.
    pub fn SelectionHasFocus<D: super::selection_focus::SelectionFocusDOM>(&self, dom: &D) -> bool {
        super::selection_focus::SelectionHasFocus(dom)
    }
    pub fn FrameIsFocusedAndActive(&self) -> bool {
        self.focused_.get() && self.FocusEffects().PageIsActive()
    }
    pub fn FrameIsFocused(&self) -> bool {
        self.focused_.get()
    }
    pub fn FocusedOrActiveStateChanged(&self) {
        let effects = self.FocusEffects();
        let page_active = effects.PageIsActive();
        let active_and_focused = self.focused_.get() && page_active;
        effects.PublishFocusState(active_and_focused, page_active);
        effects.FocusedElementFocusStateChanged();
        if effects.HasLayoutView() {
            effects.InvalidateStyleAndPaintForSelection();
        }
        effects.UpdateStyleAndLayoutTree();
        if active_and_focused {
            self.SetSelectionFromNone();
        }
        effects.SetCaretEnabled(active_and_focused);
        effects.CapsLockStateMayHaveChanged(active_and_focused);
    }
    pub fn PageActivationChanged(&self) {
        self.FocusedOrActiveStateChanged();
    }
    pub fn SetFrameIsFocused(&self, focused: bool) {
        if self.focused_.replace(focused) == focused {
            return;
        }
        self.FocusedOrActiveStateChanged();
    }
    pub fn SetSelectionFromNone(&self) {
        let effects = self.FocusEffects();
        effects.UpdateStyleAndLayoutForSelection();
        if !self.owner.Current().IsNone() || !effects.DocumentIsEditable() {
            return;
        }
        if let Some(selection) = effects.SelectionAtBodyStart() {
            self.SetSelection(&selection, SetSelectionOptions::default());
        }
    }
    // Source: frame_selection.cc:231-251.
    pub fn MoveCaretSelection(&self, point: (i32, i32)) {
        if !self.owner.HasEditableRoot() {
            return;
        }
        let selection = self.owner.CaretSelectionForContentsPoint(point);
        self.SetSelection(
            &selection,
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                set_selection_by: SetSelectionBy::kUser,
                should_show_handle: true,
                is_directional: self.is_directional.get(),
                ..Default::default()
            },
        );
    }
    // Source: frame_selection.cc:253-261. Trace instrumentation is represented
    // by the caller's dispatch span rather than another state transition.
    pub fn SetSelection(&self, selection: &S, options: SetSelectionOptions) {
        if self.SetSelectionDeprecated(selection, options) {
            self.DidSetSelectionDeprecated(selection, options);
        } else {
            self.MaybeNotifyEventHandlerForSelectionChange(options);
        }
    }
    // Source: frame_selection.cc:269-290.
    pub fn MaybeNotifyEventHandlerForSelectionChange(&self, options: SetSelectionOptions) {
        if !self.features.notify_unchanged
            || !options.should_notify_selection_controller_of_unchanged_selection
        {
            return;
        }
        if self.effects.CurrentEventType().is_some_and(|kind| {
            matches!(
                kind.as_str(),
                "mouseup" | "click" | "pointerup" | "touchend"
            )
        }) {
            return;
        }
        self.effects.NotifyEventHandlerForSelectionChange();
    }
    // Source: frame_selection.cc:292-298.
    pub fn SetSelectionAndEndTyping(&self, selection: &S) {
        self.SetSelection(
            selection,
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                ..Default::default()
            },
        );
    }
    // Source: frame_selection.cc:313-364.
    pub fn SetSelectionDeprecated(&self, selection: &S, mut options: SetSelectionOptions) -> bool {
        if self.effects.AlwaysUseDirectionalSelection() {
            options.is_directional = true;
        }
        if !options.do_not_clear_strategy {
            self.effects.ClearGranularityStrategy();
        }
        self.granularity.set(options.granularity);
        if self.features.bidi_visual_order_caret_movement && !self.is_being_modified.get() {
            self.caret_bidi_level.set(None);
            self.entered_bidi_run.set(false);
        }
        if options.should_close_typing {
            self.effects.CloseTyping();
        }
        if options.should_clear_typing_style {
            self.effects.ClearTypingStyle();
        }
        let old = self.owner.Current();
        let changed = old != *selection;
        if !changed
            && self.is_handle_visible.get() == options.should_show_handle
            && self.is_directional.get() == options.is_directional
        {
            return false;
        }
        if changed {
            self.MarkCacheDirty();
            self.owner.SetSelectionAndEndTyping(selection);
            self.effects.NotifyDisplayLock(&old, selection);
        }
        self.is_directional.set(options.is_directional);
        self.should_shrink_next_tap
            .set(options.should_shrink_next_tap);
        self.is_handle_visible.set(options.should_show_handle);
        self.effects.ScheduleVisualUpdateForVisualOverflow();
        true
    }
    // Source: frame_selection.cc:366-465. Every effect executes synchronously;
    // retained document identity is checked after focus listener reentry.
    pub fn DidSetSelectionDeprecated(&self, selection: &S, options: SetSelectionOptions) {
        let document = self.owner.DocumentIdentity();
        // TODO: frame_selection.cc:372-388 ScopedBlinkAXEventIntent.
        if !selection.IsNone() && !options.do_not_set_focus {
            self.effects.SetFocusedNodeIfNeeded();
            if !self.owner.IsAvailable() || document != self.owner.DocumentIdentity() {
                return;
            }
        }
        self.effects.StopCaretBlinkTimer();
        self.effects.UpdateAppearance();
        self.x_pos_for_vertical_arrow_navigation
            .set(super::selection_modifier::NoXPosForVerticalArrowNavigation());
        if !options.do_not_set_focus {
            self.effects.SelectFrameElementInParentIfFullySelected();
            if !self.owner.IsAvailable() || document != self.owner.DocumentIdentity() {
                return;
            }
        }
        self.effects
            .NotifyTextControlOfSelectionChange(options.set_selection_by);
        if options.set_selection_by == SetSelectionBy::kUser {
            let always = options.cursor_align_on_scroll == CursorAlignOnScroll::kAlways;
            let alignment = if self.effects.CenterAlignRevealedSelection() {
                if always {
                    SelectionAlignment::CenterAlways
                } else {
                    SelectionAlignment::CenterIfNeeded
                }
            } else if always {
                SelectionAlignment::TopAlways
            } else {
                SelectionAlignment::EdgeIfNeeded
            };
            self.effects
                .RevealSelection(alignment, RevealExtentOption::kRevealExtent);
        }
        self.effects.NotifyAccessibilityForSelectionChange();
        self.effects.NotifyCompositorForSelectionChange();
        self.effects.NotifyEventHandlerForSelectionChange();
        if self.features.per_element_selectionchange {
            if self.effects.EnclosingTextControlIsOutsideShadowTree() {
                self.effects.ScheduleTextControlSelectionchange();
            } else {
                self.effects.ScheduleDocumentSelectionchange();
            }
        } else {
            self.effects.EnqueueLegacyDocumentSelectionchange();
        }
        self.effects.RespondToChangedSelection();
    }
    // Source: frame_selection.cc:554-556,718-725.
    pub fn DidChangeFocus(&self) {
        self.effects.UpdateAppearance();
    }
    pub fn Clear(&self) {
        self.granularity.set(TextGranularity::kCharacter);
        self.effects.ClearGranularityStrategy();
        self.SetSelectionAndEndTyping(&self.owner.EmptySelection());
        self.is_handle_visible.set(false);
        self.is_directional
            .set(self.effects.AlwaysUseDirectionalSelection());
    }

    /// editing_commands_utilities.cc:796-829. Preserve source notification
    /// behavior when editing changes the visual caret without moving its DOM
    /// endpoints (the selection setter itself then has no change to report).
    pub fn ChangeSelectionAfterCommand(
        &self,
        selection: &S,
        is_caret: impl Fn(&S) -> bool,
        mut options: SetSelectionOptions,
    ) {
        if selection.IsNone() {
            return;
        }
        let unchanged = self.owner.Current() == *selection
            && options.is_directional == self.is_directional.get();
        options.should_show_handle = self.is_handle_visible.get() && !is_caret(selection);
        self.SetSelection(selection, options);
        if unchanged {
            if let Some(client) = self.effects.GetEditorFrameClient() {
                client.DidChangeSelection(
                    is_caret(&self.owner.Current()),
                    super::editor::SyncCondition::kNotForced,
                );
            }
        }
    }
}

/// Source FrameSelection::IsHidden, cc:818-836. The caller borrows the same
/// visible DOM selection and focus helper; no hidden-selection state is stored.
pub fn SelectionIsHidden<N>(
    has_focus: bool,
    start: Option<N>,
    is_range: bool,
    enclosing_text_control: impl FnOnce(&N) -> bool,
) -> bool {
    if has_focus {
        return false;
    }
    let Some(start) = start else {
        return true;
    };
    if !is_range {
        return true;
    }
    enclosing_text_control(&start)
}

// Source: frame_selection.cc:943-967. Both the retained FrameSelection and
// compatibility document selection project their original live positions here.
pub fn SelectionRangeContainsPosition<P: PartialOrd>(start: &P, end: &P, position: &P) -> bool {
    start.partial_cmp(end) == Some(std::cmp::Ordering::Less)
        && matches!(
            start.partial_cmp(position),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
        && matches!(
            position.partial_cmp(end),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
}
