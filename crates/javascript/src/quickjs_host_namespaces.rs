//! Context-local ScriptWrappable/DOMDataStore identity. A securely reused
//! Window keeps its global; each Document's bindings and HostGraph retain
//! independent native-id spaces. No clone wire format or author hook is used.
use super::*;
use std::rc::Weak;

pub(super) type WrapperOrigins = HashMap<(usize, usize), (Weak<RefCell<State>>, HostObjectId)>;
#[derive(Default)]
pub(super) struct NamespaceRegistry {
    root: RefCell<Weak<RefCell<State>>>,
    states: RefCell<Vec<Rc<RefCell<State>>>>,
    wrappers: Rc<RefCell<WrapperOrigins>>,
    active: RefCell<Option<Weak<RefCell<State>>>>,
    global_objects: RefCell<std::collections::HashSet<HostObjectId>>,
    window_host: Cell<bool>,
    document_namespaces_bound: Cell<bool>,
    current_document: RefCell<Weak<RefCell<State>>>,
    owners: RefCell<HashMap<HostObjectOwnerIdentity, Weak<RefCell<State>>>>,
}
impl NamespaceRegistry {
    pub(super) fn WithWrappers(wrappers: Rc<RefCell<WrapperOrigins>>) -> Self {
        Self {
            wrappers,
            ..Default::default()
        }
    }
    pub(super) fn RemoveWrapperIdentity(&self, identity: (usize, usize)) {
        self.wrappers.borrow_mut().remove(&identity);
    }
}
pub(super) struct NamespaceScope {
    registry: Rc<NamespaceRegistry>,
    previous: Option<Weak<RefCell<State>>>,
}
impl JavaScriptHostObjectNamespaceScope for NamespaceScope {}
impl Drop for NamespaceScope {
    fn drop(&mut self) {
        *self.registry.active.borrow_mut() = self.previous.take();
    }
}
pub(super) fn Enter(state: &Rc<RefCell<State>>) -> NamespaceScope {
    let registry = state.borrow().namespaces.clone();
    let previous = registry.active.borrow_mut().replace(Rc::downgrade(state));
    NamespaceScope { registry, previous }
}
pub(super) fn Active(state: &Rc<RefCell<State>>) -> Rc<RefCell<State>> {
    state
        .borrow()
        .namespaces
        .active
        .borrow()
        .as_ref()
        .and_then(Weak::upgrade)
        .unwrap_or_else(|| state.clone())
}
pub(super) fn Initialize(state: &Rc<RefCell<State>>) {
    *state.borrow().namespaces.root.borrow_mut() = Rc::downgrade(state);
    let window = state.borrow().global_host.borrow().IsWindowHost();
    state.borrow().namespaces.window_host.set(window);
}
pub(super) fn Create(
    state: &Rc<RefCell<State>>,
    host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    initial: bool,
) -> Result<JavaScriptHostObjectNamespace, JavaScriptException> {
    if initial {
        let mut state_ = state.borrow_mut();
        if state_.object_host.is_some() {
            return Err(JavaScriptException {
                message: "initial host namespace is already bound".into(),
                ..Default::default()
            });
        }
        match (&state_.host_graph.source, host.borrow().ObjectTraceSource()) {
            (Some(existing), Some(source)) if Rc::ptr_eq(existing, &source) => {}
            (None, None) => {}
            _ => {
                return Err(JavaScriptException {
                    message: "initial namespace must retain the original host graph".into(),
                    ..Default::default()
                })
            }
        }
        if let Some(owner) = host.borrow().NativeObjectOwnerIdentity() {
            state_
                .namespaces
                .owners
                .borrow_mut()
                .insert(owner, Rc::downgrade(state));
        }
        state_.object_host = Some(host);
        state_.namespaces.document_namespaces_bound.set(true);
        *state_.namespaces.current_document.borrow_mut() = Rc::downgrade(state);
        return Ok(JavaScriptHostObjectNamespace::new(state.clone()));
    }
    let root = state.borrow();
    if root.object_host.is_none() {
        return Err(JavaScriptException {
            message: "initial Document namespace must be installed first".into(),
            ..Default::default()
        });
    }
    let namespace = Rc::new(RefCell::new(State {
        host: host.clone(),
        realms: root.realms.clone(),
        agent_identity: root.agent_identity,
        document_object: host.borrow().DocumentHostObjectId(),
        object_host: None,
        global_host: root.global_host.clone(),
        global: root.global.clone(),
        wrappers: HashMap::new(),
        host_graph: HostGraph::new(host.borrow().ObjectTraceSource()),
        wrapper_ids: HashMap::new(),
        fallbacks: HashMap::new(),
        reflect_get: root.reflect_get.clone(),
        reflect_set: root.reflect_set.clone(),
        hooks: root.hooks.clone(),
        namespaces: root.namespaces.clone(),
    }));
    if let Some(owner) = host.borrow().NativeObjectOwnerIdentity() {
        assert!(root
            .namespaces
            .owners
            .borrow_mut()
            .insert(owner, Rc::downgrade(&namespace))
            .is_none());
    }
    root.namespaces.states.borrow_mut().push(namespace.clone());
    Ok(JavaScriptHostObjectNamespace::new(namespace))
}
pub(super) fn SelectDocument(
    state: &Rc<RefCell<State>>,
    namespace: &JavaScriptHostObjectNamespace,
) {
    let selected = StateFor(namespace);
    let registry = state.borrow().namespaces.clone();
    assert!(Rc::ptr_eq(&registry, &selected.borrow().namespaces));
    *registry.current_document.borrow_mut() = Rc::downgrade(&selected);
}
pub(super) fn StateFor(namespace: &JavaScriptHostObjectNamespace) -> Rc<RefCell<State>> {
    namespace
        .ImplementationOwner::<RefCell<State>>()
        .expect("namespace belongs to QuickJS")
}
pub(super) fn Tagged(value: &HostValue) -> Option<(Rc<RefCell<State>>, &HostValue)> {
    let HostValue::Retained(owner, value) = value else {
        return None;
    };
    let namespace = if let Some(reference) = owner.downcast_ref::<JavaScriptNativeObjectReference>()
    {
        reference.Namespace().Upgrade()?
    } else {
        owner
            .downcast_ref::<WeakJavaScriptHostObjectNamespace>()?
            .Upgrade()?
    };
    Some((StateFor(&namespace), value))
}
pub(super) fn TaggedIn<'a>(
    state: &Rc<RefCell<State>>,
    value: &'a HostValue,
) -> Option<(Rc<RefCell<State>>, &'a HostValue)> {
    if let Some(tagged) = Tagged(value) {
        return Some(tagged);
    }
    let HostValue::Retained(owner, value) = value else {
        return None;
    };
    let owner = owner.downcast_ref::<HostObjectOwnerIdentity>()?;
    let registry = state.borrow().namespaces.clone();
    let namespace = registry.owners.borrow().get(owner)?.upgrade()?;
    Some((namespace, value))
}
pub(super) fn WrapperOrigin(
    state: &Rc<RefCell<State>>,
    object: &JsObject,
) -> Option<(Rc<RefCell<State>>, HostObjectId)> {
    let registry = state.borrow().namespaces.clone();
    let entry = registry
        .wrappers
        .borrow()
        .get(&object.identity())
        .cloned()?;
    Some((entry.0.upgrade()?, entry.1))
}
pub(super) fn RegisterWrapper(state: &Rc<RefCell<State>>, object: &JsObject, id: HostObjectId) {
    state
        .borrow()
        .namespaces
        .wrappers
        .borrow_mut()
        .insert(object.identity(), (Rc::downgrade(state), id));
}
pub(super) fn RemoveWrapper(state: &Rc<RefCell<State>>, identity: (usize, usize)) {
    state
        .borrow()
        .namespaces
        .wrappers
        .borrow_mut()
        .remove(&identity);
}
pub(super) fn ObjectState(state: &Rc<RefCell<State>>, id: HostObjectId) -> Rc<RefCell<State>> {
    let registry = state.borrow().namespaces.clone();
    if id == 0 || registry.global_objects.borrow().contains(&id) {
        return state
            .borrow()
            .namespaces
            .root
            .borrow()
            .upgrade()
            .expect("live Context root");
    }
    state.clone()
}
pub(super) fn ObjectHost(
    state: &Rc<RefCell<State>>,
    id: HostObjectId,
) -> Rc<RefCell<dyn JavaScriptHostBindings>> {
    let state_ = state.borrow();
    if id == 0
        || state_.namespaces.global_objects.borrow().contains(&id)
        || state_.document_object == Some(id)
            && state_
                .namespaces
                .current_document
                .borrow()
                .upgrade()
                .is_some_and(|current| Rc::ptr_eq(&current, state))
    {
        state_.global_host.clone()
    } else {
        state_.object_host.as_ref().unwrap_or(&state_.host).clone()
    }
}
/// Called only after the host borrow is released. Conversion and GC must not
/// borrow Window while an InvokeWithRuntime continuation is already in it.
pub(super) fn RecordGlobalResult(state: &Rc<RefCell<State>>, value: &HostValue) {
    let (host, registry) = {
        let state = state.borrow();
        (state.global_host.clone(), state.namespaces.clone())
    };
    match value {
        HostValue::Object(object) if host.borrow().OwnsGlobalHostObject(object.id) => {
            registry.global_objects.borrow_mut().insert(object.id);
        }
        HostValue::Array(values) => {
            for value in values {
                RecordGlobalResult(state, value);
            }
        }
        HostValue::Record(values) => {
            for (_, value) in values {
                RecordGlobalResult(state, value);
            }
        }
        _ => {}
    }
}
pub(super) fn IsWindow(state: &Rc<RefCell<State>>) -> bool {
    let state = state.borrow();
    state.namespaces.window_host.get() && state.namespaces.document_namespaces_bound.get()
}
pub(super) fn Localize(value: &mut HostValue, state: &Rc<RefCell<State>>) {
    while TaggedIn(state, value).is_some_and(|(origin, _)| Rc::ptr_eq(&origin, state)) {
        let HostValue::Retained(_, inner) = std::mem::take(value) else {
            unreachable!()
        };
        *value = *inner;
    }
    match value {
        HostValue::Array(values) => {
            for value in values {
                Localize(value, state);
            }
        }
        HostValue::Record(values) => {
            for (_, value) in values {
                Localize(value, state);
            }
        }
        _ => {}
    }
}
pub(super) fn States(state: &Rc<RefCell<State>>) -> Vec<Rc<RefCell<State>>> {
    let registry = state.borrow().namespaces.clone();
    let mut states = vec![registry.root.borrow().upgrade().expect("live Context root")];
    states.extend(registry.states.borrow().iter().cloned());
    states
}
pub(super) fn Clear(state: &Rc<RefCell<State>>) {
    let registry = state.borrow().namespaces.clone();
    registry.active.borrow_mut().take();
    registry.wrappers.borrow_mut().retain(|_, (origin, _)| {
        origin
            .upgrade()
            .is_some_and(|origin| !Rc::ptr_eq(&origin.borrow().namespaces, &registry))
    });
    registry.owners.borrow_mut().clear();
    registry
        .current_document
        .borrow_mut()
        .clone_from(&Weak::new());
    registry.states.borrow_mut().clear();
}
