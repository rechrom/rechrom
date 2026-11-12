//! Page-facing default-action adapters over the live DOM and source owners.
#[path = "page_anchor_default.rs"]
mod anchor;

use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    input_event::MouseButton,
    input_type::{
        ActivationState, CreateInputType, InputType, InputTypeContext, IsDisabledFormControl,
    },
    ownership::{IsElement, ReadNode},
};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use interaction_default::{
    core::dom::{
        element::{self, ClickableControlOperations},
        node::Node,
    },
    core::html::forms::{
        html_label_element::{
            HTMLLabelElement, LabelClickValues, LabelDefaultOperations, PreparedLabelSelection,
        },
        text_control_inner_elements::{
            SearchCancelInputOperations, SearchFieldCancelButtonElement,
        },
    },
    core::html::html_summary_element::HTMLSummaryElement,
    public::mojom::input::focus_type::FocusType,
    public::web::web_autofill_state::WebAutofillState,
};

struct SearchCancelHost<'a, 'b> {
    context: &'a InputTypeContext<'b>,
    host: u64,
}
impl SearchCancelInputOperations for SearchCancelHost<'_, '_> {
    fn IsDisabledOrReadOnly(&self) -> bool {
        IsDisabledFormControl(&self.context.document, self.host)
            || crate::ownership::HasAttribute(&self.context.document, self.host, "readonly")
    }
    fn SetValueForUser(&self, value: &[u16]) {
        let value = String::from_utf16_lossy(value);
        let changed = ReadNode(&self.context.document, self.host, |document, _, index| {
            document.ControlValue(index) != value
        })
        .unwrap_or(false);
        (self.context.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetControlValue,
            target_node_id: self.host,
            value,
            ..Default::default()
        });
        if changed {
            let mut change = MakeSyntheticEvent(EventType::kChange, self.host);
            (self.context.dispatch_synthetic)(&mut change, self.host);
        }
    }
    fn SetAutofillState(&self, state: WebAutofillState) {
        (self.context.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetControlAutofillState,
            target_node_id: self.host,
            autofill_state: match state {
                WebAutofillState::kNotFilled => dom::DOMAutofillState::kNotFilled,
                WebAutofillState::kPreviewed => dom::DOMAutofillState::kPreviewed,
                WebAutofillState::kAutofilled => dom::DOMAutofillState::kAutofilled,
            },
            ..Default::default()
        });
    }
    fn OnSearch(&self) {
        let mut search = MakeSyntheticEvent(EventType::kCustom, self.host);
        search.custom_type = "search".into();
        search.cancelable = false;
        search.composed = false;
        (self.context.dispatch_synthetic)(&mut search, self.host);
    }
}

// cpp: interaction/default_event_handler.h:16-19
#[derive(Default)]
pub struct DefaultActivationState<'a> {
    pub input_type: Option<Box<dyn InputType + 'a>>,
    pub input_state: Option<Box<dyn ActivationState>>,
}
// cpp: interaction/default_event_handler.h:24-44
// cpp: interaction/default_event_handler.cc:40-54
pub struct DefaultEventHandler<'a> {
    context: InputTypeContext<'a>,
}
impl<'a> DefaultEventHandler<'a> {
    pub fn new(context: InputTypeContext<'a>) -> Self {
        Self { context }
    }
    // cpp: interaction/default_event_handler.cc:56-65
    fn TypeFor(&self, node: u64) -> Box<dyn InputType + 'a> {
        CreateInputType(node, self.context.clone())
    }
    // cpp: interaction/default_event_handler.cc:11-32,67-72
    fn LabelControl(&self, label: u64) -> Option<u64> {
        ReadNode(&self.context.document, label, |document, _, index| {
            document
                .LabelControl(index)
                .map(|control| document.Node(control).Id())
        })
        .flatten()
    }
    fn Element(&self, id: u64, name: &str) -> bool {
        ReadNode(&self.context.document, id, |_, n, _| IsElement(n, name)).unwrap_or(false)
    }
    fn MainDetailsForSummary(&self, id: u64) -> Option<(u64, bool)> {
        ReadNode(&self.context.document, id, |document, node, index| {
            if !document.IsDetailsMainSummary(index) {
                return None;
            }
            let details = node
                .Parent()
                .filter(|&index| document.Node(index).IsHTMLElement("details"))
                .or_else(|| document.OwnerShadowHost(index))?;
            Some((
                document.Node(details).Id(),
                document.Node(details).FindAttribute("open").is_some(),
            ))
        })
        .flatten()
    }
    fn ClickTargetIsControlInsideSummary(&self, target: u64, summary: u64) -> bool {
        element::IsClickableControlWithOperations(&summary, Some(target), self)
    }
    fn SearchCancelButtonHost(&self, id: u64) -> Option<u64> {
        ReadNode(&self.context.document, id, |document, _, index| {
            document
                .SearchCancelButtonHost(index)
                .map(|host| document.Node(host).Id())
        })
        .flatten()
    }
    // cpp: interaction/default_event_handler.cc:74-78
    pub fn HasActivationBehavior(&self, node: u64) -> bool {
        if IsDisabledFormControl(&self.context.document, node) {
            return false;
        }
        if anchor::IsLink(&self.context, node) {
            return true;
        }
        if self.Element(node, "label") {
            return self.LabelControl(node).is_some();
        }
        if self.MainDetailsForSummary(node).is_some() {
            return true;
        }
        self.TypeFor(node).HasActivationBehavior()
    }
    // cpp: interaction/default_event_handler.cc:81-90
    pub fn LegacyPreActivationBehavior(
        &self,
        node: u64,
        event: &mut Event,
    ) -> Option<DefaultActivationState<'a>> {
        if self.Element(node, "label") {
            return None;
        }
        if self.MainDetailsForSummary(node).is_some() {
            return Some(DefaultActivationState::default());
        }
        let input_type = self.TypeFor(node);
        let input_state = input_type.LegacyPreActivationBehavior(event);
        Some(DefaultActivationState {
            input_type: Some(input_type),
            input_state,
        })
    }
    // cpp: interaction/default_event_handler.cc:92-99
    pub fn RunActivationBehavior(
        &self,
        node: u64,
        event: &mut Event,
        state: Option<&DefaultActivationState<'a>>,
    ) {
        if self.MainDetailsForSummary(node).is_some() {
            return;
        }
        if self.Element(node, "label") {
            return;
        }
        if let Some(state) = state {
            if let Some(input_type) = &state.input_type {
                input_type.RunActivationBehavior(event, state.input_state.as_deref());
            }
        }
    }
    // cpp: interaction/default_event_handler.cc:101-120
    pub fn Handle(&self, node: u64, event: &mut Event) {
        if event.default_handled || event.default_prevented {
            return;
        }
        if anchor::HandleDefaultEvent(&self.context, node, event) {
            return;
        }
        if let Some((details, open)) = self.MainDetailsForSummary(node) {
            event.default_handled = HTMLSummaryElement::HandleDOMActivate(
                true,
                event.r#type == EventType::kCustom && event.custom_type == "DOMActivate",
                || self.ClickTargetIsControlInsideSummary(event.target_node_id, node),
                || {
                    (self.context.emit_mutation)(&DOMMutation {
                        mutation_type: if open {
                            DOMMutationType::kRemoveAttribute
                        } else {
                            DOMMutationType::kSetAttribute
                        },
                        target_node_id: details,
                        name: "open".into(),
                        ..Default::default()
                    })
                },
            );
            if event.default_handled {
                return;
            }
        }
        // cpp: SearchFieldCancelButtonElement::DefaultEventHandler. The real
        // retained UA-shadow button clears its search host on a primary click,
        // dispatches the SetValueForUser change event, then the search event.
        if event.r#type == EventType::kClick
            && event.target_node_id == node
            && event.button == MouseButton::kPrimary
        {
            if let Some(host) = self.SearchCancelButtonHost(node) {
                let input = SearchCancelHost {
                    context: &self.context,
                    host,
                };
                event.default_handled = SearchFieldCancelButtonElement::HandleDefaultEventWithInput(
                    &input,
                    true,
                    Some(0),
                );
                return;
            }
        }
        if self.Element(node, "label") {
            let state = self.context.default_action_state.Label(node);
            event.default_handled = HTMLLabelElement::HandleDefaultEventWithOperations(
                &state,
                &PageLabelDefault {
                    handler: self,
                    label: node,
                    event,
                },
                event.r#type == EventType::kClick,
                event.trusted,
            );
            if event.default_handled {
                return;
            }
        }
        if ["input", "textarea", "select", "button"]
            .iter()
            .any(|n| self.Element(node, n))
        {
            if IsDisabledFormControl(&self.context.document, node) {
                return;
            }
            self.TypeFor(node).HandleDefaultEvent(event);
        }
        if !event.default_handled
            && event.r#type == EventType::kClick
            && event.target_node_id == node
        {
            event.default_handled = Node::HandleClickDefaultAction(|| {
                (self.context.dispatch_dom_activate)(node, event)
            });
        }
    }
}

struct PageLabelDefault<'a, 'b> {
    handler: &'a DefaultEventHandler<'b>,
    label: u64,
    event: &'a Event,
}
impl LabelDefaultOperations<u64> for PageLabelDefault<'_, '_> {
    fn Control(&self) -> Option<u64> {
        self.handler.LabelControl(self.label)
    }
    fn RawTarget(&self) -> Option<u64> {
        Some(self.event.target_node_id)
    }
    fn ControlContains(&self, control: &u64, target: &u64) -> bool {
        ReadNode(
            &self.handler.context.document,
            *target,
            |document, _, index| {
                document.FindNodeById(*control).is_some_and(|control| {
                    document.IsShadowIncludingInclusiveAncestor(control, index)
                })
            },
        )
        .unwrap_or(false)
    }
    fn TargetIsInteractiveContent(&self, _: &u64) -> bool {
        let path = self.event.dispatch_state.borrow().path.clone();
        let Some(path) = path else {
            return false;
        };
        HTMLLabelElement::IsInteractiveContentInPath(
            path.Nodes(),
            &self.label,
            |left, right| left == right,
            |id| {
                ReadNode(
                    &self.handler.context.document,
                    *id,
                    |document, node, index| {
                        node.IsHTMLElement("button")
                            || node.IsHTMLElement("label")
                            || document.IsNativeHTMLInteractiveContent(index)
                    },
                )
                .unwrap_or(false)
            },
        )
    }
    fn PreparedSelection(&self) -> PreparedLabelSelection {
        PreparedLabelSelection {
            has_frame: true,
            label_layout_selectable: self
                .handler
                .context
                .editing_geometry
                .as_ref()
                .is_some_and(|geometry| geometry.CanStartSelection(self.label)),
            selection_is_range: self.handler.context.editor.DocumentSelectionIsRange(),
            mouse_down_was_single_click_in_selection: self
                .handler
                .context
                .selection_controller_state
                .MouseDownWasSingleClickInSelection(),
        }
    }
    fn MouseClickValues(&self) -> Option<LabelClickValues> {
        self.event
            .underlying_event
            .as_ref()
            .and_then(|event| match event {
                crate::input_event::InputEvent::Mouse(mouse) => Some(LabelClickValues {
                    has_position: true,
                    click_count: if mouse.r#type == crate::input_event::MouseEventType::kDoubleClick
                    {
                        2
                    } else {
                        1
                    },
                }),
                _ => None,
            })
    }
    fn TargetCanStartSelection(&self, target: &u64) -> bool {
        self.handler
            .context
            .editing_geometry
            .as_ref()
            .is_some_and(|geometry| geometry.CanStartSelection(*target))
    }
    fn ControlIsMouseFocusableOrDelegatesFocus(&self, control: &u64) -> bool {
        !IsDisabledFormControl(&self.handler.context.document, *control)
    }
    fn FocusControl(&self, control: &u64, focus_type: FocusType) {
        self.handler
            .context
            .focus_controller
            .FocusWithParams(*control, &HTMLLabelElement::ControlFocusParams(focus_type));
    }
    fn DispatchSimulatedClick(&self, control: &u64) {
        // Source html_label_element.cc:274-275: keep the original event and
        // source node guard/active-state lifecycle for the control click.
        (self.handler.context.dispatch_simulated_click)(*control, self.event);
    }
}

impl ClickableControlOperations<u64> for DefaultEventHandler<'_> {
    fn IsElement(&self, node: &u64) -> bool {
        ReadNode(&self.context.document, *node, |_, node, _| {
            node.Type() == dom::persistent_document::DOMNodeType::kElement
        })
        .unwrap_or(false)
    }
    fn IsFormControlElement(&self, node: &u64) -> bool {
        ReadNode(&self.context.document, *node, |document, _, index| {
            document.IsFormControlElement(index)
        })
        .unwrap_or(false)
    }
    fn OwnerShadowHost(&self, node: &u64) -> Option<u64> {
        ReadNode(&self.context.document, *node, |document, _, index| {
            document
                .OwnerShadowHost(index)
                .map(|index| document.Node(index).Id())
        })
        .flatten()
    }
    fn ParentOrShadowHostNode(&self, node: &u64) -> Option<u64> {
        ReadNode(&self.context.document, *node, |document, node, index| {
            node.Parent()
                .or_else(|| document.OwnerShadowHost(index))
                .map(|index| document.Node(index).Id())
        })
        .flatten()
    }
    fn HasActivationBehavior(&self, node: &u64) -> bool {
        DefaultEventHandler::HasActivationBehavior(self, *node)
            || ReadNode(&self.context.document, *node, |_, node, _| {
                node.IsHTMLElement("a") && node.FindAttribute("href").is_some()
            })
            .unwrap_or(false)
    }
    fn SameNode(&self, left: &u64, right: &u64) -> bool {
        left == right
    }
}
