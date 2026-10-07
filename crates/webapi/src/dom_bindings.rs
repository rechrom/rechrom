#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use dom::dom_mutation::{DOMMutation, DOMMutationEmitter, DOMMutationType};
use dom::persistent_document::{DOMNamespace, DOMNodeType};
use dom::{Document, DOM};
use layoutng_assembly::internal::layout_input::PaintPathVerb;
use javascript::javascript_runtime::{
    HostCall, HostContinuation, HostMethodRef, HostObjectId, HostObjectRef, HostOperation,
    HostResult, HostSymbol, HostValue, JavaScriptExceptionKind, JavaScriptFunction,
    JavaScriptHostBindings, JavaScriptHostRuntime, JavaScriptNull, JavaScriptRealm,
    JavaScriptValue, WeakJavaScriptRealm,
};

#[path = "dom_services.rs"]
mod services;
pub use services::DOMBindingsHost;
#[path = "dom_mutation_notification.rs"]
mod mutation_notifications;
pub use mutation_notifications::MutationNotification;

const GLOBAL_NAMES: &[&str] = &[
    "document",
    "__domInvoke",
    "__domInstall",
    "__domWrap",
    "window",
    "self",
    "parent",
    "top",
    "addEventListener",
    "removeEventListener",
    "dispatchEvent",
    "onload",
    "onerror",
    "onclick",
    "oninput",
    "onchange",
    "onkeydown",
    "onkeyup",
];

#[derive(Clone)]
struct NodeCollection {
    root: HostObjectId,
    kind: String,
    query: String,
    snapshot: Vec<HostObjectId>,
    membership_cache: RefCell<Option<CollectionMembershipCache>>,
}
#[derive(Clone)]
struct CollectionMembershipCache {
    key: (dom::persistent_document::DOMOwnerHandle, u64),
    ids: Rc<Vec<HostObjectId>>,
}
// Rc<Vec> keeps the original miss allocation without copying IDs into Rc<[Id]>.
// The caller owns this view; no RefCell cache guard escapes the traversal.
enum CollectionNodes<'a> {
    Borrowed(&'a [HostObjectId]),
    Shared(Rc<Vec<HostObjectId>>),
    Owned(Vec<HostObjectId>),
}
impl std::ops::Deref for CollectionNodes<'_> {
    type Target = [HostObjectId];
    fn deref(&self) -> &[HostObjectId] {
        match self {
            Self::Borrowed(ids) => ids,
            Self::Shared(ids) => ids.as_slice(),
            Self::Owned(ids) => ids.as_slice(),
        }
    }
}
impl CollectionNodes<'_> {
    fn into_owned(self) -> Vec<HostObjectId> {
        match self {
            Self::Borrowed(ids) => ids.to_vec(),
            Self::Shared(ids) => ids.as_ref().clone(),
            Self::Owned(ids) => ids,
        }
    }
}
struct CollectionIterator {
    collection: HostObjectId,
    index: usize,
    done: bool,
}
#[derive(Clone)]
struct Listener {
    // Shared with dispatch snapshots; removal never revives an old registration.
    active: Rc<Cell<bool>>,
    target: HostObjectId,
    kind: String,
    callback: JavaScriptFunction,
    capture: bool,
    attribute: bool,
    once: bool,
    passive: bool,
}

// Scoped ownership adapter for source DOMBindingsHost::change_focus.
pub type FocusChangeHandler = Rc<dyn Fn(HostObjectId, bool, &mut dyn JavaScriptHostRuntime)>;
// A source Emit can synchronously reenter JavaScript (image events and
// mutation notifications). Execute it after releasing the host borrow.
pub type ScopedDOMMutationEmitter = Rc<dyn Fn(&DOMMutation, &mut dyn JavaScriptHostRuntime)>;
pub struct DOMJavaScriptBindings {
    document: Rc<RefCell<DOM>>,
    services: services::DOMServices,
    emit_mutation: DOMMutationEmitter,
    scoped_emit_mutation: Option<ScopedDOMMutationEmitter>,
    prototypes: HashMap<String, JavaScriptValue>,
    method_wrappers: HashMap<String, JavaScriptFunction>,
    runtime_realm: WeakJavaScriptRealm,
    ready_state: String,
    current_script: HostObjectId,
    focused_node: Option<u64>,
    write_document: Option<Box<dyn FnMut(&str) -> bool>>,
    change_focus: Option<FocusChangeHandler>,
    collections: RefCell<HashMap<HostObjectId, NodeCollection>>,
    iterators: RefCell<HashMap<HostObjectId, CollectionIterator>>,
    records: RefCell<HashMap<HostObjectId, HashMap<String, HostValue>>>,
    next_collection_id: Cell<HostObjectId>,
    listeners: RefCell<HashMap<HostObjectId, HashMap<String, Vec<Listener>>>>,
}

impl DOMJavaScriptBindings {
    /// EventListenerProperties::blocking_event_listeners analogue used when
    /// deciding whether wheel input may run on the compositor thread.
    pub fn HasBlockingListener(&self, kind: &str) -> bool {
        self.listeners.borrow().values().any(|kinds| {
            kinds.get(kind).is_some_and(|listeners| {
                listeners
                    .iter()
                    .any(|listener| listener.active.get() && !listener.passive)
            })
        })
    }

    /// Targets whose active listeners can cancel this event. Geometry belongs
    /// to Page/PrePaint; bindings expose identities only.
    pub fn BlockingListenerTargets(&self, kind: &str) -> Vec<HostObjectId> {
        self.listeners
            .borrow()
            .iter()
            .filter_map(|(target, kinds)| {
                kinds.get(kind).and_then(|listeners| {
                    listeners
                        .iter()
                        .any(|listener| listener.active.get() && !listener.passive)
                        .then_some(*target)
                })
            })
            .collect()
    }

    // cpp: webapi/dom_bindings.cc:236-240
    pub fn new(document: Rc<RefCell<DOM>>, emit_mutation: DOMMutationEmitter) -> Self {
        Self::WithHost(document, emit_mutation, DOMBindingsHost::default())
    }
    pub fn WithHost(
        document: Rc<RefCell<DOM>>,
        emit_mutation: DOMMutationEmitter,
        host: DOMBindingsHost,
    ) -> Self {
        Self {
            services: services::DOMServices::new(host),
            document,
            emit_mutation,
            scoped_emit_mutation: None,
            prototypes: HashMap::new(),
            method_wrappers: HashMap::new(),
            runtime_realm: WeakJavaScriptRealm::default(),
            ready_state: "loading".into(),
            current_script: 0,
            focused_node: None,
            write_document: None,
            change_focus: None,
            collections: RefCell::new(HashMap::new()),
            iterators: RefCell::new(HashMap::new()),
            records: RefCell::new(HashMap::new()),
            next_collection_id: Cell::new(1 << 62),
            listeners: RefCell::new(HashMap::new()),
        }
    }

    // cpp: webapi/dom_bindings.h:33
    // scoped runtime is a Rust reborrow adapter.
    pub fn SetFocusChangeHandler(&mut self, handler: Option<FocusChangeHandler>) {
        self.change_focus = handler;
    }

    pub fn SetScopedMutationEmitter(&mut self, emitter: ScopedDOMMutationEmitter) {
        self.scoped_emit_mutation = Some(emitter);
    }

    // cpp: webapi/dom_bindings.h:50
    // The caller supplies a full or current-context runtime during delivery.
    // Retain only the source binding's validity, avoiding a host/realm cycle.
    pub fn BindRuntime(&mut self, realm: &JavaScriptRealm) {
        self.runtime_realm = realm.Downgrade();
    }

    pub fn SetReadyState(&mut self, state: String) {
        self.ready_state = state;
    }

    pub fn SetCurrentScript(&mut self, id: HostObjectId) {
        self.current_script = id;
    }

    // cpp: webapi/dom_bindings.h:58
    pub fn SetFocusedNode(&mut self, id: Option<u64>) {
        self.focused_node = id;
    }

    // Rust callback adapter for DOMBindingsHost::write_document.
    pub fn SetDocumentWriteHandler(&mut self, handler: Box<dyn FnMut(&str) -> bool>) {
        self.write_document = Some(handler);
    }

    fn value(value: HostValue) -> HostResult {
        HostResult {
            value,
            ..Default::default()
        }
    }

    fn unhandled() -> HostResult {
        HostResult {
            handled: false,
            ..Default::default()
        }
    }

    fn type_error(message: &str) -> HostResult {
        HostResult::Failure(JavaScriptExceptionKind::kTypeError, message)
    }

    fn node(&self, id: HostObjectId) -> Option<usize> {
        self.document.borrow().GetDocument().FindNodeById(id)
    }

    fn method(&self, receiver: HostObjectId, name: &str) -> HostResult {
        if let Some(wrapper) = self.method_wrappers.get(name) {
            return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
        }
        Self::value(HostValue::Method(HostMethodRef {
            receiver,
            name: name.into(),
        }))
    }

    fn get(&self, receiver: HostObjectId, member: &str) -> HostResult {
        // cpp: webapi/dom_bindings.cc:269-300
        if let Some(record) = self.records.borrow().get(&receiver) {
            return record
                .get(member)
                .cloned()
                .map_or_else(Self::unhandled, Self::value);
        }
        if self.iterators.borrow().contains_key(&receiver) {
            return if member == "next" {
                self.method(receiver, member)
            } else {
                Self::unhandled()
            };
        }
        // cpp: webapi/dom_bindings.cc:278-285
        if is_event_handler(member) {
            return self
                .listeners
                .borrow()
                .get(&receiver)
                .and_then(|kinds| kinds.get(&member[2..]))
                .and_then(|listeners| listeners.iter().find(|l| l.attribute))
                .map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |l| Self::value(HostValue::JavaScriptFunction(l.callback.clone())),
                );
        }
        if let Some(collection) = self.collections.borrow().get(&receiver) {
            if member == "item"
                || (member == "namedItem"
                    && collection.kind != "snapshot"
                    && collection.kind != "childNodes")
            {
                let class = if matches!(collection.kind.as_str(), "snapshot" | "childNodes") {
                    "NodeList"
                } else {
                    "HTMLCollection"
                };
                if self
                    .method_wrappers
                    .contains_key(&format!("__{class}_{member}"))
                {
                    // DOM-only source wrappers live on the real prototype.
                    // Preserve Reflect getter/error frames and own/prototype
                    // overrides; argument coercion runs before host traversal.
                    return Self::unhandled();
                }
                // Preserve the raw host protocol before bootstrap installation.
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver,
                    name: member.into(),
                }));
            }
            let nodes = self.collection_nodes(collection);
            if member == "length" {
                return Self::value(HostValue::Number(nodes.len() as f64));
            }
            // std::from_chars accepts decimal digits without leading +/-/space.
            if !member.is_empty() && member.bytes().all(|c| c.is_ascii_digit()) {
                if let Ok(index) = member.parse::<usize>() {
                    if let Some(&id) = nodes.get(index) {
                        return Self::value(HostValue::Object(HostObjectRef { id }));
                    }
                }
            }
            return Self::unhandled();
        }
        // cpp: webapi/dom_bindings.cc:301-322
        if receiver == 0 {
            // cpp: webapi/dom_bindings.cc:302-309
            if matches!(
                member,
                "addEventListener" | "removeEventListener" | "dispatchEvent"
            ) {
                if let Some(wrapper) = self.method_wrappers.get(&format!("__window_{member}")) {
                    return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
                }
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver: 0,
                    name: member.into(),
                }));
            }
            if matches!(member, "__domInvoke" | "__domInstall" | "__domWrap") {
                return self.method(0, member);
            }
            if member == "document" {
                let root = self.document.borrow().GetDocument().Root();
                let id = self.document.borrow().GetDocument().Node(root).Id();
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            if matches!(member, "window" | "self" | "parent" | "top") {
                return Self::value(HostValue::Object(HostObjectRef { id: 0 }));
            }
            if let Some(wrapper) = self.method_wrappers.get(member) {
                return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
            }
            return Self::unhandled();
        }
        if let Some(result) = self.GetBoundEvent(receiver, member) {
            return result;
        }
        let document = self.document.borrow();
        let document = document.GetDocument();
        let Some(index) = document.FindNodeById(receiver) else {
            return Self::type_error("host object no longer exists");
        };
        let node = document.Node(index);
        // cpp: webapi/dom_bindings.cc:384-385
        if matches!(member, "childNodes" | "children") {
            return Self::value(HostValue::Object(self.collection(receiver, member, "")));
        }
        if is_method(node.Type(), member) {
            return self.method(receiver, member);
        }
        // cpp: webapi/dom_bindings.cc:360-388
        if node.Type() == DOMNodeType::kDocument {
            match member {
                "activeElement" => {
                    if let Some(id) = self.focused_node {
                        return Self::value(HostValue::Object(HostObjectRef { id }));
                    }
                    return self.get(receiver, "body");
                }
                "readyState" => return Self::value(HostValue::String(self.ready_state.clone())),
                "currentScript" => {
                    return if self.current_script == 0 {
                        Self::value(HostValue::Null(JavaScriptNull))
                    } else {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: self.current_script,
                        }))
                    }
                }
                "defaultView" => {
                    return Self::value(if receiver == document.Node(document.Root()).Id() {
                        HostValue::Object(HostObjectRef { id: 0 })
                    } else {
                        HostValue::Null(JavaScriptNull)
                    })
                }
                "documentElement" | "scrollingElement" | "head" | "body" => {
                    let root = node
                        .Children()
                        .iter()
                        .copied()
                        .find(|&child| document.Node(child).Type() == DOMNodeType::kElement);
                    // Documents produced by this browser currently use the
                    // standards-mode scrolling model: the root element owns
                    // the viewport scroll offset.
                    if member == "documentElement" || member == "scrollingElement" {
                        return Self::node_reference(document, root);
                    }
                    if let Some(root) = root {
                        for &child in document.Node(root).Children() {
                            if document.Node(child).IsHTMLElement(member)
                                || (member == "body"
                                    && document.Node(child).IsHTMLElement("frameset"))
                            {
                                return Self::node_reference(document, Some(child));
                            }
                        }
                    }
                    return Self::node_reference(document, None);
                }
                _ => {}
            }
        }
        // cpp: webapi/dom_bindings.cc:391-411
        if node.IsHTMLElement("img") {
            let source = node.FindAttribute("src");
            let image = source.and_then(|source| document.ImageResourceFor(&source.value));
            match member {
                "complete" => {
                    return Self::value(HostValue::Boolean(
                        source.is_none_or(|source| source.value.is_empty()) || image.is_some(),
                    ))
                }
                "naturalWidth" => {
                    return Self::value(HostValue::Number(
                        image.map_or(0.0, |image| image.natural_width),
                    ))
                }
                "naturalHeight" => {
                    return Self::value(HostValue::Number(
                        image.map_or(0.0, |image| image.natural_height),
                    ))
                }
                "currentSrc" => {
                    return Self::value(HostValue::String(
                        source.map_or("", |source| &source.value).into(),
                    ))
                }
                _ => {}
            }
        }
        // cpp: webapi/dom_bindings.cc:412-442
        if node.Type() == DOMNodeType::kElement {
            match member {
                "className" | "id" => {
                    let attribute = if member == "className" { "class" } else { "id" };
                    let text = node
                        .FindAttribute(attribute)
                        .map_or("", |attribute| &attribute.value);
                    return Self::value(HostValue::String(text.into()));
                }
                "tagName" => {
                    let name = if node.Namespace() == DOMNamespace::kHTML {
                        node.Name().to_ascii_uppercase()
                    } else {
                        node.Name().into()
                    };
                    return Self::value(HostValue::String(name));
                }
                "localName" => return Self::value(HostValue::String(node.Name().into())),
                "namespaceURI" => {
                    return Self::value(match node.Namespace() {
                        DOMNamespace::kHTML => {
                            HostValue::String("http://www.w3.org/1999/xhtml".into())
                        }
                        DOMNamespace::kSVG => {
                            HostValue::String("http://www.w3.org/2000/svg".into())
                        }
                        DOMNamespace::kMathML => {
                            HostValue::String("http://www.w3.org/1998/Math/MathML".into())
                        }
                        DOMNamespace::kNone => HostValue::Null(JavaScriptNull),
                    })
                }
                _ => {}
            }
        }
        if member == "ownerDocument" {
            return Self::value(if node.Type() == DOMNodeType::kDocument {
                HostValue::Null(JavaScriptNull)
            } else {
                HostValue::Object(HostObjectRef {
                    id: node
                        .OwnerDocumentNode()
                        .expect("DOM node has no owner document")
                        .Id(),
                })
            });
        }
        if member == "content" && node.IsHTMLElement("template") {
            return Self::node_reference(
                document,
                Some(
                    document
                        .TemplateContents(index)
                        .expect("template has no contents"),
                ),
            );
        }
        // cpp: webapi/dom_bindings.cc:443-477
        match member {
            "nodeType" => Self::value(HostValue::Number(node_type_number(node.Type()))),
            "nodeName" => Self::value(HostValue::String(node_name(
                node.Type(),
                node.Name(),
                node.Namespace(),
            ))),
            "textContent"
                if matches!(
                    node.Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentType
                ) =>
            {
                Self::value(HostValue::Null(JavaScriptNull))
            }
            "textContent" => Self::value(HostValue::String(text_content(document, index))),
            "data" if is_character_data(node.Type()) => {
                Self::value(HostValue::String(node.Data().into()))
            }
            "nodeValue" => Self::value(if is_character_data(node.Type()) {
                HostValue::String(node.Data().into())
            } else {
                HostValue::Null(JavaScriptNull)
            }),
            "innerHTML"
                if matches!(
                    node.Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                ) =>
            {
                Self::value(HostValue::String(inner_html(document, index)))
            }
            "parentNode" => Self::node_reference(document, node.Parent()),
            "firstChild" => Self::node_reference(document, node.Children().first().copied()),
            "lastChild" => Self::node_reference(document, node.Children().last().copied()),
            "id" => Self::value(HostValue::String(
                node.FindAttribute("id")
                    .map_or("", |attribute| &attribute.value)
                    .into(),
            )),
            "value" if is_control(node) => {
                Self::value(HostValue::String(document.ControlValue(index)))
            }
            "checked" if node.IsHTMLElement("input") => {
                Self::value(HostValue::Boolean(document.ControlChecked(index)))
            }
            _ => Self::unhandled(),
        }
    }

    fn node_reference(document: &Document, node: Option<usize>) -> HostResult {
        Self::value(node.map_or(HostValue::Null(JavaScriptNull), |node| {
            HostValue::Object(HostObjectRef {
                id: document.Node(node).Id(),
            })
        }))
    }

    fn set(&mut self, receiver: HostObjectId, member: &str, arguments: &[HostValue]) -> HostResult {
        // cpp: webapi/dom_bindings.cc:479-495
        if is_event_handler(member) {
            let kinds = self.listeners.get_mut().entry(receiver).or_default();
            let listeners = kinds.entry(member[2..].into()).or_default();
            let position = listeners.iter().position(|l| l.attribute);
            if let Some(HostValue::JavaScriptFunction(callback)) = arguments.first() {
                if let Some(position) = position {
                    let listener = &mut listeners[position];
                    if listener.callback != *callback {
                        listener.active.set(false);
                        listener.active = Rc::new(Cell::new(true));
                        listener.callback = callback.clone();
                    }
                } else {
                    listeners.push(Listener {
                        active: Rc::new(Cell::new(true)),
                        target: receiver,
                        kind: member[2..].into(),
                        callback: callback.clone(),
                        capture: false,
                        attribute: true,
                        once: false,
                        passive: false,
                    });
                }
            } else if let Some(position) = position {
                listeners.remove(position).active.set(false);
            }
            if listeners.is_empty() {
                kinds.remove(&member[2..]);
            }
            if kinds.is_empty() {
                self.listeners.get_mut().remove(&receiver);
            }
            return HostResult::default();
        }
        // cpp: webapi/dom_bindings.cc:499-555
        if receiver == 0 {
            return Self::unhandled();
        }
        let Some(index) = self.node(receiver) else {
            return Self::unhandled();
        };
        let owner = self.document.borrow();
        let node = owner.GetDocument().Node(index);
        if member == "checked" {
            if !node.IsHTMLElement("input") {
                return Self::type_error("checked is only writable on input elements");
            }
            let Some(HostValue::Boolean(value)) = arguments.first() else {
                return Self::type_error("checked requires a boolean");
            };
            let value = *value;
            drop(owner);
            (self.emit_mutation)(&DOMMutation {
                mutation_type: DOMMutationType::kSetControlChecked,
                target_node_id: receiver,
                bool_value: value,
                ..Default::default()
            });
            return Self::value(HostValue::Boolean(value));
        }
        if !matches!(
            member,
            "textContent" | "data" | "nodeValue" | "innerHTML" | "value" | "id" | "className"
        ) {
            return Self::unhandled();
        }
        let Some(value) = dom_string_argument(arguments.first()) else {
            return Self::type_error(&format!("{member} requires a string-convertible value"));
        };
        let mutation_type = match member {
            "textContent" | "data" | "nodeValue" => {
                if matches!(
                    node.Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentType
                ) || (member != "textContent" && !is_character_data(node.Type()))
                {
                    return Self::value(HostValue::String(value));
                }
                DOMMutationType::kSetTextContent
            }
            "innerHTML" => {
                if !matches!(
                    node.Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                ) {
                    return Self::type_error(
                        "innerHTML is only writable on elements and fragments",
                    );
                }
                DOMMutationType::kSetInnerHTML
            }
            "value" => {
                if !is_control(node) {
                    return Self::type_error("value is only writable on form controls");
                }
                DOMMutationType::kSetControlValue
            }
            "id" | "className" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("id is only writable on elements");
                }
                DOMMutationType::kSetAttribute
            }
            _ => return Self::type_error("property is not writable"),
        };
        drop(owner);
        (self.emit_mutation)(&DOMMutation {
            mutation_type,
            target_node_id: receiver,
            name: match member {
                "id" => "id",
                "className" => "class",
                _ => "",
            }
            .into(),
            value: value.clone(),
            ..Default::default()
        });
        Self::value(HostValue::String(value))
    }

    fn call(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        if member == "__domCollectionBrand" {
            return self.collections.borrow().get(&receiver).map_or_else(
                || Self::type_error("Illegal invocation"),
                |collection| {
                    Self::value(HostValue::String(
                        if matches!(collection.kind.as_str(), "snapshot" | "childNodes") {
                            "NodeList"
                        } else {
                            "HTMLCollection"
                        }
                        .into(),
                    ))
                },
            );
        }
        if let Some(result) = self.CallDOMService(receiver, member, arguments) {
            return result;
        }
        // cpp: webapi/dom_bindings.cc:743-775
        if matches!(member, "addEventListener" | "removeEventListener") {
            let (Some(HostValue::String(kind)), Some(HostValue::JavaScriptFunction(callback))) =
                (arguments.first(), arguments.get(1))
            else {
                return Self::type_error(&format!("{member} requires an event type and function"));
            };
            let capture = match arguments.get(2) {
                None => false,
                Some(HostValue::Boolean(capture)) => *capture,
                _ => return Self::type_error("listener options currently require a boolean"),
            };
            // Only this target/type bucket can contain an equal registration.
            // Function identity is checked at the public add/remove boundary,
            // never by the dispatch snapshot's per-listener liveness check.
            let matches =
                |l: &Listener| l.capture == capture && !l.attribute && l.callback == *callback;
            if member == "removeEventListener" {
                let targets = self.listeners.get_mut();
                if let Some(kinds) = targets.get_mut(&receiver) {
                    if let Some(listeners) = kinds.get_mut(kind) {
                        listeners.retain(|l| {
                            if matches(l) {
                                l.active.set(false);
                                false
                            } else {
                                true
                            }
                        });
                        if listeners.is_empty() {
                            kinds.remove(kind);
                        }
                    }
                    if kinds.is_empty() {
                        targets.remove(&receiver);
                    }
                }
            } else {
                if self
                    .listeners
                    .get_mut()
                    .get(&receiver)
                    .and_then(|kinds| kinds.get(kind))
                    .is_some_and(|listeners| listeners.iter().any(matches))
                {
                    return HostResult::default();
                }
                let once = match arguments.get(3) {
                    None => false,
                    Some(HostValue::Boolean(b)) => *b,
                    _ => return Self::type_error("bad_variant_access"),
                };
                let passive = match arguments.get(4) {
                    None => false,
                    Some(HostValue::Boolean(b)) => *b,
                    _ => return Self::type_error("bad_variant_access"),
                };
                self.listeners
                    .get_mut()
                    .entry(receiver)
                    .or_default()
                    .entry(kind.clone())
                    .or_default()
                    .push(Listener {
                        active: Rc::new(Cell::new(true)),
                        target: receiver,
                        kind: kind.clone(),
                        callback: callback.clone(),
                        capture,
                        attribute: false,
                        once,
                        passive,
                    });
            }
            return HostResult::default();
        }
        // cpp: webapi/dom_bindings.cc:650-665
        if receiver == 0 && member == "cssSupports" {
            let (Some(HostValue::String(property)), Some(HostValue::String(value))) =
                (arguments.first(), arguments.get(1))
            else {
                return Self::value(HostValue::Boolean(false));
            };
            let supported = !value.is_empty()
                && match property.as_str() {
                    "color" | "background-color" | "border-color" => {
                        value == "currentcolor"
                            || layoutng_assembly::css_color_parser::ParseCSSColor(value).is_some()
                    }
                    "width" | "height" | "font-size" | "margin-left" | "padding-left" => {
                        value == "auto"
                            || css_parser::length_percentage_parser::ParseLengthPercentage(
                                value, 16.0,
                            )
                            .is_some()
                    }
                    "display" => matches!(
                        value.as_str(),
                        "none"
                            | "block"
                            | "inline"
                            | "flex"
                            | "grid"
                            | "inline-block"
                            | "contents"
                            | "table"
                    ),
                    _ => false,
                };
            return Self::value(HostValue::Boolean(supported));
        }
        // cpp: webapi/dom_bindings.cc:701-742
        if let Some(state) = self.iterators.borrow_mut().get_mut(&receiver) {
            if member == "@@iterator" {
                return Self::value(HostValue::Object(HostObjectRef { id: receiver }));
            }
            if member == "next" {
                let collections = self.collections.borrow();
                let nodes = self.collection_nodes(&collections[&state.collection]);
                let done = state.done || state.index >= nodes.len();
                state.done = done;
                let value = if done {
                    HostValue::default()
                } else {
                    let id = nodes[state.index];
                    state.index += 1;
                    HostValue::Object(HostObjectRef { id })
                };
                let id = self.allocate_collection_id();
                self.records.borrow_mut().insert(
                    id,
                    HashMap::from([
                        ("done".into(), HostValue::Boolean(done)),
                        ("value".into(), value),
                    ]),
                );
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            return Self::type_error("unknown iterator method");
        }
        if let Some(collection) = self.collections.borrow().get(&receiver) {
            if member == "@@iterator" {
                let id = self.allocate_collection_id();
                self.iterators.borrow_mut().insert(
                    id,
                    CollectionIterator {
                        collection: receiver,
                        index: 0,
                        done: false,
                    },
                );
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            let nodes = self.collection_nodes(collection);
            if member == "item" {
                if let Some(HostValue::Number(index)) = arguments.first() {
                    if *index >= 0.0 && *index < nodes.len() as f64 {
                        return Self::value(HostValue::Object(HostObjectRef {
                            id: nodes[*index as usize],
                        }));
                    }
                }
                return Self::value(HostValue::Null(JavaScriptNull));
            }
            if member == "namedItem" {
                if let Some(HostValue::String(name)) = arguments
                    .first()
                    .filter(|a| matches!(a,HostValue::String(s) if !s.is_empty()))
                {
                    let owner = self.document.borrow();
                    let tree = owner.GetDocument();
                    for &id in nodes.iter() {
                        let node = tree.Node(tree.FindNodeById(id).unwrap());
                        if node.FindAttribute("id").is_some_and(|a| a.value == *name)
                            || node.FindAttribute("name").is_some_and(|a| a.value == *name)
                        {
                            return Self::value(HostValue::Object(HostObjectRef { id }));
                        }
                    }
                }
                return Self::value(HostValue::Null(JavaScriptNull));
            }
            return Self::type_error("unknown collection method");
        }
        // cpp: webapi/dom_bindings.cc:737-759
        if receiver == 0 {
            return match member {
                "__domInstall" => {
                    if let [HostValue::String(name), HostValue::JavaScriptValue(value), ..] =
                        arguments
                    {
                        self.prototypes.insert(name.clone(), value.clone());
                        HostResult::default()
                    } else {
                        Self::type_error("prototype registration requires name and object")
                    }
                }
                "__domWrap" => {
                    if let [HostValue::String(name), HostValue::JavaScriptFunction(value), ..] =
                        arguments
                    {
                        self.method_wrappers.insert(name.clone(), value.clone());
                        if name == "__eventState" {
                            *self.services.event_state.borrow_mut() = Some(value.clone());
                        }
                        HostResult::default()
                    } else {
                        Self::type_error("method registration requires name and function")
                    }
                }
                "__domInvoke" => {
                    if let [HostValue::Object(object), HostValue::String(name), rest @ ..] =
                        arguments
                    {
                        self.call(object.id, name, rest)
                    } else {
                        Self::type_error("Illegal invocation")
                    }
                }
                _ => Self::unhandled(),
            };
        }
        // cpp: webapi/dom_bindings.cc:816-823
        if member == "write" {
            let is_document = self.node(receiver).is_some_and(|index| {
                self.document.borrow().GetDocument().Node(index).Type() == DOMNodeType::kDocument
            });
            if is_document {
                let Some(HostValue::String(markup)) = arguments.first() else {
                    return Self::type_error("document.write requires a string");
                };
                if self
                    .write_document
                    .as_mut()
                    .is_some_and(|write| write(markup))
                {
                    return HostResult::default();
                }
                return Self::type_error(
                    "document.write requires an active parser insertion point",
                );
            }
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(receiver) else {
            return Self::type_error("host object no longer exists");
        };
        let node = document.Node(index);
        match member {
            "getTotalLength"
                if node.Type() == DOMNodeType::kElement
                    && node.Namespace() == DOMNamespace::kSVG
                    && node.Name() == "path" =>
            {
                let Some(data) = node.FindAttribute("d").map(|attribute| attribute.value.as_str())
                else {
                    return Self::value(HostValue::Number(0.0));
                };
                let Some(parsed) = dom::svg_path_parser::ParseSVGPathDefault(data) else {
                    return Self::value(HostValue::Number(0.0));
                };
                let mut path = skia::PathBuilder::new();
                for command in parsed.commands {
                    match command.verb {
                        PaintPathVerb::kMoveTo => path.move_to(
                            command.point.x as f32,
                            command.point.y as f32,
                        ),
                        PaintPathVerb::kLineTo => path.line_to(
                            command.point.x as f32,
                            command.point.y as f32,
                        ),
                        PaintPathVerb::kQuadraticTo => path.quad_to(
                            command.control1.x as f32,
                            command.control1.y as f32,
                            command.point.x as f32,
                            command.point.y as f32,
                        ),
                        PaintPathVerb::kCubicTo => path.cubic_to(
                            command.control1.x as f32,
                            command.control1.y as f32,
                            command.control2.x as f32,
                            command.control2.y as f32,
                            command.point.x as f32,
                            command.point.y as f32,
                        ),
                        PaintPathVerb::kClose => path.close(),
                        PaintPathVerb::kConicTo => {
                            return Self::type_error("invalid SVG path geometry")
                        }
                    }
                }
                let length = path.finish().map_or(0.0, |path| {
                    f64::from(skia::src::core::SkContourMeasure::path_length(&path))
                });
                Self::value(HostValue::Number(length))
            }
            "contains" => {
                let Some(HostValue::Object(other)) = arguments.first() else {
                    return Self::value(HostValue::Boolean(false));
                };
                let Some(mut other) = document.FindNodeById(other.id) else {
                    return Self::value(HostValue::Boolean(false));
                };
                loop {
                    if other == index {
                        return Self::value(HostValue::Boolean(true));
                    }
                    let Some(parent) = document.Node(other).Parent() else {
                        return Self::value(HostValue::Boolean(false));
                    };
                    other = parent;
                }
            }
            "__sibling" => {
                let [HostValue::Boolean(reverse), HostValue::Boolean(elements), ..] = arguments
                else {
                    return Self::type_error(
                        "sibling lookup requires direction and element filter",
                    );
                };
                let Some(parent) = node.Parent() else {
                    return Self::node_reference(document, None);
                };
                // Blink Node::nextSibling/previousSibling read sibling pointers
                // (core/dom/node.h). This arena stores children in a Vec; scan
                // IDs directly instead of enumerating a JS NodeList and creating
                // an iterator result and wrapper lookup for every sibling.
                let siblings = document.Node(parent).Children();
                let Some(position) = siblings.iter().position(|&sibling| sibling == index) else {
                    return Self::node_reference(document, None);
                };
                let matches = |sibling: &usize| {
                    !*elements || document.Node(*sibling).Type() == DOMNodeType::kElement
                };
                let sibling = if *reverse {
                    siblings[..position].iter().rev().copied().find(matches)
                } else {
                    siblings[position + 1..].iter().copied().find(matches)
                };
                Self::node_reference(document, sibling)
            }
            // cpp: webapi/dom_bindings.cc:833-846
            "stylesheetCount" => Self::value(HostValue::Number(
                document.StyleSheetsForNode(index).len() as f64,
            )),
            "stylesheetOwner" | "stylesheetRuleCount" | "stylesheetRuleText" => {
                let sheets = document.StyleSheetsForNode(index);
                let Some(HostValue::Number(sheet_index)) = arguments.first() else {
                    return Self::value(HostValue::Null(Default::default()));
                };
                if *sheet_index < 0.0 || *sheet_index >= sheets.len() as f64 {
                    return Self::value(HostValue::Null(Default::default()));
                }
                let sheet = &sheets[*sheet_index as usize];
                if member == "stylesheetOwner" {
                    return Self::value(if sheet.owner_node_id != 0 {
                        HostValue::Object(HostObjectRef {
                            id: sheet.owner_node_id,
                        })
                    } else {
                        HostValue::Null(Default::default())
                    });
                }
                if member == "stylesheetRuleCount" {
                    return Self::value(HostValue::Number(sheet.rules.len() as f64));
                }
                let Some(HostValue::Number(rule_index)) = arguments.get(1) else {
                    return Self::value(HostValue::String(String::new()));
                };
                if *rule_index < 0.0 || *rule_index >= sheet.rules.len() as f64 {
                    return Self::value(HostValue::String(String::new()));
                }
                let rule = &sheet.rules[*rule_index as usize];
                let mut text = rule.selector_text.clone() + " {";
                for declaration in &rule.declarations {
                    text.push_str(&declaration.property);
                    text.push(':');
                    text.push_str(&declaration.value);
                    text.push(';');
                }
                text.push('}');
                Self::value(HostValue::String(text))
            }
            // cpp: webapi/dom_bindings.cc:847-869
            "stylesheetInsertRule" | "stylesheetDeleteRule" => {
                let sheets = document.StyleSheetsForNode(index);
                let (Some(HostValue::Number(sheet_index)), Some(HostValue::Number(rule_index))) =
                    (arguments.first(), arguments.get(1))
                else {
                    return Self::type_error("Invalid stylesheet index");
                };
                if *sheet_index < 0.0 || *sheet_index >= sheets.len() as f64 || *rule_index < 0.0 {
                    return Self::type_error("Invalid stylesheet index");
                }
                let mut sheet = sheets[*sheet_index as usize].clone();
                if *rule_index > sheet.rules.len() as f64
                    || (member == "stylesheetDeleteRule" && *rule_index == sheet.rules.len() as f64)
                {
                    return HostResult::Failure(
                        javascript::javascript_runtime::JavaScriptExceptionKind::kRangeError,
                        "CSS rule index out of bounds",
                    );
                }
                let index = *rule_index as usize;
                if member == "stylesheetInsertRule" {
                    let text = match arguments.get(2) {
                        Some(HostValue::String(text)) => text.as_str(),
                        _ => "",
                    };
                    let mut parsed = cssom::ParseCSS(text);
                    if parsed.rules.len() != 1 || !parsed.font_faces.is_empty() {
                        return HostResult::Failure(
                            javascript::javascript_runtime::JavaScriptExceptionKind::kSyntaxError,
                            "Expected one CSS style rule",
                        );
                    }
                    sheet.rules.insert(index, parsed.rules.remove(0));
                } else {
                    sheet.rules.remove(index);
                }
                drop(owner);
                if let Some(emit) = self.services.host.emit_style_sheet.as_mut() {
                    emit(sheet);
                }
                Self::value(HostValue::Number(index as f64))
            }
            // cpp: webapi/dom_bindings.cc:779-804
            "serializeNode" => {
                let mut output = String::new();
                serialize_node(document, index, &mut output);
                Self::value(HostValue::String(output))
            }
            "attributeList" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("attributes requires an element receiver");
                }
                let list = self.allocate_collection_id();
                let mut fields = HashMap::from([(
                    "length".into(),
                    HostValue::Number(node.Attributes().len() as f64),
                )]);
                for (position, attribute) in node.Attributes().iter().enumerate() {
                    let id = self.allocate_collection_id();
                    let name = if attribute.prefix.is_empty() {
                        attribute.local_name.clone()
                    } else {
                        format!("{}:{}", attribute.prefix, attribute.local_name)
                    };
                    self.records.borrow_mut().insert(
                        id,
                        HashMap::from([
                            ("name".into(), HostValue::String(name)),
                            (
                                "localName".into(),
                                HostValue::String(attribute.local_name.clone()),
                            ),
                            (
                                "namespaceURI".into(),
                                if attribute.namespace_uri.is_empty() {
                                    HostValue::Null(JavaScriptNull)
                                } else {
                                    HostValue::String(attribute.namespace_uri.clone())
                                },
                            ),
                            (
                                "prefix".into(),
                                if attribute.prefix.is_empty() {
                                    HostValue::Null(JavaScriptNull)
                                } else {
                                    HostValue::String(attribute.prefix.clone())
                                },
                            ),
                            ("value".into(), HostValue::String(attribute.value.clone())),
                        ]),
                    );
                    fields.insert(
                        position.to_string(),
                        HostValue::Object(HostObjectRef { id }),
                    );
                }
                self.records.borrow_mut().insert(list, fields);
                Self::value(HostValue::Object(HostObjectRef { id: list }))
            }
            // cpp: webapi/dom_bindings.cc:805-811,817-827
            "createHTMLDocument" | "cloneNode" => {
                let mut mutation = DOMMutation {
                    mutation_type: if member == "cloneNode" {
                        DOMMutationType::kCloneNode
                    } else {
                        DOMMutationType::kCreateDocument
                    },
                    target_node_id: receiver,
                    child_node_id: document.NextNodeId(),
                    bool_value: if member == "cloneNode" {
                        matches!(arguments.first(), Some(HostValue::Boolean(true)))
                    } else {
                        !arguments.is_empty()
                    },
                    ..Default::default()
                };
                if let Some(HostValue::String(title)) = arguments.first() {
                    mutation.value = title.clone();
                }
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation)(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            "hasFocus" => Self::value(HostValue::Boolean(true)),
            // cpp: webapi/dom_bindings.cc:861-883
            "createElement"
            | "createElementNS"
            | "createTextNode"
            | "createComment"
            | "createDocumentFragment" => {
                let mut mutation = DOMMutation {
                    mutation_type: DOMMutationType::kCreateElement,
                    target_node_id: if node.Type() == DOMNodeType::kDocument {
                        receiver
                    } else {
                        node.OwnerDocumentNode().unwrap().Id()
                    },
                    child_node_id: document.NextNodeId(),
                    ..Default::default()
                };
                match member {
                    "createTextNode" | "createComment" => {
                        mutation.mutation_type = if member == "createTextNode" {
                            DOMMutationType::kCreateText
                        } else {
                            DOMMutationType::kCreateComment
                        };
                        if let Some(HostValue::String(text)) = arguments.first() {
                            mutation.value = text.clone();
                        }
                    }
                    "createDocumentFragment" => {
                        mutation.mutation_type = DOMMutationType::kCreateFragment
                    }
                    _ => {
                        let Some(HostValue::String(name)) =
                            arguments.get(usize::from(member == "createElementNS"))
                        else {
                            return Self::type_error("element name is required");
                        };
                        if name.is_empty() {
                            return Self::type_error("element name is required");
                        }
                        mutation.name = if member == "createElement" {
                            name.to_ascii_lowercase()
                        } else {
                            name.clone()
                        };
                        if member == "createElementNS" {
                            if let Some(HostValue::String(ns)) = arguments.first() {
                                mutation.namespace_uri = ns.clone();
                            }
                        }
                    }
                }
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation)(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            // cpp: webapi/dom_bindings.cc:884-898
            "appendChild" | "insertBefore" | "removeChild" => {
                let Some(HostValue::Object(child)) = arguments.first() else {
                    return Self::type_error("child must be a Node");
                };
                let child = *child;
                let before_node_id = if member == "insertBefore" {
                    if let Some(HostValue::Object(before)) = arguments.get(1) {
                        before.id
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mutation = DOMMutation {
                    mutation_type: match member {
                        "appendChild" => DOMMutationType::kAppendChild,
                        "insertBefore" => DOMMutationType::kInsertBefore,
                        _ => DOMMutationType::kRemoveChild,
                    },
                    target_node_id: receiver,
                    child_node_id: child.id,
                    before_node_id,
                    ..Default::default()
                };
                drop(owner);
                if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (self.emit_mutation)(&mutation)
                })) {
                    if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
                        return Self::type_error(error.message);
                    }
                    std::panic::resume_unwind(payload);
                }
                Self::value(HostValue::Object(child))
            }
            // cpp: webapi/dom_bindings.cc:899-921
            "getStyle" | "setStyle" => {
                let Some(HostValue::String(property)) = arguments.first() else {
                    return Self::type_error("style property name is required");
                };
                let mut declarations = cssom::ParseCSSDeclarationList(
                    node.FindAttribute("style").map_or("", |a| a.value.as_str()),
                );
                if member == "getStyle" {
                    return Self::value(HostValue::String(
                        declarations
                            .iter()
                            .rev()
                            .find(|d| d.property == *property)
                            .map_or_else(String::new, |d| d.value.clone()),
                    ));
                }
                let Some(HostValue::String(value)) = arguments.get(1) else {
                    return Self::type_error("style value is required");
                };
                declarations.retain(|d| d.property != *property);
                if !value.is_empty() {
                    declarations.push(cssom::CSSDeclaration {property:property.clone(),value:value.clone(),important:matches!(arguments.get(2),Some(HostValue::String(p)) if p=="important")});
                }
                let value = declarations
                    .iter()
                    .map(|d| {
                        format!(
                            "{}: {}{};",
                            d.property,
                            d.value,
                            if d.important { " !important" } else { "" }
                        )
                    })
                    .collect::<String>();
                drop(owner);
                (self.emit_mutation)(&DOMMutation {
                    mutation_type: DOMMutationType::kSetAttribute,
                    target_node_id: receiver,
                    name: "style".into(),
                    value,
                    ..Default::default()
                });
                HostResult::default()
            }
            // cpp: webapi/dom_bindings.cc:922-949
            "querySelectorAll" | "getElementsByTagName" | "getElementsByClassName" => {
                let Some(HostValue::String(query)) = arguments.first() else {
                    return Self::type_error(&format!("{member} requires a string"));
                };
                Self::value(HostValue::Object(self.collection(
                    receiver,
                    if member == "querySelectorAll" {
                        "snapshot"
                    } else {
                        member
                    },
                    query,
                )))
            }
            "querySelector" | "matches" | "closest" => {
                let Some(HostValue::String(selector)) = arguments.first() else {
                    return Self::type_error(&format!("{member} requires a selector"));
                };
                let matches = |candidate| {
                    dom::style_resolver::persistent_selector::MatchesSelector(
                        document, candidate, selector,
                    )
                };
                if member == "matches" {
                    return Self::value(HostValue::Boolean(matches(index)));
                }
                let found = if member == "closest" {
                    let mut candidate = Some(index);
                    let mut result = None;
                    while let Some(node) = candidate {
                        if matches(node) {
                            result = Some(node);
                            break;
                        }
                        candidate = document.Node(node).Parent();
                    }
                    result
                } else {
                    find_selector(document, index, selector)
                };
                found.map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |node| {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: document.Node(node).Id(),
                        }))
                    },
                )
            }
            // cpp: webapi/dom_bindings.cc:950-962
            "getElementById" => {
                if node.Type() != DOMNodeType::kDocument {
                    return Self::type_error("getElementById requires a document receiver");
                }
                let Some(HostValue::String(id)) = arguments.first() else {
                    return Self::type_error("getElementById requires a string");
                };
                let found = find_element_by_id(document, document.Root(), id);
                found.map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |found| {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: document.Node(found).Id(),
                        }))
                    },
                )
            }
            // cpp: webapi/dom_bindings.cc:963-995
            "getAttribute" | "hasAttribute" | "removeAttribute" | "setAttribute" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("attribute method requires an element receiver");
                }
                let name = if member == "setAttribute" {
                    dom_string_argument(arguments.first())
                } else {
                    match arguments.first() {
                        Some(HostValue::String(name)) => Some(name.clone()),
                        _ => None,
                    }
                };
                let Some(name) = name else {
                    return Self::type_error(if member == "setAttribute" {
                        "setAttribute requires string-convertible name and value"
                    } else {
                        "attribute method requires a name"
                    });
                };
                let attribute = node
                    .FindAttribute(&name)
                    .map(|attribute| attribute.value.clone());
                if member == "getAttribute" {
                    return Self::value(
                        attribute.map_or(HostValue::Null(JavaScriptNull), HostValue::String),
                    );
                }
                if member == "hasAttribute" {
                    return Self::value(HostValue::Boolean(attribute.is_some()));
                }
                let value = if member == "setAttribute" {
                    let Some(value) = dom_string_argument(arguments.get(1)) else {
                        return Self::type_error(
                            "setAttribute requires string-convertible name and value",
                        );
                    };
                    value
                } else {
                    String::new()
                };
                // The emitter synchronously applies the mutation to this
                // arena; release the read borrow before invoking it.
                drop(owner);
                (self.emit_mutation)(&DOMMutation {
                    mutation_type: if member == "setAttribute" {
                        DOMMutationType::kSetAttribute
                    } else {
                        DOMMutationType::kRemoveAttribute
                    },
                    target_node_id: receiver,
                    name,
                    value,
                    ..Default::default()
                });
                HostResult::default()
            }
            _ => Self::type_error("host method is not installed"),
        }
    }
}

impl JavaScriptHostBindings for DOMJavaScriptBindings {
    fn PrepareInvocation(&mut self, call: &HostCall<'_>) -> Option<HostContinuation> {
        // Reads never dispatch JS or emit a mutation through this seam. Leave
        // their receiver validation to get(), instead of looking up every DOM
        // Proxy receiver once here and again in the actual property read.
        if call.symbol != HostSymbol::kNone || call.operation == HostOperation::kGet {
            return None;
        }
        let (id, member) = if call.operation == HostOperation::kCall
            && call.receiver == 0
            && call.member == "__domInvoke"
        {
            if let [HostValue::Object(o), HostValue::String(n), ..] = call.arguments {
                (o.id, n.as_str())
            } else {
                return None;
            }
        } else {
            (call.receiver, call.member)
        };
        // This internal getter only reads the tree. Validate its receiver in
        // call(), without a second node lookup or preparing a mutation emitter.
        if call.operation == HostOperation::kCall && member == "__sibling" {
            return None;
        }
        // cpp: webapi/dom_bindings.cc:634-653
        // Synthetic dispatch can call back into this host. Prepare under the
        // host borrow, then enter Interaction on the active runtime after it
        // has been released (including Window's delegating host borrow).
        if call.operation == HostOperation::kCall && member == "dispatchEvent" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            return Some(self.PrepareSyntheticDispatch(id, arguments));
        }
        // Preserve call routing: collections/bound events/invalid objects must
        // go through Invoke; source Node(receiver) precedes focus/blur.
        // cpp: webapi/dom_bindings.cc:773-774
        if self.node(id).is_none() {
            return None;
        }
        // cpp: webapi/dom_bindings.cc:818-821
        // Copy callback out of bindings
        //
        // caller executes only after host/window/DOM borrows are released.
        if call.operation == HostOperation::kCall && matches!(member, "focus" | "blur") {
            let handler = self.change_focus.clone();
            let focus = member == "focus";
            return Some(Box::new(move |runtime| {
                if let Some(handler) = handler {
                    handler(id, focus, runtime);
                }
                HostResult::default()
            }));
        }
        if call.operation == HostOperation::kCall && matches!(member, "submit" | "requestSubmit") {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let submitter = arguments.first().and_then(|value| {
                if let HostValue::Object(object) = value {
                    Some(object.id)
                } else {
                    None
                }
            });
            let handler = self.services.host.submit_form.clone();
            let dispatch_event = member == "requestSubmit";
            return Some(Box::new(move |runtime| {
                if let Some(handler) = handler {
                    handler(id, submitter, dispatch_event, runtime);
                }
                HostResult::default()
            }));
        }
        let emitter = self.scoped_emit_mutation.clone()?;
        let emits = match call.operation {
            HostOperation::kSet => matches!(
                member,
                "checked"
                    | "textContent"
                    | "data"
                    | "nodeValue"
                    | "innerHTML"
                    | "value"
                    | "id"
                    | "className"
            ),
            HostOperation::kCall => matches!(
                member,
                "createHTMLDocument"
                    | "cloneNode"
                    | "createElement"
                    | "createElementNS"
                    | "createTextNode"
                    | "createComment"
                    | "createDocumentFragment"
                    | "appendChild"
                    | "insertBefore"
                    | "removeChild"
                    | "setStyle"
                    | "setAttribute"
                    | "removeAttribute"
            ),
            _ => false,
        };
        if !emits {
            return None;
        }
        // Each of these source Set/Call branches checks arguments, constructs
        // one mutation, emits it, then returns a value independent of the DOM.
        // Reuse that exact argument/return-value logic; no tree mutation or
        // connected-subtree preparation occurs under the bindings borrow.
        let mutation = Rc::new(RefCell::new(None));
        let pending = mutation.clone();
        let original = std::mem::replace(
            &mut self.emit_mutation,
            Box::new(move |m| {
                assert!(
                    pending.borrow().is_none(),
                    "a DOM host invocation emits one mutation"
                );
                *pending.borrow_mut() = Some(m.clone());
            }),
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.Invoke(call)));
        self.emit_mutation = original;
        let result = result.unwrap_or_else(|error| std::panic::resume_unwind(error));
        let mutation = mutation.borrow_mut().take();
        let catches_errors = matches!(member, "appendChild" | "insertBefore" | "removeChild");
        Some(Box::new(move |runtime| {
            if let Some(mutation) = mutation {
                let emitted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    emitter(&mutation, runtime);
                }));
                if let Err(error) = emitted {
                    if catches_errors {
                        if let Some(error) = error.downcast_ref::<dom::error::DOMException>() {
                            return Self::type_error(error.message);
                        }
                        if let Some(error) = error.downcast_ref::<std::io::Error>() {
                            return Self::type_error(&error.to_string());
                        }
                    }
                    std::panic::resume_unwind(error);
                }
            }
            result
        }))
    }
    // cpp: webapi/dom_bindings.cc:243-264
    fn GlobalNames(&self) -> Vec<String> {
        GLOBAL_NAMES.iter().map(|name| (*name).into()).collect()
    }

    fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
        if call.symbol != HostSymbol::kNone {
            if call.operation == HostOperation::kGet
                && call.symbol == HostSymbol::kIterator
                && (self.collections.borrow().contains_key(&call.receiver)
                    || self.iterators.borrow().contains_key(&call.receiver))
            {
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver: call.receiver,
                    name: "@@iterator".into(),
                }));
            }
            return Self::unhandled();
        }
        match call.operation {
            HostOperation::kGet => self.get(call.receiver, call.member),
            HostOperation::kSet => self.set(call.receiver, call.member, call.arguments),
            HostOperation::kCall => self.call(call.receiver, call.member, call.arguments),
            HostOperation::kConstruct => Self::type_error("DOM constructors are not installed yet"),
        }
    }

    fn PrototypeFor(&mut self, id: HostObjectId) -> HostResult {
        if self.services.IsEvent(id) {
            return self
                .prototypes
                .get("Event")
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        if let Some(collection) = self.collections.borrow().get(&id) {
            let name = if matches!(collection.kind.as_str(), "snapshot" | "childNodes") {
                "NodeList"
            } else {
                "HTMLCollection"
            };
            return self
                .prototypes
                .get(name)
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        // cpp: webapi/dom_bindings.cc:1043-1094
        let document = self.document.borrow();
        let document = document.GetDocument();
        let Some(index) = document.FindNodeById(id) else {
            return Self::unhandled();
        };
        let node = document.Node(index);
        let prototype = match node.Type() {
            DOMNodeType::kDocument => "Document",
            DOMNodeType::kElement => match node.Namespace() {
                DOMNamespace::kHTML => match node.Name() {
                    "input" => "HTMLInputElement",
                    "textarea" => "HTMLTextAreaElement",
                    "button" => "HTMLButtonElement",
                    "form" => "HTMLFormElement",
                    "select" => "HTMLSelectElement",
                    "option" => "HTMLOptionElement",
                    "label" => "HTMLLabelElement",
                    "details" => "HTMLDetailsElement",
                    "dialog" => "HTMLDialogElement",
                    "script" => "HTMLScriptElement",
                    "style" => "HTMLStyleElement",
                    "link" => "HTMLLinkElement",
                    "meta" => "HTMLMetaElement",
                    "iframe" => "HTMLIFrameElement",
                    "img" => "HTMLImageElement",
                    "template" => "HTMLTemplateElement",
                    "a" => "HTMLAnchorElement",
                    "video" => "HTMLVideoElement",
                    "audio" => "HTMLAudioElement",
                    _ => "HTMLElement",
                },
                DOMNamespace::kSVG => match node.Name() {
                    "path" => "SVGPathElement",
                    _ => "SVGElement",
                },
                DOMNamespace::kMathML => "MathMLElement",
                DOMNamespace::kNone => "Element",
            },
            DOMNodeType::kText => "Text",
            DOMNodeType::kComment => "Comment",
            DOMNodeType::kDocumentFragment => "DocumentFragment",
            _ => "Node",
        };
        self.prototypes
            .get(prototype)
            .cloned()
            .map_or_else(Self::unhandled, |value| {
                Self::value(HostValue::JavaScriptValue(value))
            })
    }
}

fn is_method(node_type: DOMNodeType, member: &str) -> bool {
    if matches!(member, "addEventListener" | "removeEventListener") {
        return matches!(node_type, DOMNodeType::kDocument | DOMNodeType::kElement);
    }
    if matches!(
        member,
        "querySelector" | "querySelectorAll" | "getElementsByTagName" | "getElementsByClassName"
    ) {
        return matches!(
            node_type,
            DOMNodeType::kDocument | DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        );
    }
    if matches!(member, "matches" | "closest") {
        return node_type == DOMNodeType::kElement;
    }
    if member == "getElementById" {
        return node_type == DOMNodeType::kDocument;
    }
    if matches!(
        member,
        "getAttribute" | "hasAttribute" | "setAttribute" | "removeAttribute"
    ) {
        return node_type == DOMNodeType::kElement;
    }
    false
}

// cpp: webapi/dom_bindings.cc:189-197
fn is_event_handler(name: &str) -> bool {
    matches!(
        name,
        "onload"
            | "onerror"
            | "onabort"
            | "oninput"
            | "onchange"
            | "onclick"
            | "onkeydown"
            | "onkeyup"
            | "onkeypress"
            | "onfocus"
            | "onblur"
            | "onsubmit"
            | "onreset"
            | "onmousedown"
            | "onmouseup"
            | "onmousemove"
            | "onreadystatechange"
    )
}

fn is_character_data(node_type: DOMNodeType) -> bool {
    matches!(
        node_type,
        DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
    )
}

impl DOMJavaScriptBindings {
    fn allocate_collection_id(&self) -> HostObjectId {
        let id = self.next_collection_id.get();
        self.next_collection_id.set(id + 1);
        id
    }
    // cpp: webapi/dom_bindings.cc:990-1006
    fn collection(&self, root: HostObjectId, kind: &str, query: &str) -> HostObjectRef {
        if kind != "snapshot" {
            for (&id, collection) in self.collections.borrow().iter() {
                if collection.root == root && collection.kind == kind && collection.query == query {
                    return HostObjectRef { id };
                }
            }
        }
        let mut collection = NodeCollection {
            root,
            kind: kind.into(),
            query: query.into(),
            snapshot: Vec::new(),
            membership_cache: RefCell::new(None),
        };
        if kind == "snapshot" {
            collection.kind = "query".into();
            collection.snapshot = self.collection_nodes(&collection).into_owned();
            collection.kind = "snapshot".into();
        }
        let id = self.allocate_collection_id();
        self.collections.borrow_mut().insert(id, collection);
        HostObjectRef { id }
    }
    // cpp: webapi/dom_bindings.cc:1008-1037
    fn collection_nodes<'a>(&self, collection: &'a NodeCollection) -> CollectionNodes<'a> {
        if collection.kind == "snapshot" {
            return CollectionNodes::Borrowed(&collection.snapshot);
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let cache_key = matches!(
            collection.kind.as_str(),
            "childNodes" | "children" | "getElementsByTagName" | "getElementsByClassName"
        )
        .then(|| document.CollectionMembershipKey())
        .flatten();
        if let Some(key) = cache_key {
            if let Some(cache) = collection
                .membership_cache
                .borrow()
                .as_ref()
                .filter(|cache| cache.key == key)
            {
                return CollectionNodes::Shared(cache.ids.clone());
            }
        }
        let Some(root) = document.FindNodeById(collection.root) else {
            return CollectionNodes::Owned(Vec::new());
        };
        fn visit(
            document: &Document,
            index: usize,
            collection: &NodeCollection,
            result: &mut Vec<HostObjectId>,
        ) {
            for &child in document.Node(index).Children() {
                let node = document.Node(child);
                let element = node.Type() == DOMNodeType::kElement;
                let matched = match collection.kind.as_str() {
                    "childNodes" => true,
                    "children" => element,
                    "getElementsByTagName" => {
                        element
                            && (collection.query == "*"
                                || node.Name() == collection.query
                                || (node.Namespace() == DOMNamespace::kHTML
                                    && node.Name() == collection.query.to_ascii_lowercase()))
                    }
                    "getElementsByClassName" => element && has_classes(node, &collection.query),
                    "query" => {
                        element
                            && dom::style_resolver::persistent_selector::MatchesSelector(
                                document,
                                child,
                                &collection.query,
                            )
                    }
                    _ => false,
                };
                if matched {
                    result.push(node.Id());
                }
                if collection.kind != "childNodes" && collection.kind != "children" {
                    visit(document, child, collection, result);
                }
            }
        }
        let mut result = Vec::new();
        visit(document, root, collection, &mut result);
        if let Some(key) = cache_key {
            let ids = Rc::new(result);
            *collection.membership_cache.borrow_mut() = Some(CollectionMembershipCache {
                key,
                ids: ids.clone(),
            });
            CollectionNodes::Shared(ids)
        } else {
            // Overflow, unknown kinds and transient query snapshots keep fresh traversal.
            collection.membership_cache.borrow_mut().take();
            CollectionNodes::Owned(result)
        }
    }
}
// cpp: webapi/dom_bindings.cc:199-213
fn has_classes(node: &dom::persistent_document::DOMNode, query: &str) -> bool {
    let Some(classes) = node.FindAttribute("class") else {
        return false;
    };
    let mut any = false;
    for word in query
        .split(|c: char| matches!(c, ' ' | '\t'..='\r'))
        .filter(|s| !s.is_empty())
    {
        any = true;
        if !classes
            .value
            .split_ascii_whitespace()
            .any(|candidate| candidate == word)
        {
            return false;
        }
    }
    any
}
// cpp: webapi/dom_bindings.cc:937-946
fn find_selector(document: &Document, index: usize, selector: &str) -> Option<usize> {
    for &child in document.Node(index).Children() {
        if dom::style_resolver::persistent_selector::MatchesSelector(document, child, selector) {
            return Some(child);
        }
        if let Some(found) = find_selector(document, child, selector) {
            return Some(found);
        }
    }
    None
}

fn is_control(node: &dom::persistent_document::DOMNode) -> bool {
    node.IsHTMLElement("input") || node.IsHTMLElement("textarea") || node.IsHTMLElement("select")
}

// cpp: webapi/dom_bindings.cc:55-64
fn append_descendant_text(document: &Document, node: usize, output: &mut String) {
    let node = document.Node(node);
    if node.Type() == DOMNodeType::kText {
        output.push_str(node.Data());
        return;
    }
    for &child in node.Children() {
        append_descendant_text(document, child, output);
    }
}

// cpp: webapi/dom_bindings.cc:66-75
fn text_content(document: &Document, node: usize) -> String {
    if is_character_data(document.Node(node).Type()) {
        return document.Node(node).Data().into();
    }
    let mut output = String::new();
    append_descendant_text(document, node, &mut output);
    output
}

// cpp: webapi/dom_bindings.cc:77-85
fn append_escaped(text: &str, output: &mut String, attribute: bool) {
    for character in text.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' if !attribute => output.push_str("&gt;"),
            '"' if attribute => output.push_str("&quot;"),
            _ => output.push(character),
        }
    }
}

// cpp: webapi/dom_bindings.cc:87-92
fn is_void_html_element(name: &str) -> bool {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ]
    .contains(&name)
}

// cpp: webapi/dom_bindings.cc:94-130
fn serialize_node(document: &Document, index: usize, output: &mut String) {
    let node = document.Node(index);
    if node.Type() == DOMNodeType::kText {
        append_escaped(node.Data(), output, false);
        return;
    }
    if node.Type() == DOMNodeType::kComment {
        output.push_str("<!--");
        output.push_str(node.Data());
        output.push_str("-->");
        return;
    }
    if matches!(
        node.Type(),
        DOMNodeType::kDocument | DOMNodeType::kDocumentFragment
    ) {
        for &child in node.Children() {
            serialize_node(document, child, output);
        }
        return;
    }
    if node.Type() != DOMNodeType::kElement {
        return;
    }
    output.push('<');
    output.push_str(node.Name());
    for attribute in node.Attributes() {
        output.push(' ');
        if !attribute.prefix.is_empty() {
            output.push_str(&attribute.prefix);
            output.push(':');
        }
        output.push_str(&attribute.local_name);
        output.push_str("=\"");
        append_escaped(&attribute.value, output, true);
        output.push('"');
    }
    output.push('>');
    if node.Namespace() == DOMNamespace::kHTML && is_void_html_element(node.Name()) {
        return;
    }
    let contents = document.TemplateContents(index).unwrap_or(index);
    for &child in document.Node(contents).Children() {
        serialize_node(document, child, output);
    }
    output.push_str("</");
    output.push_str(node.Name());
    output.push('>');
}

// cpp: webapi/dom_bindings.cc:132-138
fn inner_html(document: &Document, index: usize) -> String {
    let mut output = String::new();
    let contents = document.TemplateContents(index).unwrap_or(index);
    for &child in document.Node(contents).Children() {
        serialize_node(document, child, &mut output);
    }
    output
}

// cpp: webapi/dom_bindings.cc:140-161
fn node_type_number(node_type: DOMNodeType) -> f64 {
    match node_type {
        DOMNodeType::kElement => 1.0,
        DOMNodeType::kText => 3.0,
        DOMNodeType::kProcessingInstruction => 7.0,
        DOMNodeType::kComment => 8.0,
        DOMNodeType::kDocument => 9.0,
        DOMNodeType::kDocumentType => 10.0,
        DOMNodeType::kDocumentFragment => 11.0,
    }
}

// cpp: webapi/dom_bindings.cc:163-191
fn node_name(node_type: DOMNodeType, name: &str, namespace: DOMNamespace) -> String {
    match node_type {
        DOMNodeType::kDocument => "#document".into(),
        DOMNodeType::kText => "#text".into(),
        DOMNodeType::kComment => "#comment".into(),
        DOMNodeType::kDocumentFragment => "#document-fragment".into(),
        DOMNodeType::kElement if namespace == DOMNamespace::kHTML => name.to_ascii_uppercase(),
        _ => name.into(),
    }
}

fn find_element_by_id(document: &dom::Document, index: usize, id: &str) -> Option<usize> {
    let node = document.Node(index);
    if node.Type() == DOMNodeType::kElement
        && node
            .FindAttribute("id")
            .is_some_and(|attribute| attribute.value == id)
    {
        return Some(index);
    }
    for &child in node.Children() {
        if let Some(found) = find_element_by_id(document, child, id) {
            return Some(found);
        }
    }
    None
}

// cpp: webapi/dom_bindings.cc:35-53
fn dom_string_argument(argument: Option<&HostValue>) -> Option<String> {
    match argument? {
        HostValue::String(value) => Some(value.clone()),
        HostValue::Boolean(value) => Some(value.to_string()),
        HostValue::Number(value) => Some(dom_number_string(*value)),
        HostValue::Null(_) => Some("null".into()),
        HostValue::Undefined(_) => Some("undefined".into()),
        _ => None,
    }
}

// std::ostringstream uses defaultfloat with six significant decimal digits.
// Round before selecting fixed/scientific notation, including exponent carry.
fn dom_number_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .into();
    }
    let rounded = format!("{value:.5e}");
    let (mantissa, exponent) = rounded.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    if (-4..6).contains(&exponent) {
        let fixed = format!("{value:.precision$}", precision = (5 - exponent) as usize);
        if fixed.contains('.') {
            fixed.trim_end_matches('0').trim_end_matches('.').into()
        } else {
            fixed
        }
    } else {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{mantissa}e{exponent:+03}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DOMBootstrapSource;
    use dom::dom_mutation::ApplyDOMMutations;
    use dom::persistent_document::DOMAttribute;
    use javascript::javascript_runtime::JavaScriptRuntime;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;

    #[test]
    fn blocking_listener_targets_report_only_live_non_passive_registrations() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body><div id='passive'></div><div id='blocking'></div></body></html>",
        )));
        let blocking_id = {
            let owner = document.borrow();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .find_map(|index| {
                    document
                        .Node(index)
                        .FindAttribute("id")
                        .filter(|attribute| attribute.value == "blocking")
                        .map(|_| document.Node(index).Id())
                })
                .unwrap()
        };
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        assert!(runtime
            .Evaluate(
                &realm,
                r#"
                    const passiveWheel = () => {};
                    const blockingWheel = () => {};
                    document.getElementById('passive').addEventListener(
                        'wheel', passiveWheel, { passive: true });
                    document.getElementById('blocking').addEventListener(
                        'wheel', blockingWheel);
                "#,
                "blocking-wheel-targets.js",
            )
            .Succeeded());
        assert_eq!(
            bindings.borrow().BlockingListenerTargets("wheel"),
            vec![blocking_id]
        );
        assert!(runtime
            .Evaluate(
                &realm,
                "document.getElementById('blocking').removeEventListener('wheel', blockingWheel);",
                "remove-blocking-wheel-target.js",
            )
            .Succeeded());
        assert!(bindings
            .borrow()
            .BlockingListenerTargets("wheel")
            .is_empty());
        assert!(runtime
            .Evaluate(
                &realm,
                "window.addEventListener('wheel', blockingWheel);",
                "window-blocking-wheel-target.js",
            )
            .Succeeded());
        assert_eq!(bindings.borrow().BlockingListenerTargets("wheel"), vec![0]);
    }

    #[test]
    fn sibling_getters_follow_live_tree_without_creating_collections() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body><div id='box'>alpha<!-- comment --><span id='left'></span>omega<b id='right'></b></div></body></html>",
        )));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let collections = bindings.borrow().collections.borrow().len();
        assert!(runtime.Evaluate(&realm, r#"
            function check(value) { if (!value) throw new Error('sibling query'); }
            const box = document.getElementById('box');
            const left = document.getElementById('left'), right = document.getElementById('right');
            const first = box.firstChild, comment = first.nextSibling, text = left.nextSibling;
            check(first.previousSibling === null && first.previousElementSibling === null);
            check(comment.nodeType === 8 && comment.previousSibling === first && comment.nextSibling === left);
            check(first.nextElementSibling === left && text.previousElementSibling === left && text.nextElementSibling === right);
            check(left.previousElementSibling === null && left.nextElementSibling === right);
            check(right.previousSibling === text && right.nextSibling === null && right.nextElementSibling === null);
        "#, "sibling-query.js").Succeeded());
        assert_eq!(bindings.borrow().collections.borrow().len(), collections);
        assert!(runtime.Evaluate(&realm, r#"
            const inserted = document.createElement('i');
            check(inserted.nextSibling === null && inserted.previousSibling === null);
            box.insertBefore(inserted, left);
            check(comment.nextSibling === inserted && left.previousSibling === inserted && first.nextElementSibling === inserted);
            box.removeChild(inserted);
            check(inserted.previousSibling === null && inserted.nextElementSibling === null && comment.nextSibling === left);
            box.appendChild(left);
            check(left.previousSibling === right && right.nextElementSibling === left && left.nextSibling === null);
        "#, "sibling-mutation.js").Succeeded());
    }

    #[test]
    fn parsed_dom_properties_mutations_and_template_serialization_match_source() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            r#"<!doctype html><html><head></head><body><div id="box">one<!-- c -->two<span>three</span></div><input id="check" type="checkbox" checked value="seed"><textarea id="area">initial</textarea><select id="choice"><option>A</option><option selected value="b">B</option></select><img id="missing" src="x.png"><img id="blank"><template id="tpl"><span data-quote="&quot;&amp;&lt;&gt;">nested &amp;&lt;&gt;</span><!-- c --><br></template><svg id="vector"></svg></body></html>"#,
        )));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let result = runtime.Evaluate(&realm, r#"
            function check(condition, message) { if (!condition) throw new Error(message); }
            check(document.documentElement.tagName === 'HTML' && document.head.tagName === 'HEAD' && document.body.tagName === 'BODY', 'document structure');
            check(document.activeElement === document.body && document.defaultView === globalThis && document.ownerDocument === null && document.textContent === null, 'document properties');
            const box = document.getElementById('box');
            check(box.parentNode === document.body && box.ownerDocument === document && box.namespaceURI === 'http://www.w3.org/1999/xhtml', 'node ownership');
            check(box.textContent === 'onetwothree' && box.nodeValue === null, 'descendant text excludes comments');
            const text = box.firstChild;
            text.data = 'changed';
            check(text.nodeValue === 'changed' && box.textContent === 'changedtwothree', 'character data mutation');
            box.nodeValue = 'ignored';
            check(box.textContent === 'changedtwothree', 'element nodeValue setter');
            box.textContent = 'new';
            check(box.firstChild === box.lastChild && box.firstChild.parentNode === box && box.innerHTML === 'new', 'replace child text');
            const control = document.getElementById('check');
            check(control.checked === true && control.value === 'seed', 'initial input state');
            control.checked = false; control.value = 'changed value';
            check(control.checked === false && control.hasAttribute('checked') && control.value === 'changed value' && control.getAttribute('value') === 'seed', 'live control state differs from attribute');
            const area = document.getElementById('area'), choice = document.getElementById('choice');
            check(area.value === 'initial' && choice.value === 'b', 'initial textarea/select');
            area.value = 'edited'; choice.value = 'chosen';
            check(area.value === 'edited' && choice.value === 'chosen' && area.textContent === 'initial', 'live textarea/select state');
            const missing = document.getElementById('missing'), blank = document.getElementById('blank');
            check(missing.complete === false && missing.naturalWidth === 0 && missing.naturalHeight === 0 && missing.currentSrc === 'x.png' && blank.complete === true, 'unloaded images');
            const template = document.getElementById('tpl');
            check(template.content.nodeType === 11 && template.content.ownerDocument === document && template.textContent === '', 'template ownership');
            check(template.innerHTML === '<span data-quote="&quot;&amp;&lt;>">nested &amp;&lt;&gt;</span><!-- c --><br>', 'template serialization');
            check(document.getElementById('vector').namespaceURI === 'http://www.w3.org/2000/svg', 'SVG namespace');
            box.setAttribute('rounded', 1.23456789); box.setAttribute('scientific', 1000000); box.setAttribute('large-fixed', 100000); box.setAttribute('infinity', Infinity);
            check(box.getAttribute('rounded') === '1.23457' && box.getAttribute('scientific') === '1e+06' && box.getAttribute('large-fixed') === '100000' && box.getAttribute('infinity') === 'Infinity', 'source number formatting');
        "#, "properties.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let check_id = {
            let mut owner = document.borrow_mut();
            owner.GetDocumentMut().SetImageResource(
                "x.png".into(),
                dom::ImageResourceMetadata {
                    id: 1,
                    natural_width: 17.0,
                    natural_height: 11.0,
                    ..Default::default()
                },
            );
            let tree = owner.GetDocument();
            tree.Node(find_element_by_id(tree, tree.Root(), "check").unwrap())
                .Id()
        };
        bindings.borrow_mut().SetFocusedNode(Some(check_id));
        let result = runtime.Evaluate(&realm,
            "check(document.activeElement === control && missing.complete && missing.naturalWidth === 17 && missing.naturalHeight === 11, 'focused node and loaded image')", "properties.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn snapshot_collection_reads_borrow_fixed_ids_without_copying() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<body></body>")));
        let bindings = DOMJavaScriptBindings::new(document, Box::new(|_| {}));
        let snapshot = NodeCollection {
            root: 0,
            kind: "snapshot".into(),
            query: String::new(),
            snapshot: (1..=1024).collect(),
            membership_cache: RefCell::new(None),
        };
        for _ in 0..16 {
            let nodes = bindings.collection_nodes(&snapshot);
            assert!(
                matches!(nodes, CollectionNodes::Borrowed(_)),
                "immutable snapshot must not materialize N ids per read"
            );
            assert_eq!(nodes.as_ptr(), snapshot.snapshot.as_ptr());
            assert_eq!(nodes.len(), 1024);
            assert_eq!(nodes[0], 1);
            assert_eq!(nodes[1023], 1024);
        }
        let live = NodeCollection {
            root: u64::MAX,
            kind: "children".into(),
            query: String::new(),
            snapshot: vec![7],
            membership_cache: RefCell::new(None),
        };
        assert!(
            matches!(bindings.collection_nodes(&live),CollectionNodes::Owned(ref nodes) if nodes.is_empty()),
            "live collection must recompute, never consume a fixed snapshot"
        );
    }
    #[test]
    fn snapshot_iterator_retains_detached_ids_and_sticky_done() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
            const root=document.getElementById('root'), snap=root.querySelectorAll('p'), live=root.children;
            const a=snap[0],b=snap[1],iter=snap[Symbol.iterator]();
            if(iter.next().value!==a)throw Error('first iterator id');
            root.removeChild(a);root.removeChild(b);
            if(live.length!==0 || snap.length!==2 || snap.item(0)!==a || snap[1]!==b)throw Error('detached snapshot');
            if(iter.next().value!==b || !iter.next().done)throw Error('detached iterator');
            root.appendChild(a);root.appendChild(b);root.appendChild(document.createElement('p'));
            if(live.length!==3 || !iter.next().done || snap.length!==2)throw Error('done or live snapshot');
            if(Array.from(snap).map(n=>n.id).join('|')!=='a|b')throw Error('snapshot iterable order');
        "#,"snapshot.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
    // Artifact: insert inside dom_bindings.rs's existing tests module.
    #[test]
    fn live_membership_cache_reuses_only_matching_arena_revision_and_reads_fresh_names() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main><p class=one id=a></p><p id=b></p></main>",
        )));
        let bindings = DOMJavaScriptBindings::new(document.clone(), Box::new(|_| {}));
        let collection = NodeCollection {
            root: 1,
            kind: "getElementsByClassName".into(),
            query: "one".into(),
            snapshot: vec![],
            membership_cache: RefCell::new(None),
        };
        let CollectionNodes::Shared(first) = bindings.collection_nodes(&collection) else {
            panic!("cacheable live membership")
        };
        let CollectionNodes::Shared(second) = bindings.collection_nodes(&collection) else {
            panic!("repeat cacheable live membership")
        };
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(first.len(), 1);
        {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let index = find_element_by_id(tree, root, "b").unwrap();
            tree.SetAttribute(
                index,
                DOMAttribute {
                    local_name: "class".into(),
                    value: "one".into(),
                    ..Default::default()
                },
            );
        }
        let CollectionNodes::Shared(third) = bindings.collection_nodes(&collection) else {
            panic!("updated cacheable membership")
        };
        assert!(!Rc::ptr_eq(&first, &third));
        assert_eq!(third.len(), 2);
        assert_eq!(first.len(), 1);
        {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let a = find_element_by_id(tree, root, "a").unwrap();
            tree.Remove(a);
        }
        assert_eq!(bindings.collection_nodes(&collection).len(), 1);
        let mut other = html::html_parser::ParseHTML(
            "<main><p class=one></p><p class=one></p><p class=one></p></main>",
        );
        document.borrow_mut().SwapDocument(other.GetDocumentMut());
        assert_eq!(
            bindings.collection_nodes(&collection).len(),
            3,
            "arena identity must prevent cross-document ID reuse"
        );
        document.borrow_mut().SwapDocument(other.GetDocumentMut());
        assert_eq!(
            bindings.collection_nodes(&collection).len(),
            1,
            "restored arena must recompute after foreign cache publication"
        );
    }
    #[test]
    fn live_membership_cache_preserves_mutation_between_reads_iterators_and_argument_coercion() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a name=first class=one></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
        function check(ok,message){if(!ok)throw Error(message)}
        const root=document.getElementById('root'),a=document.getElementById('a'),b=document.getElementById('b');
        const kids=root.children,tags=root.getElementsByTagName('P'),classes=root.getElementsByClassName('one');
        const snap=root.querySelectorAll('p'),nodes=root.childNodes;
        check(kids.length===2 && tags.length===2 && classes.length===1,'warm memberships');
        root.removeChild(a);check(kids.item(0)===b && kids.length===1 && classes.length===0,'remove between length/item');
        root.insertBefore(a,b);check(kids.item(0)===a && tags.item(1)===b,'insert current order');
        const iterator=kids[Symbol.iterator]();check(iterator.next().value===a,'iterator first');
        root.removeChild(b);const c=document.createElement('p');c.id='c';root.appendChild(c);
        check(iterator.next().value===c && iterator.next().done,'iterator membership mutation');
        root.appendChild(b);check(iterator.next().done,'sticky completed iterator');
        check(kids.item({valueOf(){root.removeChild(a);return 0;}})===c,'argument coercion changes membership before item');
        c.setAttribute('name','fresh');check(kids.namedItem('fresh')===c,'fresh name');
        c.setAttribute('name','new');check(kids.namedItem('fresh')===null && kids.namedItem('new')===c,'name changed after warm lookup');
        check(kids.namedItem({toString(){c.id='coerced';return 'coerced'}})===c,'name coercion side effect');
        b.className='one';check(classes.length===1 && classes[0]===b,'class change invalidation');
        const fragment=document.createDocumentFragment();fragment.appendChild(c);
        check(kids.length===1 && fragment.children.item(0)===c,'reparent roots');
        fragment.appendChild(document.createTextNode('text'));check(fragment.childNodes.length===2,'text child membership');
        c.appendChild(a);check(tags.length===1 && fragment.getElementsByTagName('P').length===2,'detached subtree insertion');
        check(snap.length===2 && snap[0]===a && snap[1]===b,'fixed snapshot identity retains detached nodes');
        root.textContent='new';check(kids.length===0 && nodes.length===1 && nodes[0].nodeType===3,'text replacement');
        let coercion=false;try{kids.item({valueOf(){throw Error('coercion')}})}catch(e){coercion=e.message==='coercion'}
        check(coercion,'coercion exception is not suppressed');
    "#,"live-membership-cache.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    // Insert in existing DOM binding tests. All existing mutation assertions retained.
    #[test]
    fn collection_webidl_receiver_conversion_exceptions_and_prototype_semantics() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
        function check(ok,label){if(!ok)throw Error(label)}
        function typeError(run,label){let seen=false;try{run()}catch(e){seen=e instanceof TypeError}check(seen,label)}
        const root=document.getElementById('root'),a=document.getElementById('a'),b=document.getElementById('b');
        const live=root.children, snap=root.querySelectorAll('p');
        const htmlItem=HTMLCollection.prototype.item,nodeItem=NodeList.prototype.item,named=HTMLCollection.prototype.namedItem;
        check(live.item===htmlItem && live.item===live.item && snap.item===nodeItem && htmlItem!==nodeItem,'per-interface prototype identity');
        check(htmlItem.name==='item' && htmlItem.length===1 && named.name==='namedItem' && named.length===1,'method descriptors');
        check(!Object.hasOwn(htmlItem,'prototype') && !Object.hasOwn(live,'item'),'nonconstructor prototype method');
        const d=Object.getOwnPropertyDescriptor(HTMLCollection.prototype,'item');check(d.writable && d.configurable && d.enumerable,'prototype method attributes');
        let coerced=0;const bad={valueOf(){coerced++;return 0},toString(){coerced++;return 'a'}};
        typeError(()=>htmlItem(bad),'detached receiver');
        typeError(()=>htmlItem.call(snap,bad),'wrong interface');
        typeError(()=>nodeItem.call(live,bad),'opposite wrong interface');
        typeError(()=>htmlItem.call(Object.create(HTMLCollection.prototype),bad),'prototype impostor');
        typeError(()=>named.call(snap,bad),'named wrong interface');
        check(coerced===0,'receiver errors precede argument coercion');
        typeError(()=>live.item(),'required item argument');typeError(()=>live.namedItem(),'required named argument');
        for(const value of [0,0.9,'0',null,undefined,NaN,Infinity,-Infinity,4294967296,-4294967296])check(live.item(value)===a,'unsigned long conversion '+value);
        check(live.item(4294967297)===b && live.item(-1)===null,'modulo and negative wrap');
        typeError(()=>live.item(1n),'BigInt ToNumber');typeError(()=>live.item(Symbol('index')),'Symbol ToNumber');
        let seenHint='';check(live.namedItem({[Symbol.toPrimitive](hint){seenHint=hint;b.id='fresh';return 'fresh'}})===b && seenHint==='string','DOMString hint and mutation');
        typeError(()=>live.namedItem(Symbol('name')),'primitive symbol DOMString');
        typeError(()=>live.namedItem({[Symbol.toPrimitive](){return Symbol('name')}}),'object symbol DOMString');
        let error=false;try{live.item({valueOf(){throw Error('coercion')}})}catch(e){error=e.message==='coercion' && e.stack.includes('collection-webidl.js')}
        check(error,'coercion failure and user source stack');
        const oldString=String;globalThis.String=()=>{throw Error('must not replace IDL intrinsic')};
        try{check(live.namedItem({toString(){return 'fresh'}})===b,'captured original String')}finally{globalThis.String=oldString}
        HTMLCollection.prototype.item=function(){return 'overridden'};
        check(live.item(0)==='overridden','prototype override');HTMLCollection.prototype.item=htmlItem;
        Object.defineProperty(live,'item',{value:()=>17,configurable:true});check(live.item(0)===17,'own override');delete live.item;
        Object.defineProperty(HTMLCollection.prototype,'item',{get(){throw Error('prototype getter')},configurable:true});
        let getter=false;try{live.item}catch(e){getter=e.message==='prototype getter' && e.stack.includes('collection-webidl.js')}
        check(getter,'Reflect getter error');Object.defineProperty(HTMLCollection.prototype,'item',d);
        let primitiveCalls=0;check(live.item({valueOf(){primitiveCalls++;root.removeChild(a);return 0}})===b && primitiveCalls===1,'coercion before membership lookup once');
        check(snap.item(0)===a && snap.item(1)===b,'fixed snapshot ID retention after coercion mutation');
    "#,"collection-webidl.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn selectors_and_collections_keep_live_identity_and_snapshot_semantics() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<main id=root><p id=a name=first class='one two'>text</p><!--comment--><p id=b class=two></p><section><span class=leaf></span></section><input id=field placeholder=hint><svg viewBox='0 0 10 10'><linearGradient id='gradient'/></svg></main>")));
        let emit_document = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let bootstrap = runtime.Evaluate(&realm, DOMBootstrapSource(), "dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let result = runtime.Evaluate(&realm,r#"
            function check(value,label){if(!value)throw Error(label)}
            const root=document.querySelector('#root'), a=root.querySelector('p.one'), b=root.querySelector('#b');
            check(root === document.getElementById('root') && root.querySelector('#root') === null,'query excludes receiver');
            check(a.matches('main > p.one.two:first-child') && b.matches('p + p:nth-child(2)') && b.closest('main') === root,'combinators');
            check(root.matches(':has(> section)') && !root.matches(':has(> section .leaf)') && !root.matches(':has(> .leaf)') && root.matches(':has(.leaf)'),'has source traversal');
            check(a.matches(':is(#a,.absent):not(.absent):where(p)') && !a.matches('p::before'),'selector lists and pseudo');
            check(document.querySelector(':root') === document.documentElement,'document root');
            const detached=document.createElement('div');check(!detached.matches(':root') && detached.matches(':first-child') && !detached.matches(':last-child'),'detached source semantics');
            const field=document.getElementById('field');field.value='live';check(field.matches(':placeholder-shown'),'source public matcher reads attribute');field.setAttribute('value','attribute');check(!field.matches(':placeholder-shown'),'placeholder attribute');
            const svg=document.querySelector('svg');check(svg.matches('[viewBox]') && !svg.matches('[viewbox]'),'SVG attribute case');
            check(document.querySelector('linearGradient').id === 'gradient' && document.querySelector('lineargradient') === null,'SVG type case');
            const live=root.children, childNodes=root.childNodes;
            check(live instanceof HTMLCollection && childNodes instanceof NodeList && root.children === live && root.childNodes === childNodes,'live identity and prototypes');
            check(live.length === 5 && childNodes.length === 6 && childNodes[1].nodeType === 8,'children node kinds');
            check(live.item(0.9) === a && live.item(-1) === null && live.namedItem('first') === a && live.namedItem('') === null,'collection methods');
            const snapshot=root.querySelectorAll('p'), tags=root.getElementsByTagName('P'), classes=root.getElementsByClassName('one two');
            check(snapshot instanceof NodeList && snapshot.length === 2 && tags.length === 2 && classes.length === 1,'query collections');
            let visits=0;Array.prototype.forEach.call(snapshot,()=>visits++);
            check(visits===2 && (0 in snapshot) && !(2 in snapshot),'host indices participate in HasProperty');
            check(Array.prototype.map.call(snapshot,n=>n.id).join(',')==='a,b','array methods traverse host collection');
            check(root.getElementsByTagName('P') === tags && root.querySelectorAll('p') !== snapshot,'cache and snapshots');
            const scripts=document.scripts, script=document.createElement('script');script.id='live-script';
            check(scripts instanceof HTMLCollection && scripts === document.scripts && scripts.length === 0,'document scripts collection');
            root.appendChild(script);
            check(scripts.length === 1 && scripts[0] === script && scripts.namedItem('live-script') === script,'document scripts insertion');
            root.removeChild(script);check(scripts.length === 0,'document scripts removal');
            check(document.images === document.getElementsByTagName('img') && document.forms === document.getElementsByTagName('form') && document.plugins === document.embeds,'document collection filters');
            const added=document.createElement('p');added.className='one two';root.appendChild(added);
            check(live.length === 6 && tags.length === 3 && classes.length === 2 && snapshot.length === 2,'live mutation');
            root.removeChild(a);check(snapshot[0] === a && tags.length === 2 && classes.length === 1,'snapshot retains detached node');
            const iterator=live[Symbol.iterator]();check(iterator[Symbol.iterator]() === iterator,'iterator identity');
            while(!iterator.next().done){}
            root.appendChild(a);check(iterator.next().done,'iterator remains done');
            check(Array.from(live).length === live.length && Array.from(snapshot).length === 2,'iterability');
            const fragment=document.createDocumentFragment();fragment.appendChild(document.createElement('aside'));
            check(fragment.querySelector('aside') === fragment.firstChild && fragment.children.length === 1,'fragment query');
            const clone=root.cloneNode(true);check(clone !== root && clone.children.length === root.children.length && clone.querySelector('p') !== root.querySelector('p'),'deep clone');
            const shallow=a.cloneNode(false);check(shallow.childNodes.length === 0 && shallow.className === a.className,'shallow clone');
            const text=document.createTextNode('text'), comment=document.createComment('note');
            fragment.insertBefore(text,fragment.firstChild);fragment.appendChild(comment);check(fragment.firstChild === text && fragment.lastChild === comment,'create and insert node kinds');
            let invalid=false;try{root.appendChild(root)}catch(e){invalid=e instanceof TypeError}check(invalid && root.parentNode !== null,'invalid cycle');
            a.style.setProperty('color','red');a.style.setProperty('color','blue','important');check(a.getAttribute('style') === 'color: blue !important;' && a.style.getPropertyValue('color') === 'blue','inline style');
            const separate=document.implementation.createHTMLDocument('title');check(separate.defaultView === null && separate.head.firstChild.textContent === 'title' && separate.hasFocus(),'detached document');
            a.className='one\u000btwo';check(a.matches('.one.two[class~=two]') && root.getElementsByClassName('one\u000btwo').length >= 1 && !a.matches('\u00a0p'),'source C-locale whitespace');
            true
        "#,"collections.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }

    #[test]
    fn bootstrap_dom_script_mutation_is_synchronously_visible() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let input_id = {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let html = tree.CreateElementDefault(DOMNamespace::kHTML, "html".into());
            tree.AppendChild(root, html);
            let body = tree.CreateElementDefault(DOMNamespace::kHTML, "body".into());
            tree.AppendChild(html, body);
            let input = tree.CreateElementDefault(DOMNamespace::kHTML, "input".into());
            tree.SetAttribute(
                input,
                DOMAttribute {
                    local_name: "id".into(),
                    value: "kw".into(),
                    ..Default::default()
                },
            );
            tree.AppendChild(body, input);
            tree.Node(input).Id()
        };
        let emit_document = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let bootstrap = runtime.Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let script = runtime.Evaluate(
            &realm,
            "const field = document.getElementById('kw');\
             field.className = 'night';\
             document instanceof Document && field instanceof HTMLInputElement && field.getAttribute('class') === 'night'",
            "page.js",
        );
        assert!(script.Succeeded(), "{:?}", script.exception);
        assert_eq!(
            script
                .value
                .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        let owner = document.borrow();
        let tree = owner.GetDocument();
        let input = tree.FindNodeById(input_id).unwrap();
        assert_eq!(
            tree.Node(input).FindAttribute("class").unwrap().value,
            "night"
        );
    }
}
