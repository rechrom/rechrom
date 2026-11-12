//! Single-local-frame assembly of Chromium's retained sequential focus owner.
//!
//! Source: core/page/focus_controller.cc:2043-2062,2100-2274 and
//! core/dom/document.cc:6257-6351. Navigation runs in the translated
//! FocusController/ScopedFocusNavigation, over the original DOM and Layout.
//! No tree, ComputedStyle, or layout owner is copied into this adapter.
//!
//! Interaction must retain one assembly per Document/LocalFrame lifetime; a new
//! assembly on each key would lose FocusController's focused-frame state. Its
//! Rc views are stable during listener reentry, while the caller supplies the
//! current call-scoped FocusDispatchScope at each entry. Missing
//! element, focusgroup, pseudo-element and embedder operations are required
//! ports, rather than feature guesses or successful no-op default handlers.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use dom::{
    persistent_document::DOMShadowRootMode, DOMNamespace, DOMNodeType, Document,
    UserInteractionState,
};
use interaction_default::{
    core::{
        dom::focus_params::FocusParams,
        page::{
            focus_controller::{
                FocusController, FocusDispatchScope, FocusDocument, FocusDocumentRef,
                FocusEffectEmitter, FocusElement, FocusElementRef, FocusFrame, FocusFrameRef,
                FocusPage, FocusPageRef,
            },
            focus_navigation::{
                FocusNavigationFacts, FocusNavigationNode as Node, FocusNavigationTree,
                FocusgroupSegment,
            },
            focusgroup_controller_utils,
        },
    },
    public::mojom::input::focus_type::FocusType,
};
use interaction_dispatch::core::{
    dom::events::event::EventTargetRef, events::ui_event::InputDeviceCapabilitiesRef,
};
use layoutng_assembly::layout_engine::LayoutFocusNavigationFacts;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::{
    dom_event::DomEventTargets, event::EventTargetResolver,
    interaction_contract::InteractionDocument, page_focus_controller,
    page_input_type::DefaultActionState, page_editing_owners::PageEditingOwners,
};

#[path = "page_focusgroup.rs"]
mod focusgroup;
pub use focusgroup::QueryGridFocusgroup;
pub use interaction_default::core::page::grid_focusgroup_structure_info::{
    GridQuery as FocusgroupGridQuery, GridResult as FocusgroupGridResult,
};

/// Borrowed resident Layout queries at the synchronous focus lifecycle barrier.
/// The embedding retains its Layout owner; no engine or object crosses this port.
pub trait FocusNavigationLayout: super::page_element_focusability::PageElementFocusLayout {
    fn UpdateStyleAndLayoutForFocus(&self);
    fn Facts(&self, node_id: u64) -> Option<LayoutFocusNavigationFacts>;
    fn ReadingFlowNodes(&self, node_id: u64) -> Vec<u64>;
    fn GridFocusgroup(&self, root: u64, query: FocusgroupGridQuery)
        -> Option<FocusgroupGridResult>;
}

#[derive(Clone, Copy, Debug)]
pub struct FocusNavigationPolicy {
    pub tabs_to_links: bool,
    pub spatial_navigation_enabled: bool,
}

/// Element virtual results which are not owned by the navigation algorithm.
/// Focusability is queried separately at its source call sites: structural
/// Facts must not perform an extra IsFocusable/IsKeyboardFocusable calculation.
#[derive(Clone, Copy, Debug)]
pub struct FocusNavigationElementFacts {
    pub has_custom_focus_logic: bool,
    pub is_plugin: bool,
    pub is_scroll_marker_scope: bool,
    /// Element::GetIntegralAttribute's HTML integer parsing, owned by Element.
    pub integral_tab_index: Option<i32>,
}

/// Source owners absent from the DOM's structural navigation facts. Every
/// operation is required: an embedding with a disabled feature supplies its
/// actual disabled state here, rather than the adapter silently disabling it.
pub trait FocusNavigationSpecialOwners {
    fn KeyboardPolicy(&self) -> FocusNavigationPolicy;
    fn ElementFacts(&self, target: &EventTargetRef) -> FocusNavigationElementFacts;
    /// Includes the source scroller/focusgroup policy, not just tabindex.
    fn IsFocusable(&self, target: &EventTargetRef) -> bool;
    fn IsKeyboardFocusable(&self, target: &EventTargetRef) -> bool;
    fn ScrollMarkerOrigin(&self, node_id: u64) -> Option<u64>;
    fn ScrollMarkerFor(&self, node_id: u64) -> Option<u64>;
    fn FocusgroupSegment(&self, node_id: u64) -> Option<FocusgroupSegment>;
    fn FocusgroupSegmentEntry(&self, segment: FocusgroupSegment) -> Node;
    fn FocusedDirectionalKeyHandlerRoot(&self) -> Option<u64>;
    fn FocusgroupOwner(&self, node_id: u64) -> Option<u64>;
    /// An iframe element is not proof of an attached child frame. Resolve the
    /// actual frame owner, or return None when this single-Page domain has none.
    fn ContentFrame(&self, target: &EventTargetRef) -> Option<FocusFrameRef>;
    /// DOM's LayoutTreeBuilderTraversal includes pseudo-element siblings and
    /// skips display:contents/column pseudo ancestors for LayoutParent. It must
    /// use that DOM/pseudo owner, not substitute the LayoutObject tree.
    fn LayoutTreeBuilderLayoutParent(&self, node_id: u64) -> Option<u64>;
    fn LayoutTreeBuilderFirstChild(&self, node_id: u64) -> Option<u64>;
    fn LayoutTreeBuilderNextSibling(&self, node_id: u64) -> Option<u64>;
}

#[derive(Clone, Copy, Debug)]
pub struct FocusNavigationFrameState {
    pub has_view: bool,
    pub is_active: bool,
    pub unload_started: bool,
    pub is_page_paused: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum FocusNavigationFrameEffect {
    SelectionFrameIsFocused(bool),
    DidFocus,
    StopAutoscroll,
    WillChangeInputMethodFocus,
    SelectionPageActivationChanged,
}

#[derive(Clone, Copy, Debug)]
pub enum FocusNavigationElementEffect {
    HandleInterestForHoverOrFocus(bool),
    SetFocused {
        focused: bool,
        focus_type: FocusType,
    },
    SetHasFocusWithinUpToAncestor {
        focused: bool,
        need_snap_container_search: bool,
    },
}

/// Synchronous source focus actions. Scope carries the current listener
/// binding, so none of these callbacks retain a dispatcher borrowed from a
/// previous event. The implementation invokes the existing Document focus
/// transaction/Element::Focus; navigation never edits focused_node_id itself.
pub trait FocusNavigationActions {
    fn FrameState(&self) -> FocusNavigationFrameState;
    fn CanTakeFocus(&self, direction: FocusType) -> bool;
    fn TakeFocus(&self, direction: FocusType);
    fn Focus(&self, target: &EventTargetRef, params: &FocusParams, scope: &FocusDispatchScope<'_>);
    fn SetFocusedElement(
        &self,
        target: Option<&EventTargetRef>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool;
    fn ClearFocusedElement(&self, scope: &FocusDispatchScope<'_>);
    fn FrameEffect(&self, effect: FocusNavigationFrameEffect);
    fn StopAutoscroll(&self, _scope: &FocusDispatchScope<'_>) {
        self.FrameEffect(FocusNavigationFrameEffect::StopAutoscroll);
    }
    fn ElementEffect(&self, target: &EventTargetRef, effect: FocusNavigationElementEffect);
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>);
    fn DispatchElementEvent(
        &self,
        target: &EventTargetRef,
        event_type: &'static str,
        focus_type: FocusType,
        scope: &FocusDispatchScope<'_>,
    );
}

/// Frame/embedder effects which are outside DOM's focus transaction. Browser
/// projects only these typed host facts; focus traversal and DOM state remain
/// in Interaction.
/// Native FrameTree relations and the Page-owned focus controller. Only retained
/// Interaction identities cross this port; DOM nodes remain in their Document.
pub trait FocusNavigationFrameTree {
    fn PageIdentity(&self) -> Rc<()>;
    fn Frame(&self) -> Option<FocusFrameRef>;
    fn Root(&self) -> Option<FocusFrameRef>;
    fn Parent(&self) -> Option<FocusFrameRef>;
    fn Owner(&self) -> Option<FocusElementRef>;
    fn TraverseNext(&self) -> Option<FocusFrameRef>;
    fn ContentFrame(&self, owner_node: u64) -> Option<FocusFrameRef>;
    fn Controller(&self) -> Option<Rc<FocusController>>;
    fn DocumentEvents(&self, document: u128) -> Option<Rc<dyn FocusDocumentEventTarget>>;
}
/// A narrow retained destination for a call-scoped focus event. The listener
/// invoker is lent by the current script call, while the original destination
/// owns event paths, default actions and its Document identity.
pub trait FocusDocumentEventTarget {
    /// Borrow-free dispatcher over this original Document. Native selection
    /// focus creates it for the current synchronous call, never retaining an
    /// event-time Layout loan or a previous listener invocation.
    fn FocusEvents(
        &self,
    ) -> Rc<dyn interaction_default::core::page::focus_controller::FocusEventDispatch>;
    fn Dispatch(
        &self,
        event: &mut crate::event::Event,
        target: u64,
        listeners: Option<crate::event::EventListenerDispatcher<'_>>,
    );
}

pub trait FocusNavigationFramePort {
    fn FrameTree(&self) -> Option<Rc<dyn FocusNavigationFrameTree>> {
        None
    }

    fn HasView(&self) -> bool;
    fn IsPagePaused(&self) -> bool;
    fn CanTakeFocus(&self, direction: FocusType) -> bool;
    fn TakeFocus(&self, direction: FocusType);
    fn FrameEffect(&self, effect: FocusNavigationFrameEffect);
    fn StopAutoscroll(&self, _scope: &FocusDispatchScope<'_>) {
        self.FrameEffect(FocusNavigationFrameEffect::StopAutoscroll);
    }
    fn HandleInterestForHoverOrFocus(&self, node_id: u64, focused: bool);
    fn UpdateFocusDataForSnapAreas(&self, update: page_focus_controller::FocusWithinSnapUpdate);
}

/// Concrete Document transaction owner used by retained navigation. Each
/// method borrows the event dispatcher from FocusDispatchScope, so no listener
/// or per-event geometry survives the synchronous key transaction.
pub struct PageFocusNavigationActions {
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    targets: Rc<DomEventTargets>,
    editor: Rc<PageEditingOwners>,
    default_action_state: Rc<DefaultActionState>,
    focus_services: Rc<page_focus_controller::DocumentFocusServices>,
    layout: Rc<dyn FocusNavigationLayout>,
    frame: Rc<dyn FocusNavigationFramePort>,
}

impl PageFocusNavigationActions {
    pub fn new(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        targets: Rc<DomEventTargets>,
        editor: Rc<PageEditingOwners>,
        default_action_state: Rc<DefaultActionState>,
        layout: Rc<dyn FocusNavigationLayout>,
        frame: Rc<dyn FocusNavigationFramePort>,
        focus_services: Rc<page_focus_controller::DocumentFocusServices>,
    ) -> Rc<Self> {
        let owner = Rc::new(Self {
            document,
            state,
            targets,
            editor,
            default_action_state,
            focus_services,
            layout,
            frame,
        });
        owner.focus_services.selection.BindFocusActions(&owner);
        owner
    }

    /// FrameSelection::SetFocusedNodeIfNeeded, frame_selection.cc:1283-1313.
    /// Query the native visible selection and flat parent at each source call;
    /// release all owner borrows before the synchronous focus transaction.
    pub(crate) fn SetFocusedNodeForSelection(&self) {
        interaction_default::core::editing::selection_focus::SetFocusedNodeIfNeeded(self);
    }

    fn Controller(&self) -> page_focus_controller::FocusController<'static> {
        page_focus_controller::FocusController::new_scoped(
            self.document.clone(),
            self.state.clone(),
            self.editor.clone(),
            self.default_action_state.clone(),
            self.focus_services.clone(),
        )
    }

    fn NodeId(&self, target: &EventTargetRef) -> Option<u64> {
        self.targets.NodeId(target)
    }
    fn SelectionFocusEvents(
        &self,
    ) -> Rc<dyn interaction_default::core::page::focus_controller::FocusEventDispatch> {
        let identity = self.document.borrow().GetDocument().RootHandle().Identity();
        self.frame
            .FrameTree()
            .and_then(|tree| tree.DocumentEvents(identity))
            .expect("native selection focus has its live Document event owner")
            .FocusEvents()
    }
}

impl interaction_default::core::editing::selection_focus::FocusedNodeSelectionSource
    for PageFocusNavigationActions
{
    type Element = u64;
    fn UpdateStyleAndLayoutForSelection(&self) {
        self.focus_services.selection.PrepareEditing();
    }
    fn VisibleSelectionIsNone(&self) -> bool {
        let selection = &self.focus_services.selection;
        selection
            .CreateVisibleSelection(selection.Current())
            .is_none()
    }
    fn FrameIsFocused(&self) -> bool {
        self.focus_services.selection.FrameIsFocused()
    }
    fn VisibleSelectionRootEditableElement(&self) -> Option<u64> {
        self.focus_services
            .selection
            .VisibleSelectionRootEditableElement()
    }
    fn UpdateStyleAndLayoutTree(&self) {
        self.focus_services.lifecycle.UpdateStyleAndLayoutTree();
    }
    fn IsFocusable(&self, target: u64) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document.FindNodeById(target).is_some_and(|index| {
            super::page_element_focusability::PageElementFocusability::new(
                document,
                self.layout.as_ref(),
                self.state.borrow().focused_node_id,
            )
            .IsFocusable(index)
        })
    }
    fn IsFrameElement(&self, target: u64) -> bool {
        self.layout
            .Facts(target)
            .is_some_and(|facts| facts.is_embedded_content)
    }
    fn FlatTreeParentElement(&self, target: u64) -> Option<u64> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let parent = document.FlatTreeParentElement(document.FindNodeById(target)?)?;
        Some(document.Node(parent).Id())
    }
    fn SetFocusedElement(&self, target: u64) {
        let events = self.SelectionFocusEvents();
        let scope = FocusDispatchScope {
            focus_events: Some(events.as_ref()),
            ..Default::default()
        };
        if let Some((controller, frame, element)) =
            self.focus_services.lifecycle.PageFocusTarget(target)
        {
            controller.SetFocusedElementWithScope(Some(element), Some(frame), &scope);
        } else {
            use interaction_default::core::dom::element::SelectionBehaviorOnFocus;
            self.Controller().SetFocusedElementWithScope(
                Some(target),
                &FocusParams {
                    selection_behavior: SelectionBehaviorOnFocus::kNone,
                    ..Default::default()
                },
                &scope,
            );
        }
    }
    fn ClearFocusedElement(&self) {
        let events = self.SelectionFocusEvents();
        let scope = FocusDispatchScope {
            focus_events: Some(events.as_ref()),
            ..Default::default()
        };
        self.Controller().ClearFocusedElementWithScope(&scope);
    }
}

impl FocusNavigationActions for PageFocusNavigationActions {
    fn FrameState(&self) -> FocusNavigationFrameState {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        FocusNavigationFrameState {
            has_view: self.frame.HasView(),
            is_active: document.IsActive(),
            unload_started: document.UnloadStarted(),
            is_page_paused: self.frame.IsPagePaused(),
        }
    }

    fn CanTakeFocus(&self, direction: FocusType) -> bool {
        self.frame.CanTakeFocus(direction)
    }

    fn TakeFocus(&self, direction: FocusType) {
        self.frame.TakeFocus(direction)
    }

    fn Focus(&self, target: &EventTargetRef, params: &FocusParams, scope: &FocusDispatchScope<'_>) {
        if let Some(id) = self.NodeId(target) {
            let focusable = {
                let owner = self.document.borrow();
                let document = owner.GetDocument();
                let focused = self.state.borrow().focused_node_id;
                document.FindNodeById(id).is_some_and(|index| {
                    super::page_element_focusability::PageElementFocusability::new(
                        document,
                        self.layout.as_ref(),
                        focused,
                    )
                    .IsFocusable(index)
                })
            };
            if focusable {
                self.Controller().FocusWithParamsAndScope(id, params, scope);
            }
        }
    }

    fn SetFocusedElement(
        &self,
        target: Option<&EventTargetRef>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        self.Controller().SetFocusedElementWithScope(
            target.and_then(|target| self.NodeId(target)),
            params,
            scope,
        )
    }

    fn ClearFocusedElement(&self, scope: &FocusDispatchScope<'_>) {
        self.Controller().ClearFocusedElementWithScope(scope)
    }

    fn StopAutoscroll(&self, scope: &FocusDispatchScope<'_>) {
        self.frame.StopAutoscroll(scope);
    }
    fn FrameEffect(&self, effect: FocusNavigationFrameEffect) {
        match effect {
            FocusNavigationFrameEffect::SelectionFrameIsFocused(focused) => {
                self.focus_services.selection.SetFrameIsFocused(focused);
            }
            FocusNavigationFrameEffect::SelectionPageActivationChanged => {
                self.focus_services.selection.PageActivationChanged();
            }
            _ => {}
        }
        self.frame.FrameEffect(effect)
    }

    fn ElementEffect(&self, target: &EventTargetRef, effect: FocusNavigationElementEffect) {
        let Some(id) = self.NodeId(target) else {
            return;
        };
        match effect {
            FocusNavigationElementEffect::HandleInterestForHoverOrFocus(focused) => {
                self.frame.HandleInterestForHoverOrFocus(id, focused)
            }
            FocusNavigationElementEffect::SetFocused { focused, .. } => {
                page_focus_controller::StoreElementFocusedState(
                    &self.document,
                    &self.state,
                    id,
                    focused,
                )
            }
            FocusNavigationElementEffect::SetHasFocusWithinUpToAncestor {
                focused,
                need_snap_container_search,
            } => {
                let update = page_focus_controller::StoreFocusWithinUpToAncestor(
                    &self.document,
                    id,
                    focused,
                    None,
                    need_snap_container_search,
                );
                if need_snap_container_search {
                    self.frame.UpdateFocusDataForSnapAreas(update);
                }
            }
        }
    }

    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>) {
        let document = self.document.borrow().GetDocument().RootHandle().Identity();
        scope
            .focus_events
            .expect("focus navigation requires a call-scoped canonical dispatcher")
            .DispatchWindowFocusEventInDocument(document, event_type)
    }

    fn DispatchElementEvent(
        &self,
        target: &EventTargetRef,
        event_type: &'static str,
        focus_type: FocusType,
        scope: &FocusDispatchScope<'_>,
    ) {
        let Some(id) = self.NodeId(target) else {
            return;
        };
        let document = self.document.borrow().GetDocument().RootHandle().Identity();
        scope
            .focus_events
            .expect("focus navigation requires a call-scoped canonical dispatcher")
            .DispatchElementFocusEventInDocument(document, id, event_type, None, focus_type)
    }
}

/// DOM-owned virtuals used by the retained traversal. Only scalar Layout facts
/// cross the package boundary; focusgroup and pseudo identity stay canonical.
pub struct PageFocusNavigationSpecialOwners {
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    targets: Rc<DomEventTargets>,
    layout: Rc<dyn FocusNavigationLayout>,
    policy: FocusNavigationPolicy,
    frame_tree: Option<Rc<dyn FocusNavigationFrameTree>>,
}

impl PageFocusNavigationSpecialOwners {
    pub fn new(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        targets: Rc<DomEventTargets>,
        layout: Rc<dyn FocusNavigationLayout>,
        policy: FocusNavigationPolicy,
        frame_tree: Option<Rc<dyn FocusNavigationFrameTree>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            document,
            state,
            targets,
            layout,
            policy,
            frame_tree,
        })
    }

    fn NodeId(&self, target: &EventTargetRef) -> Option<u64> {
        self.targets.NodeId(target)
    }

    fn Read<T>(&self, node_id: u64, read: impl FnOnce(&Document, usize) -> T) -> Option<T> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        Some(read(document, document.FindNodeById(node_id)?))
    }

    fn Focusability<T>(
        &self,
        target: &EventTargetRef,
        read: impl FnOnce(&super::page_element_focusability::PageElementFocusability<'_>, usize) -> T,
    ) -> Option<T> {
        let id = self.NodeId(target)?;
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let index = document.FindNodeById(id)?;
        let focused = self.state.borrow().focused_node_id;
        let focusability = super::page_element_focusability::PageElementFocusability::new(
            document,
            self.layout.as_ref(),
            focused,
        );
        Some(read(&focusability, index))
    }

    fn Focusgroup<T>(&self, read: impl FnOnce(&focusgroup::PageFocusgroup<'_>) -> T) -> T {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        read(&focusgroup::PageFocusgroup::new(
            document,
            self.layout.as_ref(),
            self.state.borrow().focused_node_id,
            self.policy,
        ))
    }
}

impl FocusNavigationSpecialOwners for PageFocusNavigationSpecialOwners {
    fn KeyboardPolicy(&self) -> FocusNavigationPolicy {
        self.policy
    }
    fn ElementFacts(&self, target: &EventTargetRef) -> FocusNavigationElementFacts {
        self.NodeId(target)
            .and_then(|id| {
                self.Read(id, |document, index| {
                    let node = document.Node(index);
                    FocusNavigationElementFacts {
                        has_custom_focus_logic: document.InputType(index).is_some()
                            || node.IsHTMLElement("textarea"),
                        is_plugin: document.ContextMenuIsPluginElement(index),
                        is_scroll_marker_scope: document.PseudoElementFacts(index).is_some_and(
                            |facts| facts.pseudo_id == PseudoId::kPseudoIdScrollMarker,
                        ),
                        integral_tab_index: document.IntegralAttribute(index, "tabindex"),
                    }
                })
            })
            .unwrap_or(FocusNavigationElementFacts {
                has_custom_focus_logic: false,
                is_plugin: false,
                is_scroll_marker_scope: false,
                integral_tab_index: None,
            })
    }

    fn IsFocusable(&self, target: &EventTargetRef) -> bool {
        self.Focusability(target, |focusability, index| {
            focusability.IsFocusable(index)
        })
        .unwrap_or(false)
    }

    fn IsKeyboardFocusable(&self, target: &EventTargetRef) -> bool {
        self.Focusability(target, |focusability, index| {
            focusability.IsKeyboardFocusable(
                index,
                self.policy.tabs_to_links,
                self.policy.spatial_navigation_enabled,
            )
        })
        .unwrap_or(false)
    }

    fn ScrollMarkerOrigin(&self, node_id: u64) -> Option<u64> {
        self.Read(node_id, |document, index| {
            document
                .PseudoElementFacts(index)
                .filter(|facts| facts.pseudo_id == PseudoId::kPseudoIdScrollMarker)
                .map(|facts| document.Node(facts.originating_element).Id())
        })?
    }

    fn ScrollMarkerFor(&self, node_id: u64) -> Option<u64> {
        self.Read(node_id, |document, index| {
            document
                .GeneratedPseudoElement(index, PseudoId::kPseudoIdScrollMarker)
                .map(|marker| document.Node(marker).Id())
        })?
    }

    fn FocusgroupSegment(&self, node_id: u64) -> Option<FocusgroupSegment> {
        self.Focusgroup(|tree| focusgroup_controller_utils::Segment(tree, Node(node_id)))
    }

    fn FocusgroupSegmentEntry(&self, segment: FocusgroupSegment) -> Node {
        self.Focusgroup(|tree| {
            focusgroup_controller_utils::GetEntryElementForFocusgroupSegmentFromFirst(
                tree,
                segment.first,
                segment.owner,
            )
        })
    }

    fn FocusedDirectionalKeyHandlerRoot(&self) -> Option<u64> {
        self.Focusgroup(|tree| {
            focusgroup_controller_utils::GetDirectionalKeyHandlerRootForFocusedElement(tree)
                .map(|node| node.0)
        })
    }

    fn FocusgroupOwner(&self, node_id: u64) -> Option<u64> {
        self.Focusgroup(|tree| {
            focusgroup_controller_utils::GetFocusgroupOwnerOfItem(tree, Node(node_id))
                .map(|node| node.0)
        })
    }

    fn ContentFrame(&self, target: &EventTargetRef) -> Option<FocusFrameRef> {
        self.frame_tree.as_ref()?.ContentFrame(self.NodeId(target)?)
    }

    fn LayoutTreeBuilderLayoutParent(&self, node_id: u64) -> Option<u64> {
        self.Read(node_id, |document, index| {
            document
                .LayoutTreeBuilderParent(index)
                .map(|parent| document.Node(parent).Id())
        })?
    }

    fn LayoutTreeBuilderFirstChild(&self, node_id: u64) -> Option<u64> {
        self.Read(node_id, |document, index| {
            document
                .LayoutTreeBuilderFirstChild(index)
                .map(|child| document.Node(child).Id())
        })?
    }

    fn LayoutTreeBuilderNextSibling(&self, node_id: u64) -> Option<u64> {
        self.Read(node_id, |document, index| {
            document
                .LayoutTreeBuilderNextSibling(index)
                .map(|sibling| document.Node(sibling).Id())
        })?
    }
}

/// Retained assembly: construct once for this Document/LocalFrame lifetime.
/// Page/Frame/Document views share one Rc allocation, preserving the source
/// pointer comparisons in FocusController across repeated trait projections.
/// The embedding applies genuine activation/focus transitions to Controller;
/// construction never synthesizes a transition to initialize its state.
pub struct PageFocusNavigation {
    owner: Rc<PageFocusNavigationOwner>,
    controller: Rc<FocusController>,
}

impl PageFocusNavigation {
    pub(crate) fn HandleFocusgroupKeyboardEvent(
        &self,
        dispatcher: &super::page_event_dispatcher::EventDispatcher<'_, '_>,
        event: &interaction_dispatch::core::events::keyboard_event::KeyboardEventRef,
    ) -> bool {
        focusgroup::HandleKeyboardEvent(self, dispatcher, event)
    }
    pub fn new(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        targets: Rc<DomEventTargets>,
        layout: Rc<dyn FocusNavigationLayout>,
        special: Rc<dyn FocusNavigationSpecialOwners>,
        actions: Rc<dyn FocusNavigationActions>,
        effects: FocusEffectEmitter,
        frame_tree: Option<Rc<dyn FocusNavigationFrameTree>>,
    ) -> Self {
        assert!(
            Rc::ptr_eq(&document, &targets.DocumentOwner()),
            "focus navigation and dispatch must retain the same Document owner"
        );
        let owner = Rc::new_cyclic(|weak| PageFocusNavigationOwner {
            self_: weak.clone(),
            identity: Rc::new(()),
            document,
            state,
            targets,
            layout,
            special,
            actions,
            frame_tree: frame_tree.clone(),
            elements: RefCell::new(HashMap::new()),
            element_nodes: RefCell::new(HashMap::new()),
        });
        let controller = frame_tree
            .as_ref()
            .and_then(|tree| tree.Controller())
            .unwrap_or_else(|| Rc::new(FocusController::new(owner.clone(), effects)));
        Self { owner, controller }
    }

    pub fn AdvanceFocus(
        &self,
        direction: FocusType,
        capabilities: InputDeviceCapabilitiesRef,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        self.controller
            .AdvanceFocusWithScope(direction, false, Some(capabilities), scope)
    }

    pub(crate) fn DocumentEvents(
        &self,
        document: u128,
    ) -> Option<Rc<dyn FocusDocumentEventTarget>> {
        self.owner.frame_tree.as_ref()?.DocumentEvents(document)
    }
    pub fn ControllerOwner(&self) -> Rc<FocusController> {
        self.controller.clone()
    }
    pub fn Element(&self, node: u64) -> Option<FocusElementRef> {
        self.owner.ElementForNode(Node(node))
    }
    pub fn Controller(&self) -> &FocusController {
        &self.controller
    }
    pub(crate) fn AdjustedTabIndex(&self, node: u64) -> i32 {
        interaction_default::core::page::focus_navigation::AdjustedTabIndex(
            self.owner.as_ref(),
            Node(node),
        )
    }
    pub fn Page(&self) -> FocusPageRef {
        self.owner.clone()
    }
    pub fn Frame(&self) -> FocusFrameRef {
        self.owner
            .frame_tree
            .as_ref()
            .and_then(|tree| tree.Frame())
            .unwrap_or_else(|| self.owner.clone())
    }
    pub fn LocalFrameEvents(&self) -> FocusFrameRef {
        self.owner.clone()
    }
    pub fn Document(&self) -> FocusDocumentRef {
        self.owner.clone()
    }
    pub fn ActiveElementNode(&self) -> Option<u64> {
        self.owner
            .NodeForHandle(&FocusDocument::ActiveElement(self.owner.as_ref())?)
            .map(|node| node.0)
    }
    pub fn DocumentHasFocus(&self, page_active: bool, page_focused: bool) -> bool {
        let document: FocusDocumentRef = self.owner.clone();
        self.owner.document.borrow().GetDocument().IsActive()
            && self
                .controller
                .IsDocumentFocusedWithPageState(&document, page_active, page_focused)
    }
}

struct PageFocusNavigationOwner {
    self_: Weak<Self>,
    identity: Rc<()>,
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    targets: Rc<DomEventTargets>,
    layout: Rc<dyn FocusNavigationLayout>,
    special: Rc<dyn FocusNavigationSpecialOwners>,
    actions: Rc<dyn FocusNavigationActions>,
    frame_tree: Option<Rc<dyn FocusNavigationFrameTree>>,
    // Keep one wrapper per canonical DOM identity for this owner's lifetime.
    // Elements point back with Weak, so Rc::ptr_eq remains stable across key
    // transactions without creating an owner cycle.
    elements: RefCell<HashMap<u64, Rc<PageFocusNavigationElement>>>,
    element_nodes: RefCell<HashMap<usize, u64>>,
}

impl PageFocusNavigationOwner {
    fn Owner(&self) -> Rc<Self> {
        self.self_.upgrade().expect("live focus owner")
    }

    fn Read<T>(&self, node: Node, read: impl FnOnce(&Document, usize) -> T) -> Option<T> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        Some(read(document, document.FindNodeById(node.0)?))
    }

    fn Relation(
        &self,
        node: Node,
        read: impl FnOnce(&Document, usize) -> Option<usize>,
    ) -> Option<Node> {
        self.Read(node, |document, index| {
            read(document, index).map(|index| Node(document.Node(index).Id()))
        })?
    }

    fn ElementForNode(&self, node: Node) -> Option<FocusElementRef> {
        if let Some(element) = self.elements.borrow().get(&node.0).cloned() {
            return Some(element);
        }
        let target = self.Read(node, |document, index| {
            (document.Node(index).Type() == DOMNodeType::kElement)
                .then(|| self.targets.TargetFromBorrowedDocument(document, node.0))
                .flatten()
        })??;
        let element = Rc::new(PageFocusNavigationElement {
            owner: self.self_.clone(),
            target,
            node,
        });
        self.elements.borrow_mut().insert(node.0, element.clone());
        self.element_nodes
            .borrow_mut()
            .insert(Rc::as_ptr(&element) as usize, node.0);
        Some(element)
    }

    fn ElementHandle(&self, element: &FocusElementRef) -> Option<Rc<PageFocusNavigationElement>> {
        // Rc pointer identity is the contract of translated SameElement. No
        // downcast or numeric ID can accidentally admit another Document's node.
        let key = Rc::as_ptr(element) as *const () as usize;
        let id = *self.element_nodes.borrow().get(&key)?;
        let cached = self.elements.borrow().get(&id)?.clone();
        let view: FocusElementRef = cached.clone();
        Rc::ptr_eq(&view, element).then_some(cached)
    }

    fn NodeForHandle(&self, element: &FocusElementRef) -> Option<Node> {
        self.ElementHandle(element).map(|element| element.node)
    }

    fn TargetForHandle(&self, element: &FocusElementRef) -> EventTargetRef {
        self.ElementHandle(element)
            .expect("focus element belongs to this Document")
            .target
            .clone()
    }

    fn Focused(&self) -> Option<FocusElementRef> {
        let id = self.state.borrow().focused_node_id?;
        self.ElementForNode(Node(id))
    }

    fn ElementFacts(&self, node: Node) -> Option<FocusNavigationElementFacts> {
        Some(self.special.ElementFacts(&self.ElementTarget(node)?))
    }

    fn ElementTarget(&self, node: Node) -> Option<EventTargetRef> {
        if self.Read(node, |document, index| {
            document.Node(index).Type() == DOMNodeType::kElement
        }) != Some(true)
        {
            return None;
        }
        self.targets.ResolveTarget(node.0)
    }

    fn Sibling(&self, node: Node, forward: bool) -> Option<Node> {
        self.Relation(node, |document, index| {
            let parent = document.Node(index).Parent()?;
            let children = document.Node(parent).Children();
            let offset = children.iter().position(|&child| child == index)?;
            children
                .get(if forward {
                    offset.checked_add(1)?
                } else {
                    offset.checked_sub(1)?
                })
                .copied()
        })
    }
}

impl FocusPage for PageFocusNavigationOwner {
    fn Identity(&self) -> Rc<()> {
        self.frame_tree
            .as_ref()
            .map_or_else(|| self.identity.clone(), |tree| tree.PageIdentity())
    }
    fn MainFrame(&self) -> Option<FocusFrameRef> {
        self.frame_tree
            .as_ref()
            .and_then(|tree| tree.Root())
            .or_else(|| Some(self.Owner()))
    }
    // This assembly is the outermost local Page domain, never a fenced frame.
    fn IsMainFrameFencedFrameRoot(&self) -> bool {
        false
    }
    fn CanTakeFocus(&self, direction: FocusType) -> bool {
        self.actions.CanTakeFocus(direction)
    }
    fn TakeFocus(&self, direction: FocusType) {
        self.actions.TakeFocus(direction)
    }
}

impl FocusFrame for PageFocusNavigationOwner {
    fn PageIdentity(&self) -> Option<Rc<()>> {
        Some(FocusPage::Identity(self))
    }
    fn IsLocalFrame(&self) -> bool {
        true
    }
    fn IsAttached(&self) -> bool {
        self.targets.WindowDocument().IsAttached()
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.frame_tree
            .as_ref()
            .is_none_or(|tree| tree.Parent().is_none())
    }
    fn HasView(&self) -> bool {
        self.actions.FrameState().has_view
    }
    fn Owner(&self) -> Option<FocusElementRef> {
        self.frame_tree.as_ref()?.Owner()
    }
    fn Document(&self) -> Option<FocusDocumentRef> {
        Some(self.Owner())
    }
    fn LocalFrameRootDocument(&self) -> FocusDocumentRef {
        self.Top().Document().expect("attached local root Document")
    }
    fn Parent(&self) -> Option<FocusFrameRef> {
        self.frame_tree.as_ref()?.Parent()
    }
    fn Top(&self) -> FocusFrameRef {
        self.frame_tree
            .as_ref()
            .and_then(|tree| tree.Root())
            .unwrap_or_else(|| self.Owner())
    }
    fn LocalFrameRoot(&self) -> FocusFrameRef {
        self.Top()
    }
    fn AdvanceFocusFromRemote(&self, _: FocusType, _: &FocusFrameRef) {
        unreachable!("a single local Page frame cannot receive a remote-frame focus handoff")
    }
    fn TraverseNext(&self) -> Option<FocusFrameRef> {
        self.frame_tree.as_ref()?.TraverseNext()
    }
    fn IsDescendantOf(&self, frame: Option<&FocusFrameRef>) -> bool {
        let Some(frame) = frame else {
            return false;
        };
        let mut current: Option<FocusFrameRef> = Some(self.Owner());
        while let Some(candidate) = current {
            if Rc::ptr_eq(frame, &candidate) {
                return true;
            }
            current = candidate.Parent();
        }
        false
    }
    fn SetSelectionFrameIsFocused(&self, focused: bool) {
        self.actions
            .FrameEffect(FocusNavigationFrameEffect::SelectionFrameIsFocused(focused));
    }
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>) {
        self.actions.DispatchWindowEvent(event_type, scope)
    }
    fn DidFocus(&self) {
        self.actions
            .FrameEffect(FocusNavigationFrameEffect::DidFocus)
    }
    fn StopAutoscroll(&self) {
        self.actions
            .FrameEffect(FocusNavigationFrameEffect::StopAutoscroll)
    }
    fn StopAutoscrollWithScope(&self, scope: &FocusDispatchScope<'_>) {
        self.actions.StopAutoscroll(scope);
    }
    fn WillChangeInputMethodFocus(&self) {
        self.actions
            .FrameEffect(FocusNavigationFrameEffect::WillChangeInputMethodFocus)
    }
    fn SelectionPageActivationChanged(&self) {
        self.actions
            .FrameEffect(FocusNavigationFrameEffect::SelectionPageActivationChanged)
    }
}

impl FocusDocument for PageFocusNavigationOwner {
    fn FocusedElement(&self) -> Option<FocusElementRef> {
        self.Focused()
    }
    fn ActiveElement(&self) -> Option<FocusElementRef> {
        // TreeScope::AdjustedFocusedElement retargets a focused shadow descendant
        // to the host for Document.activeElement. Stay in the document's scope.
        let focused = self.Focused().or_else(|| {
            let tree = self.frame_tree.as_ref()?;
            tree.Controller()?.FocusedFrameOwnerElement(&tree.Frame()?)
        });
        if let Some(focused) = focused {
            let mut node = self.NodeForHandle(&focused)?;
            loop {
                let root = FocusNavigationTree::TreeScopeRoot(self, node);
                match FocusNavigationTree::ShadowHost(self, root) {
                    Some(host) => node = host,
                    None => return self.ElementForNode(node),
                }
            }
        }
        let node = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document
                .BodyElement()
                .or_else(|| document.DocumentElement())
                .map(|index| Node(document.Node(index).Id()))?
        };
        self.ElementForNode(node)
    }
    fn Frame(&self) -> Option<FocusFrameRef> {
        self.frame_tree
            .as_ref()
            .and_then(|tree| tree.Frame())
            .or_else(|| Some(self.Owner()))
    }
    fn ClearFocusedElement(&self, scope: &FocusDispatchScope<'_>) {
        self.actions.ClearFocusedElement(scope)
    }
    fn SetFocusedElement(
        &self,
        element: Option<FocusElementRef>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        let target = element
            .as_ref()
            .map(|element| self.TargetForHandle(element));
        self.actions
            .SetFocusedElement(target.as_ref(), params, scope)
    }
    fn IsActive(&self) -> bool {
        self.actions.FrameState().is_active
    }
    fn UnloadStarted(&self) -> bool {
        self.actions.FrameState().unload_started
    }
    fn HasView(&self) -> bool {
        self.actions.FrameState().has_view
    }
    fn IsPagePaused(&self) -> bool {
        self.actions.FrameState().is_page_paused
    }
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>) {
        self.actions.DispatchWindowEvent(event_type, scope)
    }
    fn FocusNavigation(&self) -> Rc<dyn FocusNavigationTree> {
        self.Owner()
    }
    fn SequentialFocusNavigationStartingPoint(&self, direction: FocusType) -> Option<Node> {
        let focused = self.state.borrow().focused_node_id;
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let focused = focused.and_then(|id| document.FindNodeById(id));
        document
            .SequentialFocusNavigationStartingPoint(direction == FocusType::kForward, focused)
            .map(|index| Node(document.Node(index).Id()))
    }
    fn ClearSequentialFocusNavigationStartingPoint(&self) {
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .SetSequentialFocusNavigationStartingPoint(None)
    }
    fn UpdateStyleAndLayoutForFocus(&self) {
        self.layout.UpdateStyleAndLayoutForFocus()
    }
}

impl FocusNavigationTree for PageFocusNavigationOwner {
    fn Root(&self) -> Node {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        Node(document.Node(document.Root()).Id())
    }
    fn Facts(&self, node: Node) -> FocusNavigationFacts {
        // Host queries happen outside the DOM borrow, including virtual element
        // focusability which can cross the embedding's lifecycle barrier.
        let layout = self.layout.Facts(node.0);
        let element = self.ElementFacts(node);
        self.Read(node, |document, index| {
            let value = document.Node(index);
            let style = document.ResolvedStyleFor(index);
            FocusNavigationFacts {
                is_element: value.Type() == DOMNodeType::kElement,
                is_dialog: value.IsHTMLElement("dialog"),
                is_autofocusable: matches!(
                    value.Namespace(),
                    DOMNamespace::kHTML | DOMNamespace::kSVG
                ) && value.FindAttribute("autofocus").is_some(),
                is_slot: value.IsHTMLElement("slot"),
                has_custom_focus_logic: element.is_some_and(|facts| facts.has_custom_focus_logic),
                has_author_shadow_root: document.ShadowRoot(index).is_some_and(|root| {
                    document.ShadowRootMode(root) != Some(DOMShadowRootMode::kUserAgent)
                }),
                delegates_focus: document.ShadowRootDelegatesFocus(index),
                integral_tab_index: element.and_then(|facts| facts.integral_tab_index),
                has_display_contents_style: style.is_some_and(|style| style.own_display_contents),
                is_iframe: value.IsHTMLElement("iframe"),
                reading_order: style
                    .and_then(|style| style.ComputedStyle())
                    .map_or(0, |style| style.ReadingOrder()),
                is_reading_flow_container: layout
                    .is_some_and(|facts| facts.is_reading_flow_container),
                is_scroll_marker_scope: element.is_some_and(|facts| facts.is_scroll_marker_scope),
                is_frame_owner: value.IsHTMLElement("iframe")
                    || value.IsHTMLElement("frame")
                    || value.IsHTMLElement("object")
                    || value.IsHTMLElement("embed")
                    || value.IsHTMLElement("fencedframe"),
                is_plugin: element.is_some_and(|facts| facts.is_plugin),
                is_pseudo: document.PseudoElementFacts(index).is_some(),
                has_layout_box: layout.is_some_and(|facts| facts.is_box),
            }
        })
        .expect("focus navigation node belongs to the live Document")
    }
    fn IsFocusable(&self, node: Node) -> bool {
        self.ElementTarget(node)
            .is_some_and(|target| self.special.IsFocusable(&target))
    }
    fn IsKeyboardFocusable(&self, node: Node) -> bool {
        self.ElementTarget(node)
            .is_some_and(|target| self.special.IsKeyboardFocusable(&target))
    }
    fn Parent(&self, node: Node) -> Option<Node> {
        self.Relation(node, |document, index| document.Node(index).Parent())
    }
    fn FlatTreeParent(&self, node: Node) -> Option<Node> {
        self.Relation(node, Document::FlatTreeParent)
    }
    fn LayoutParent(&self, node: Node) -> Option<Node> {
        self.special.LayoutTreeBuilderLayoutParent(node.0).map(Node)
    }
    fn LayoutTreeBuilderFirstChild(&self, node: Node) -> Option<Node> {
        self.special.LayoutTreeBuilderFirstChild(node.0).map(Node)
    }
    fn LayoutTreeBuilderNextSibling(&self, node: Node) -> Option<Node> {
        self.special.LayoutTreeBuilderNextSibling(node.0).map(Node)
    }
    fn FirstChild(&self, node: Node) -> Option<Node> {
        self.Relation(node, |document, index| {
            document.Node(index).Children().first().copied()
        })
    }
    fn LastChild(&self, node: Node) -> Option<Node> {
        self.Relation(node, |document, index| {
            document.Node(index).Children().last().copied()
        })
    }
    fn NextSibling(&self, node: Node) -> Option<Node> {
        self.Sibling(node, true)
    }
    fn PreviousSibling(&self, node: Node) -> Option<Node> {
        self.Sibling(node, false)
    }
    fn TreeScopeRoot(&self, node: Node) -> Node {
        self.Relation(node, |document, index| Some(document.TreeScopeRoot(index)))
            .expect("live TreeScope")
    }
    fn ShadowRoot(&self, host: Node) -> Option<Node> {
        self.Relation(host, Document::ShadowRoot)
    }
    fn ShadowHost(&self, root: Node) -> Option<Node> {
        self.Relation(root, Document::ShadowHost)
    }
    fn AssignedSlot(&self, node: Node) -> Option<Node> {
        self.Relation(node, Document::AssignedSlot)
    }
    fn HasAssignedNodes(&self, slot: Node) -> bool {
        self.Read(slot, Document::HasAssignedNodesNoRecalc)
            .unwrap_or(false)
    }
    fn OpenPopoverInvoker(&self, node: Node) -> Option<Node> {
        self.Relation(node, Document::OpenPopoverInvoker)
    }
    fn OpenPopoverTarget(&self, invoker: Node) -> Option<Node> {
        self.Relation(invoker, Document::OpenPopoverTarget)
    }
    fn LayoutReadingFlowNodes(&self, owner: Node) -> Vec<Node> {
        self.layout
            .ReadingFlowNodes(owner.0)
            .into_iter()
            .map(Node)
            .collect()
    }
    fn ScrollMarkerOrigin(&self, marker: Node) -> Option<Node> {
        self.special.ScrollMarkerOrigin(marker.0).map(Node)
    }
    fn ScrollMarkerFor(&self, node: Node) -> Option<Node> {
        self.special.ScrollMarkerFor(node.0).map(Node)
    }
    fn ContentFrame(&self, node: Node) -> Option<FocusFrameRef> {
        self.special
            .ContentFrame(&self.targets.ResolveTarget(node.0)?)
    }
    fn FrameOwner(&self) -> Option<Node> {
        None
    }
    fn FrameOwnerElement(&self) -> Option<FocusElementRef> {
        self.frame_tree.as_ref()?.Owner()
    }
    fn Element(&self, node: Node) -> Option<FocusElementRef> {
        self.ElementForNode(node)
    }
    fn NodeForElement(&self, element: &FocusElementRef) -> Option<Node> {
        self.NodeForHandle(element)
    }
    fn FocusgroupSegment(&self, node: Node) -> Option<FocusgroupSegment> {
        self.special.FocusgroupSegment(node.0)
    }
    fn FocusgroupSegmentEntry(&self, segment: FocusgroupSegment) -> Node {
        self.special.FocusgroupSegmentEntry(segment)
    }
    fn FocusedDirectionalKeyHandlerRoot(&self) -> Option<Node> {
        self.special.FocusedDirectionalKeyHandlerRoot().map(Node)
    }
    fn FocusgroupOwner(&self, node: Node) -> Option<Node> {
        self.special.FocusgroupOwner(node.0).map(Node)
    }
}

struct PageFocusNavigationElement {
    owner: Weak<PageFocusNavigationOwner>,
    target: EventTargetRef,
    node: Node,
}

impl PageFocusNavigationElement {
    fn Owner(&self) -> Rc<PageFocusNavigationOwner> {
        self.owner.upgrade().expect("live focus navigation owner")
    }
    fn Effect(&self, effect: FocusNavigationElementEffect) {
        self.Owner().actions.ElementEffect(&self.target, effect)
    }
    fn Dispatch(
        &self,
        event_type: &'static str,
        focus_type: FocusType,
        scope: &FocusDispatchScope<'_>,
    ) {
        self.Owner()
            .actions
            .DispatchElementEvent(&self.target, event_type, focus_type, scope)
    }
}

impl FocusElement for PageFocusNavigationElement {
    fn Document(&self) -> FocusDocumentRef {
        self.Owner()
    }
    fn IsFrameOwnerElement(&self) -> bool {
        self.Owner().Facts(self.node).is_frame_owner
    }
    fn IsFencedFrameElement(&self) -> bool {
        self.Owner()
            .Read(self.node, |document, index| {
                document.Node(index).IsHTMLElement("fencedframe")
            })
            .unwrap_or(false)
    }
    fn Focus(&self, params: &FocusParams, scope: &FocusDispatchScope<'_>) {
        self.Owner().actions.Focus(&self.target, params, scope)
    }
    fn ContentFrame(&self) -> Option<FocusFrameRef> {
        self.Owner().special.ContentFrame(&self.target)
    }
    fn IsPlugin(&self) -> bool {
        self.Owner().special.ElementFacts(&self.target).is_plugin
    }
    fn IsKeyboardFocusable(&self) -> bool {
        self.Owner().special.IsKeyboardFocusable(&self.target)
    }
    fn IsRootEditableElement(&self) -> bool {
        self.Owner()
            .Read(self.node, |document, _| {
                foundation::editable::IsRootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(document),
                    self.node.0,
                    foundation::editable::EditableFeatures::default(),
                )
            })
            .unwrap_or(false)
    }
    fn IsEditable(&self) -> bool {
        self.Owner()
            .Read(self.node, Document::IsEditableForEditing)
            .unwrap_or(false)
    }
    fn HasRootEditableElement(&self) -> bool {
        self.Owner()
            .Read(self.node, |document, index| {
                foundation::editable::RootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(document),
                    document.Node(index).Id(),
                    foundation::editable::EditableFeatures::default(),
                )
                .is_some()
            })
            .unwrap_or(false)
    }
    fn HandleInterestForHoverOrFocus(&self, focused: bool) {
        self.Effect(FocusNavigationElementEffect::HandleInterestForHoverOrFocus(
            focused,
        ))
    }
    fn SetFocused(&self, focused: bool, focus_type: FocusType) {
        self.Effect(FocusNavigationElementEffect::SetFocused {
            focused,
            focus_type,
        })
    }
    fn SetHasFocusWithinUpToAncestor(&self, focused: bool, need_snap_container_search: bool) {
        self.Effect(
            FocusNavigationElementEffect::SetHasFocusWithinUpToAncestor {
                focused,
                need_snap_container_search,
            },
        )
    }
    fn DispatchBlurEvent(&self, focus_type: FocusType, scope: &FocusDispatchScope<'_>) {
        self.Dispatch("blur", focus_type, scope)
    }
    fn DispatchFocusEvent(&self, focus_type: FocusType, scope: &FocusDispatchScope<'_>) {
        self.Dispatch("focus", focus_type, scope)
    }
    fn DispatchFocusOutEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>) {
        self.Dispatch(event_type, FocusType::kNone, scope)
    }
    fn DispatchFocusInEvent(
        &self,
        event_type: &'static str,
        focus_type: FocusType,
        scope: &FocusDispatchScope<'_>,
    ) {
        self.Dispatch(event_type, focus_type, scope)
    }
}
