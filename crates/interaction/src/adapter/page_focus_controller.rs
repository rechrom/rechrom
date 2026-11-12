//! Live DOM/selection ports for the translated Document focus transaction.
//!
//! The document remains the focus-state owner. No facade borrow or DOM borrow
//! survives a focus listener; source Document::SetFocusedElement controls event
//! order and detects synchronous changes made by those listeners.

use crate::{
    event::{EventType, MakeSyntheticEvent},
    ownership::{InteractionDocument, ReadNode},
    editing_owners::{SyntheticEventDispatcher, PageEditingOwners},
};
use dom::UserInteractionState;
use interaction_default::core::editing::frame_selection::{RevealExtentOption, SelectionAlignment};
use interaction_default::core::editing::set_selection_options::SetSelectionOptions;
use interaction_default::{
    core::{
        dom::{
            document_focus::{self, DocumentFocusElement, DocumentFocusOwner},
            element::{self, ElementSelectionOnFocus, SelectionBehaviorOnFocus},
            focus_params::{FocusParams, FocusTrigger},
        },
        page::focus_controller::FocusDispatchScope,
    },
    public::mojom::input::focus_type::FocusType,
};
use layoutng_assembly::editing_state::DOMSelection;
use std::{cell::RefCell, rc::Rc};

pub use super::page_editing_visible_position::{
    BeginDisplayLockForcedSelectionForFocus, CaretBoundsForSelectionForFocus,
    CommitLayoutSelectionForFocus, CreateVisibleDOMSelectionForFocus,
    EnclosingTextControlForSelectionForFocus, EndDisplayLockForcedSelectionForFocus,
    ForcedSelectionLayoutScope, NotifyDisplayLockForSelectionChangeForFocus,
    SelectionAtFirstPositionInOrBeforeNodeForFocus, VisibleDOMSelectionCoversDocumentForFocus,
    VisibleDOMSelectionRootEditableElementForFocus, VisibleDOMSelectionStartAnchorForFocus,
};

/// Synchronous Document/LocalFrame effects outside the DOM transaction owner.
/// Implementations enter the existing lifecycle, FrameSelection and embedder
/// owners at each source call point. These operations must not be queued until
/// after focus/blur listeners, and none has a successful missing-owner default.
pub trait DocumentFocusLifecycle {
    fn PageFocusTarget(
        &self,
        _node: u64,
    ) -> Option<(
        Rc<interaction_default::core::page::focus_controller::FocusController>,
        interaction_default::core::page::focus_controller::FocusFrameRef,
        interaction_default::core::page::focus_controller::FocusElementRef,
    )> {
        None
    }

    fn StopClearFocusedElementTimer(&self);
    fn HasFrame(&self) -> bool;
    fn IsPageFocused(&self) -> bool;
    fn UpdateStyleAndLayoutTreeForElement(&self, node_id: u64);
    fn UpdateStyleAndLayoutForNode(&self, node_id: u64);
    fn UpdateStyleAndLayoutTree(&self);
    /// Inject native effects and borrowed geometry into Interaction's retained
    /// FrameSelection owner. Selection algorithms remain in Interaction.
    fn FrameSelectionHost(&self) -> Rc<dyn super::page_frame_selection::PageFrameSelectionHost>;
    fn IsLayoutEmbeddedContent(&self, node_id: u64) -> bool;
    /// Call the resident Layout scroll owner with partial horizontal scrolling
    /// disabled (NativeScrollSource::ScrollFocusIntoView(true)).
    fn ScrollBoundingBoxForFocusIntoView(&self, node_id: u64);
    fn UpdateFocusDataForSnapAreas(&self, update: FocusWithinSnapUpdate);
    fn NotifyFocusedElementChanged(
        &self,
        old_node_id: Option<u64>,
        new_node_id: Option<u64>,
        focus_type: FocusType,
    );
}

/// Element::SetHasFocusWithinUpToAncestor's two source traversals expressed as
/// stable DOM identities. `editing_fact_nodes` receive the new focus-within
/// input; `snap_container_candidates` continue past the common ancestor so an
/// unchanged outer snap container still refreshes its descendant focus data.
pub struct FocusWithinSnapUpdate {
    pub editing_fact_nodes: Vec<u64>,
    pub snap_container_candidates: Vec<u64>,
}

/// The same live Layout and document lifecycle owners are shared by every
/// focus entry point, including listener reentry and scoped keyboard dispatch.
pub struct DocumentFocusServices {
    pub layout: Rc<dyn super::page_element_focusability::PageElementFocusLayout>,
    pub lifecycle: Rc<dyn DocumentFocusLifecycle>,
    pub selection: Rc<super::page_frame_selection::PageFrameSelection>,
}

/// Store Chromium's mutation-aware sequential-focus Range on the canonical
/// Document owner. Callers pass an identity captured before listener reentry;
/// a detached identity is therefore rejected by the live lookup.
pub(crate) fn SetSequentialFocusNavigationStartingPoint(
    document: &InteractionDocument,
    node_id: Option<u64>,
) {
    let mut owner = document.borrow_mut();
    let document = owner.GetDocumentMut();
    let node = node_id.and_then(|id| document.FindNodeById(id));
    document.SetSequentialFocusNavigationStartingPoint(node);
}

pub(crate) fn StoreElementFocusedState(
    document: &InteractionDocument,
    state: &Rc<RefCell<UserInteractionState>>,
    id: u64,
    focused: bool,
) {
    {
        let mut owner = document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(index) = document.FindNodeById(id) {
            if document.IsElementFocused(index) != focused {
                document.StoreElementFocused(index, focused);
                document
                    .InvalidateFocusPseudo(index, dom::style_state::UserActionPseudoState::Focus);
                document.InvalidateFocusPseudo(
                    index,
                    dom::style_state::UserActionPseudoState::FocusVisible,
                );
            }
            // HTMLInputElement::SetFocused updates the group's retained slot
            // after HTMLElement has stored its focused user-action flag.
            if focused {
                document.UpdateRadioLastFocusedState(index);
            }
        }
    }
    if !focused {
        let mut state = state.borrow_mut();
        if state.focus_visible_node_id == Some(id) {
            state.focus_visible_node_id = None;
        }
    }
}

pub(crate) fn StoreFocusWithinUpToAncestor(
    document: &InteractionDocument,
    id: u64,
    focused: bool,
    stop_id: Option<u64>,
    search_snap_containers: bool,
) -> FocusWithinSnapUpdate {
    let mut owner = document.borrow_mut();
    let document = owner.GetDocumentMut();
    let mut current = document.FindNodeById(id);
    let mut reached_ancestor = false;
    let mut update = FocusWithinSnapUpdate {
        editing_fact_nodes: Vec::new(),
        snap_container_candidates: Vec::new(),
    };
    while let Some(index) = current.filter(|_| search_snap_containers || !reached_ancestor) {
        let current_id = document.Node(index).Id();
        if !reached_ancestor && Some(current_id) != stop_id {
            if document.HasElementFocusWithin(index) != focused {
                document.StoreElementFocusWithin(index, focused);
                document.InvalidateFocusPseudo(
                    index,
                    dom::style_state::UserActionPseudoState::FocusWithin,
                );
            }
            update.editing_fact_nodes.push(current_id);
        }
        if current_id != id && search_snap_containers {
            update.snap_container_candidates.push(current_id);
        }
        reached_ancestor |= Some(current_id) == stop_id;
        current = document.UserActionElementParent(index);
    }
    update
}

#[derive(Clone)]
pub struct FocusController<'a> {
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    editor: Rc<PageEditingOwners>,
    dispatch: Option<SyntheticEventDispatcher<'a>>,
    focus_dispatch:
        Option<Rc<dyn interaction_default::core::page::focus_controller::FocusEventDispatch + 'a>>,
    default_action_state: Rc<crate::input_type::DefaultActionState>,
    services: Rc<DocumentFocusServices>,
}

/// An identity into the existing document, carrying only its existing state
/// owner. The source transaction never needs a projected element/frame tree.
#[derive(Clone)]
pub struct FocusElementIdentity {
    id: u64,
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
}

struct ElementSelectionOnFocusView<'a, 'dispatch> {
    controller: &'a FocusController<'dispatch>,
    element: &'a FocusElementIdentity,
}

impl ElementSelectionOnFocus for ElementSelectionOnFocusView<'_, '_> {
    type Selection = Option<DOMSelection>;
    fn IsRootEditableElement(&self) -> bool {
        self.element.IsRootEditableElement()
    }
    fn HasFrame(&self) -> bool {
        self.controller.services.lifecycle.HasFrame()
    }
    fn VisibleSelectionRootEditableElementIsThis(&self) -> bool {
        self.controller
            .services
            .selection
            .VisibleSelectionRootEditableElement()
            == Some(self.element.id)
    }
    fn SelectionAtFirstPositionInOrBeforeNode(&self) -> Self::Selection {
        self.controller
            .services
            .selection
            .SelectionAtFirstPositionInOrBeforeNode(self.element.id)
    }
    fn RemoveVisibleSelectionInDOMSelectionEnabled(&self) -> bool {
        self.controller
            .services
            .selection
            .RemoveVisibleSelectionInDOMSelectionEnabled()
    }
    fn CreateVisibleSelection(&self, selection: Self::Selection) -> Self::Selection {
        self.controller
            .services
            .selection
            .CreateVisibleSelection(selection)
    }
    fn SetSelection(&self, selection: Self::Selection, options: SetSelectionOptions) {
        self.controller
            .services
            .selection
            .SetSelection(selection, options)
    }
    fn RevealSelection(&self) {
        self.controller.services.selection.RevealSelection(
            SelectionAlignment::CenterIfNeeded,
            RevealExtentOption::kDoNotRevealExtent,
        )
    }
    fn HasLayoutObject(&self) -> bool {
        self.controller
            .services
            .layout
            .HasLayoutObject(self.element.id)
    }
    fn IsLayoutEmbeddedContent(&self) -> bool {
        self.controller
            .services
            .lifecycle
            .IsLayoutEmbeddedContent(self.element.id)
    }
    fn ScrollBoundingBoxForFocusIntoView(&self) {
        self.controller
            .services
            .lifecycle
            .ScrollBoundingBoxForFocusIntoView(self.element.id)
    }
}
impl DocumentFocusElement for FocusElementIdentity {
    fn Same(&self, other: &Self) -> bool {
        self.id == other.id && Rc::ptr_eq(&self.state, &other.state)
    }
    fn IsRootEditableElement(&self) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        foundation::editable::IsRootEditableElement(
            &dom::persistent_document::DOMEditableFacts(document),
            self.id,
            foundation::editable::EditableFeatures::default(),
        )
    }
    fn SetFocused(&self, focused: bool, _: FocusType) {
        StoreElementFocusedState(&self.document, &self.state, self.id, focused)
    }
}

impl<'a> FocusController<'a> {
    pub fn new(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        editor: Rc<PageEditingOwners>,
        dispatch: SyntheticEventDispatcher<'a>,
        default_action_state: Rc<crate::input_type::DefaultActionState>,
        services: Rc<DocumentFocusServices>,
    ) -> Self {
        Self {
            document,
            state,
            editor,
            dispatch: Some(dispatch),
            focus_dispatch: None,
            default_action_state,
            services,
        }
    }
    pub(crate) fn new_scoped(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        editor: Rc<PageEditingOwners>,
        default_action_state: Rc<crate::input_type::DefaultActionState>,
        services: Rc<DocumentFocusServices>,
    ) -> Self {
        Self {
            document,
            state,
            editor,
            dispatch: None,
            focus_dispatch: None,
            default_action_state,
            services,
        }
    }
    pub(crate) fn WithFocusDispatch(
        mut self,
        dispatch: Rc<
            dyn interaction_default::core::page::focus_controller::FocusEventDispatch + 'a,
        >,
    ) -> Self {
        self.focus_dispatch = Some(dispatch);
        self
    }
    fn Element(&self, id: u64) -> FocusElementIdentity {
        FocusElementIdentity {
            id,
            document: self.document.clone(),
            state: self.state.clone(),
        }
    }
    fn NotifyDisplayLockFocusChanged(&self, element: &FocusElementIdentity, focused: bool) {
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(index) = document.FindNodeById(element.id) {
            document.NotifyDisplayLockFocusChanged(index, focused);
        }
    }
    fn EditContext(
        &self,
        element: &FocusElementIdentity,
    ) -> Option<Rc<dyn foundation::focus::ElementEditContextFocus>> {
        ReadNode(&self.document, element.id, |document, _, index| {
            document.ElementEditContext(index)
        })
        .flatten()
    }
    pub(crate) fn ValidationAnchorIsFocusable(&self, id: u64) -> bool {
        self.IsFocusable(&self.Element(id))
    }
    pub fn Focus(&self, id: u64, focus_visible: bool) {
        let params = FocusParams {
            r#type: if focus_visible {
                FocusType::kForward
            } else {
                FocusType::kMouse
            },
            ..FocusParams::default()
        };
        self.FocusWithParams(id, &params);
    }
    pub(crate) fn FocusWithScriptTrigger(&self, id: u64) {
        self.FocusWithParams(id, &FocusParams::WithTrigger(FocusTrigger::kScript));
    }
    pub(crate) fn FocusWithParams(&self, id: u64, params: &FocusParams) -> bool {
        let scope = FocusDispatchScope {
            focus_events: self.focus_dispatch.as_deref(),
            ..Default::default()
        };
        self.FocusWithParamsAndScope(id, params, &scope)
    }
    pub(crate) fn FocusWithParamsAndScope(
        &self,
        id: u64,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        let element = self.Element(id);
        // Element::Focus rejects ineligible requests before document focus.
        if !self.IsFocusable(&element) {
            return false;
        }
        if let Some((controller, frame, target)) = self.services.lifecycle.PageFocusTarget(id) {
            return controller.SetFocusedElementWithParamsAndScope(
                Some(target),
                Some(frame),
                params,
                scope,
            );
        }
        document_focus::SetFocusedElementWithScope(self, Some(element), params, scope)
    }
    pub(crate) fn SetFocusedElementWithScope(
        &self,
        id: Option<u64>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        document_focus::SetFocusedElementWithScope(
            self,
            id.map(|id| self.Element(id)),
            params,
            scope,
        )
    }
    pub(crate) fn ClearFocusedElementWithScope(&self, scope: &FocusDispatchScope<'_>) {
        document_focus::ClearFocusedElementWithScope(self, scope)
    }
    // Source: focus_controller.cc:1167-1235; mouse_event_manager.cc:617-638.
    pub(crate) fn SetFocusedElementWithMouse(
        &self,
        id: Option<u64>,
        selection_behavior: SelectionBehaviorOnFocus,
        source_capabilities: Option<
            interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef,
        >,
        focus_trigger: FocusTrigger,
    ) -> bool {
        let params = FocusParams {
            selection_behavior,
            r#type: FocusType::kMouse,
            source_capabilities,
            focus_trigger,
            ..FocusParams::default()
        };
        if let Some(id) = id {
            if let Some((controller, frame, target)) = self.services.lifecycle.PageFocusTarget(id) {
                let scope = FocusDispatchScope {
                    focus_events: self.focus_dispatch.as_deref(),
                    ..Default::default()
                };
                return controller.SetFocusedElementWithParamsAndScope(
                    Some(target),
                    Some(frame),
                    &params,
                    &scope,
                );
            }
        }
        document_focus::SetFocusedElement(self, id.map(|id| self.Element(id)), &params)
    }
    pub fn Blur(&self, id: u64) {
        if self.state.borrow().focused_node_id == Some(id) {
            document_focus::ClearFocusedElement(self);
        }
    }
}

impl DocumentFocusOwner<FocusElementIdentity> for FocusController<'_> {
    fn StopClearFocusedElementTimer(&self) {
        self.services.lifecycle.StopClearFocusedElementTimer()
    }
    fn FocusedElement(&self) -> Option<FocusElementIdentity> {
        self.state
            .borrow()
            .focused_node_id
            .map(|id| self.Element(id))
    }
    fn BelongsToDocument(&self, element: &FocusElementIdentity) -> bool {
        ReadNode(&self.document, element.id, |document, _, index| {
            document.NodeDocumentHandle(index) == document.RootHandle()
        })
        .unwrap_or(false)
    }
    fn IsBeingRemoved(&self, element: &FocusElementIdentity) -> bool {
        ReadNode(&self.document, element.id, |document, _, index| {
            document.IsBeingRemoved(index)
        })
        .unwrap_or(false)
    }
    fn StoreFocusedElement(
        &self,
        element: Option<FocusElementIdentity>,
        params: Option<&FocusParams>,
    ) {
        {
            let mut state = self.state.borrow_mut();
            let next = element.as_ref().map(|element| element.id);
            // The same-element fast path changes only Document.focus_options_.
            if state.focused_node_id != next {
                state.focused_node_id = next;
                state.focus_visible_node_id = element
                    .filter(|_| {
                        params.is_some_and(|params| {
                            matches!(params.r#type, FocusType::kForward | FocusType::kBackward)
                        })
                    })
                    .map(|element| element.id);
            }
        }
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .SetFocusOptions(params.map(|params| params.options.clone()));
    }
    fn CommonAncestor(
        &self,
        old: &FocusElementIdentity,
        new: &FocusElementIdentity,
    ) -> Option<FocusElementIdentity> {
        ReadNode(&self.document, old.id, |document, _, old_index| {
            if !document.IsConnectedToDocument(old_index) {
                return None;
            }
            let new_index = document.FindNodeById(new.id)?;
            document
                .UserActionElementCommonAncestor(old_index, new_index)
                .map(|index| self.Element(document.Node(index).Id()))
        })
        .flatten()
    }
    // Element's flags and ancestor relation stay on the canonical DOM owner.
    fn SetFocusWithin(
        &self,
        element: &FocusElementIdentity,
        focused: bool,
        ancestor: Option<&FocusElementIdentity>,
        search_snap_container: bool,
    ) {
        let snap_update = StoreFocusWithinUpToAncestor(
            &self.document,
            element.id,
            focused,
            ancestor.map(|ancestor| ancestor.id),
            search_snap_container,
        );
        if search_snap_container {
            self.services
                .lifecycle
                .UpdateFocusDataForSnapAreas(snap_update);
        }
    }
    fn ElementLostFocus(&self, element: &FocusElementIdentity) {
        self.NotifyDisplayLockFocusChanged(element, false);
        self.editor.FinishComposition(element.id);
    }
    fn ElementGainedFocus(&self, element: &FocusElementIdentity) {
        self.NotifyDisplayLockFocusChanged(element, true);
    }
    fn UpdateFocusgroupLastFocused(&self, element: &FocusElementIdentity) {
        if !self
            .document
            .borrow()
            .GetDocument()
            .FocusgroupFeatures()
            .enabled
        {
            return;
        }
        // GetFocusgroupOwnerOfItem rechecks Element::IsFocusable at this call
        // site, after SetFocused and display-lock notifications may reenter.
        let is_focusable = self.IsFocusable(element);
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(index) = document.FindNodeById(element.id) {
            document.UpdateFocusgroupLastFocused(index, is_focusable);
        }
    }
    fn IsPageFocused(&self) -> bool {
        self.services.lifecycle.IsPageFocused()
    }
    fn DispatchFocusEvent(
        &self,
        element: &FocusElementIdentity,
        kind: &'static str,
        related: Option<&FocusElementIdentity>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) {
        // HTMLSelectElement::DispatchBlurEvent, cc:1159, resets the
        // element's retained TypeAhead before author blur listeners run.
        if kind == "blur" {
            self.default_action_state
                .ResetSelectTypeAheadForBlur(element.id);
        }
        if let Some(events) = scope.focus_events {
            let document = self.document.borrow().GetDocument().RootHandle().Identity();
            events.DispatchElementFocusEventInDocument(
                document,
                element.id,
                kind,
                related.map(|element| element.id),
                params.r#type,
            );
            return;
        }
        let event_type = match kind {
            "blur" => EventType::kBlur,
            "focusout" => EventType::kFocusOut,
            "focus" => EventType::kFocus,
            "focusin" => EventType::kFocusIn,
            _ => EventType::kCustom,
        };
        let mut event = MakeSyntheticEvent(event_type, element.id);
        if event_type == EventType::kCustom {
            event.custom_type = kind.into();
            event.bubbles = true;
        }
        event.related_target_node_id = related.map(|element| element.id);
        self.dispatch
            .as_ref()
            .expect("focus transaction requires a call-scoped event dispatcher")(
            &mut event, element.id,
        );
    }
    fn BlurEditContext(&self, element: &FocusElementIdentity) {
        // Clone the actual EditContext owner before invoking it: Blur can
        // synchronously dispatch composition events and reenter this Document.
        if let Some(context) = self.EditContext(element) {
            context.Blur();
        }
    }
    fn FocusEditContext(&self, element: &FocusElementIdentity) {
        if let Some(context) = self.EditContext(element) {
            context.Focus();
        }
    }
    fn UpdateStyleAndLayoutTreeForElement(&self, element: &FocusElementIdentity) {
        self.services
            .lifecycle
            .UpdateStyleAndLayoutTreeForElement(element.id)
    }
    fn IsFocusable(&self, element: &FocusElementIdentity) -> bool {
        self.services
            .lifecycle
            .UpdateStyleAndLayoutForNode(element.id);
        ReadNode(&self.document, element.id, |document, _, index| {
            super::page_element_focusability::PageElementFocusability::new(
                document,
                self.services.layout.as_ref(),
                self.state.borrow().focused_node_id,
            )
            .IsFocusable(index)
        })
        .unwrap_or(false)
    }
    fn AcceptsEditingFocus(&self, element: &FocusElementIdentity) -> bool {
        // document.cc:742-746 checks only the actual frame and editable root;
        // this is not an embedder veto or a second editing-focus policy.
        self.services.lifecycle.HasFrame()
            && ReadNode(&self.document, element.id, |document, _, _| {
                foundation::editable::RootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(document),
                    element.id,
                    foundation::editable::EditableFeatures::default(),
                )
                .is_some()
            })
            .unwrap_or(false)
    }
    // Source: document.cc:6017-6030,6232-6247. This callback is deliberately
    // after StoreFocusedElement and before Element::SetFocused: temporary
    // clears and same-element early returns must not discard the live Range.
    fn DidStoreFocusedElement(&self, element: &FocusElementIdentity, params: &FocusParams) {
        SetSequentialFocusNavigationStartingPoint(&self.document, Some(element.id));
        let observers = {
            let mut owner = self.document.borrow_mut();
            let document = owner.GetDocumentMut();
            if !matches!(params.r#type, FocusType::kNone | FocusType::kScript) {
                document.SetLastFocusType(params.r#type);
            }
            document.FocusedElementChangeObservers()
        };
        for observer in observers {
            observer.DidChangeFocus();
        }
    }
    fn UpdateSelectionOnFocus(&self, element: &FocusElementIdentity, params: &FocusParams) {
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .SetShouldUpdateSelectionAfterLayout(false);
        self.services
            .lifecycle
            .UpdateStyleAndLayoutForNode(element.id);
        if super::page_editing_owners::IsTextControl(&self.document, element.id) {
            // TextControlElement supplies a subclass override. Its retained
            // editing owner remains isolated from Element's generic editable-
            // root and ordinary-element scroll algorithm below.
            if params.selection_behavior == SelectionBehaviorOnFocus::kNone {
                return;
            }
            self.editor.Focus(&self.document, element.id);
            return;
        }
        element::UpdateSelectionOnFocus(
            &ElementSelectionOnFocusView {
                controller: self,
                element,
            },
            params.selection_behavior,
            &params.options,
        );
    }
    fn NotifyFocusedElementChanged(
        &self,
        old: Option<&FocusElementIdentity>,
        new: Option<&FocusElementIdentity>,
        params: &FocusParams,
    ) {
        self.services.lifecycle.NotifyFocusedElementChanged(
            old.map(|element| element.id),
            new.map(|element| element.id),
            params.r#type,
        )
    }
    fn UpdateStyleAndLayoutTree(&self) {
        self.services.lifecycle.UpdateStyleAndLayoutTree()
    }
    fn SelectionDidChangeFocus(&self) {
        self.services.selection.DidChangeFocus();
    }
}
