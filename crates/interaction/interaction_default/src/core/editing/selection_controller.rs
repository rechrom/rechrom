// Copyright (C) 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
// This library is free software under the GNU Library General Public License,
// version 2 or (at your option) any later version; see Chromium COPYING.LIB.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/selection_controller.h:55-62,116-123,132-153
// Source: third_party/blink/renderer/core/editing/selection_controller.cc:72-136,162-262,271-413,417-826,881-900,951-1356,1430-1489,1495-1499,1517-1524
// Positions and selections below are the owning DOM engine's native values;
// only source dispatch flags and the original bidi anchor are retained here.

use super::{
    set_selection_options::{CursorAlignOnScroll, SetSelectionOptions},
    text_granularity::TextGranularity,
};
use interaction_dispatch::{
    core::dom::events::{event::EventTargetRef, event_dispatch_result::DispatchEventResult},
    public::platform::web_input_event_result::WebInputEventResult,
};
use interaction_input::public::common::input::{
    web_input_event::{Type, WebInputEvent},
    web_mouse_event::WebMouseEvent,
    web_pointer_properties::Button,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub trait SelectionPosition: Clone + PartialEq + PartialOrd {}
pub trait FlatPositionIterator<P: SelectionPosition> {
    fn AtStart(&self) -> bool;
    fn AtEnd(&self) -> bool;
    fn Position(&self) -> P;
    fn Node(&self) -> Option<EventTargetRef>;
    fn Decrement(&mut self);
    fn Increment(&mut self);
}
pub trait FlatSelection: Clone + PartialEq {
    type Position: SelectionPosition;
    fn IsNone(&self) -> bool;
    fn IsRange(&self) -> bool;
    fn IsCaret(&self) -> bool;
    fn IsAnchorFirst(&self) -> bool;
    fn Anchor(&self) -> Option<Self::Position>;
    fn Focus(&self) -> Option<Self::Position>;
    fn Start(&self) -> Option<Self::Position>;
    fn End(&self) -> Option<Self::Position>;
}

pub struct MouseEventWithSelectionHit<'a, H> {
    pub event: &'a WebMouseEvent,
    pub hit: &'a H,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionState {
    kHaveNotStartedSelection,
    kPlacedCaret,
    kExtendedSelection,
}
#[derive(Clone, Copy)]
pub enum SelectionContextMenu {
    Touch,
    AdjustSelectionReset,
}

/// Call-scoped projection of the original MouseEventObject's immutable fields.
#[derive(Clone, Copy)]
pub struct SelectionContextMenuEvent {
    pub click_count: i32,
    pub from_touch: bool,
    pub menu_source: interaction_input::ui::base::mojom::menu_source_type::MenuSourceType,
}
pub trait SelectionControllerOwner<S: FlatSelection> {
    fn IsAvailable(&self) -> bool;
    fn Current(&self) -> S;
    fn CurrentStartIsEditable(&self) -> bool;
    fn IsDirectional(&self) -> bool;
    fn Granularity(&self) -> TextGranularity;
    fn IsHandleVisible(&self) -> bool;
    fn ShouldShrinkNextTap(&self) -> bool;
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool;
    fn ShouldToggleMenuWhenCaretTapped(&self) -> bool;
    fn IsSelectTrailingWhitespaceEnabled(&self) -> bool;
    fn ShouldSelectOnContextualMenuClick(&self) -> bool;
    fn SpellCheckEnabledByDefault(&self) -> bool;

    fn Contains(&self, contents_point: (f64, f64)) -> bool;
    fn SetSelection(&self, selection: &S, options: SetSelectionOptions);
    fn SetSelectionAndEndTyping(&self, selection: &S);
    fn NotifyTextControlOfSelectionChange(&self);
    fn SelectFrameElementInParentIfFullySelected(&self);
    fn ShowContextMenu(&self, kind: SelectionContextMenu);
    fn MarkSelectionEndpointsForRepaint(&self, selection: &S);
    fn BindOriginalAnchorToCurrentContext(&self);
    fn HasPage(&self) -> bool;
    fn MiddleClickPasteAllowed(&self) -> bool;
    fn IsFocusedOrMainFrame(&self) -> bool;
    fn ExecutePasteGlobalSelection(&self) -> bool;
}

/// Read-only facts and native position operations from the resident hit/DOM
/// owner. No live tree or LayoutObject is copied across this boundary.
pub trait SelectionGeometry<S: FlatSelection, H> {
    fn InnerNode(&self, hit: &H) -> Option<EventTargetRef>;
    fn InnerPossiblyPseudoNode(&self, hit: &H) -> Option<EventTargetRef>;
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool;
    fn CanStartSelection(&self, node: &EventTargetRef) -> bool;
    fn IsSelectable(&self, node: &EventTargetRef) -> bool;
    fn HasActivationBehavior(&self, node: &EventTargetRef) -> bool;
    fn IsPseudoElement(&self, node: &EventTargetRef) -> bool;
    fn IsEditable(&self, node: &EventTargetRef) -> bool;
    fn IsConnected(&self, node: &EventTargetRef) -> bool;
    fn TextAreaEmptyPlaceholderBreakEnabled(&self) -> bool;
    fn EnclosingTextControlValueIsEmpty(&self, node: &EventTargetRef) -> Option<bool>;
    fn RootEditableHasChildren(&self, node: &EventTargetRef) -> Option<bool>;
    fn IsOverScrollbar(&self, hit: &H) -> bool;
    fn IsOverLink(&self, hit: &H) -> bool;
    fn IsUserNodeDraggable(&self, hit: &H) -> bool;
    fn IsLiveLink(&self, hit: &H) -> bool;
    fn HasImage(&self, hit: &H) -> bool;
    fn HitTestResultIsMisspelled(&self, hit: &H, page_default: bool) -> bool;
    fn SpellCheckMarkerGroupAtPosition(
        &self,
        position: &S::Position,
    ) -> Option<(S::Position, S::Position)>;
    fn AnnotationAtHit(&self, hit: &H) -> Option<super::selection_hit_test::MarkerAnnotationType>;
    fn IsOverAnnotation(&self, hit: &H) -> bool {
        self.AnnotationAtHit(hit).is_some()
    }

    fn URLElement(&self, hit: &H) -> Option<EventTargetRef>;
    fn PositionIsDescendantOf(&self, position: &S::Position, node: &EventTargetRef) -> bool;
    fn ParentAnchoredPosition(&self, position: &S::Position) -> Option<S::Position>;

    fn HasView(&self) -> bool;
    fn ConvertRootPointToContents(&self, point: (f64, f64)) -> (f64, f64);
    fn PositionOfHit(&self, hit: &H, zero_image_offset: bool) -> Option<S::Position>;
    fn PositionRespectingEditingBoundary(&self, hit: &H) -> Option<S::Position>;
    fn SelectionHasFocus(&self) -> bool;
    fn FirstPositionInOrBefore(&self, node: &EventTargetRef) -> Option<S::Position>;
    fn PositionIsValid(&self, position: &S::Position) -> bool;
    fn PositionIsConnected(&self, position: &S::Position) -> bool;
    fn PositionIsEquivalent(&self, a: &S::Position, b: &S::Position) -> bool;
    fn VisiblePosition(&self, position: &S::Position) -> Option<S::Position>;
    fn VisiblePositionOfPlain(&self, position: &S::Position) -> Option<S::Position>;
    fn Empty(&self) -> S;
    fn Collapse(&self, position: &S::Position) -> S;
    fn Build(&self, anchor: &S::Position, focus: &S::Position) -> S;
    fn WithBaseAndExtent(&self, selection: &S, anchor: &S::Position, focus: &S::Position) -> S;
    fn WithAffinityOf(&self, selection: &S, position: &S::Position) -> S;
    fn VisibleSelection(&self, selection: &S) -> S;
    fn ResetInvalidSelectionAfterEvent(&self, selection: &S) -> S;
    fn ExpandWithGranularity(&self, selection: &S, granularity: TextGranularity) -> S;
    fn RootUserSelectAllBounds(&self, node: &EventTargetRef) -> Option<(S::Position, S::Position)>;
    fn EnclosingBlock(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn NodeIsText(&self, node: &EventTargetRef) -> bool;
    fn FirstPositionInNode(&self, node: &EventTargetRef) -> S::Position;
    fn LastPositionInNode(&self, node: &EventTargetRef) -> S::Position;
    fn PositionIterator(
        &self,
        position: &S::Position,
    ) -> Box<dyn FlatPositionIterator<S::Position> + '_>;
    fn StartRespectingGranularity(
        &self,
        position: &S::Position,
        granularity: TextGranularity,
    ) -> Option<S::Position>;
    fn EndRespectingGranularity(
        &self,
        start: &S::Position,
        position: &S::Position,
        granularity: TextGranularity,
    ) -> Option<S::Position>;
    fn PreviousPosition(&self, position: &S::Position) -> Option<S::Position>;
    fn TextDistance(&self, start: &S::Position, end: &S::Position) -> usize;
    fn AdjustAtBidiBoundary(&self, anchor: &S::Position, focus: &S::Position) -> S;
    fn PlainText(&self, selection: &S) -> Vec<u16>;
    fn ContainsOnlyWhitespaceAfterSimplifying(&self, text: &[u16]) -> bool;
    fn IsLastPositionInEditableRoot(&self, position: &S::Position, selection: &S) -> bool;
    fn SkipWhitespace(&self, position: &S::Position) -> S::Position;
    fn DispatchSelectStart(&self, node: &EventTargetRef) -> DispatchEventResult;
    fn HandlePotentialSuggestionTap(&self, position: &S::Position);
    fn HitTestAtContentsPoint(&self, point: (f64, f64), move_request: bool) -> H;
}
pub trait PreparedSelectionGeometry {
    fn Prepare(&self, preparation: SelectionGeometryPreparation);
}
#[derive(Clone, Copy)]
pub enum SelectionGeometryPreparation {
    StyleAndLayoutForSelection,
    StyleAndLayoutForInput,
    StyleAndLayoutTree,
}

// Source: selection_controller.h:132-138; selection_controller.cc:72-84,1430-1436.
// Shared native input state, retained by both the full controller and facade.
#[derive(Default)]
pub struct SelectionControllerState {
    pub mouse_down_may_start_select: Cell<bool>,
    pub mouse_down_was_single_click_in_selection: Cell<bool>,
    /// The native adapter specializes source Position without exposing its
    /// document strategy to this shared input owner. This is one controller's
    /// source gesture/bidi state, never a second selection endpoint store.
    pub gesture_owner: RefCell<Option<Rc<dyn std::any::Any>>>,
}
impl SelectionControllerState {
    pub fn SetMouseDownMayStartSelect(&self, may_start: bool) {
        self.mouse_down_may_start_select.set(may_start);
    }
    pub fn MouseDownMayStartSelect(&self) -> bool {
        self.mouse_down_may_start_select.get()
    }
    // Source: selection_controller.cc:1162-1163,1467-1469.
    pub fn ResetSingleClickInSelection(&self) {
        self.mouse_down_was_single_click_in_selection.set(false);
    }
    pub fn MouseDownWasSingleClickInSelection(&self) -> bool {
        self.mouse_down_was_single_click_in_selection.get()
    }
    // Source: selection_controller.cc:468-479. The original FrameSelection
    // owner supplies Contains; the input owner retains only the source flag.
    pub fn RecordSingleClickInSelection(&self, extend_selection: bool, contains: bool) -> bool {
        if extend_selection || !contains {
            return false;
        }
        self.mouse_down_was_single_click_in_selection.set(true);
        true
    }
}

pub struct SelectionController<'a, S: FlatSelection, H> {
    pub owner: Rc<dyn SelectionControllerOwner<S> + 'a>,
    pub geometry: Rc<dyn SelectionGeometry<S, H> + 'a>,
    input_state: Rc<SelectionControllerState>,
    pub mouse_down_was_single_click_on_caret: Rc<Cell<bool>>,
    pub mouse_down_allows_multi_click: Rc<Cell<bool>>,
    pub selection_state: Rc<Cell<SelectionState>>,
    original_anchor: Rc<RefCell<Option<S::Position>>>,
    pub left_click_to_handle_suggestion: bool,
}

impl<S: FlatSelection, H> std::ops::Deref for SelectionController<'_, S, H> {
    type Target = SelectionControllerState;
    fn deref(&self) -> &Self::Target {
        &self.input_state
    }
}

impl<'a, S: FlatSelection, H> SelectionController<'a, S, H> {
    // Source: selection_controller.cc:72-84,271-273.
    pub fn new(
        owner: Rc<dyn SelectionControllerOwner<S> + 'a>,
        geometry: Rc<dyn SelectionGeometry<S, H> + 'a>,
        left_click_to_handle_suggestion: bool,
    ) -> Self {
        Self::WithState(
            owner,
            geometry,
            left_click_to_handle_suggestion,
            Rc::new(SelectionControllerState::default()),
        )
    }
    pub fn WithState(
        owner: Rc<dyn SelectionControllerOwner<S> + 'a>,
        geometry: Rc<dyn SelectionGeometry<S, H> + 'a>,
        left_click_to_handle_suggestion: bool,
        input_state: Rc<SelectionControllerState>,
    ) -> Self {
        Self {
            owner,
            geometry,
            input_state,
            mouse_down_was_single_click_on_caret: Rc::new(Cell::new(false)),
            mouse_down_allows_multi_click: Rc::new(Cell::new(false)),
            selection_state: Rc::new(Cell::new(SelectionState::kHaveNotStartedSelection)),
            original_anchor: Rc::new(RefCell::new(None)),
            left_click_to_handle_suggestion,
        }
    }
    /// Borrow native ports for this synchronous transaction while preserving
    /// the original controller's flags and bidi anchor across listener reentry.
    pub fn WithSharedState(
        owner: Rc<dyn SelectionControllerOwner<S> + 'a>,
        geometry: Rc<dyn SelectionGeometry<S, H> + 'a>,
        left_click_to_handle_suggestion: bool,
        input_state: Rc<SelectionControllerState>,
        state: &super::gesture_selection::GestureSelectionState<S::Position>,
    ) -> Self {
        Self {
            owner,
            geometry,
            input_state,
            left_click_to_handle_suggestion,
            mouse_down_was_single_click_on_caret: state
                .mouse_down_was_single_click_on_caret
                .clone(),
            mouse_down_allows_multi_click: state.mouse_down_allows_multi_click.clone(),
            selection_state: state.selection_state.clone(),
            original_anchor: state.original_anchor.clone(),
        }
    }
    pub fn ContextDestroyed(&self) {
        *self.original_anchor.borrow_mut() = None;
    }
    // Source: selection_controller.cc:88-136.
    fn DispatchSelectStart(&self, node: Option<&EventTargetRef>) -> DispatchEventResult {
        let Some(node) = node else {
            return DispatchEventResult::kNotCanceled;
        };
        if !self.geometry.HasLayoutObject(node) {
            return DispatchEventResult::kNotCanceled;
        }
        self.geometry.DispatchSelectStart(node)
    }
    fn ExpandSelectionToRespectUserSelectAll(&self, node: &EventTargetRef, selection: &S) -> S {
        if selection.IsNone() {
            return self.geometry.Empty();
        }
        if let Some((start, end)) = self.geometry.RootUserSelectAllBounds(node) {
            return self.geometry.WithBaseAndExtent(selection, &start, &end);
        }
        selection.clone()
    }
    fn CanMouseDownStartSelect(&self, node: Option<&EventTargetRef>) -> bool {
        node.is_none_or(|node| {
            !self.geometry.HasLayoutObject(node) || self.geometry.CanStartSelection(node)
        })
    }
    // Source: selection_controller.cc:162-185.
    fn IsNonSelectable(&self, node: Option<&EventTargetRef>) -> bool {
        node.is_some_and(|node| {
            self.geometry.HasLayoutObject(node) && !self.geometry.IsSelectable(node)
        })
    }
    fn ShouldIgnoreNodeForCheckSelectable(
        &self,
        block: Option<&EventTargetRef>,
        node: Option<&EventTargetRef>,
    ) -> bool {
        match (block, node) {
            (Some(block), Some(node)) if Rc::ptr_eq(block, node) => true,
            (None, None) => true,
            (_, Some(node)) => self.geometry.NodeIsText(node),
            _ => false,
        }
    }
    fn IsEditableBoxEmpty(&self, node: &EventTargetRef) -> bool {
        if self.geometry.TextAreaEmptyPlaceholderBreakEnabled() {
            if let Some(empty) = self.geometry.EnclosingTextControlValueIsEmpty(node) {
                return empty;
            }
        }
        self.geometry
            .RootEditableHasChildren(node)
            .is_none_or(|children| !children)
    }
    // Source: selection_controller.cc:189-207.
    fn AdjustSelectionWithTrailingWhitespace(&self, selection: &S) -> S {
        if selection.IsNone() || !selection.IsRange() {
            return selection.clone();
        }
        let Some(end) = selection.End() else {
            return selection.clone();
        };
        let new_end = self.geometry.SkipWhitespace(&end);
        if end == new_end {
            return selection.clone();
        }
        if selection.IsAnchorFirst() {
            selection.Anchor().map_or_else(
                || selection.clone(),
                |anchor| {
                    self.geometry
                        .WithBaseAndExtent(selection, &anchor, &new_end)
                },
            )
        } else {
            selection.Focus().map_or_else(
                || selection.clone(),
                |focus| self.geometry.WithBaseAndExtent(selection, &new_end, &focus),
            )
        }
    }
    // Source: selection_controller.cc:209-262. Iterators borrow the native
    // flat-tree owner; no descendant list is copied.
    fn AdjustSelectionByUserSelect(&self, node: &EventTargetRef, selection: &S) -> S {
        if selection.IsNone() {
            return self.geometry.Empty();
        }
        let expanded = self.ExpandSelectionToRespectUserSelectAll(node, selection);
        let block = self.geometry.EnclosingBlock(node);
        let (Some(anchor), Some(focus)) = (expanded.Anchor(), expanded.Focus()) else {
            return self.geometry.Empty();
        };
        let mut start = self.geometry.FirstPositionInNode(node);
        {
            let mut iter = self.geometry.PositionIterator(&start);
            while !iter.AtStart() {
                let current = iter.Position();
                if current <= anchor {
                    start = anchor.clone();
                    break;
                }
                let current_node = iter.Node();
                if !self.ShouldIgnoreNodeForCheckSelectable(block.as_ref(), current_node.as_ref())
                    && self.IsNonSelectable(current_node.as_ref())
                {
                    start = current;
                    break;
                }
                iter.Decrement();
            }
        }
        let mut end = self.geometry.LastPositionInNode(node);
        {
            let mut iter = self.geometry.PositionIterator(&end);
            while !iter.AtEnd() {
                let current = iter.Position();
                if current >= focus {
                    end = focus.clone();
                    break;
                }
                let current_node = iter.Node();
                if !self.ShouldIgnoreNodeForCheckSelectable(block.as_ref(), current_node.as_ref())
                    && self.IsNonSelectable(current_node.as_ref())
                {
                    end = current;
                    break;
                }
                iter.Increment();
            }
        }
        self.geometry.Build(&start, &end)
    }
    // Source: selection_controller.cc:275-302.
    fn AdjustPositionRespectUserSelectAll(
        &self,
        node: &EventTargetRef,
        selection: &S,
        position: Option<S::Position>,
    ) -> Option<S::Position> {
        let collapsed = position.as_ref().map_or_else(
            || self.geometry.Empty(),
            |position| self.geometry.Collapse(position),
        );
        let all = self
            .geometry
            .VisibleSelection(&self.ExpandSelectionToRespectUserSelectAll(node, &collapsed));
        if !all.IsRange() {
            return position;
        }
        if all.Start()? < selection.Start()? {
            return all.Start();
        }
        if selection.End()? < all.End()? {
            return all.End();
        }
        position
    }
    // Source: selection_controller.cc:304-369.
    fn ExtendSelectionAsDirectional(
        &self,
        position: &S::Position,
        selection: &S,
        granularity: TextGranularity,
    ) -> S {
        let Some(anchor) = selection.Anchor() else {
            return selection.clone();
        };
        let (start, end, backward) = if *position < anchor {
            let Some(start) = self
                .geometry
                .StartRespectingGranularity(position, granularity)
            else {
                return selection.clone();
            };
            let end = if selection.IsAnchorFirst() {
                self.geometry
                    .EndRespectingGranularity(&start, &anchor, granularity)
            } else {
                Some(anchor)
            };
            (Some(start), end, true)
        } else {
            let start = if selection.IsAnchorFirst() || granularity == TextGranularity::kCharacter {
                Some(anchor.clone())
            } else {
                self.geometry
                    .PreviousPosition(&anchor)
                    .and_then(|previous| {
                        self.geometry
                            .StartRespectingGranularity(&previous, granularity)
                    })
            };
            let end = start.as_ref().and_then(|start| {
                self.geometry
                    .EndRespectingGranularity(start, position, granularity)
            });
            (start, end, false)
        };
        let (Some(start), Some(end)) = (start, end) else {
            return selection.clone();
        };
        let built = if backward {
            self.geometry.Build(&end, &start)
        } else {
            self.geometry.Build(&start, &end)
        };
        if start == end {
            self.geometry.WithAffinityOf(&built, position)
        } else {
            built
        }
    }
    // Source: selection_controller.cc:372-413.
    fn ExtendSelectionAsNonDirectional(
        &self,
        position: &S::Position,
        selection: &S,
        granularity: TextGranularity,
    ) -> S {
        let (Some(start), Some(end)) = (selection.Start(), selection.End()) else {
            return selection.clone();
        };
        if start == end && *position == start {
            return selection.clone();
        }
        if *position < start {
            return self
                .geometry
                .StartRespectingGranularity(position, granularity)
                .map_or_else(
                    || self.geometry.Empty(),
                    |new_start| self.geometry.Build(&end, &new_start),
                );
        }
        if end < *position {
            return self
                .geometry
                .EndRespectingGranularity(&start, position, granularity)
                .map_or_else(
                    || self.geometry.Empty(),
                    |new_end| self.geometry.Build(&start, &new_end),
                );
        }
        if self.geometry.TextDistance(&start, position)
            <= self.geometry.TextDistance(position, &end)
        {
            self.geometry
                .StartRespectingGranularity(position, granularity)
                .map_or_else(
                    || self.geometry.Empty(),
                    |new_start| self.geometry.Build(&end, &new_start),
                )
        } else {
            self.geometry
                .EndRespectingGranularity(&start, position, granularity)
                .map_or_else(
                    || self.geometry.Empty(),
                    |new_end| self.geometry.Build(&start, &new_end),
                )
        }
    }
    // Source: selection_controller.cc:417-559.
    pub fn HandleSingleClick(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        let (Some(node), Some(pseudo)) = (
            self.geometry.InnerNode(&event.hit),
            self.geometry.InnerPossiblyPseudoNode(&event.hit),
        ) else {
            return false;
        };
        if !self.geometry.HasLayoutObject(&node)
            || !self.geometry.HasLayoutObject(&pseudo)
            || !self.mouse_down_may_start_select.get()
        {
            return false;
        }
        let extend = event.event.GetModifiers() & WebInputEvent::kShiftKey != 0
            && !self.geometry.IsOverLink(&event.hit)
            && !self.geometry.HasImage(&event.hit)
            && !self.geometry.IsUserNodeDraggable(&event.hit);
        let position = self
            .geometry
            .PositionOfHit(&event.hit, false)
            .and_then(|position| self.geometry.VisiblePosition(&position))
            .or_else(|| {
                self.geometry
                    .FirstPositionInOrBefore(&node)
                    .and_then(|position| self.geometry.VisiblePosition(&position))
            });
        let selection = self.owner.Current();
        let editable = self.geometry.IsEditable(&node);
        if editable
            && selection.IsCaret()
            && selection
                .Anchor()
                .zip(position.as_ref())
                .is_some_and(|(anchor, pos)| self.geometry.PositionIsEquivalent(&anchor, pos))
        {
            self.mouse_down_was_single_click_on_caret.set(true);
            if event.event.FromTouch() && self.owner.ShouldToggleMenuWhenCaretTapped() {
                self.HandleTapOnCaret(&node, &selection, prepared);
                return false;
            }
        }
        if self.geometry.HasView() {
            let point = event.event.PositionInRootFrame();
            let point = self
                .geometry
                .ConvertRootPointToContents((point.x().floor() as f64, point.y().floor() as f64));
            if self
                .input_state
                .RecordSingleClickInSelection(extend, self.owner.Contains(point))
            {
                if !event.event.FromTouch() {
                    return false;
                }
                if self.HandleTapInsideSelection(event, &selection, prepared) {
                    return false;
                }
            }
        }
        if extend && !selection.IsNone() {
            let position = self.AdjustPositionRespectUserSelectAll(&node, &selection, position);
            let granularity = self.owner.Granularity();
            let new_selection = if let Some(position) = position {
                if self.owner.ShouldConsiderSelectionAsDirectional() {
                    self.ExtendSelectionAsDirectional(&position, &selection, granularity)
                } else {
                    self.ExtendSelectionAsNonDirectional(&position, &selection, granularity)
                }
            } else {
                selection
            };
            self.UpdateSelectionForMouseDownDispatchingSelectStart(
                Some(&node),
                &new_selection,
                SetSelectionOptions {
                    granularity,
                    ..Default::default()
                },
                prepared,
            );
            return false;
        }
        if self.selection_state.get() == SelectionState::kExtendedSelection {
            self.UpdateSelectionForMouseDownDispatchingSelectStart(
                Some(&node),
                &selection,
                Default::default(),
                prepared,
            );
            return false;
        }
        let Some(position) = position else {
            self.UpdateSelectionForMouseDownDispatchingSelectStart(
                Some(&node),
                &self.geometry.Empty(),
                Default::default(),
                prepared,
            );
            return false;
        };
        let show_handle = editable
            && (!self.IsEditableBoxEmpty(&node)
                || event.event.pointer_properties.button != Button::kLeft)
            && event.event.FromTouch();
        let selection =
            self.ExpandSelectionToRespectUserSelectAll(&node, &self.geometry.Collapse(&position));
        if !self.UpdateSelectionForMouseDownDispatchingSelectStart(
            Some(&node),
            &selection,
            SetSelectionOptions {
                should_show_handle: show_handle,
                ..Default::default()
            },
            prepared,
        ) {
            return false;
        }
        let suggestion = event.event.FromTouch()
            || (self.left_click_to_handle_suggestion
                && event.event.pointer_properties.button == Button::kLeft);
        if editable && suggestion && self.geometry.PositionIsValid(&position) {
            self.geometry.HandlePotentialSuggestionTap(&position);
        }
        false
    }
    // Source: selection_controller.cc:562-618.
    fn HandleTapOnCaret(
        &self,
        node: &EventTargetRef,
        selection: &S,
        prepared: &dyn PreparedSelectionGeometry,
    ) {
        let show = !self.IsEditableBoxEmpty(node) || !self.owner.IsHandleVisible();
        self.owner.MarkSelectionEndpointsForRepaint(selection);
        if self.UpdateSelectionForMouseDownDispatchingSelectStart(
            Some(node),
            selection,
            SetSelectionOptions {
                should_show_handle: show,
                ..Default::default()
            },
            prepared,
        ) {
            self.owner.ShowContextMenu(SelectionContextMenu::Touch);
        }
    }
    fn HandleTapInsideSelection(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        selection: &S,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        if self.owner.ShouldShrinkNextTap() {
            if self.SelectClosestWordFromHitTestResult(&event.hit, false, true, prepared) {
                self.owner
                    .ShowContextMenu(SelectionContextMenu::AdjustSelectionReset);
            }
            return true;
        }
        if self.owner.IsHandleVisible() {
            return false;
        }
        self.owner.MarkSelectionEndpointsForRepaint(selection);
        let node = self.geometry.InnerNode(&event.hit);
        if self.UpdateSelectionForMouseDownDispatchingSelectStart(
            node.as_ref(),
            selection,
            SetSelectionOptions {
                should_show_handle: true,
                ..Default::default()
            },
            prepared,
        ) {
            self.owner.ShowContextMenu(SelectionContextMenu::Touch);
        }
        true
    }
    // Source: selection_controller.cc:620-710.
    pub fn UpdateSelectionForMouseDrag(
        &self,
        hit: &H,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> WebInputEventResult {
        if !self.mouse_down_may_start_select.get() {
            return WebInputEventResult::kNotHandled;
        }
        let Some(target) = self.geometry.InnerPossiblyPseudoNode(hit) else {
            return WebInputEventResult::kNotHandled;
        };
        prepared.Prepare(SelectionGeometryPreparation::StyleAndLayoutForSelection);
        let position = if self.geometry.SelectionHasFocus() {
            self.geometry.PositionRespectingEditingBoundary(hit)
        } else {
            None
        };
        let Some(position) = position.and_then(|position| self.geometry.VisiblePosition(&position))
        else {
            return WebInputEventResult::kNotHandled;
        };
        if self.selection_state.get() == SelectionState::kHaveNotStartedSelection
            && self.DispatchSelectStart(Some(&target)) != DispatchEventResult::kNotCanceled
        {
            return WebInputEventResult::kHandledApplication;
        }
        if !self.owner.IsAvailable() || !self.geometry.PositionIsValid(&position) {
            return WebInputEventResult::kNotHandled;
        }
        let extend = self.selection_state.get() == SelectionState::kExtendedSelection;
        self.selection_state.set(SelectionState::kExtendedSelection);
        let selection = self.owner.Current();
        if selection.IsNone() {
            return WebInputEventResult::kNotHandled;
        }
        let Some(position) =
            self.AdjustPositionRespectUserSelectAll(&target, &selection, Some(position))
        else {
            return WebInputEventResult::kNotHandled;
        };
        let adjusted = if extend {
            self.ExtendSelectionAsDirectional(&position, &selection, self.owner.Granularity())
        } else {
            self.geometry.Collapse(&position)
        };
        let visible = if adjusted.IsRange() {
            self.geometry.VisibleSelection(&adjusted)
        } else {
            adjusted
        };
        if visible.IsNone() {
            return WebInputEventResult::kNotHandled;
        }
        self.SetNonDirectionalSelectionIfNeeded(
            &visible,
            SetSelectionOptions {
                granularity: self.owner.Granularity(),
                is_directional: extend && self.owner.IsDirectional(),
                ..Default::default()
            },
            true,
        );
        WebInputEventResult::kHandledSystem
    }
    // Source: selection_controller.cc:712-747.
    pub fn UpdateSelectionForMouseDownDispatchingSelectStart(
        &self,
        target: Option<&EventTargetRef>,
        selection: &S,
        options: SetSelectionOptions,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        super::gesture_selection::GestureSelectionController {
            owner: self.owner.as_ref(),
            geometry: self.geometry.as_ref(),
            selection_state: &self.selection_state,
            original_anchor: &self.original_anchor,
            prepared,
            marker: std::marker::PhantomData,
        }
        .UpdateSelection(target, selection, options)
    }
    // Source: selection_controller.cc:749-826,881-900.
    pub fn SelectClosestWordFromHitTestResult(
        &self,
        hit: &H,
        append_whitespace: bool,
        touch: bool,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        super::gesture_selection::GestureSelectionController {
            owner: self.owner.as_ref(),
            geometry: self.geometry.as_ref(),
            selection_state: &self.selection_state,
            original_anchor: &self.original_anchor,
            prepared,
            marker: std::marker::PhantomData,
        }
        .SelectClosestWordFromHitTestResult(hit, append_whitespace, touch)
    }
    fn SelectClosestWordFromMouseEvent(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        if !self.mouse_down_may_start_select.get() {
            return false;
        }
        self.SelectClosestWordFromHitTestResult(
            &event.hit,
            event.event.click_count == 2 && self.owner.IsSelectTrailingWhitespaceEnabled(),
            event.event.FromTouch(),
            prepared,
        )
    }
    // Source: selection_controller.cc:955-1024.
    fn SetNonDirectionalSelectionIfNeeded(
        &self,
        selection: &S,
        mut options: SetSelectionOptions,
        adjust_bidi: bool,
    ) {
        let original = self
            .original_anchor
            .borrow()
            .clone()
            .filter(|position| self.geometry.PositionIsConnected(position))
            .and_then(|position| self.geometry.VisiblePositionOfPlain(&position));
        let anchor = original.clone().or_else(|| {
            selection
                .Anchor()
                .and_then(|position| self.geometry.VisiblePositionOfPlain(&position))
        });
        let focus = selection
            .Focus()
            .and_then(|position| self.geometry.VisiblePositionOfPlain(&position));
        let (Some(anchor), Some(focus)) = (anchor, focus) else {
            let directional =
                self.owner.ShouldConsiderSelectionAsDirectional() || options.is_directional;
            if self.owner.Current() == self.geometry.VisibleSelection(selection)
                && self.owner.IsHandleVisible() == options.should_show_handle
                && directional == self.owner.IsDirectional()
            {
                return;
            }
            options.should_close_typing = true;
            options.should_clear_typing_style = true;
            options.is_directional = directional;
            options.cursor_align_on_scroll = CursorAlignOnScroll::kIfNeeded;
            self.owner.SetSelection(selection, options);
            return;
        };
        let adjusted = if adjust_bidi {
            self.geometry.AdjustAtBidiBoundary(&anchor, &focus)
        } else {
            self.geometry.Build(&anchor, &focus)
        };
        let mut chosen = selection.clone();
        if adjusted.Anchor() != Some(anchor.clone()) || adjusted.Focus() != Some(focus.clone()) {
            *self.original_anchor.borrow_mut() = Some(anchor.clone());
            self.owner.BindOriginalAnchorToCurrentContext();
            chosen = adjusted;
        } else if let Some(original) = original {
            if self
                .owner
                .Current()
                .Anchor()
                .and_then(|position| self.geometry.VisiblePositionOfPlain(&position))
                == selection
                    .Anchor()
                    .and_then(|position| self.geometry.VisiblePositionOfPlain(&position))
            {
                if let Some(focus) = selection.Focus() {
                    chosen = self.geometry.Build(&original, &focus);
                }
            }
            *self.original_anchor.borrow_mut() = None;
        }
        let directional =
            self.owner.ShouldConsiderSelectionAsDirectional() || options.is_directional;
        if self.owner.Current() == self.geometry.VisibleSelection(&chosen)
            && self.owner.IsHandleVisible() == options.should_show_handle
            && directional == self.owner.IsDirectional()
        {
            return;
        }
        options.should_close_typing = true;
        options.should_clear_typing_style = true;
        options.is_directional = directional;
        options.cursor_align_on_scroll = CursorAlignOnScroll::kIfNeeded;
        self.owner.SetSelection(&chosen, options);
    }
    // Source: selection_controller.cc:1026-1053.
    pub fn SetCaretAtHitTestResult(&self, hit: &H, prepared: &dyn PreparedSelectionGeometry) {
        super::gesture_selection::GestureSelectionController {
            owner: self.owner.as_ref(),
            geometry: self.geometry.as_ref(),
            selection_state: &self.selection_state,
            original_anchor: &self.original_anchor,
            prepared,
            marker: std::marker::PhantomData,
        }
        .SetCaretAtHitTestResult(hit)
    }
    // Source: selection_controller.cc:1055-1140.
    pub fn HandleDoubleClick(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        if !self.owner.IsAvailable() {
            return false;
        }
        if !self.mouse_down_allows_multi_click.get() {
            return self.HandleSingleClick(event, prepared);
        }
        if event.event.pointer_properties.button != Button::kLeft {
            return false;
        }
        if self.owner.Current().IsRange() {
            self.selection_state.set(SelectionState::kExtendedSelection);
            return true;
        }
        if !self.SelectClosestWordFromMouseEvent(event, prepared) || !self.owner.IsHandleVisible() {
            return true;
        }
        self.owner.ShowContextMenu(SelectionContextMenu::Touch);
        true
    }
    pub fn HandleTripleClick(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        if !self.owner.IsAvailable() {
            return false;
        }
        if !self.mouse_down_allows_multi_click.get() {
            return self.HandleSingleClick(event, prepared);
        }
        if event.event.pointer_properties.button != Button::kLeft {
            return false;
        }
        let (Some(node), Some(pseudo)) = (
            self.geometry.InnerNode(&event.hit),
            self.geometry.InnerPossiblyPseudoNode(&event.hit),
        ) else {
            return false;
        };
        if !self.geometry.HasLayoutObject(&node)
            || !self.geometry.HasLayoutObject(&pseudo)
            || !self.mouse_down_may_start_select.get()
        {
            return false;
        }
        let selection = self.geometry.PositionOfHit(&event.hit, false).map_or_else(
            || self.geometry.Empty(),
            |position| {
                self.geometry.ExpandWithGranularity(
                    &self.geometry.Collapse(&position),
                    TextGranularity::kParagraph,
                )
            },
        );
        let adjusted = self.AdjustSelectionByUserSelect(&node, &selection);
        if !self.UpdateSelectionForMouseDownDispatchingSelectStart(
            Some(&node),
            &adjusted,
            SetSelectionOptions {
                granularity: TextGranularity::kParagraph,
                should_show_handle: event.event.FromTouch() && selection.IsRange(),
                ..Default::default()
            },
            prepared,
        ) {
            return false;
        }
        if self.owner.IsHandleVisible() {
            self.owner.ShowContextMenu(SelectionContextMenu::Touch);
        }
        true
    }
    // Source: selection_controller.cc:1142-1181.
    pub fn HandleMousePressEvent(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        let node = self.geometry.InnerNode(&event.hit);
        let selection_over_link = event.event.GetModifiers() & WebInputEvent::kAltKey != 0
            && self.geometry.IsOverLink(&event.hit);
        let may_select = (self.CanMouseDownStartSelect(node.as_ref()) || selection_over_link)
            && !self.geometry.IsOverScrollbar(&event.hit);
        self.mouse_down_may_start_select.set(may_select);
        if may_select
            && self
                .geometry
                .InnerPossiblyPseudoNode(&event.hit)
                .is_some_and(|pseudo| {
                    self.geometry.IsPseudoElement(&pseudo)
                        && self.geometry.HasActivationBehavior(&pseudo)
                })
        {
            self.mouse_down_may_start_select.set(false);
        }
        self.mouse_down_was_single_click_on_caret.set(false);
        self.input_state.ResetSingleClickInSelection();
        self.mouse_down_allows_multi_click.set(
            !event.event.FromTouch()
                || (self.owner.IsAvailable() && self.owner.CurrentStartIsEditable()),
        );
        if event.event.click_count >= 3 {
            return self.HandleTripleClick(event, prepared);
        }
        if event.event.click_count == 2 {
            return self.HandleDoubleClick(event, prepared);
        }
        self.HandleSingleClick(event, prepared)
    }
    // Source: selection_controller.cc:1183-1220.
    pub fn HandleMouseDraggedEvent(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        mouse_down: (f64, f64),
        prepared: &dyn PreparedSelectionGeometry,
    ) -> WebInputEventResult {
        if !self.owner.IsAvailable() {
            return WebInputEventResult::kNotHandled;
        }
        if self.selection_state.get() != SelectionState::kExtendedSelection {
            let hit = self.geometry.HitTestAtContentsPoint(mouse_down, false);
            self.UpdateSelectionForMouseDrag(&hit, prepared);
        }
        self.UpdateSelectionForMouseDrag(&event.hit, prepared)
    }
    // Source: selection_controller.cc:1203-1220. The native frame/view owner
    // supplies its resident hit-test query without materializing another tree.
    pub fn UpdateSelectionForMouseDragInRootFrame(
        &self,
        root_position: (f64, f64),
        has_content_layout: bool,
        prepared: &dyn PreparedSelectionGeometry,
    ) {
        if !self.geometry.HasView() || !has_content_layout {
            return;
        }
        let point = self.geometry.ConvertRootPointToContents(root_position);
        let hit = self.geometry.HitTestAtContentsPoint(point, true);
        self.UpdateSelectionForMouseDrag(&hit, prepared);
    }
    // Source: selection_controller.cc:1222-1319.
    pub fn HandleMouseReleaseEvent(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        drag_start: (f64, f64),
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        if !self.owner.IsAvailable() {
            return false;
        }
        let mut handled = false;
        self.mouse_down_may_start_select.set(false);
        let point = event.event.PositionInRootFrame();
        if self.mouse_down_was_single_click_in_selection.get()
            && self.selection_state.get() != SelectionState::kExtendedSelection
            && drag_start == (point.x().floor() as f64, point.y().floor() as f64)
            && self.owner.Current().IsRange()
            && event.event.pointer_properties.button != Button::kRight
        {
            prepared.Prepare(SelectionGeometryPreparation::StyleAndLayoutForSelection);
            let selection = self
                .geometry
                .InnerNode(&event.hit)
                .filter(|node| {
                    self.geometry.HasLayoutObject(node) && self.geometry.IsEditable(node)
                })
                .and_then(|_| self.geometry.PositionOfHit(&event.hit, false))
                .and_then(|position| self.geometry.VisiblePosition(&position))
                .map_or_else(
                    || self.geometry.Empty(),
                    |position| self.geometry.Collapse(&position),
                );
            if self.owner.Current() != self.geometry.VisibleSelection(&selection) {
                self.owner.SetSelectionAndEndTyping(&selection);
            }
            handled = true;
        }
        self.owner.NotifyTextControlOfSelectionChange();
        self.owner.SelectFrameElementInParentIfFullySelected();
        if event.event.pointer_properties.button == Button::kMiddle
            && !self.geometry.IsOverLink(&event.hit)
        {
            handled = self.HandlePasteGlobalSelection(&event.event) || handled;
        }
        handled
    }
    pub fn HandlePasteGlobalSelection(&self, event: &WebMouseEvent) -> bool {
        if event.GetType() != Type::kMouseUp
            || !self.owner.HasPage()
            || !self.owner.MiddleClickPasteAllowed()
        {
            return false;
        }
        self.owner.IsFocusedOrMainFrame() && self.owner.ExecutePasteGlobalSelection()
    }
    // Source: selection_controller.cc:1321-1356. The caller prepares the
    // original node's style/layout at the synchronous source lifecycle point.
    pub fn HandleGestureLongPress(
        &self,
        hit: &H,
        long_press_link_select_text: bool,
        prepared: &dyn PreparedSelectionGeometry,
    ) -> bool {
        super::gesture_selection::GestureSelectionController {
            owner: self.owner.as_ref(),
            geometry: self.geometry.as_ref(),
            selection_state: &self.selection_state,
            original_anchor: &self.original_anchor,
            prepared,
            marker: std::marker::PhantomData,
        }
        .HandleGestureLongPress(hit, long_press_link_select_text)
    }
    pub fn HandleGestureTwoFingerTap(&self, hit: &H, prepared: &dyn PreparedSelectionGeometry) {
        self.SetCaretAtHitTestResult(hit, prepared);
    }
    // Source: selection_controller.cc:1430-1453. Selection belongs to the
    // parent frame; a subframe press collapses that original source selection.
    /// Source selection_controller.cc:828-880,903-953,1373-1426.
    pub fn UpdateSelectionForContextMenuEvent(
        &self,
        event: SelectionContextMenuEvent,
        hit: &H,
        position: (f64, f64),
        prepared: &dyn PreparedSelectionGeometry,
    ) {
        use interaction_input::ui::base::mojom::menu_source_type::MenuSourceType;
        if !self.owner.IsAvailable() {
            return;
        }
        if self.owner.Contains(position)
            || self.geometry.IsOverScrollbar(hit)
            || !(self.owner.CurrentStartIsEditable()
                || self
                    .geometry
                    .InnerNode(hit)
                    .is_some_and(|node| self.geometry.NodeIsText(&node)))
        {
            return;
        }
        struct Reset<'a> {
            cell: &'a Cell<bool>,
            old: bool,
        }
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.cell.set(self.old)
            }
        }
        let _restore = Reset {
            cell: &self.mouse_down_may_start_select,
            old: self.mouse_down_may_start_select.replace(true),
        };
        let append = event.click_count == 2 && self.owner.IsSelectTrailingWhitespaceEnabled();
        if event.menu_source != MenuSourceType::kTouchHandle
            && self
                .geometry
                .HitTestResultIsMisspelled(hit, self.owner.SpellCheckEnabledByDefault())
        {
            let Some(node) = self.geometry.InnerPossiblyPseudoNode(hit) else {
                return;
            };
            if !self.geometry.HasLayoutObject(&node) {
                return;
            }
            let position = self
                .geometry
                .PositionOfHit(hit, false)
                .and_then(|p| self.geometry.VisiblePosition(&p));
            let group = position
                .as_ref()
                .and_then(|p| self.geometry.ParentAnchoredPosition(p))
                .and_then(|p| self.geometry.SpellCheckMarkerGroupAtPosition(&p));
            let options = SetSelectionOptions {
                granularity: TextGranularity::kWord,
                ..Default::default()
            };
            let Some((start, end)) = group else {
                self.UpdateSelectionForMouseDownDispatchingSelectStart(
                    Some(&node),
                    &self.geometry.Empty(),
                    options,
                    prepared,
                );
                return;
            };
            let selection = self
                .geometry
                .VisibleSelection(&self.geometry.Build(&start, &end));
            let adjusted = if append {
                self.AdjustSelectionWithTrailingWhitespace(&selection)
            } else {
                selection
            };
            self.UpdateSelectionForMouseDownDispatchingSelectStart(
                Some(&node),
                &self.ExpandSelectionToRespectUserSelectAll(&node, &adjusted),
                options,
                prepared,
            );
            return;
        }
        if self.mouse_down_was_single_click_on_caret.get()
            || !self.owner.ShouldSelectOnContextualMenuClick()
            || self.geometry.IsOverAnnotation(hit)
            || matches!(
                event.menu_source,
                MenuSourceType::kLongPress | MenuSourceType::kKeyboard
            )
        {
            return;
        }
        if !self.geometry.IsLiveLink(hit) {
            self.SelectClosestWordFromHitTestResult(hit, append, event.from_touch, prepared);
            return;
        }
        let Some(node) = self.geometry.InnerNode(hit) else {
            return;
        };
        if !self.geometry.HasLayoutObject(&node) {
            return;
        }
        let selection = self
            .geometry
            .URLElement(hit)
            .and_then(|url| {
                let position = self
                    .geometry
                    .PositionOfHit(hit, false)
                    .and_then(|p| self.geometry.VisiblePosition(&p))?;
                self.geometry
                    .PositionIsDescendantOf(&position, &url)
                    .then(|| {
                        self.geometry.Build(
                            &self.geometry.FirstPositionInNode(&url),
                            &self.geometry.LastPositionInNode(&url),
                        )
                    })
            })
            .unwrap_or_else(|| self.geometry.Empty());
        self.UpdateSelectionForMouseDownDispatchingSelectStart(
            Some(&node),
            &self.ExpandSelectionToRespectUserSelectAll(&node, &selection),
            SetSelectionOptions {
                granularity: TextGranularity::kWord,
                ..Default::default()
            },
            prepared,
        );
    }
    pub fn PassMousePressEventToSubframe(
        &self,
        event: &MouseEventWithSelectionHit<'_, H>,
        prepared: &dyn PreparedSelectionGeometry,
    ) {
        prepared.Prepare(SelectionGeometryPreparation::StyleAndLayoutForInput);
        let point = event.event.PositionInRootFrame();
        let contents = self
            .geometry
            .ConvertRootPointToContents((point.x().floor() as f64, point.y().floor() as f64));
        if !self.owner.Contains(contents) {
            return;
        }
        let selection = self
            .geometry
            .PositionOfHit(event.hit, false)
            .and_then(|position| self.geometry.VisiblePosition(&position))
            .map_or_else(
                || self.geometry.Empty(),
                |position| self.geometry.Collapse(&position),
            );
        self.owner.SetSelectionAndEndTyping(&selection);
    }
    // Source: selection_controller.cc:1455-1489.
    pub fn InitializeSelectionState(&self) {
        self.selection_state
            .set(SelectionState::kHaveNotStartedSelection);
    }
    // Source: selection_controller.h:79-81.
    pub fn HasExtendedSelection(&self) -> bool {
        self.selection_state.get() == SelectionState::kExtendedSelection
    }
    pub fn SetMouseDownMayStartSelect(&self, may_start: bool) {
        self.mouse_down_may_start_select.set(may_start);
    }
    pub fn MouseDownMayStartSelect(&self) -> bool {
        self.mouse_down_may_start_select.get()
    }
    pub fn MouseDownWasSingleClickInSelection(&self) -> bool {
        self.mouse_down_was_single_click_in_selection.get()
    }
    pub fn NotifySelectionChanged(&self) {
        let selection = self.owner.Current();
        self.selection_state.set(if selection.IsNone() {
            SelectionState::kHaveNotStartedSelection
        } else if selection.IsCaret() {
            SelectionState::kPlacedCaret
        } else {
            debug_assert!(selection.IsRange());
            SelectionState::kExtendedSelection
        });
    }
}

// Source selection_controller.cc:1495-1524. Both the mouse manager and
// EventHandler use this exact event-modifier predicate.
pub fn IsSelectionOverLink(event: &WebMouseEvent, is_over_link: bool) -> bool {
    event.GetModifiers() & WebInputEvent::kAltKey != 0 && is_over_link
}
pub fn IsExtendingSelection(
    event: &WebMouseEvent,
    is_over_link: bool,
    has_image: bool,
    is_user_node_draggable: bool,
) -> bool {
    event.GetModifiers() & WebInputEvent::kShiftKey != 0
        && !is_over_link
        && !has_image
        && !is_user_node_draggable
}
