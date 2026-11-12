//! Call-scoped SelectionController gesture ports over the canonical
//! FrameSelection and live native position queries.
use super::*;
use crate::adapter::{
    page_editing_visible_position::{
        CompositionTextQuery, CompositionTextResult, SelectionGeometryQuery as Query,
        SelectionGeometryResult as Value,
    },
    page_frame_selection::PageFrameSelection,
    page_pointer_hit_test::{InteractionTextSettings, PagePointerGeometry, PointerHitResult},
};
use interaction_default::core::editing::{
    gesture_selection::{
        GestureSelectionController, GestureSelectionGeometry, GestureSelectionOwner,
        GestureSelectionState,
    },
    selection_controller::{
        FlatSelection, PreparedSelectionGeometry, SelectionGeometryPreparation, SelectionPosition,
        SelectionState as SourceSelectionState,
    },
    set_selection_options::SetSelectionOptions,
    text_granularity::TextGranularity,
};
use layoutng_assembly::{
    editing_state::DOMSelection, EditingPosition, EditingPositionAnchorType,
    EditingPositionWithAffinity, EditingTextAffinity,
};
use std::{cmp::Ordering, ffi::CString};

#[derive(Clone)]
pub(crate) struct GesturePosition {
    pub(crate) value: EditingPositionWithAffinity,
    pub(crate) document: InteractionDocument,
    pub(crate) frame: Rc<PageFrameSelection>,
    pub(crate) locale: CString,
}
fn GeometryQuery(frame: &PageFrameSelection, locale: &CString, query: Query) -> Option<Value> {
    match frame.CompositionText(CompositionTextQuery::SelectionGeometry {
        query,
        ui_locale: locale.clone(),
    })? {
        CompositionTextResult::SelectionGeometry(value) => Some(value),
        _ => None,
    }
}
impl PartialEq for GesturePosition {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.document, &other.document) && self.value.position == other.value.position
    }
}
impl PartialOrd for GesturePosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !Rc::ptr_eq(&self.document, &other.document) {
            return None;
        }
        match GeometryQuery(
            &self.frame,
            &self.locale,
            Query::Compare(self.value.position, other.value.position),
        )? {
            Value::Compare(order) => order,
            _ => None,
        }
    }
}
impl SelectionPosition for GesturePosition {}
#[derive(Clone)]
pub(crate) struct GestureFlatSelection {
    pub(crate) value: Option<DOMSelection>,
    pub(crate) document: InteractionDocument,
    pub(crate) frame: Rc<PageFrameSelection>,
    pub(crate) locale: CString,
}
impl GestureFlatSelection {
    fn Position(&self, position: EditingPosition) -> GesturePosition {
        GesturePosition {
            value: EditingPositionWithAffinity {
                position,
                affinity: self
                    .value
                    .map_or(EditingTextAffinity::Downstream, |s| s.affinity),
            },
            document: self.document.clone(),
            frame: self.frame.clone(),
            locale: self.locale.clone(),
        }
    }
}
impl PartialEq for GestureFlatSelection {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.document, &other.document) && self.value == other.value
    }
}
impl FlatSelection for GestureFlatSelection {
    type Position = GesturePosition;
    fn IsNone(&self) -> bool {
        self.value.is_none()
    }
    fn IsRange(&self) -> bool {
        self.value.is_some_and(|s| s.anchor != s.focus)
    }
    fn IsCaret(&self) -> bool {
        self.value.is_some_and(|s| s.anchor == s.focus)
    }
    fn IsAnchorFirst(&self) -> bool {
        self.Anchor()
            .zip(self.Focus())
            .is_none_or(|(a, f)| a.partial_cmp(&f) != Some(Ordering::Greater))
    }
    fn Anchor(&self) -> Option<GesturePosition> {
        Some(self.Position(self.value?.anchor))
    }
    fn Focus(&self) -> Option<GesturePosition> {
        Some(self.Position(self.value?.focus))
    }
    fn Start(&self) -> Option<GesturePosition> {
        if self.IsAnchorFirst() {
            self.Anchor()
        } else {
            self.Focus()
        }
    }
    fn End(&self) -> Option<GesturePosition> {
        if self.IsAnchorFirst() {
            self.Focus()
        } else {
            self.Anchor()
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum GestureSelectionError {
    MissingTextSettings,
    UnavailableGeometry,
    Geometry(crate::adapter::page_pointer_hit_test::PointerGeometryError),
}

/// FrameSelection::Contains uses the existing SelectionEditor endpoints and
/// a second native read-only active hit test, exactly at the source query.
pub(crate) fn ContainsPoint(
    frame: Rc<PageFrameSelection>,
    document: InteractionDocument,
    geometry: Rc<dyn PagePointerGeometry>,
    settings: InteractionTextSettings,
    point: foundation::PhysicalOffset,
    has_layout_view: bool,
) -> Result<bool, GestureSelectionError> {
    use crate::adapter::page_frame_selection::FrameSelectionValue;
    use interaction_default::core::editing::frame_selection::SelectionContainmentSource;
    struct Source {
        frame: Rc<PageFrameSelection>,
        document: InteractionDocument,
        geometry: Rc<dyn PagePointerGeometry>,
        settings: InteractionTextSettings,
        has_layout_view: bool,
        error: RefCell<Option<GestureSelectionError>>,
    }
    impl Source {
        fn Position(&self, value: EditingPositionWithAffinity) -> GesturePosition {
            GesturePosition {
                value,
                document: self.document.clone(),
                frame: self.frame.clone(),
                locale: self.settings.ui_locale.clone(),
            }
        }
    }
    impl SelectionContainmentSource<FrameSelectionValue> for Source {
        type Position = GesturePosition;
        fn HasLayoutView(&self) -> bool {
            self.has_layout_view
        }
        fn ComputeVisibleSelectionInDOMTreeIsNonNull(
            &self,
            selection: &FrameSelectionValue,
        ) -> bool {
            self.frame.CreateVisibleSelection(selection.0).is_some()
        }
        fn VisibleFlatSelectionEndpoints(
            &self,
            selection: &FrameSelectionValue,
        ) -> Option<(GesturePosition, GesturePosition)> {
            let Value::Selection(selection) = GeometryQuery(
                &self.frame,
                &self.settings.ui_locale,
                Query::VisibleSelection {
                    selection: selection.0?,
                    word: false,
                },
            )?
            else {
                return None;
            };
            let selection = selection?;
            let a = self.Position(EditingPositionWithAffinity {
                position: selection.anchor,
                affinity: selection.affinity,
            });
            let f = self.Position(EditingPositionWithAffinity {
                position: selection.focus,
                affinity: selection.affinity,
            });
            Some(if a.partial_cmp(&f) == Some(Ordering::Greater) {
                (f, a)
            } else {
                (a, f)
            })
        }
        fn HitTestPositionReadOnlyActive(
            &self,
            point: &foundation::PhysicalOffset,
        ) -> Option<GesturePosition> {
            use interaction_hit_test::core::layout::{
                hit_test_location::HitTestLocation,
                hit_test_request::RequestType,
                hit_test_result::{HitTestPositionAnchor as A, HitTestTextAffinity as F},
            };
            let hit = match self.geometry.HitTest(
                HitTestLocation::FromPhysicalOffset(*point),
                RequestType::kReadOnly as u32 | RequestType::kActive as u32,
            ) {
                Ok(hit) => hit,
                Err(error) => {
                    *self.error.borrow_mut() = Some(GestureSelectionError::Geometry(error));
                    return None;
                }
            };
            let position = hit.result.GetPositionForInnerNodeOrImageMapImage()?.Value();
            if position.document != self.document.borrow().GetDocument().RootHandle().Identity() {
                return None;
            }
            Some(self.Position(EditingPositionWithAffinity {
                position: EditingPosition::WithAnchor(
                    position.node_id?,
                    position.offset,
                    match position.anchor {
                        A::OffsetInAnchor => EditingPositionAnchorType::OffsetInAnchor,
                        A::BeforeAnchor => EditingPositionAnchorType::BeforeAnchor,
                        A::AfterAnchor => EditingPositionAnchorType::AfterAnchor,
                        A::AfterChildren => EditingPositionAnchorType::AfterChildren,
                    },
                ),
                affinity: match position.affinity {
                    F::Downstream => EditingTextAffinity::Downstream,
                    F::Upstream => EditingTextAffinity::Upstream,
                },
            }))
        }
    }
    let source = Source {
        frame: frame.clone(),
        document,
        geometry,
        settings,
        has_layout_view,
        error: RefCell::new(None),
    };
    let contains = frame.ContainsWithSource(&source, &point);
    let error = source.error.borrow_mut().take();
    error.map_or(Ok(contains), Err)
}
pub(crate) struct PageGestureSelection<'a, 'listener, 'frame> {
    pub(crate) dispatcher: &'a EventDispatcher<'listener, 'frame>,
    pub(crate) frame: Rc<PageFrameSelection>,
    pub(crate) settings: Option<InteractionTextSettings>,
    pub(crate) state: Rc<GestureSelectionState<GesturePosition>>,
    pub(crate) error: RefCell<Option<GestureSelectionError>>,
}
impl<'a, 'listener, 'frame> PageGestureSelection<'a, 'listener, 'frame> {
    pub(crate) fn new(
        dispatcher: &'a EventDispatcher<'listener, 'frame>,
        geometry: Rc<dyn PagePointerGeometry>,
    ) -> Self {
        let state = {
            let mut slot = dispatcher
                .selection_controller_state
                .gesture_owner
                .borrow_mut();
            slot.get_or_insert_with(|| Rc::new(GestureSelectionState::<GesturePosition>::default()))
                .clone()
                .downcast::<GestureSelectionState<GesturePosition>>()
                .expect("same SelectionController position specialization")
        };
        Self {
            dispatcher,
            frame: dispatcher.focus_services.selection.clone(),
            settings: geometry.InteractionTextSettings(),
            state,
            error: RefCell::new(None),
        }
    }
    pub(crate) fn TakeError(&self) -> Option<GestureSelectionError> {
        self.error.borrow_mut().take()
    }
    pub(crate) fn Locale(&self) -> Option<&CString> {
        if self.settings.is_none() {
            *self.error.borrow_mut() = Some(GestureSelectionError::MissingTextSettings);
        }
        self.settings.as_ref().map(|settings| &settings.ui_locale)
    }
    pub(crate) fn Query(&self, query: Query) -> Option<Value> {
        // The outer Option describes the availability of the host's resident
        // query. Source null positions, selections, user-select-all bounds and
        // unordered positions are successful Value variants containing None.
        let value = GeometryQuery(&self.frame, self.Locale()?, query);
        if value.is_none() {
            *self.error.borrow_mut() = Some(GestureSelectionError::UnavailableGeometry);
        }
        value
    }
    pub(crate) fn Position(&self, value: EditingPositionWithAffinity) -> Option<GesturePosition> {
        Some(GesturePosition {
            value,
            document: self.dispatcher.document.clone(),
            frame: self.frame.clone(),
            locale: self.Locale()?.clone(),
        })
    }
    pub(crate) fn Selection(&self, value: Option<DOMSelection>) -> GestureFlatSelection {
        GestureFlatSelection {
            value,
            document: self.dispatcher.document.clone(),
            frame: self.frame.clone(),
            locale: self
                .settings
                .as_ref()
                .map(|s| s.ui_locale.clone())
                .unwrap_or_default(),
        }
    }
    pub(crate) fn PositionQuery(&self, query: Query) -> Option<GesturePosition> {
        match self.Query(query)? {
            Value::Position(value) => self.Position(value?),
            _ => None,
        }
    }
    pub(crate) fn NativeController(
        &self,
    ) -> GestureSelectionController<'_, GestureFlatSelection, PointerHitResult, Self, Self> {
        GestureSelectionController {
            owner: self,
            geometry: self,
            selection_state: &self.state.selection_state,
            original_anchor: &self.state.original_anchor,
            prepared: self,
            marker: std::marker::PhantomData,
        }
    }
    pub(crate) fn InitializeSelectionState(&self) {
        self.state
            .selection_state
            .set(SourceSelectionState::kHaveNotStartedSelection);
    }
    pub(crate) fn HandleGestureLongPress(&self, result: &PointerHitResult) -> bool {
        if self.Locale().is_none() {
            return false;
        }
        self.NativeController().HandleGestureLongPress(
            result,
            foundation::RuntimeEnabledFeatures::LongPressLinkSelectTextEnabled(),
        )
    }
    pub(crate) fn HandleGestureTwoFingerTap(&self, result: &PointerHitResult) {
        if self.Locale().is_some() {
            self.NativeController().SetCaretAtHitTestResult(result);
        }
    }
    pub(crate) fn NodeId(&self, node: &EventTargetRef) -> Option<u64> {
        let owner = self.dispatcher.document.borrow();
        (self.dispatcher.targets.OwnerDocumentHandle(node)
            == Some(owner.GetDocument().RootHandle()))
        .then(|| self.dispatcher.targets.NodeId(node))
        .flatten()
    }
    pub(crate) fn NodeFact(
        &self,
        node: &EventTargetRef,
        read: impl FnOnce(&dom::Document, usize) -> bool,
    ) -> bool {
        let Some(id) = self.NodeId(node) else {
            return false;
        };
        let owner = self.dispatcher.document.borrow();
        let d = owner.GetDocument();
        d.FindNodeById(id).is_some_and(|i| read(d, i))
    }
}
impl PreparedSelectionGeometry for PageGestureSelection<'_, '_, '_> {
    fn Prepare(&self, preparation: SelectionGeometryPreparation) {
        match preparation {
            SelectionGeometryPreparation::StyleAndLayoutTree => self
                .dispatcher
                .focus_services
                .lifecycle
                .UpdateStyleAndLayoutTree(),
            SelectionGeometryPreparation::StyleAndLayoutForSelection
            | SelectionGeometryPreparation::StyleAndLayoutForInput => self.frame.PrepareEditing(),
        }
    }
}
impl GestureSelectionOwner<GestureFlatSelection> for PageGestureSelection<'_, '_, '_> {
    fn IsAvailable(&self) -> bool {
        self.frame.IsAvailable()
    }
    fn Current(&self) -> GestureFlatSelection {
        self.Selection(self.frame.Current())
    }
    fn IsDirectional(&self) -> bool {
        self.frame.GestureSelectionFlags().1
    }
    fn IsHandleVisible(&self) -> bool {
        self.frame.GestureSelectionFlags().0
    }
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool {
        self.settings
            .as_ref()
            .is_some_and(|s| s.editing_behavior.ShouldConsiderSelectionAsDirectional())
    }
    fn SetSelection(&self, s: &GestureFlatSelection, options: SetSelectionOptions) {
        self.frame.SetSelection(s.value, options)
    }
    fn BindOriginalAnchorToCurrentContext(&self) {
        assert!(self
            .state
            .original_anchor
            .borrow()
            .as_ref()
            .is_none_or(|p| Rc::ptr_eq(&p.document, &self.dispatcher.document)));
    }
}
impl GestureSelectionGeometry<GestureFlatSelection, PointerHitResult>
    for PageGestureSelection<'_, '_, '_>
{
    fn InnerPossiblyPseudoNode(&self, h: &PointerHitResult) -> Option<EventTargetRef> {
        self.dispatcher
            .targets
            .ResolveTarget(h.InnerPossiblyPseudoNode()?.DOMNodeId()?)
    }
    fn IsLiveLink(&self, h: &PointerHitResult) -> bool {
        h.IsLiveLink()
    }
    fn HasLayoutObject(&self, n: &EventTargetRef) -> bool {
        self.NodeId(n)
            .is_some_and(|id| self.dispatcher.focus_services.layout.HasLayoutObject(id))
    }
    fn IsSelectable(&self, n: &EventTargetRef) -> bool {
        self.NodeId(n)
            .and_then(|n| self.Query(Query::NodeSelectable(n)))
            .is_some_and(|v| matches!(v, Value::Boolean(true)))
    }
    fn IsEditable(&self, n: &EventTargetRef) -> bool {
        self.NodeFact(n, |d, i| {
            foundation::editable::IsEditable(
                &dom::persistent_document::DOMEditableFacts(d),
                d.Node(i).Id(),
                Default::default(),
            )
        })
    }
    fn NodeIsText(&self, n: &EventTargetRef) -> bool {
        self.NodeFact(n, |d, i| d.Node(i).Type() == dom::DOMNodeType::kText)
    }
    fn CanStartSelection(&self, n: &EventTargetRef) -> bool {
        let Some(id) = self.NodeId(n) else {
            return false;
        };
        if self
            .Query(Query::NodeSelectionLocked(id))
            .is_some_and(|v| matches!(v, Value::Boolean(true)))
        {
            self.dispatcher
                .focus_services
                .lifecycle
                .UpdateStyleAndLayoutTreeForElement(id);
        }
        self.Query(Query::NodeCanStartSelection(id))
            .is_some_and(|v| matches!(v, Value::Boolean(true)))
    }
    fn IsConnected(&self, n: &EventTargetRef) -> bool {
        self.NodeFact(n, |d, i| d.IsConnectedToDocument(i))
    }
    fn HasImage(&self, h: &PointerHitResult) -> bool {
        h.GetImage().is_some()
    }
    fn PositionOfHit(
        &self,
        h: &PointerHitResult,
        zero_image_offset: bool,
    ) -> Option<GesturePosition> {
        // The source hit retains a weak resident query service. Invoke it with
        // no DOM/Layout loan, because the query may prepare the native frame.
        let adjusted = if zero_image_offset {
            let mut hit = h.clone();
            hit.SetNodeAndPosition(hit.InnerPossiblyPseudoNode().cloned(), Default::default());
            Some(hit)
        } else {
            None
        };
        let hit = adjusted.as_ref().unwrap_or(h);
        let p = hit.GetPosition()?.Value();
        if p.document
            != self
                .dispatcher
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity()
        {
            return None;
        }
        use interaction_hit_test::core::layout::hit_test_result::{
            HitTestPositionAnchor as A, HitTestTextAffinity as F,
        };
        let native = EditingPositionWithAffinity {
            position: EditingPosition::WithAnchor(
                p.node_id?,
                p.offset,
                match p.anchor {
                    A::OffsetInAnchor => EditingPositionAnchorType::OffsetInAnchor,
                    A::BeforeAnchor => EditingPositionAnchorType::BeforeAnchor,
                    A::AfterAnchor => EditingPositionAnchorType::AfterAnchor,
                    A::AfterChildren => EditingPositionAnchorType::AfterChildren,
                },
            ),
            affinity: match p.affinity {
                F::Downstream => EditingTextAffinity::Downstream,
                F::Upstream => EditingTextAffinity::Upstream,
            },
        };
        self.PositionQuery(Query::Visible(native))
    }
    fn FirstPositionInOrBefore(&self, n: &EventTargetRef) -> Option<GesturePosition> {
        self.PositionQuery(Query::FirstPosition(self.NodeId(n)?))
    }
    fn Empty(&self) -> GestureFlatSelection {
        self.Selection(None)
    }
    fn Collapse(&self, p: &GesturePosition) -> GestureFlatSelection {
        self.Build(p, p)
    }
    fn Build(&self, a: &GesturePosition, f: &GesturePosition) -> GestureFlatSelection {
        self.Selection(Some(DOMSelection {
            anchor: a.value.position,
            focus: f.value.position,
            affinity: a.value.affinity,
        }))
    }
    fn WithBaseAndExtent(
        &self,
        selection: &GestureFlatSelection,
        a: &GesturePosition,
        f: &GesturePosition,
    ) -> GestureFlatSelection {
        let mut result = self.Build(a, f);
        if let (Some(old), Some(new)) = (selection.value, result.value.as_mut()) {
            new.affinity = old.affinity;
        }
        result
    }
    fn ExpandWithGranularity(
        &self,
        s: &GestureFlatSelection,
        g: TextGranularity,
    ) -> GestureFlatSelection {
        assert!(
            matches!(g, TextGranularity::kWord),
            "gesture source expands word granularity"
        );
        s.value
            .and_then(|selection| {
                match self.Query(Query::VisibleSelection {
                    selection,
                    word: true,
                })? {
                    Value::Selection(s) => s,
                    _ => None,
                }
            })
            .map_or_else(|| self.Empty(), |s| self.Selection(Some(s)))
    }
    fn RootUserSelectAllBounds(
        &self,
        n: &EventTargetRef,
    ) -> Option<(GesturePosition, GesturePosition)> {
        match self.Query(Query::UserSelectAll(self.NodeId(n)?))? {
            Value::Bounds(Some((a, b))) => Some((self.Position(a)?, self.Position(b)?)),
            _ => None,
        }
    }
    fn PlainText(&self, s: &GestureFlatSelection) -> Vec<u16> {
        s.value
            .and_then(|s| self.Query(Query::PlainText(s)))
            .and_then(|v| {
                if let Value::Text(v) = v {
                    Some(v)
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
    fn ContainsOnlyWhitespaceAfterSimplifying(&self, text: &[u16]) -> bool {
        text.iter()
            .all(|ch| foundation::unicode::IsSpaceOrNewline(*ch))
    }
    fn IsLastPositionInEditableRoot(&self, p: &GesturePosition, s: &GestureFlatSelection) -> bool {
        s.value
            .and_then(|selection| {
                self.Query(Query::LastPositionInEditableRoot {
                    position: p.value.position,
                    selection,
                })
            })
            .is_some_and(|v| matches!(v, Value::Boolean(true)))
    }
    fn SkipWhitespace(&self, p: &GesturePosition) -> GesturePosition {
        self.PositionQuery(Query::SkipWhitespace(p.value))
            .unwrap_or_else(|| p.clone())
    }
    fn ResetInvalidSelectionAfterEvent(&self, s: &GestureFlatSelection) -> GestureFlatSelection {
        if [s.Anchor(), s.Focus()].iter().flatten().all(|p| {
            self.Query(Query::IsValid(p.value))
                .is_some_and(|v| matches!(v, Value::Boolean(true)))
        }) {
            s.clone()
        } else {
            self.Empty()
        }
    }
    fn VisibleSelection(&self, s: &GestureFlatSelection) -> GestureFlatSelection {
        s.value
            .and_then(|selection| {
                match self.Query(Query::VisibleSelection {
                    selection,
                    word: false,
                })? {
                    Value::Selection(s) => s,
                    _ => None,
                }
            })
            .map_or_else(|| self.Empty(), |s| self.Selection(Some(s)))
    }
    fn PositionIsConnected(&self, p: &GesturePosition) -> bool {
        p.value.position.node_id.is_some_and(|id| {
            let owner = self.dispatcher.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(id)
                .is_some_and(|i| d.IsConnectedToDocument(i))
        })
    }
    fn VisiblePositionOfPlain(&self, p: &GesturePosition) -> Option<GesturePosition> {
        self.PositionQuery(Query::Visible(p.value))
    }
    fn DispatchSelectStart(&self, n: &EventTargetRef) -> NativeDispatchEventResult {
        let event = interaction_dispatch::core::dom::events::event::Event::CreateCancelableBubble(
            "selectstart",
            self.dispatcher.clock.as_ref(),
        );
        self.dispatcher
            .DispatchCanonicalEventTarget(n, EventRef::new(event))
            .0
    }
}
