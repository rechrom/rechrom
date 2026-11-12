//! Call-scoped Page ports for MouseEventManager::HandleMouseFocus.
//!
//! Chromium source: core/input/mouse_event_manager.cc:541-639 and
//! core/dom/element.cc:8954-8975,8994-9017.  DOM identity, style, selection
//! and focus remain in their canonical owners; this adapter retains no tree.

use super::{
    dom_event::DomEventTargets, event::EventTargetResolver,
    page_event_dispatcher::MouseFrameGeometryRef, page_focus_controller::FocusController,
};
use crate::{
    input_type::IsTextField,
    ownership::InteractionDocument,
    editing_owners::{SyntheticEventDispatcher, PageEditingOwners},
};
use dom::{DOMBoundaryPoint, Document};
use interaction_default::core::dom::{
    element::SelectionBehaviorOnFocus, focus_params::FocusTrigger,
};
use interaction_dispatch::core::{
    dom::events::event::EventTargetRef, events::ui_event::InputDeviceCapabilitiesRef,
    input::event_handling_util::WebInputEventResult,
};
use layoutng_assembly::{editing_state::SelectionState, EditingPositionAnchorType};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::core::input::mouse_event_manager::{
    MouseEventManager, MouseEventManagerState, MouseFocusFeatures, MouseFocusHitTest,
    MouseFocusPagePort, MouseFocusPort, MouseFocusPorts, MouseFocusSelectionPort,
};

struct PageMouseFocusLayout<'a>(Option<&'a dyn super::page_event_dispatcher::MouseFrameGeometry>);
impl super::page_element_focusability::PageElementFocusLayout for PageMouseFocusLayout<'_> {
    fn HasLayoutObject(&self, id: u64) -> bool {
        self.0.is_some_and(|frame| frame.HasLayoutObject(id))
    }
    fn IsUserScrollable(&self, id: u64) -> bool {
        self.0.is_some_and(|frame| frame.IsUserScrollable(id))
    }
    fn IsCanvasLayoutObject(&self, id: u64) -> bool {
        self.0.is_some_and(|frame| frame.IsCanvasLayoutObject(id))
    }
}

pub(crate) struct PageMouseFocus<'a> {
    document: InteractionDocument,
    targets: Rc<DomEventTargets>,
    state: Rc<RefCell<dom::UserInteractionState>>,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    focus: FocusController<'a>,
    frame: Option<MouseFrameGeometryRef>,
    focus_document_view: Rc<dyn Fn() + 'a>,
}

impl<'a> PageMouseFocus<'a> {
    pub(crate) fn new(
        document: InteractionDocument,
        targets: Rc<DomEventTargets>,
        state: Rc<RefCell<dom::UserInteractionState>>,
        editor: Rc<PageEditingOwners>,
        selections: Rc<SelectionState>,
        dispatch: SyntheticEventDispatcher<'a>,
        default_action_state: Rc<crate::input_type::DefaultActionState>,
        frame: Option<MouseFrameGeometryRef>,
        focus_services: Rc<super::page_focus_controller::DocumentFocusServices>,
        focus_document_view: Rc<dyn Fn() + 'a>,
        focus_dispatch: Rc<
            dyn interaction_default::core::page::focus_controller::FocusEventDispatch + 'a,
        >,
    ) -> Self {
        let focus = FocusController::new(
            document.clone(),
            state.clone(),
            editor.clone(),
            dispatch,
            default_action_state,
            focus_services,
        )
        .WithFocusDispatch(focus_dispatch);
        Self {
            document,
            targets,
            state,
            editor,
            selections,
            focus,
            frame,
            focus_document_view,
        }
    }

    fn IndexIn(&self, document: &Document, target: &EventTargetRef) -> Option<usize> {
        document.FindNodeById(self.targets.NodeId(target)?)
    }

    fn IsFocusableIndex(&self, document: &Document, index: usize, mouse: bool) -> bool {
        let layout = PageMouseFocusLayout(self.frame.as_deref());
        let focus = super::page_element_focusability::PageElementFocusability::new(
            document,
            &layout,
            self.state.borrow().focused_node_id,
        );
        if mouse {
            focus.IsMouseFocusable(index)
        } else {
            focus.IsFocusable(index)
        }
    }

    fn BoundaryPoint(
        document: &Document,
        position: layoutng_assembly::EditingPosition,
    ) -> Option<DOMBoundaryPoint> {
        let node_id = position.node_id?;
        let index = document.FindNodeById(node_id)?;
        match position.anchor {
            EditingPositionAnchorType::OffsetInAnchor => Some(DOMBoundaryPoint {
                node_id,
                offset: position.offset,
            }),
            EditingPositionAnchorType::BeforeAnchor | EditingPositionAnchorType::AfterAnchor => {
                let parent = document.Node(index).Parent()?;
                let child = document
                    .Node(parent)
                    .Children()
                    .iter()
                    .position(|&child| child == index)?;
                Some(DOMBoundaryPoint {
                    node_id: document.Node(parent).Id(),
                    offset: child
                        + usize::from(position.anchor == EditingPositionAnchorType::AfterAnchor),
                })
            }
            EditingPositionAnchorType::AfterChildren => Some(DOMBoundaryPoint {
                node_id,
                offset: document.Node(index).Children().len(),
            }),
        }
    }

    fn IsDescendant(document: &Document, node: usize, ancestor: usize) -> bool {
        let mut current = document.Node(node).Parent();
        while let Some(index) = current {
            if index == ancestor {
                return true;
            }
            current = document.Node(index).Parent();
        }
        false
    }

    fn FirstFocusableShadowDescendant(&self, document: &Document, root: usize) -> Option<usize> {
        let mut stack: Vec<_> = document
            .Node(root)
            .Children()
            .iter()
            .rev()
            .copied()
            .collect();
        while let Some(index) = stack.pop() {
            if self.IsFocusableIndex(document, index, false) {
                return Some(index);
            }
            stack.extend(document.Node(index).Children().iter().rev().copied());
        }
        None
    }
}

impl MouseFocusPagePort for PageMouseFocus<'_> {
    fn HasPage(&self) -> bool {
        self.frame.is_some()
    }
}

impl MouseFocusSelectionPort for PageMouseFocus<'_> {
    fn IsRange(&self) -> bool {
        self.editor.DocumentSelectionIsRange()
    }
    fn Clear(&self) {
        self.selections.Clear()
    }
}

impl MouseFocusFeatures for PageMouseFocus<'_> {
    fn PseudoElementsFocusableEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::PseudoElementsFocusableEnabled()
    }
}

impl MouseFocusPort for PageMouseFocus<'_> {
    fn IsMouseFocusable(&self, element: &EventTargetRef) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        self.IndexIn(document, element)
            .is_some_and(|index| self.IsFocusableIndex(document, index, true))
    }

    fn IsFocusedElementInDocument(&self, element: &EventTargetRef) -> bool {
        self.targets
            .NodeId(element)
            .is_some_and(|id| self.state.borrow().focused_node_id == Some(id))
    }

    fn IsShadowHostWithDelegatesFocus(&self, element: &EventTargetRef) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        self.IndexIn(document, element)
            .is_some_and(|index| document.ShadowRootDelegatesFocus(index))
    }

    fn FlatTreeParentElement(&self, element: &EventTargetRef) -> Option<EventTargetRef> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let parent = document.FlatTreeParentElement(self.IndexIn(document, element)?)?;
        self.targets
            .TargetFromBorrowedDocument(document, document.Node(parent).Id())
    }

    fn IsScrollControlPseudoElement(&self, element: &EventTargetRef) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        self.IndexIn(document, element).is_some_and(|index| {
            let layout = PageMouseFocusLayout(self.frame.as_deref());
            super::page_element_focusability::PageElementFocusability::new(
                document,
                &layout,
                self.state.borrow().focused_node_id,
            )
            .IsScrollControlPseudoElement(index)
        })
    }

    fn IsNodeFullyContainedInSelection(&self, element: &EventTargetRef) -> bool {
        let Some(selection) = self.selections.DOMSelection() else {
            return false;
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(element) = self.IndexIn(document, element) else {
            return false;
        };
        let Some(parent) = document.Node(element).Parent() else {
            return false;
        };
        let Some(child) = document
            .Node(parent)
            .Children()
            .iter()
            .position(|&child| child == element)
        else {
            return false;
        };
        let Some(anchor) = Self::BoundaryPoint(document, selection.anchor) else {
            return false;
        };
        let Some(focus) = Self::BoundaryPoint(document, selection.focus) else {
            return false;
        };
        let (start, end) =
            if document.CompareBoundaryPoints(anchor, focus).ok() == Some(Ordering::Greater) {
                (focus, anchor)
            } else {
                (anchor, focus)
            };
        let before = DOMBoundaryPoint {
            node_id: document.Node(parent).Id(),
            offset: child,
        };
        let after = DOMBoundaryPoint {
            node_id: document.Node(parent).Id(),
            offset: child + 1,
        };
        document
            .CompareBoundaryPoints(start, before)
            .is_ok_and(|order| order != Ordering::Greater)
            && document
                .CompareBoundaryPoints(after, end)
                .is_ok_and(|order| order != Ordering::Greater)
    }

    fn IsDescendantOfFocusedElement(&self, element: &EventTargetRef) -> bool {
        let Some(focused) = self.state.borrow().focused_node_id else {
            return false;
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        self.IndexIn(document, element)
            .zip(document.FindNodeById(focused))
            .is_some_and(|(element, focused)| Self::IsDescendant(document, element, focused))
    }

    fn GetFocusableArea(&self, element: &EventTargetRef) -> Option<EventTargetRef> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let host = self.IndexIn(document, element)?;
        let root = document.ShadowRoot(host)?;
        if let Some(focused) = self
            .state
            .borrow()
            .focused_node_id
            .and_then(|id| document.FindNodeById(id))
            .filter(|&focused| Self::IsDescendant(document, focused, root))
        {
            return self
                .targets
                .TargetFromBorrowedDocument(document, document.Node(focused).Id());
        }
        let target = self.FirstFocusableShadowDescendant(document, root)?;
        self.targets
            .TargetFromBorrowedDocument(document, document.Node(target).Id())
    }

    fn FocusDelegatedTargetWithMouseAndResetSelection(&self, element: &EventTargetRef) {
        if let Some(id) = self.targets.NodeId(element) {
            self.focus.SetFocusedElementWithMouse(
                Some(id),
                SelectionBehaviorOnFocus::kReset,
                None,
                FocusTrigger::kUserGesture,
            );
        }
    }

    fn IsTextControl(&self, element: &EventTargetRef) -> bool {
        self.targets
            .NodeId(element)
            .is_some_and(|id| IsTextField(&self.document, id))
    }

    fn SetFocusedElementWithMouse(
        &self,
        element: Option<&EventTargetRef>,
        source_capabilities: Option<&InputDeviceCapabilitiesRef>,
    ) -> bool {
        self.focus.SetFocusedElementWithMouse(
            element.and_then(|element| self.targets.NodeId(element)),
            SelectionBehaviorOnFocus::kNone,
            source_capabilities.cloned(),
            FocusTrigger::kScript,
        )
    }

    fn FocusDocumentView(&self) {
        (self.focus_document_view)();
    }

    // Source: mouse_event_manager.cc:724-727; document.cc:6232-6247.
    // Keep Chromium's live Range owner on PersistentDocument. The target is
    // resolved again under the mutation borrow because focus listeners may
    // have synchronously detached it before this stage of the transaction.
    fn SetSequentialFocusNavigationStartingPoint(&self, element: Option<&EventTargetRef>) {
        let id = element.and_then(|element| self.targets.NodeId(element));
        super::page_focus_controller::SetSequentialFocusNavigationStartingPoint(&self.document, id);
    }
}

pub(crate) struct PageMouseFocusHit {
    pub over_scrollbar: bool,
    pub frame_view_scrollbar: bool,
}
impl MouseFocusHitTest for PageMouseFocusHit {
    fn HasScrollbar(&self) -> bool {
        self.over_scrollbar
    }
    fn ScrollbarIsFrameViewScrollbar(&self) -> bool {
        self.frame_view_scrollbar
    }
}

pub(crate) fn HandleMouseFocus(
    mouse: &Rc<MouseEventManagerState>,
    ports: &PageMouseFocus<'_>,
    hit: &PageMouseFocusHit,
    capabilities: Option<&InputDeviceCapabilitiesRef>,
) -> WebInputEventResult {
    MouseEventManager::WithState(mouse.clone()).HandleMouseFocusWithPorts(
        &MouseFocusPorts {
            page: ports,
            focus: ports,
            selection: ports,
            features: ports,
        },
        hit,
        capabilities,
    )
}
