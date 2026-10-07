use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    input_event::MouseButton,
    input_type::{
        ActivationState, CreateInputType, InputType, InputTypeContext, IsDisabledFormControl,
        IsTextField,
    },
    ownership::{IsElement, ReadNode},
};
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
    // cpp: interaction/default_event_handler.cc:74-78
    pub fn HasActivationBehavior(&self, node: u64) -> bool {
        if IsDisabledFormControl(&self.context.document, node) {
            return false;
        }
        if self.Element(node, "label") {
            return self.LabelControl(node).is_some();
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
