use dom::{
    dom_mutation::ApplyDOMMutations,
    persistent_document::{DOMAttribute, DOMNamespace},
    DOM,
};
use interaction::{
    event::{EventListenerDispatcher, EventListenerResult, EventPhase, EventType, EventTypeName},
    input_event::*,
    ownership::{InteractionDOMMutationEmitter, InteractionDocument},
    Interaction,
};
use layoutng_assembly::fragment_tree::FragmentNode;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn opt(id: Option<u64>) -> String {
    id.map_or("-".into(), |id| id.to_string())
}
struct Fixture {
    doc: InteractionDocument,
    n: BTreeMap<String, u64>,
    engine: Interaction<'static>,
    output: Rc<RefCell<String>>,
    cancel: Rc<Cell<bool>>,
    reenter: Rc<Cell<bool>>,
    append: Rc<Cell<bool>>,
    switch: Rc<Cell<bool>>,
    clear: Rc<Cell<bool>>,
    unicode: Rc<Cell<bool>>,
    listeners: EventListenerDispatcher<'static>,
}
impl Fixture {
    fn new() -> Self {
        let doc = Rc::new(RefCell::new(DOM::new()));
        let mut n = BTreeMap::new();
        let mut owner = doc.borrow_mut();
        let d = owner.GetDocumentMut();
        fn add(
            d: &mut dom::Document,
            n: &mut BTreeMap<String, u64>,
            key: &str,
            parent: usize,
            tag: &str,
            attrs: &[(&str, &str)],
        ) -> usize {
            let attrs = attrs
                .iter()
                .map(|&(name, value)| DOMAttribute {
                    local_name: name.into(),
                    value: value.into(),
                    ..Default::default()
                })
                .collect();
            let i = d.CreateElement(DOMNamespace::kHTML, tag.into(), attrs);
            d.AppendChild(parent, i);
            n.insert(key.into(), d.Node(i).Id());
            i
        }
        let root = d.Root();
        let html = add(d, &mut n, "html", root, "html", &[]);
        let body = add(d, &mut n, "body", html, "body", &[]);
        add(d, &mut n, "text", body, "input", &[("value", "A")]);
        let ta = add(d, &mut n, "ta", body, "textarea", &[]);
        d.AppendText(ta, "Hi");
        add(d, &mut n, "check", body, "input", &[("type", "checkbox")]);
        add(
            d,
            &mut n,
            "r1",
            body,
            "input",
            &[("type", "radio"), ("name", "g"), ("checked", "")],
        );
        add(
            d,
            &mut n,
            "r2",
            body,
            "input",
            &[("type", "radio"), ("name", "g")],
        );
        add(
            d,
            &mut n,
            "range",
            body,
            "input",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "10"),
                ("value", "5"),
            ],
        );
        let select = add(d, &mut n, "select", body, "select", &[]);
        let o1 = add(d, &mut n, "o1", select, "option", &[]);
        d.AppendText(o1, "one");
        let o2 = add(d, &mut n, "o2", select, "option", &[]);
        d.AppendText(o2, "two");
        let form = add(d, &mut n, "form", body, "form", &[]);
        add(d, &mut n, "ft", form, "input", &[("value", "q")]);
        add(d, &mut n, "submit", form, "button", &[]);
        add(d, &mut n, "reset", form, "button", &[("type", "reset")]);
        add(
            d,
            &mut n,
            "readonly",
            body,
            "input",
            &[("readonly", ""), ("value", "R")],
        );
        add(
            d,
            &mut n,
            "disabled",
            body,
            "input",
            &[("disabled", ""), ("type", "checkbox")],
        );
        drop(owner);
        let output = Rc::new(RefCell::new(String::new()));
        let cancel = Rc::new(Cell::new(false));
        let reenter = Rc::new(Cell::new(false));
        let append = Rc::new(Cell::new(false));
        let switch = Rc::new(Cell::new(false));
        let clear = Rc::new(Cell::new(false));
        let unicode = Rc::new(Cell::new(false));
        let slot = Rc::new(RefCell::new(None::<Interaction>));
        let (out, c, r, a, u, s, ids) = (
            output.clone(),
            cancel.clone(),
            reenter.clone(),
            append.clone(),
            unicode.clone(),
            slot.clone(),
            n.clone(),
        );
        let listeners: EventListenerDispatcher = Rc::new(move |v, doc, _| {
            if v.phase == EventPhase::kAtTarget && !v.capture_listeners {
                out.borrow_mut().push_str(&format!(
                    "E {} {} {} {}\n",
                    EventTypeName(v.event.r#type),
                    v.target_node_id,
                    opt(v.event.related_target_node_id),
                    opt(v.focused_node_id)
                ));
                if a.get() && v.event.r#type == EventType::kClick {
                    a.set(false);
                    let id = {
                        let mut owner = doc.borrow_mut();
                        let d = owner.GetDocumentMut();
                        let node = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
                        let body = d.FindNodeById(ids["body"]).unwrap();
                        d.AppendChild(body, node);
                        d.Node(node).Id()
                    };
                    out.borrow_mut().push_str(&format!("A {id}\n"));
                }
                if r.get() && v.event.r#type == EventType::kBlur {
                    r.set(false);
                    let engine = s.borrow().as_ref().unwrap().clone();
                    engine.Dispatch(
                        &InputEvent::Focus(FocusEvent {
                            target_node_id: ids["range"],
                            ..Default::default()
                        }),
                        doc,
                        &FragmentNode::default(),
                    );
                }
                if u.get() && v.event.r#type == EventType::kBeforeInput {
                    u.set(false);
                    let mut owner = doc.borrow_mut();
                    let d = owner.GetDocumentMut();
                    let i = d.FindNodeById(ids["text"]).unwrap();
                    d.SetControlValue(i, "é".into());
                }
                if c.get() && v.event.r#type == EventType::kClick {
                    return EventListenerResult {
                        prevent_default: true,
                        ..Default::default()
                    };
                }
            }
            EventListenerResult::default()
        });
        let (out, document, sw, cl, s) = (
            output.clone(),
            doc.clone(),
            switch.clone(),
            clear.clone(),
            slot.clone(),
        );
        let emit: InteractionDOMMutationEmitter = Rc::new(move |m| {
            out.borrow_mut().push_str(&format!(
                "M {} {} {} {}\n",
                m.mutation_type as u8,
                m.target_node_id,
                m.bool_value as u8,
                hex(&m.value)
            ));
            ApplyDOMMutations(
                document.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(m),
            );
            if sw.replace(false) {
                let engine = s.borrow().as_ref().unwrap().clone();
                let out = out.clone();
                engine.SetEventListenerDispatcher(Some(Rc::new(move |v, _, _| {
                    if v.phase == EventPhase::kAtTarget && !v.capture_listeners {
                        out.borrow_mut().push_str(&format!(
                            "N {} {}\n",
                            EventTypeName(v.event.r#type),
                            v.target_node_id
                        ));
                    }
                    EventListenerResult::default()
                })));
            }
            if cl.replace(false) {
                let engine = s.borrow().as_ref().unwrap().clone();
                engine.SetEventListenerDispatcher(None);
            }
        });
        let engine = Interaction::WithDOMMutationEmitter(emit, Some(listeners.clone()));
        *slot.borrow_mut() = Some(engine.clone());
        Self {
            doc,
            n,
            engine,
            output,
            cancel,
            reenter,
            append,
            switch,
            clear,
            unicode,
            listeners,
        }
    }
    fn focus(&self, name: &str) -> InputEvent {
        InputEvent::Focus(FocusEvent {
            target_node_id: self.n[name],
            ..Default::default()
        })
    }
    fn click(&self, name: &str) -> InputEvent {
        InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kClick,
            button: MouseButton::kPrimary,
            target_node_id: Some(self.n[name]),
            ..Default::default()
        })
    }
    fn key(&self, name: &str, key: &str, text: &str) -> InputEvent {
        InputEvent::Key(KeyEvent {
            key: key.into(),
            text: text.into(),
            target_node_id: Some(self.n[name]),
            ..Default::default()
        })
    }
    fn value(&self, name: &str) -> String {
        let o = self.doc.borrow();
        let d = o.GetDocument();
        d.ControlValue(d.FindNodeById(self.n[name]).unwrap())
    }
    fn checked(&self, name: &str) -> bool {
        let o = self.doc.borrow();
        let d = o.GetDocument();
        d.ControlChecked(d.FindNodeById(self.n[name]).unwrap())
    }
    fn run(&self, name: &str, input: InputEvent) {
        self.output.borrow_mut().push_str(&format!("CASE {name}\n"));
        self.engine
            .Dispatch(&input, &self.doc, &FragmentNode::default());
        let s = self.engine.State();
        self.output.borrow_mut().push_str(&format!(
            "S {} {} {} {} {} {} {} {} {} {} {}\n",
            opt(s.focused_node_id),
            opt(s.focus_visible_node_id),
            opt(s.pressed_node_id),
            self.checked("check") as u8,
            self.checked("r1") as u8,
            self.checked("r2") as u8,
            hex(&self.value("text")),
            hex(&self.value("ta")),
            hex(&self.value("range")),
            hex(&self.value("select")),
            hex(&self.value("ft"))
        ));
    }
    fn sequence(&self) {
        self.run("focus_text", self.focus("text"));
        self.run("focus_repeat", self.focus("text"));
        self.run("focus_ta", self.focus("ta"));
        self.run(
            "blur_unfocused",
            InputEvent::Focus(FocusEvent {
                r#type: FocusEventType::kBlur,
                target_node_id: self.n["text"],
                ..Default::default()
            }),
        );
        self.run(
            "blur_ta",
            InputEvent::Focus(FocusEvent {
                r#type: FocusEventType::kBlur,
                target_node_id: self.n["ta"],
                ..Default::default()
            }),
        );
        self.run("focus_reentry_setup", self.focus("text"));
        self.reenter.set(true);
        self.run("focus_reentry", self.focus("ta"));
        {
            let mut o = self.doc.borrow_mut();
            let d = o.GetDocumentMut();
            let ta = d.FindNodeById(self.n["ta"]).unwrap();
            d.Remove(ta);
        }
        self.run("focus_detached_old", self.focus("text"));
        self.cancel.set(true);
        self.run("checkbox_cancel", self.click("check"));
        self.run("radio_cancel", self.click("r2"));
        self.cancel.set(false);
        self.append.set(true);
        self.run("checkbox_append", self.click("check"));
        self.run("radio_arrow", self.key("r1", "ArrowRight", ""));
        self.run("range_arrow", self.key("range", "ArrowRight", ""));
        self.run("select_arrow", self.key("select", "ArrowDown", ""));
        self.run(
            "readonly",
            InputEvent::TextInput(TextInputEvent {
                text: "Z".into(),
                target_node_id: Some(self.n["readonly"]),
            }),
        );
        self.run("disabled", self.click("disabled"));
        self.run("form_enter", self.key("ft", "Enter", "\r"));
        {
            let mut o = self.doc.borrow_mut();
            let d = o.GetDocumentMut();
            let i = d.FindNodeById(self.n["ft"]).unwrap();
            d.SetControlValue(i, "changed".into());
        }
        self.run("form_reset", self.click("reset"));
        self.run("text_focus", self.focus("text"));
        self.run(
            "text_unicode",
            InputEvent::TextInput(TextInputEvent {
                text: "😀".into(),
                target_node_id: Some(self.n["text"]),
            }),
        );
        self.run("text_backspace", self.key("text", "Backspace", ""));
        self.run("space_down", self.key("check", " ", ""));
        self.run(
            "space_up",
            InputEvent::Key(KeyEvent {
                r#type: KeyEventType::kUp,
                key: " ".into(),
                target_node_id: Some(self.n["check"]),
                ..Default::default()
            }),
        );
        self.switch.set(true);
        self.run("listener_switch", self.click("check"));
        self.clear.set(true);
        self.run("listener_clear", self.click("check"));
        self.engine
            .SetEventListenerDispatcher(Some(self.listeners.clone()));
        self.run("utf_boundary_focus", self.focus("text"));
        let stop = Rc::new(Cell::new(false));
        let (out, stop_read, body) = (self.output.clone(), stop.clone(), self.n["body"]);
        self.engine
            .SetEventListenerDispatcher(Some(Rc::new(move |v, _, _| {
                out.borrow_mut().push_str(&format!(
                    "P {} {} {} {} {} {} {}\n",
                    v.phase as u8,
                    v.target_node_id,
                    v.current_target_node_id,
                    v.capture_listeners as u8,
                    v.event.phase as u8,
                    v.event.current_target_node_id,
                    v.event.capture_listeners as u8
                ));
                EventListenerResult {
                    stop_propagation: stop_read.get() && v.current_target_node_id == body,
                    stop_immediate_propagation: stop_read.get() && v.target_node_id == 0,
                    ..Default::default()
                }
            })));
        let probe = |name: &str, target: u64| {
            self.output.borrow_mut().push_str(&format!("CASE {name}\n"));
            let mut e = interaction::event::MakeSyntheticEvent(EventType::kCustom, target);
            e.bubbles = true;
            e.cancelable = true;
            let r = self.engine.DispatchDOMEvent(&mut e, target, &self.doc);
            self.output.borrow_mut().push_str(&format!(
                "D {} {} {} {} {}\n",
                opt(r.target_node_id),
                r.propagation_stopped as u8,
                e.phase as u8,
                e.current_target_node_id,
                e.capture_listeners as u8
            ));
        };
        probe("event_path", self.n["text"]);
        stop.set(true);
        probe("event_path_stop", self.n["text"]);
        stop.set(false);
        probe("target_zero", 0);
        stop.set(true);
        probe("target_zero_stop", 0);
        self.engine
            .SetEventListenerDispatcher(Some(self.listeners.clone()));
    }
}
#[test]
fn actual_native_focus_default_actions_cancellation_reentry_and_slot_replacement_match() {
    let f = Fixture::new();
    f.sequence();
    let native=include_str!("../../../artifacts/parallel-zero-parity/shadow-state/interaction-native/interaction-results.txt");
    let expected = native.split("CASE utf_boundary\n").next().unwrap();
    assert_eq!(*f.output.borrow(), expected);
}
#[test]
fn native_byte_offset_can_produce_invalid_utf8_after_beforeinput_value_change() {
    let f = Fixture::new();
    f.run("setup", f.focus("text"));
    f.unicode.set(true);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        f.engine.Dispatch(
            &InputEvent::TextInput(TextInputEvent {
                text: "X".into(),
                target_node_id: Some(f.n["text"]),
            }),
            &f.doc,
            &FragmentNode::default(),
        )
    }));
    assert!(
        result.is_err(),
        "this source byte-string edge is currently unrepresentable by DOM's Rust String"
    );
    assert_eq!(f.value("text"), "é");
    let native=include_str!("../../../artifacts/parallel-zero-parity/shadow-state/interaction-native/interaction-results.txt");
    assert!(native
        .split("CASE utf_boundary\n")
        .nth(1)
        .unwrap()
        .contains("c358a9"));
}
