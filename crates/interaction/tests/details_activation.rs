use dom::{dom_mutation::ApplyDOMMutations, persistent_document::DOMNamespace, DOM};
use interaction::{
    event::{EventListenerResult, EventPhase, EventType},
    input_event::{InputEvent, MouseButton, MouseEvent, MouseEventType},
    ownership::{InteractionDOMMutationEmitter, InteractionDocument},
    Interaction,
};
use std::{cell::Cell, cell::RefCell, rc::Rc};

struct Fixture {
    document: InteractionDocument,
    interaction: Interaction<'static>,
    details: u64,
    summary: u64,
    cancel: Rc<Cell<bool>>,
}

impl Fixture {
    fn new() -> Self {
        let document = Rc::new(RefCell::new(DOM::new()));
        let (details, summary) = {
            let mut owner = document.borrow_mut();
            let dom = owner.GetDocumentMut();
            let html = dom.CreateElementDefault(DOMNamespace::kHTML, "html".into());
            dom.AppendChild(dom.Root(), html);
            let body = dom.CreateElementDefault(DOMNamespace::kHTML, "body".into());
            dom.AppendChild(html, body);
            let details = dom.CreateElementDefault(DOMNamespace::kHTML, "details".into());
            dom.AppendChild(body, details);
            let summary = dom.CreateElementDefault(DOMNamespace::kHTML, "summary".into());
            dom.AppendChild(details, summary);
            let content = dom.CreateElementDefault(DOMNamespace::kHTML, "p".into());
            dom.AppendChild(details, content);
            (dom.Node(details).Id(), dom.Node(summary).Id())
        };
        let mutation_document = document.clone();
        let emit: InteractionDOMMutationEmitter = Rc::new(move |mutation| {
            ApplyDOMMutations(
                mutation_document.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        });
        let cancel = Rc::new(Cell::new(false));
        let cancel_listener = cancel.clone();
        let interaction = Interaction::WithDOMMutationEmitter(
            emit,
            Some(Rc::new(move |view, _, _| EventListenerResult {
                prevent_default: cancel_listener.get()
                    && view.phase == EventPhase::kAtTarget
                    && !view.capture_listeners
                    && view.event.r#type == EventType::kClick,
                ..Default::default()
            })),
        );
        Self {
            document,
            interaction,
            details,
            summary,
            cancel,
        }
    }

    fn click_summary(&self) {
        self.interaction.DispatchTargeted(
            &InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                target_node_id: Some(self.summary),
                ..Default::default()
            }),
            &self.document,
        );
    }

    fn open(&self) -> bool {
        let owner = self.document.borrow();
        let dom = owner.GetDocument();
        dom.Node(dom.FindNodeById(self.details).unwrap())
            .FindAttribute("open")
            .is_some()
    }
}

#[test]
fn main_summary_click_toggles_details_and_honors_cancellation() {
    let fixture = Fixture::new();
    assert!(!fixture.open());

    fixture.click_summary();
    assert!(fixture.open());

    fixture.cancel.set(true);
    fixture.click_summary();
    assert!(fixture.open());

    fixture.cancel.set(false);
    fixture.click_summary();
    assert!(!fixture.open());
}
