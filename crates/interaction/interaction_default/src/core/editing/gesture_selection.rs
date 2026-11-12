// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: selection_controller.cc:189-207,712-826,955-1053,1321-1356.
// These are the same SelectionController transactions used by mouse and touch.
// The smaller call-scoped ports admit a gesture without manufacturing unused
// drag, clipboard, spellcheck, or context-menu capabilities.
use super::{
    selection_controller::{FlatSelection, SelectionState},
    set_selection_options::{CursorAlignOnScroll, SetSelectionOptions},
    text_granularity::TextGranularity,
};
use interaction_dispatch::core::dom::events::{
    event::EventTargetRef, event_dispatch_result::DispatchEventResult,
};
use std::cell::{Cell, RefCell};

pub trait GestureSelectionOwner<S: FlatSelection> {
    fn IsAvailable(&self) -> bool;
    fn Current(&self) -> S;
    fn IsDirectional(&self) -> bool;
    fn IsHandleVisible(&self) -> bool;
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool;
    fn SetSelection(&self, selection: &S, options: SetSelectionOptions);
    fn BindOriginalAnchorToCurrentContext(&self);
}
pub trait GestureSelectionGeometry<S: FlatSelection, H> {
    fn InnerPossiblyPseudoNode(&self, hit: &H) -> Option<EventTargetRef>;
    fn IsLiveLink(&self, hit: &H) -> bool;
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool;
    fn IsSelectable(&self, node: &EventTargetRef) -> bool;
    fn IsEditable(&self, node: &EventTargetRef) -> bool;
    fn NodeIsText(&self, node: &EventTargetRef) -> bool;
    fn CanStartSelection(&self, node: &EventTargetRef) -> bool;
    fn IsConnected(&self, node: &EventTargetRef) -> bool;
    fn HasImage(&self, hit: &H) -> bool;
    fn PositionOfHit(&self, hit: &H, zero_image_offset: bool) -> Option<S::Position>;
    fn FirstPositionInOrBefore(&self, node: &EventTargetRef) -> Option<S::Position>;
    fn Empty(&self) -> S;
    fn Collapse(&self, position: &S::Position) -> S;
    fn Build(&self, anchor: &S::Position, focus: &S::Position) -> S;
    fn WithBaseAndExtent(&self, selection: &S, anchor: &S::Position, focus: &S::Position) -> S;
    fn ExpandWithGranularity(&self, selection: &S, granularity: TextGranularity) -> S;
    fn RootUserSelectAllBounds(&self, node: &EventTargetRef) -> Option<(S::Position, S::Position)>;
    fn PlainText(&self, selection: &S) -> Vec<u16>;
    fn ContainsOnlyWhitespaceAfterSimplifying(&self, text: &[u16]) -> bool;
    fn IsLastPositionInEditableRoot(&self, position: &S::Position, selection: &S) -> bool;
    fn SkipWhitespace(&self, position: &S::Position) -> S::Position;
    fn ResetInvalidSelectionAfterEvent(&self, selection: &S) -> S;
    fn VisibleSelection(&self, selection: &S) -> S;
    fn PositionIsConnected(&self, position: &S::Position) -> bool;
    fn VisiblePositionOfPlain(&self, position: &S::Position) -> Option<S::Position>;
    fn DispatchSelectStart(&self, node: &EventTargetRef) -> DispatchEventResult;
}

/// Source fields, retained by the original SelectionController owner. Endpoints
/// in this state are only the source's optional bidi anchor, never a selection.
pub struct GestureSelectionState<P> {
    pub selection_state: std::rc::Rc<Cell<SelectionState>>,
    pub original_anchor: std::rc::Rc<RefCell<Option<P>>>,
    pub mouse_down_was_single_click_on_caret: std::rc::Rc<Cell<bool>>,
    pub mouse_down_allows_multi_click: std::rc::Rc<Cell<bool>>,
}
impl<P> Default for GestureSelectionState<P> {
    fn default() -> Self {
        Self {
            selection_state: std::rc::Rc::new(Cell::new(SelectionState::kHaveNotStartedSelection)),
            original_anchor: std::rc::Rc::new(RefCell::new(None)),
            mouse_down_was_single_click_on_caret: std::rc::Rc::new(Cell::new(false)),
            mouse_down_allows_multi_click: std::rc::Rc::new(Cell::new(false)),
        }
    }
}
pub struct GestureSelectionController<'a, S: FlatSelection, H, O: ?Sized, G: ?Sized> {
    pub owner: &'a O,
    pub geometry: &'a G,
    pub selection_state: &'a Cell<SelectionState>,
    pub original_anchor: &'a RefCell<Option<S::Position>>,
    pub prepared: &'a dyn super::selection_controller::PreparedSelectionGeometry,
    pub marker: std::marker::PhantomData<H>,
}
impl<
        S: FlatSelection,
        H,
        O: GestureSelectionOwner<S> + ?Sized,
        G: GestureSelectionGeometry<S, H> + ?Sized,
    > GestureSelectionController<'_, S, H, O, G>
{
    fn ExpandUserSelectAll(&self, node: &EventTargetRef, selection: &S) -> S {
        if selection.IsNone() {
            return self.geometry.Empty();
        }
        self.geometry.RootUserSelectAllBounds(node).map_or_else(
            || selection.clone(),
            |(start, end)| self.geometry.WithBaseAndExtent(selection, &start, &end),
        )
    }
    // Source: selection_controller.cc:955-1024, adjust_bidi=false branch.
    pub fn SetNonDirectionalSelectionIfNeeded(
        &self,
        selection: &S,
        mut options: SetSelectionOptions,
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
        let mut chosen = selection.clone();
        if let (Some(anchor), Some(focus)) = (anchor, focus) {
            let adjusted = self.geometry.Build(&anchor, &focus);
            if adjusted.Anchor() != Some(anchor.clone()) || adjusted.Focus() != Some(focus.clone())
            {
                *self.original_anchor.borrow_mut() = Some(anchor);
                self.owner.BindOriginalAnchorToCurrentContext();
                chosen = adjusted;
            } else if let Some(original) = original {
                if self
                    .owner
                    .Current()
                    .Anchor()
                    .and_then(|p| self.geometry.VisiblePositionOfPlain(&p))
                    == selection
                        .Anchor()
                        .and_then(|p| self.geometry.VisiblePositionOfPlain(&p))
                {
                    if let Some(focus) = selection.Focus() {
                        chosen = self.geometry.Build(&original, &focus);
                    }
                }
                *self.original_anchor.borrow_mut() = None;
            }
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
    // Source: selection_controller.cc:712-747. No native borrow survives
    // selectstart; positions are checked again in the live document afterward.
    pub fn UpdateSelection(
        &self,
        target: Option<&EventTargetRef>,
        selection: &S,
        options: SetSelectionOptions,
    ) -> bool {
        use super::selection_controller::SelectionGeometryPreparation;
        if target.is_some_and(|node| {
            self.geometry.HasLayoutObject(node) && !self.geometry.IsSelectable(node)
        }) {
            return false;
        }
        let dispatch = target
            .filter(|node| self.geometry.HasLayoutObject(node))
            .map_or(DispatchEventResult::kNotCanceled, |node| {
                self.geometry.DispatchSelectStart(node)
            });
        let selection = self.geometry.ResetInvalidSelectionAfterEvent(selection);
        if dispatch != DispatchEventResult::kNotCanceled || !self.owner.IsAvailable() {
            return false;
        }
        self.prepared
            .Prepare(SelectionGeometryPreparation::StyleAndLayoutForSelection);
        let visible = self.geometry.VisibleSelection(&selection);
        self.selection_state.set(if visible.IsRange() {
            SelectionState::kExtendedSelection
        } else {
            SelectionState::kPlacedCaret
        });
        self.SetNonDirectionalSelectionIfNeeded(&visible, options);
        true
    }
    // Source: selection_controller.cc:749-826,881-900.
    pub fn SelectClosestWordFromHitTestResult(
        &self,
        hit: &H,
        append_whitespace: bool,
        touch: bool,
    ) -> bool {
        let Some(node) = self.geometry.InnerPossiblyPseudoNode(hit) else {
            return false;
        };
        if !self.geometry.HasLayoutObject(&node) || !self.geometry.IsSelectable(&node) {
            return false;
        }
        let position = self
            .geometry
            .PositionOfHit(hit, touch && self.geometry.HasImage(hit));
        let selection = position.as_ref().map_or_else(
            || self.geometry.Empty(),
            |position| {
                self.geometry.ExpandWithGranularity(
                    &self.geometry.Collapse(position),
                    TextGranularity::kWord,
                )
            },
        );
        if selection.IsNone() || selection.Start() > selection.End() {
            return false;
        }
        if touch {
            let word = self.geometry.PlainText(&selection);
            if word.first() == Some(&10)
                || self.geometry.ContainsOnlyWhitespaceAfterSimplifying(&word)
            {
                return false;
            }
            if position.as_ref().is_some_and(|position| {
                self.geometry
                    .IsLastPositionInEditableRoot(position, &selection)
            }) {
                return false;
            }
        }
        let adjusted = if append_whitespace {
            selection.End().map_or_else(
                || selection.clone(),
                |end| {
                    let new_end = self.geometry.SkipWhitespace(&end);
                    if new_end == end {
                        return selection.clone();
                    }
                    if selection.IsAnchorFirst() {
                        selection.Anchor().map_or_else(
                            || selection.clone(),
                            |anchor| {
                                self.geometry
                                    .WithBaseAndExtent(&selection, &anchor, &new_end)
                            },
                        )
                    } else {
                        selection.Focus().map_or_else(
                            || selection.clone(),
                            |focus| {
                                self.geometry
                                    .WithBaseAndExtent(&selection, &new_end, &focus)
                            },
                        )
                    }
                },
            )
        } else {
            selection
        };
        let expanded = self.ExpandUserSelectAll(&node, &adjusted);
        self.UpdateSelection(
            Some(&node),
            &expanded,
            SetSelectionOptions {
                granularity: TextGranularity::kWord,
                should_show_handle: touch,
                ..Default::default()
            },
        )
    }
    // Source: selection_controller.cc:1026-1053.
    pub fn SetCaretAtHitTestResult(&self, hit: &H) {
        let Some(node) = self.geometry.InnerPossiblyPseudoNode(hit) else {
            return;
        };
        let position = self
            .geometry
            .PositionOfHit(hit, false)
            .or_else(|| self.geometry.FirstPositionInOrBefore(&node));
        let selection = position.as_ref().map_or_else(
            || self.geometry.Empty(),
            |position| self.ExpandUserSelectAll(&node, &self.geometry.Collapse(position)),
        );
        self.UpdateSelection(
            Some(&node),
            &selection,
            SetSelectionOptions {
                should_show_handle: true,
                ..Default::default()
            },
        );
    }
    // Source: selection_controller.cc:1321-1356.
    pub fn HandleGestureLongPress(&self, hit: &H, long_press_link_select_text: bool) -> bool {
        use super::selection_controller::SelectionGeometryPreparation;
        if !self.owner.IsAvailable()
            || (!long_press_link_select_text && self.geometry.IsLiveLink(hit))
        {
            return false;
        }
        let Some(node) = self.geometry.InnerPossiblyPseudoNode(hit) else {
            return false;
        };
        self.prepared
            .Prepare(SelectionGeometryPreparation::StyleAndLayoutTree);
        if !(self.geometry.IsEditable(&node)
            || self.geometry.NodeIsText(&node)
            || self.geometry.CanStartSelection(&node))
        {
            return false;
        }
        if self.SelectClosestWordFromHitTestResult(hit, false, true) {
            return self.owner.IsAvailable();
        }
        if !self.geometry.IsConnected(&node) || !self.geometry.HasLayoutObject(&node) {
            return false;
        }
        self.SetCaretAtHitTestResult(hit);
        false
    }
}

impl<S: FlatSelection, T: super::selection_controller::SelectionControllerOwner<S> + ?Sized>
    GestureSelectionOwner<S> for T
{
    fn IsAvailable(&self) -> bool {
        super::selection_controller::SelectionControllerOwner::IsAvailable(self)
    }
    fn Current(&self) -> S {
        super::selection_controller::SelectionControllerOwner::Current(self)
    }
    fn IsDirectional(&self) -> bool {
        super::selection_controller::SelectionControllerOwner::IsDirectional(self)
    }
    fn IsHandleVisible(&self) -> bool {
        super::selection_controller::SelectionControllerOwner::IsHandleVisible(self)
    }
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool {
        super::selection_controller::SelectionControllerOwner::ShouldConsiderSelectionAsDirectional(
            self,
        )
    }
    fn SetSelection(&self, s: &S, o: SetSelectionOptions) {
        super::selection_controller::SelectionControllerOwner::SetSelection(self, s, o)
    }
    fn BindOriginalAnchorToCurrentContext(&self) {
        super::selection_controller::SelectionControllerOwner::BindOriginalAnchorToCurrentContext(
            self,
        )
    }
}
macro_rules! delegate_geometry {
    ($($name:ident($($arg:ident:$ty:ty),*) -> $ret:ty;)+)=>{$(fn $name(&self,$($arg:$ty),*)->$ret{super::selection_controller::SelectionGeometry::$name(self,$($arg),*)})+};
}
impl<S: FlatSelection, H, T: super::selection_controller::SelectionGeometry<S, H> + ?Sized>
    GestureSelectionGeometry<S, H> for T
{
    delegate_geometry! {
        InnerPossiblyPseudoNode(hit:&H)->Option<EventTargetRef>;
        IsLiveLink(hit:&H)->bool;
        HasLayoutObject(node:&EventTargetRef)->bool;
        IsSelectable(node:&EventTargetRef)->bool;
        IsEditable(node:&EventTargetRef)->bool;
        NodeIsText(node:&EventTargetRef)->bool;
        CanStartSelection(node:&EventTargetRef)->bool;
        IsConnected(node:&EventTargetRef)->bool;
        HasImage(hit:&H)->bool;
        PositionOfHit(hit:&H,zero_image_offset:bool)->Option<S::Position>;
        FirstPositionInOrBefore(node:&EventTargetRef)->Option<S::Position>;
        Empty()->S;
        Collapse(position:&S::Position)->S;
        Build(anchor:&S::Position,focus:&S::Position)->S;
        WithBaseAndExtent(selection:&S,anchor:&S::Position,focus:&S::Position)->S;
        ExpandWithGranularity(selection:&S,granularity:TextGranularity)->S;
        RootUserSelectAllBounds(node:&EventTargetRef)->Option<(S::Position,S::Position)>;
        PlainText(selection:&S)->Vec<u16>;
        ContainsOnlyWhitespaceAfterSimplifying(text:&[u16])->bool;
        IsLastPositionInEditableRoot(position:&S::Position,selection:&S)->bool;
        SkipWhitespace(position:&S::Position)->S::Position;
        ResetInvalidSelectionAfterEvent(selection:&S)->S;
        VisibleSelection(selection:&S)->S;
        PositionIsConnected(position:&S::Position)->bool;
        VisiblePositionOfPlain(position:&S::Position)->Option<S::Position>;
        DispatchSelectStart(node:&EventTargetRef)->DispatchEventResult;
    }
}
