//! Full source SelectionController ports over the same FrameSelection used by
//! gestures, editing commands and DOM selection. Ports live for one synchronous
//! input transaction; only source controller flags and bidi anchor persist.
use super::gesture_selection::{
    GestureFlatSelection as S, GesturePosition as P, PageGestureSelection,
};
use super::*;
use crate::adapter::page_editing_visible_position::{
    CompositionTextQuery, CompositionTextResult, MouseSelectionCursor,
    MouseSelectionGeometryQuery as Q, MouseSelectionGeometryResult as V,
};
use crate::adapter::page_pointer_hit_test::{PagePointerGeometry, PointerHitResult as H};
use interaction_default::core::editing::{
    gesture_selection::{GestureSelectionGeometry as G, GestureSelectionOwner as O},
    selection_controller::{
        self as source, FlatPositionIterator, FlatSelection, PreparedSelectionGeometry,
        SelectionContextMenu, SelectionControllerOwner, SelectionGeometry,
        SelectionGeometryPreparation,
    },
    set_selection_options::SetSelectionOptions,
    text_granularity::TextGranularity,
};
use layoutng_assembly::{editing_state::DOMSelection, EditingPositionWithAffinity};

#[derive(Clone, Copy)]
pub(crate) struct MouseSelectionSettings {
    pub select_trailing_whitespace: bool,
    pub spellcheck_enabled_by_default: bool,
    pub middle_click_paste_allowed: bool,
}
/// Native frame effects which are outside selection's algorithm/position
/// owner. Missing effects are errors, never successful placeholder methods.
pub(crate) trait PageMouseSelectionServices {
    fn MarkerOwner(&self) -> crate::adapter::page_editing_visible_position::CompositionMarkerOwner;
    fn Settings(&self) -> MouseSelectionSettings;
    fn ShowContextMenu(&self, kind: SelectionContextMenu) -> Result<(), &'static str>;
    fn MarkSelectionEndpointsForRepaint(
        &self,
        selection: Option<DOMSelection>,
    ) -> Result<(), &'static str>;
    fn HandlePotentialSuggestionTap(
        &self,
        position: EditingPositionWithAffinity,
    ) -> Result<(), &'static str>;
    fn ExecutePasteGlobalSelection(&self) -> Result<bool, &'static str>;
    fn IsFocusedOrMainFrame(&self) -> bool;
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum MouseSelectionError {
    SourceLeaf(&'static str),
    Geometry(super::gesture_selection::GestureSelectionError),
}
pub(crate) struct PageMouseSelection<'a, 'listener, 'frame> {
    pub(crate) gesture: PageGestureSelection<'a, 'listener, 'frame>,
    geometry: Rc<dyn PagePointerGeometry>,
    services: &'a dyn PageMouseSelectionServices,
    error: RefCell<Option<MouseSelectionError>>,
}
impl<'a, 'listener, 'frame> PageMouseSelection<'a, 'listener, 'frame> {
    pub(crate) fn new(
        dispatcher: &'a EventDispatcher<'listener, 'frame>,
        geometry: Rc<dyn PagePointerGeometry>,
        services: &'a dyn PageMouseSelectionServices,
    ) -> Rc<Self> {
        Rc::new(Self {
            gesture: PageGestureSelection::new(dispatcher, geometry.clone()),
            geometry,
            services,
            error: Default::default(),
        })
    }
    pub(crate) fn Controller(self: &Rc<Self>) -> source::SelectionController<'_, S, H> {
        source::SelectionController::WithSharedState(
            self.clone(),
            self.clone(),
            foundation::RuntimeEnabledFeatures::LeftClickToHandleSuggestionEnabled(),
            self.gesture.dispatcher.selection_controller_state.clone(),
            &self.gesture.state,
        )
    }
    pub(crate) fn TakeError(&self) -> Option<MouseSelectionError> {
        self.error
            .borrow_mut()
            .take()
            .or_else(|| self.gesture.TakeError().map(MouseSelectionError::Geometry))
    }
    fn Fault(&self, leaf: &'static str) {
        *self.error.borrow_mut() = Some(MouseSelectionError::SourceLeaf(leaf));
    }
    fn Effect(&self, result: Result<(), &'static str>) {
        if let Err(leaf) = result {
            self.Fault(leaf);
        }
    }
    fn Query(&self, q: Q) -> Option<V> {
        let result =
            self.gesture
                .frame
                .CompositionText(CompositionTextQuery::MouseSelectionGeometry {
                    query: q,
                    ui_locale: self.gesture.Locale()?.clone(),
                });
        match result {
            Some(CompositionTextResult::MouseSelectionGeometry(v)) => Some(v),
            _ => {
                self.Fault("SelectionController native mouse geometry");
                None
            }
        }
    }
    fn PositionQuery(&self, q: Q) -> Option<P> {
        match self.Query(q)? {
            V::Position(p) => self.gesture.Position(p?),
            _ => None,
        }
    }
    fn CurrentVisible(&self) -> S {
        G::VisibleSelection(&self.gesture, &O::Current(&self.gesture))
    }
    fn NodeFact(&self, n: &EventTargetRef, f: impl FnOnce(&dom::Document, usize) -> bool) -> bool {
        self.gesture.NodeFact(n, f)
    }
}
impl PreparedSelectionGeometry for PageMouseSelection<'_, '_, '_> {
    fn Prepare(&self, p: SelectionGeometryPreparation) {
        self.gesture.Prepare(p)
    }
}
impl SelectionControllerOwner<S> for PageMouseSelection<'_, '_, '_> {
    fn IsAvailable(&self) -> bool {
        O::IsAvailable(&self.gesture)
    }
    fn Current(&self) -> S {
        self.CurrentVisible()
    }
    fn CurrentStartIsEditable(&self) -> bool {
        self.gesture
            .frame
            .VisibleSelectionRootEditableElement()
            .is_some()
    }
    fn IsDirectional(&self) -> bool {
        O::IsDirectional(&self.gesture)
    }
    fn Granularity(&self) -> TextGranularity {
        self.gesture.frame.SelectionControllerFlags().0
    }
    fn IsHandleVisible(&self) -> bool {
        O::IsHandleVisible(&self.gesture)
    }
    fn ShouldShrinkNextTap(&self) -> bool {
        self.gesture.frame.SelectionControllerFlags().1
    }
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool {
        O::ShouldConsiderSelectionAsDirectional(&self.gesture)
    }
    fn ShouldToggleMenuWhenCaretTapped(&self) -> bool {
        self.gesture.settings.as_ref().is_some_and(|s|s.editing_behavior==interaction_default::core::editing::editing_behavior::EditingBehaviorType::ChromeOS)
    }
    fn IsSelectTrailingWhitespaceEnabled(&self) -> bool {
        self.services.Settings().select_trailing_whitespace
    }
    fn ShouldSelectOnContextualMenuClick(&self) -> bool {
        self.gesture
            .settings
            .as_ref()
            .is_some_and(|s| s.editing_behavior.ShouldSelectOnContextualMenuClick())
    }
    fn SpellCheckEnabledByDefault(&self) -> bool {
        self.services.Settings().spellcheck_enabled_by_default
    }
    fn Contains(&self, p: (f64, f64)) -> bool {
        let Some(settings) = self.gesture.settings.clone() else {
            self.Fault("Settings::editing behavior and UI locale");
            return false;
        };
        let point = foundation::PhysicalOffset::new(
            foundation::LayoutUnit::FromDoubleRound(p.0),
            foundation::LayoutUnit::FromDoubleRound(p.1),
        );
        match self.gesture.frame.ContainsPoint(
            self.gesture.dispatcher.document.clone(),
            self.geometry.clone(),
            settings,
            point,
            self.gesture.dispatcher.focus_services.lifecycle.HasFrame(),
        ) {
            Ok(value) => value,
            Err(error) => {
                *self.error.borrow_mut() = Some(MouseSelectionError::Geometry(error));
                false
            }
        }
    }
    fn SetSelection(&self, s: &S, o: SetSelectionOptions) {
        O::SetSelection(&self.gesture, s, o)
    }
    fn SetSelectionAndEndTyping(&self, s: &S) {
        self.gesture.frame.SetSelectionAndEndTyping(s.value)
    }
    fn NotifyTextControlOfSelectionChange(&self) {
        self.gesture.frame.NotifyTextControlOfSelectionChange()
    }
    fn SelectFrameElementInParentIfFullySelected(&self) {
        self.gesture
            .frame
            .SelectFrameElementInParentIfFullySelected()
    }
    fn ShowContextMenu(&self, k: SelectionContextMenu) {
        self.Effect(self.services.ShowContextMenu(k))
    }
    fn MarkSelectionEndpointsForRepaint(&self, s: &S) {
        self.Effect(self.services.MarkSelectionEndpointsForRepaint(s.value))
    }
    fn BindOriginalAnchorToCurrentContext(&self) {
        O::BindOriginalAnchorToCurrentContext(&self.gesture)
    }
    fn HasPage(&self) -> bool {
        self.gesture.dispatcher.focus_services.lifecycle.HasFrame()
    }
    fn MiddleClickPasteAllowed(&self) -> bool {
        self.services.Settings().middle_click_paste_allowed
    }
    fn IsFocusedOrMainFrame(&self) -> bool {
        self.services.IsFocusedOrMainFrame()
    }
    fn ExecutePasteGlobalSelection(&self) -> bool {
        match self.services.ExecutePasteGlobalSelection() {
            Ok(v) => v,
            Err(e) => {
                self.Fault(e);
                false
            }
        }
    }
}

impl SelectionGeometry<S, H> for PageMouseSelection<'_, '_, '_> {
    fn InnerPossiblyPseudoNode(&self, hit: &H) -> Option<EventTargetRef> {
        G::InnerPossiblyPseudoNode(&self.gesture, hit)
    }
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool {
        G::HasLayoutObject(&self.gesture, node)
    }
    fn CanStartSelection(&self, node: &EventTargetRef) -> bool {
        G::CanStartSelection(&self.gesture, node)
    }
    fn IsSelectable(&self, node: &EventTargetRef) -> bool {
        G::IsSelectable(&self.gesture, node)
    }
    fn IsEditable(&self, node: &EventTargetRef) -> bool {
        G::IsEditable(&self.gesture, node)
    }
    fn IsConnected(&self, node: &EventTargetRef) -> bool {
        G::IsConnected(&self.gesture, node)
    }
    fn IsLiveLink(&self, hit: &H) -> bool {
        G::IsLiveLink(&self.gesture, hit)
    }
    fn HasImage(&self, hit: &H) -> bool {
        G::HasImage(&self.gesture, hit)
    }
    fn PositionOfHit(&self, hit: &H, zero_image_offset: bool) -> Option<P> {
        G::PositionOfHit(&self.gesture, hit, zero_image_offset)
    }
    fn FirstPositionInOrBefore(&self, node: &EventTargetRef) -> Option<P> {
        G::FirstPositionInOrBefore(&self.gesture, node)
    }
    fn PositionIsConnected(&self, position: &P) -> bool {
        G::PositionIsConnected(&self.gesture, position)
    }
    fn VisiblePositionOfPlain(&self, position: &P) -> Option<P> {
        G::VisiblePositionOfPlain(&self.gesture, position)
    }
    fn Empty(&self) -> S {
        G::Empty(&self.gesture)
    }
    fn Collapse(&self, position: &P) -> S {
        G::Collapse(&self.gesture, position)
    }
    fn Build(&self, anchor: &P, focus: &P) -> S {
        G::Build(&self.gesture, anchor, focus)
    }
    fn WithBaseAndExtent(&self, selection: &S, anchor: &P, focus: &P) -> S {
        G::WithBaseAndExtent(&self.gesture, selection, anchor, focus)
    }
    fn VisibleSelection(&self, selection: &S) -> S {
        G::VisibleSelection(&self.gesture, selection)
    }
    fn ResetInvalidSelectionAfterEvent(&self, selection: &S) -> S {
        G::ResetInvalidSelectionAfterEvent(&self.gesture, selection)
    }
    fn RootUserSelectAllBounds(&self, node: &EventTargetRef) -> Option<(P, P)> {
        G::RootUserSelectAllBounds(&self.gesture, node)
    }
    fn NodeIsText(&self, node: &EventTargetRef) -> bool {
        G::NodeIsText(&self.gesture, node)
    }
    fn PlainText(&self, selection: &S) -> Vec<u16> {
        G::PlainText(&self.gesture, selection)
    }
    fn ContainsOnlyWhitespaceAfterSimplifying(&self, text: &[u16]) -> bool {
        G::ContainsOnlyWhitespaceAfterSimplifying(&self.gesture, text)
    }
    fn IsLastPositionInEditableRoot(&self, position: &P, selection: &S) -> bool {
        G::IsLastPositionInEditableRoot(&self.gesture, position, selection)
    }
    fn SkipWhitespace(&self, position: &P) -> P {
        G::SkipWhitespace(&self.gesture, position)
    }
    fn DispatchSelectStart(&self, node: &EventTargetRef) -> NativeDispatchEventResult {
        G::DispatchSelectStart(&self.gesture, node)
    }
    fn InnerNode(&self, h: &H) -> Option<EventTargetRef> {
        self.gesture
            .dispatcher
            .targets
            .ResolveTarget(h.InnerNode()?.DOMNodeId()?)
    }
    fn HasActivationBehavior(&self, n: &EventTargetRef) -> bool {
        self.NodeFact(n, |d, i| {
            d.PseudoElementFacts(i)
                .is_some_and(|f| f.HasActivationBehavior())
        })
    }
    fn IsPseudoElement(&self, n: &EventTargetRef) -> bool {
        n.IsPseudoElement()
    }
    fn TextAreaEmptyPlaceholderBreakEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::TextAreaEmptyPlaceholderBreakEnabled()
    }
    fn EnclosingTextControlValueIsEmpty(&self, n: &EventTargetRef) -> Option<bool> {
        let id = self.gesture.NodeId(n)?;
        let owner = self.gesture.dispatcher.document.borrow();
        let d = owner.GetDocument();
        let mut i = d.FindNodeById(id)?;
        loop {
            if d.TextControlInnerEditorElement(i).is_some() {
                return Some(d.TextControlInnerEditorValue(i).is_empty());
            }
            i = d.Node(i).Parent().or_else(|| d.OwnerShadowHost(i))?;
        }
    }
    fn RootEditableHasChildren(&self, n: &EventTargetRef) -> Option<bool> {
        match self.Query(Q::RootEditableHasChildren(self.gesture.NodeId(n)?))? {
            V::OptionalBoolean(v) => v,
            _ => None,
        }
    }
    fn IsOverScrollbar(&self, h: &H) -> bool {
        h.GetScrollbar().is_some()
    }
    fn IsOverLink(&self, h: &H) -> bool {
        h.IsOverLink()
    }
    fn IsUserNodeDraggable(&self, h: &H) -> bool {
        h.InnerNode()
            .and_then(|n| n.DOMNodeId())
            .and_then(|id| self.Query(Q::NodeDraggable(id)))
            .is_some_and(|v| matches!(v, V::Boolean(true)))
    }
    fn HitTestResultIsMisspelled(&self, h: &H, page_default: bool) -> bool {
        let Some(p) = G::PositionOfHit(&self.gesture, h, false) else {
            return false;
        };
        let focused = self.gesture.dispatcher.state.borrow().focused_node_id;
        self.Query(Q::Misspelled {
            position: p.value,
            owner: self.services.MarkerOwner(),
            focused,
            page_default,
        })
        .is_some_and(|v| matches!(v, V::Boolean(true)))
    }
    fn SpellCheckMarkerGroupAtPosition(&self, p: &P) -> Option<(P, P)> {
        match self.Query(Q::MarkerGroup {
            position: p.value,
            owner: self.services.MarkerOwner(),
        })? {
            V::Bounds(Some((a, b))) => Some((self.gesture.Position(a)?, self.gesture.Position(b)?)),
            _ => None,
        }
    }
    fn AnnotationAtHit(
        &self,
        h: &H,
    ) -> Option<interaction_default::core::editing::selection_hit_test::MarkerAnnotationType> {
        if h.InnerNode().is_none() || h.InnerNodeFrame().is_none() {
            return None;
        }
        let p = G::PositionOfHit(&self.gesture, h, false)?;
        match self.Query(Q::Annotation {
            position: p.value,
            owner: self.services.MarkerOwner(),
            has_inner_node: true,
            has_frame: true,
        })? {
            V::Annotation(a) => a,
            _ => None,
        }
    }
    fn URLElement(&self, h: &H) -> Option<EventTargetRef> {
        self.gesture
            .dispatcher
            .targets
            .ResolveTarget(h.URLElement()?.DOMNodeId()?)
    }
    fn PositionIsDescendantOf(&self, p: &P, n: &EventTargetRef) -> bool {
        let Some(node) = self.gesture.NodeId(n) else {
            return false;
        };
        let owner = self.gesture.dispatcher.document.borrow();
        let d = owner.GetDocument();
        let (Some(mut child), Some(parent)) = (
            p.value.position.node_id.and_then(|id| d.FindNodeById(id)),
            d.FindNodeById(node),
        ) else {
            return false;
        };
        while let Some(next) = d.FlatTreeParent(child) {
            if next == parent {
                return true;
            }
            child = next;
        }
        false
    }
    fn ParentAnchoredPosition(&self, p: &P) -> Option<P> {
        self.PositionQuery(Q::ParentAnchored(p.value))
    }
    fn HasView(&self) -> bool {
        self.gesture
            .dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .is_some_and(|g| g.HasFrameView())
    }
    fn ConvertRootPointToContents(&self, p: (f64, f64)) -> (f64, f64) {
        use interaction_input::public::common::input::web_pointer_properties::PointF;
        let frame = self
            .gesture
            .dispatcher
            .mouse_frame_geometry
            .borrow()
            .clone();
        match frame.and_then(|g| g.ConvertFromRootFrame(PointF::new(p.0 as f32, p.1 as f32))) {
            Some(p) => (p.x() as f64, p.y() as f64),
            None => {
                self.Fault("LocalFrameView::ConvertFromRootFrame");
                p
            }
        }
    }
    fn PositionRespectingEditingBoundary(&self, h: &H) -> Option<P> {
        let selection = self.CurrentVisible();
        let focus = selection.Focus()?;
        let hit = G::PositionOfHit(&self.gesture, h, false)?;
        let behavior = self.gesture.settings.as_ref()?.editing_behavior;
        self.PositionQuery(Q::EditingBoundary {
            anchor: focus.value.position,
            target: h.InnerNode()?.DOMNodeId()?,
            hit: hit.value,
            local_point: *h.LocalPoint(),
            options: layoutng_assembly::EditingPositionOptions {
                move_caret_to_horizontal_boundary_when_past_top_or_bottom: behavior
                    .ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(),
                ..Default::default()
            },
        })
    }
    fn SelectionHasFocus(&self) -> bool {
        self.gesture.frame.PrepareEditing();
        let focused = self.gesture.dispatcher.state.borrow().focused_node_id;
        self.Query(Q::SelectionFocus {
            selection: self.gesture.frame.Current(),
            focused,
        })
        .is_some_and(|v| matches!(v, V::Boolean(true)))
    }
    fn PositionIsValid(&self, p: &P) -> bool {
        self.gesture
            .Query(
                crate::adapter::page_editing_visible_position::SelectionGeometryQuery::IsValid(
                    p.value,
                ),
            )
            .is_some_and(|v| {
                matches!(
                    v,
                    crate::adapter::page_editing_visible_position::SelectionGeometryResult::Boolean(
                        true
                    )
                )
            })
    }
    fn PositionIsEquivalent(&self, a: &P, b: &P) -> bool {
        self.Query(Q::Equivalent(a.value.position, b.value.position))
            .is_some_and(|v| matches!(v, V::Boolean(true)))
    }
    fn VisiblePosition(&self, p: &P) -> Option<P> {
        G::VisiblePositionOfPlain(&self.gesture, p)
    }
    fn WithAffinityOf(&self, s: &S, p: &P) -> S {
        let mut s = s.clone();
        if let Some(v) = s.value.as_mut() {
            v.affinity = p.value.affinity;
        }
        s
    }
    fn ExpandWithGranularity(&self, s: &S, g: TextGranularity) -> S {
        s.value
            .and_then(|selection| {
                self.Query(Q::Expand {
                    selection,
                    granularity: g,
                })
            })
            .and_then(|v| match v {
                V::Selection(s) => s,
                _ => None,
            })
            .map_or_else(
                || self.gesture.Selection(None),
                |s| self.gesture.Selection(Some(s)),
            )
    }
    fn EnclosingBlock(&self, n: &EventTargetRef) -> Option<EventTargetRef> {
        let V::Node(node) = self.Query(Q::EnclosingBlock(self.gesture.NodeId(n)?))? else {
            return None;
        };
        self.gesture.dispatcher.targets.ResolveTarget(node?)
    }
    fn FirstPositionInNode(&self, n: &EventTargetRef) -> P {
        self.PositionQuery(Q::FirstInNode(
            self.gesture.NodeId(n).expect("live source node"),
        ))
        .expect("source node has first position")
    }
    fn LastPositionInNode(&self, n: &EventTargetRef) -> P {
        self.PositionQuery(Q::LastInNode(
            self.gesture.NodeId(n).expect("live source node"),
        ))
        .expect("source node has last position")
    }
    fn PositionIterator(&self, p: &P) -> Box<dyn FlatPositionIterator<P> + '_> {
        let state = match self.Query(Q::IteratorStart(p.value)) {
            Some(V::Cursor(v)) => Some(v),
            _ => None,
        };
        Box::new(MousePositionIterator {
            owner: self,
            state: RefCell::new(state),
            fallback: p.clone(),
        })
    }
    fn StartRespectingGranularity(&self, p: &P, g: TextGranularity) -> Option<P> {
        self.PositionQuery(Q::Start(p.value, g))
    }
    fn EndRespectingGranularity(&self, start: &P, p: &P, g: TextGranularity) -> Option<P> {
        self.PositionQuery(Q::End(start.value, p.value, g))
    }
    fn PreviousPosition(&self, p: &P) -> Option<P> {
        self.PositionQuery(Q::Previous(p.value))
    }
    fn TextDistance(&self, a: &P, b: &P) -> usize {
        G::PlainText(&self.gesture, &G::Build(&self.gesture, a, b)).len()
    }
    fn AdjustAtBidiBoundary(&self, a: &P, b: &P) -> S {
        match self.Query(Q::AdjustBidi(a.value, b.value)) {
            Some(V::Selection(s)) => self.gesture.Selection(s),
            _ => self.gesture.Selection(None),
        }
    }
    fn HandlePotentialSuggestionTap(&self, p: &P) {
        self.Effect(self.services.HandlePotentialSuggestionTap(p.value))
    }
    fn HitTestAtContentsPoint(&self, p: (f64, f64), m: bool) -> H {
        use interaction_hit_test::core::layout::{
            hit_test_location::HitTestLocation, hit_test_request::RequestType,
            hit_test_result::HitTestResultRequest,
        };
        let location = HitTestLocation::FromPhysicalOffset(foundation::PhysicalOffset::new(
            foundation::LayoutUnit::FromDoubleRound(p.0),
            foundation::LayoutUnit::FromDoubleRound(p.1),
        ));
        let flags = RequestType::kReadOnly as u32
            | RequestType::kActive as u32
            | if m { RequestType::kMove as u32 } else { 0 };
        match self.geometry.HitTest(location.clone(), flags) {
            Ok(hit) => hit.result,
            Err(e) => {
                *self.error.borrow_mut() = Some(MouseSelectionError::Geometry(
                    super::gesture_selection::GestureSelectionError::Geometry(e),
                ));
                H::new(&HitTestResultRequest::new(flags, None, None), &location)
            }
        }
    }
}
struct MousePositionIterator<'a, 'scope, 'listener, 'frame> {
    owner: &'a PageMouseSelection<'scope, 'listener, 'frame>,
    state: RefCell<Option<MouseSelectionCursor>>,
    fallback: P,
}
impl MousePositionIterator<'_, '_, '_, '_> {
    fn Step(&self, forward: bool) {
        let cursor = self.state.borrow_mut().take();
        let next = cursor.and_then(|c| self.owner.Query(Q::IteratorStep(c.iterator, forward)));
        *self.state.borrow_mut() = match next {
            Some(V::Cursor(c)) => Some(c),
            _ => None,
        };
    }
}
impl FlatPositionIterator<P> for MousePositionIterator<'_, '_, '_, '_> {
    fn AtStart(&self) -> bool {
        self.state.borrow().as_ref().is_none_or(|c| c.at_start)
    }
    fn AtEnd(&self) -> bool {
        self.state.borrow().as_ref().is_none_or(|c| c.at_end)
    }
    fn Position(&self) -> P {
        self.state
            .borrow()
            .as_ref()
            .and_then(|c| c.position)
            .and_then(|p| self.owner.gesture.Position(p))
            .unwrap_or_else(|| self.fallback.clone())
    }
    fn Node(&self) -> Option<EventTargetRef> {
        self.owner
            .gesture
            .dispatcher
            .targets
            .ResolveTarget(self.state.borrow().as_ref()?.node?)
    }
    fn Decrement(&mut self) {
        self.Step(false)
    }
    fn Increment(&mut self) {
        self.Step(true)
    }
}
