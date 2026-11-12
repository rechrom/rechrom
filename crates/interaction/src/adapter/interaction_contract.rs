//! Rechrom's Page/DOM boundary values, independent of translated dispatch.
//!
//! These are adapters, not Chromium source objects. DOM retains its canonical
//! interaction state and mutations; callbacks carry those same values without
//! introducing a second document, state model, or dispatch implementation.

#![allow(non_snake_case)]

use dom::{
    dom_mutation::DOMMutation,
    persistent_document::{DOMNamespace, DOMNode},
    Document, DOM,
};
use std::{cell::RefCell, rc::Rc};

pub use dom::InteractionStateMutation;
pub type InteractionState = dom::UserInteractionState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractionResult {
    pub target_node_id: Option<u64>,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    /// Authoritative native input disposition, independent of DOM cancelation.
    pub input_result:
        Option<interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult>,
}

/// Immutable submission admission lease and original DOM identities. Browser
/// checks them when the scheduled task runs before it creates a new context.
#[derive(Clone, Debug)]
pub struct FormSubmissionScheduling {
    pub task: interaction_default::core::html::forms::html_form_element::FormSubmissionTaskToken,
    pub form_owner: dom::persistent_document::DOMOwnerHandle,
    pub document_token: [u64; 2],
}
/// Navigation decisions frozen before the synchronous formdata event. DOM
/// entry construction stays in Interaction; the browser receives only values.
#[derive(Clone, Debug)]
pub struct FormSubmissionParameters {
    pub scheduling: Option<FormSubmissionScheduling>,
    pub form: u64,
    pub submitter: Option<u64>,
    pub action: String,
    pub base_href: Option<String>,
    pub target: String,
    pub method: interaction_default::core::html::forms::html_form_element::SubmitMethod,
    pub enctype: String,
}

/// Synchronous FrameLoader preparation; this borrowed port can dispatch only
/// the current frame's lifecycle event and cannot escape the listener scope.
pub trait FormNavigationPreparation {
    fn Prepare(
        &self,
        target: &interaction_default::core::loader::frame_load_request::FrameNavigationTarget,
        events: &dyn interaction_default::core::loader::frame_navigation_lifecycle::FrameLifecycleEventDispatcher,
    ) -> bool;
}
#[derive(Clone, Debug)]
pub struct PreparedFormSubmission {
    pub token: crate::input_type::FormSubmissionToken,
    pub parameters: FormSubmissionParameters,
}

#[derive(Clone)]
pub enum InteractionEffect {
    MouseClickActivation {
        target_node_id: u64,
    },
    SignalCloseWatchers,
    NotifyHistoryUserActivation(interaction_input::base::time::time::TimeTicks),
    ConsumeHistoryUserActivation,
    CaptureMouseEventsToWidget(bool),
    PrepareFormSubmission(PreparedFormSubmission),
    SubmitForm {
        parameters: FormSubmissionParameters,
        entries: Rc<interaction_dispatch::core::fileapi::form_data::FormData>,
    },
    FormSubmissionError(crate::input_type::FormDataAdmission),
}

#[derive(Clone)]
pub enum InteractionOutput {
    DocumentMutation(DOMMutation),
    StateMutation(InteractionStateMutation),
    Effect(InteractionEffect),
}

/// Invocation is synchronous: DOM effects must finish before listener reentry.
pub type InteractionOutputEmitter = Rc<dyn Fn(InteractionOutput)>;

/// A retained owner, never a copied tree or a node address held across callbacks.
pub type InteractionDocument = Rc<RefCell<DOM>>;
pub type InteractionDOMMutationEmitter = Rc<dyn Fn(&DOMMutation)>;

/// Borrow the live node only for this read; the returned value cannot borrow it.
pub fn ReadNode<R>(
    document: &InteractionDocument,
    id: u64,
    read: impl FnOnce(&Document, &DOMNode, usize) -> R,
) -> Option<R> {
    let owner = document.borrow();
    let document = owner.GetDocument();
    let index = document.FindNodeById(id)?;
    Some(read(document, document.Node(index), index))
}

pub fn IsElement(node: &DOMNode, name: &str) -> bool {
    node.IsElement(name, DOMNamespace::kHTML)
}

pub fn Attribute(document: &InteractionDocument, id: u64, name: &str) -> String {
    ReadNode(document, id, |_, node, _| {
        node.FindAttribute(name)
            .map_or_else(String::new, |attribute| attribute.value.clone())
    })
    .unwrap_or_default()
}

pub fn HasAttribute(document: &InteractionDocument, id: u64, name: &str) -> bool {
    ReadNode(document, id, |_, node, _| {
        node.FindAttribute(name).is_some()
    })
    .unwrap_or(false)
}

/// Identifies the Element user-action transition, including its paint policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionPseudoState {
    Hover,
    Active,
}

/// Narrow, read-only queries supplied by the canonical DOM owner.
///
/// Interaction does not inspect ComputedStyle or reconstruct an ancestor path.
/// The DOM owner resolves identities and answers appearance queries using the
/// same elements affected by its user-action transition.
pub trait InteractionStateDocument {
    fn NodeExists(&self, node_id: u64) -> bool;

    /// Whether the Element pseudo-state transition needs native-theme paint.
    ///
    /// Chromium Element::SetHovered/SetActive always notify pseudo-state style
    /// invalidation separately. This query covers only their
    /// InvalidateIfHasEffectiveAppearance branch; Active excludes disabled form
    /// controls. It must not treat ordinary hover/active as blanket paint damage.
    ///
    /// Source: third_party/blink/renderer/core/dom/element.cc:13146-13193
    fn PseudoStateChangeNeedsPaint(&self, node_id: u64, state: InteractionPseudoState) -> bool;
}

/// DOM alone can validate identities. Native-theme paint additionally needs
/// the resident LayoutObject's appearance, supplied with `WithAppearance`.
/// A script-only document has no native appearance to invalidate.
impl InteractionStateDocument for Document {
    fn NodeExists(&self, node_id: u64) -> bool {
        self.FindNodeById(node_id).is_some()
    }

    fn PseudoStateChangeNeedsPaint(&self, _node_id: u64, _state: InteractionPseudoState) -> bool {
        false
    }
}

/// Borrowed admission facts; no Layout owner or callbacks into an engine cross
/// the Interaction package boundary. The predicate answers one native-appearance
/// fact from the caller's current resident LayoutObject.
pub struct InteractionStateDocumentWithAppearance<'a> {
    document: &'a Document,
    appearance: &'a dyn Fn(u64) -> bool,
}

impl<'a> InteractionStateDocumentWithAppearance<'a> {
    pub fn new(document: &'a Document, appearance: &'a dyn Fn(u64) -> bool) -> Self {
        Self {
            document,
            appearance,
        }
    }
}

impl InteractionStateDocument for InteractionStateDocumentWithAppearance<'_> {
    fn NodeExists(&self, node_id: u64) -> bool {
        self.document.FindNodeById(node_id).is_some()
    }

    fn PseudoStateChangeNeedsPaint(&self, node_id: u64, state: InteractionPseudoState) -> bool {
        let Some(node) = self.document.FindNodeById(node_id) else {
            return false;
        };
        if self.document.Node(node).Type() != dom::persistent_document::DOMNodeType::kElement {
            return false;
        }
        if state == InteractionPseudoState::Active && self.document.IsDisabledFormControl(node) {
            return false;
        }
        (self.appearance)(node_id)
    }
}

pub fn ValidateInteractionState(
    document: &(impl InteractionStateDocument + ?Sized),
    state: InteractionState,
) -> bool {
    [
        state.focused_node_id,
        state.focus_visible_node_id,
        state.hovered_node_id,
        state.pressed_node_id,
        state.multi_select_focused_option_id,
    ]
    .into_iter()
    .flatten()
    .all(|id| document.NodeExists(id))
}

/// Paint admission only. The DOM/Style lifecycle still receives every changed
/// pseudo-state, even when this function returns false.
pub fn InteractionStateNeedsPaint(
    document: &(impl InteractionStateDocument + ?Sized),
    old: InteractionState,
    new: InteractionState,
) -> bool {
    if old.focused_node_id != new.focused_node_id
        || old.focus_visible_node_id != new.focus_visible_node_id
        || old.multi_select_focused_option_id != new.multi_select_focused_option_id
    {
        return true;
    }

    let changed_pseudo_state_needs_paint = |old, new, state| {
        old != new
            && [old, new]
                .into_iter()
                .flatten()
                .any(|id| document.PseudoStateChangeNeedsPaint(id, state))
    };
    changed_pseudo_state_needs_paint(
        old.hovered_node_id,
        new.hovered_node_id,
        InteractionPseudoState::Hover,
    ) || changed_pseudo_state_needs_paint(
        old.pressed_node_id,
        new.pressed_node_id,
        InteractionPseudoState::Active,
    )
}
