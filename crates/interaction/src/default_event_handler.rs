use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    input_event::MouseButton,
    input_type::{
        ActivationState, CreateInputType, InputType, InputTypeContext, IsDisabledFormControl,
        IsTextField,
    },
    ownership::{IsElement, ReadNode},
};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
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
        ReadNode(&self.context.document, label, |d, n, _| {
            if !IsElement(n, "label") {
                return None;
            }
            if let Some(target) = n.FindAttribute("for") {
                fn find(d: &dom::Document, i: usize, id: &str) -> Option<u64> {
                    let n = d.Node(i);
                    if n.Type() == dom::persistent_document::DOMNodeType::kElement
                        && n.FindAttribute("id").is_some_and(|a| a.value == id)
                    {
                        return Some(n.Id());
                    }
                    for &c in n.Children() {
                        if let Some(v) = find(d, c, id) {
                            return Some(v);
                        }
                    }
                    None
                }
                return find(d, d.Root(), &target.value);
            }
            fn first(d: &dom::Document, i: usize) -> Option<u64> {
                for &c in d.Node(i).Children() {
                    let n = d.Node(c);
                    if IsElement(n, "input")
                        || IsElement(n, "textarea")
                        || IsElement(n, "select")
                        || IsElement(n, "button")
                    {
                        return Some(n.Id());
                    }
                    if let Some(v) = first(d, c) {
                        return Some(v);
                    }
                }
                None
            }
            first(d, d.FindNodeById(label).unwrap())
        })
        .flatten()
    }
    fn Element(&self, id: u64, name: &str) -> bool {
        ReadNode(&self.context.document, id, |_, n, _| IsElement(n, name)).unwrap_or(false)
    }
    // cpp: html_summary_element.cc:145-166. Only the first direct summary is
    // the details element's main summary and owns its activation behavior.
    fn MainDetailsForSummary(&self, id: u64) -> Option<(u64, bool)> {
        ReadNode(&self.context.document, id, |document, node, index| {
            if !IsElement(node, "summary") {
                return None;
            }
            let details = node.Parent()?;
            if !document.Node(details).IsHTMLElement("details")
                || document
                    .Node(details)
                    .Children()
                    .iter()
                    .copied()
                    .find(|child| document.Node(*child).IsHTMLElement("summary"))
                    != Some(index)
            {
                return None;
            }
            Some((
                document.Node(details).Id(),
                document.Node(details).FindAttribute("open").is_some(),
            ))
        })
        .flatten()
    }
    fn ClickTargetIsControlInsideSummary(&self, target: u64, summary: u64) -> bool {
        ReadNode(&self.context.document, target, |document, _, mut index| {
            loop {
                let node = document.Node(index);
                if node.Id() == summary {
                    return false;
                }
                if ["a", "button", "input", "select", "textarea"]
                    .iter()
                    .any(|name| IsElement(node, name))
                {
                    return true;
                }
                let Some(parent) = node.Parent() else {
                    return false;
                };
                index = parent;
            }
        })
        .unwrap_or(false)
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
        if let Some((details, open)) = self.MainDetailsForSummary(node) {
            if event.r#type == EventType::kClick
                && event.button == MouseButton::kPrimary
                && !event.default_prevented
                && !self.ClickTargetIsControlInsideSummary(event.target_node_id, node)
            {
                (self.context.emit_mutation)(&DOMMutation {
                    mutation_type: if open {
                        DOMMutationType::kRemoveAttribute
                    } else {
                        DOMMutationType::kSetAttribute
                    },
                    target_node_id: details,
                    name: "open".into(),
                    ..Default::default()
                });
                event.default_handled = true;
            }
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
        // cpp: SearchFieldCancelButtonElement::DefaultEventHandler. The real
        // retained UA-shadow button clears its search host on a primary click,
        // dispatches the SetValueForUser change event, then the search event.
        if event.r#type == EventType::kClick
            && event.target_node_id == node
            && event.button == MouseButton::kPrimary
        {
            if let Some(host) = self.SearchCancelButtonHost(node) {
                if IsDisabledFormControl(&self.context.document, host)
                    || crate::ownership::HasAttribute(&self.context.document, host, "readonly")
                {
                    return;
                }
                let value_changed = ReadNode(&self.context.document, host, |document, _, index| {
                    !document.ControlValue(index).is_empty()
                })
                .unwrap_or(false);
                (self.context.emit_mutation)(&DOMMutation {
                    mutation_type: DOMMutationType::kSetControlValue,
                    target_node_id: host,
                    value: String::new(),
                    ..Default::default()
                });
                if value_changed {
                    let mut change = MakeSyntheticEvent(EventType::kChange, host);
                    (self.context.dispatch_synthetic)(&mut change, host);
                }
                let mut search = MakeSyntheticEvent(EventType::kCustom, host);
                search.custom_type = "search".into();
                search.cancelable = false;
                search.composed = false;
                (self.context.dispatch_synthetic)(&mut search, host);
                event.default_handled = true;
                return;
            }
        }
        if self.Element(node, "label")
            && event.r#type == EventType::kClick
            && event.target_node_id == node
            && event.button == MouseButton::kPrimary
        {
            let Some(control) = self.LabelControl(node) else {
                return;
            };
            if IsDisabledFormControl(&self.context.document, control) {
                return;
            }
            if ["input", "textarea", "select", "button"]
                .iter()
                .any(|n| self.Element(control, n))
            {
                self.context
                    .focus_controller
                    .Focus(control, IsTextField(&self.context.document, control));
            }
            let mut click = MakeSyntheticEvent(EventType::kClick, control);
            click.button = MouseButton::kPrimary;
            (self.context.dispatch_synthetic)(&mut click, control);
            event.default_handled = true;
            return;
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
    }
}
