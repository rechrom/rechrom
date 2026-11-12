//! HTMLFrameElementBase.cc:62-87,98-125,174-209 and
//! HTMLFrameOwnerElement.cc:115-161,681-713,727-852,858-881.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//!
//! Source policy remains here. The client receives a CreateFrame continuation,
//! not a Page constructor. It must complete that synchronous transaction before
//! yielding to script/parser execution. No Document borrow crosses that call.
use crate::{
    content_frames::{ContentFrameSource, LazyLoadFrameObserverSource},
    document_base_url::DocumentURLValue,
    document_policy::DocumentPolicyFeatureState,
    persistent_document::{DOMOwnerHandle, PersistentDocument},
};
use foundation::security::SandboxFlags;
use navigation_history::{FrameLoadType, ReferrerPolicy};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

pub use crate::network_permissions_policy::ParsedPermissionsPolicy as FrameContainerPolicy;
pub trait FrameTrustTokenParams {}

#[derive(Clone)]
pub struct FrameOwnerPolicy {
    pub sandbox: SandboxFlags,
    pub container: Rc<FrameContainerPolicy>,
    pub required_document: Rc<DocumentPolicyFeatureState>,
}
#[derive(Clone)]
pub struct FrameOwnerResourceRequest {
    pub url: Rc<DocumentURLValue>,
    /// FrameLoader retains the submitted request through the existing pending
    /// navigation owner; a child navigation must not project a POST to a URL.
    pub method: String,
    pub referrer: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub encoded_body: Option<std::sync::Arc<fileapi::encoded_form_data::EncodedFormData>>,
    pub navigation: Option<navigation_history::NavigationRequestMetadata>,
    pub referrer_policy: ReferrerPolicy,
    pub trust_token: Option<Rc<dyn FrameTrustTokenParams>>,
    pub bypass_cache: bool,
    pub skip_service_worker: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameOwnerNavigationReason {
    None,
    FrameNavigation,
    InitialFrameNavigation,
}
#[derive(Clone)]
pub struct FrameOwnerNavigation {
    pub owner: DOMOwnerHandle,
    pub request: FrameOwnerResourceRequest,
    pub policy: FrameOwnerPolicy,
    pub load_type: FrameLoadType,
    pub reason: FrameOwnerNavigationReason,
    pub is_container_initiated: bool,
}
/// Source locals retained across LocalFrameClient::CreateFrame's script reentry.
/// No DOM/Layout tree or borrowed engine is retained by this continuation.
#[derive(Clone)]
pub struct CreateLocalFrameRequest {
    pub owner: DOMOwnerHandle,
    pub name: Rc<[u16]>,
    /// Snapshot of the binding/parser entry that requested frame creation.
    pub is_created_by_script: bool,
    navigation: FrameOwnerNavigation,
    original_url: Rc<DocumentURLValue>,
}
#[derive(Clone)]
pub enum FrameOwnerLoadEffect {
    CreateLocalFrame(CreateLocalFrameRequest),
    /// Internal continuation: route the preceding DidChangeAttributes first.
    ContinueInitialNavigation {
        pending: CreateLocalFrameRequest,
        frame: Rc<dyn ContentFrameSource>,
    },
    Navigate {
        frame: Rc<dyn ContentFrameSource>,
        navigation: FrameOwnerNavigation,
    },
    DidChangeAttributes {
        owner: DOMOwnerHandle,
        frame_identity: u64,
    },
    DidChangeFramePolicy {
        frame_identity: u64,
        policy: FrameOwnerPolicy,
    },
    StartNavigation {
        frame: Rc<dyn ContentFrameSource>,
        navigation: FrameOwnerNavigation,
    },
    InvalidRelativeURL {
        owner: DOMOwnerHandle,
        source: String,
    },
    SrcDocChanged {
        frame_identity: u64,
        value: Rc<[u16]>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameOwnerLoadError {
    MissingClient,
    ParentFrameFacts,
    CreateLocalFrame,
    HostEffect,
    MissingOwner,
    WrongOwnerDocument,
    ContainerPolicy,
    TrustTokenParams,
    NavigationPolicy,
    LazyLoadObserver,
    RequestAdmission,
    UnexpectedCreatedFrame,
    InitialLocalFrameDocument,
    LocalFrameNavigationWindow,
}

/// Facts supplied by the actual parent Frame/Page owner for this call. They
/// are not inferred from an iframe tag, parent rendering state or another Page.
#[derive(Clone, Copy)]
pub struct FrameOwnerParentFacts {
    pub subframe_count: usize,
    pub maximum_frames: usize,
    pub load_type: FrameLoadType,
    pub lazy_load_enabled: bool,
    pub page_visible: bool,
    pub can_execute_scripts: bool,
}
/// Missing source facilities are required ports. Implementations may query
/// only the named original policy/resource owners, never synchronously a Page.
pub trait FrameOwnerLoadingServices {
    fn ParentFrame(
        &self,
        document: DOMOwnerHandle,
    ) -> Result<Option<FrameOwnerParentFacts>, FrameOwnerLoadError>;
    fn CanNavigate(
        &self,
        frame: &dyn ContentFrameSource,
        url: &DocumentURLValue,
    ) -> Result<bool, FrameOwnerLoadError>;
    fn TrustTokenParams(
        &self,
        owner: DOMOwnerHandle,
        value: &str,
    ) -> Result<Option<Rc<dyn FrameTrustTokenParams>>, FrameOwnerLoadError>;
    fn MonotonicTime(&self) -> std::time::Instant;
    fn CanAccessURL(&self, url: &DocumentURLValue) -> bool;
    fn NewLazyLoadObserver(
        &self,
        owner: DOMOwnerHandle,
        frame: &dyn ContentFrameSource,
    ) -> Result<Rc<dyn LazyLoadFrameObserverSource>, FrameOwnerLoadError>;
    fn AllowRequestForThisFrame(
        &self,
        frame: &dyn ContentFrameSource,
        navigation: &FrameOwnerNavigation,
    ) -> Result<bool, FrameOwnerLoadError>;
}

/// The assembly routes effects and constructs the original native Frame. It
/// performs no navigation/lazy-load/policy decisions. A null source CreateFrame
/// result is distinct from a missing implementation (the typed error case).
pub trait FrameOwnerLoadingClient {
    /// Prepare small live Frame facts and shared policy owners before taking
    /// the exclusive Document borrow. A service may not reach back into DOM.
    fn ServicesForOwner(
        &self,
        owner: DOMOwnerHandle,
    ) -> Result<Box<dyn FrameOwnerLoadingServices + '_>, FrameOwnerLoadError>;
    fn CreateLocalFrame(
        &self,
        owner: DOMOwnerHandle,
        name: &[u16],
        is_created_by_script: bool,
    ) -> Result<Option<Rc<dyn ContentFrameSource>>, FrameOwnerLoadError>;
    fn RouteEffect(&self, effect: &FrameOwnerLoadEffect) -> Result<(), FrameOwnerLoadError>;
}

/// Own the synchronous source transaction across RefCell/script reentry.
/// Creation may call this function again; there is deliberately no global
/// "already draining" flag that would defer a nested source load until later.
pub fn DrainFrameOwnerLoading(
    document: &Rc<RefCell<crate::DOM>>,
    client: &dyn FrameOwnerLoadingClient,
) -> Result<(), FrameOwnerLoadError> {
    // Hold this call's siblings locally. A nested drain must see only its own
    // newly produced notifications, not resume an older outer sibling early.
    let (mut effects, mut notifications) = {
        let owner = document.borrow();
        let document = owner.GetDocument();
        (
            document.TakeFrameOwnerLoadEffects(),
            document.TakeFrameOwnerLoadNotifications(),
        )
    };
    while !effects.is_empty() || !notifications.is_empty() {
        let outcome = if let Some(effect) = effects.pop_front() {
            let retained = effect.clone();
            let outcome = (|| {
                match effect {
                    FrameOwnerLoadEffect::CreateLocalFrame(pending) => {
                        let created = client.CreateLocalFrame(
                            pending.owner,
                            &pending.name,
                            pending.is_created_by_script,
                        )?;
                        document
                            .borrow_mut()
                            .GetDocumentMut()
                            .CompleteLocalFrameCreation(pending, created)?;
                    }
                    FrameOwnerLoadEffect::ContinueInitialNavigation { pending, frame } => {
                        let services = client.ServicesForOwner(pending.owner)?;
                        document
                            .borrow_mut()
                            .GetDocumentMut()
                            .ContinueInitialFrameNavigation(pending, frame, services.as_ref())?;
                    }
                    effect => client.RouteEffect(&effect)?,
                }
                Ok(())
            })();
            if outcome.is_err() {
                effects.push_front(retained);
            }
            outcome
        } else {
            let (context, notification) = notifications.pop_front().unwrap();
            let outcome = (|| {
                let services = client.ServicesForOwner(match notification {
                    FrameOwnerLoadNotification::DidNotifySubtreeInsertions(owner)
                    | FrameOwnerLoadNotification::LoadingAttributeChanged(owner)
                    | FrameOwnerLoadNotification::UpdatePolicy(owner)
                    | FrameOwnerLoadNotification::OpenURL { owner, .. } => owner,
                })?;
                document
                    .borrow_mut()
                    .GetDocumentMut()
                    .ProcessFrameOwnerLoadNotificationWithContext(
                        notification,
                        context,
                        services.as_ref(),
                    )?;
                Ok(())
            })();
            if outcome.is_err() {
                notifications.push_front((context, notification));
            }
            outcome
        };
        if let Err(error) = outcome {
            let owner = document.borrow();
            let loading = owner.GetDocument().FrameLoadingDocument();
            loading.effects.borrow_mut().extend(effects);
            loading.notifications.borrow_mut().extend(notifications);
            return Err(error);
        }
        if let Err(error) = DrainFrameOwnerLoading(document, client) {
            let owner = document.borrow();
            let loading = owner.GetDocument().FrameLoadingDocument();
            loading.effects.borrow_mut().extend(effects);
            loading.notifications.borrow_mut().extend(notifications);
            return Err(error);
        }
    }
    Ok(())
}

/// Canonical HTMLFrameElementBase members, attached to the original DOM node.
pub(crate) struct FrameElementLoadingState {
    url: RefCell<Rc<[u16]>>,
    name: RefCell<Rc<[u16]>>,
    should_lazy_load_children: Cell<bool>,
    policy: RefCell<Option<FrameOwnerPolicy>>,
    sandbox: Cell<SandboxFlags>,
    referrer_policy: Cell<ReferrerPolicy>,
    fallback_timing: RefCell<Option<FrameOwnerFallbackTiming>>,
}
impl FrameElementLoadingState {
    fn new(
        url: Rc<[u16]>,
        name: Rc<[u16]>,
        should_lazy_load_children: bool,
        sandbox: SandboxFlags,
        referrer_policy: ReferrerPolicy,
    ) -> Self {
        Self {
            url: RefCell::new(url),
            name: RefCell::new(name),
            should_lazy_load_children: Cell::new(should_lazy_load_children),
            policy: RefCell::new(None),
            sandbox: Cell::new(sandbox),
            referrer_policy: Cell::new(referrer_policy),
            fallback_timing: RefCell::new(None),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum FrameOwnerLoadNotification {
    DidNotifySubtreeInsertions(DOMOwnerHandle),
    OpenURL {
        owner: DOMOwnerHandle,
        replace_current_item: bool,
    },
    LoadingAttributeChanged(DOMOwnerHandle),
    UpdatePolicy(DOMOwnerHandle),
}
pub(crate) enum FrameOwnerLoadChange {
    Notification(FrameOwnerCreationContext, FrameOwnerLoadNotification),
    Effect(FrameOwnerLoadEffect),
}

/// Bind the canonical released-borrow transaction queue as well as the native
/// client. The queue's ordinary local continuation preserves sibling order.
pub fn BindFrameOwnerLoadingClient(
    document: &Rc<RefCell<crate::DOM>>,
    client: &Rc<dyn FrameOwnerLoadingClient>,
) {
    let owner = document.borrow();
    let document_owner = owner.GetDocument();
    document_owner.BindFrameOwnerLoadingClient(client);
    document_owner
        .AttributeLifecycle()
        .BindFrameOwnerDocument(Rc::downgrade(document));
}
pub(crate) fn DispatchFrameOwnerLoadChange(
    document: &Rc<RefCell<crate::DOM>>,
    change: FrameOwnerLoadChange,
) {
    {
        let owner = document.borrow();
        let loading = owner.GetDocument().FrameLoadingDocument();
        match change {
            FrameOwnerLoadChange::Notification(context, change) => loading
                .notifications
                .borrow_mut()
                .push_back((context, change)),
            FrameOwnerLoadChange::Effect(effect) => loading.effects.borrow_mut().push_back(effect),
        }
    }
    let _ = DrainBoundFrameOwnerLoading(document);
}
#[derive(Default)]
pub(crate) struct FrameOwnerLoadingDocument {
    pub notifications: RefCell<VecDeque<(FrameOwnerCreationContext, FrameOwnerLoadNotification)>>,
    creation_context: Rc<Cell<FrameOwnerCreationContext>>,
    pub effects: RefCell<VecDeque<FrameOwnerLoadEffect>>,
    client: RefCell<Option<std::rc::Weak<dyn FrameOwnerLoadingClient>>>,
    required_port: Cell<Option<FrameOwnerLoadError>>,
}
/// Explicit caller provenance, captured while the mutation runs. Parser and
/// host work never query later parser state to reconstruct a script context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameOwnerCreationContext {
    #[default]
    ParserOrHost,
    Script,
}
pub struct FrameOwnerCreationScope {
    context: Rc<Cell<FrameOwnerCreationContext>>,
    previous: FrameOwnerCreationContext,
}
impl Drop for FrameOwnerCreationScope {
    fn drop(&mut self) {
        self.context.set(self.previous);
    }
}
#[derive(Clone)]
pub struct FrameOwnerFallbackTiming {
    pub started_at: std::time::Instant,
    pub url: Rc<DocumentURLValue>,
}

pub fn DrainBoundFrameOwnerLoading(
    document: &Rc<RefCell<crate::DOM>>,
) -> Result<(), FrameOwnerLoadError> {
    let client = {
        let owner = document.borrow();
        let loading = owner.GetDocument().FrameLoadingDocument();
        if loading.effects.borrow().is_empty() && loading.notifications.borrow().is_empty() {
            return Ok(());
        }
        let client = loading
            .client
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade);
        client
    };
    let result = client.map_or(Err(FrameOwnerLoadError::MissingClient), |client| {
        DrainFrameOwnerLoading(document, client.as_ref())
    });
    document
        .borrow()
        .GetDocument()
        .FrameLoadingDocument()
        .required_port
        .set(result.as_ref().err().copied());
    result
}

fn utf16(value: &str) -> Rc<[u16]> {
    value.encode_utf16().collect::<Vec<_>>().into()
}
fn strip_html_spaces(value: &[u16]) -> Rc<[u16]> {
    let is_space = |c: &u16| matches!(*c, 9 | 10 | 12 | 13 | 32);
    let start = value
        .iter()
        .position(|c| !is_space(c))
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|c| !is_space(c))
        .map_or(start, |i| i + 1);
    Rc::from(&value[start..end])
}
// SecurityPolicy.cc:245-301, HTMLIFrameElement.cc:266-275 (legacy keywords).
fn referrer_policy(value: Option<&str>) -> ReferrerPolicy {
    match value.unwrap_or("").to_ascii_lowercase().as_str() {
        "no-referrer" | "never" | "none" => ReferrerPolicy::Never,
        "unsafe-url" | "always" => ReferrerPolicy::Always,
        "origin" => ReferrerPolicy::Origin,
        "origin-when-cross-origin" | "origin-when-crossorigin" => {
            ReferrerPolicy::OriginWhenCrossOrigin
        }
        "same-origin" => ReferrerPolicy::SameOrigin,
        "strict-origin" => ReferrerPolicy::StrictOrigin,
        "strict-origin-when-cross-origin" | "default" => {
            ReferrerPolicy::StrictOriginWhenCrossOrigin
        }
        "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
        _ => ReferrerPolicy::Default,
    }
}
impl PersistentDocument {
    pub fn FrameOwnerPolicy(&self, node: usize) -> Option<FrameOwnerPolicy> {
        self.Node(node)
            .frame_element_loading
            .as_ref()?
            .policy
            .borrow()
            .clone()
    }
    pub fn BindFrameOwnerLoadingClient(&self, client: &Rc<dyn FrameOwnerLoadingClient>) {
        *self.FrameLoadingDocument().client.borrow_mut() = Some(Rc::downgrade(client));
    }
    pub fn FrameOwnerLoadingRequiredPort(&self) -> Option<FrameOwnerLoadError> {
        self.FrameLoadingDocument().required_port.get()
    }
    pub fn FrameOwnerFallbackTiming(&self, node: usize) -> Option<FrameOwnerFallbackTiming> {
        self.Node(node)
            .frame_element_loading
            .as_ref()?
            .fallback_timing
            .borrow()
            .clone()
    }
    fn FrameLoadingDocument(&self) -> &FrameOwnerLoadingDocument {
        &self
            .document_metadata
            .get(&self.Node(self.Root()).Id())
            .expect("root document metadata")
            .frame_owner_loading
    }
    fn FrameLoadingIndex(&self, owner: DOMOwnerHandle) -> Result<usize, FrameOwnerLoadError> {
        let index = self
            .FindNodeById(owner.Id())
            .ok_or(FrameOwnerLoadError::MissingOwner)?;
        if self.NodeHandle(index) != owner {
            return Err(FrameOwnerLoadError::WrongOwnerDocument);
        }
        Ok(index)
    }
    pub(crate) fn InitializeFrameElementLoading(&mut self, node: usize) {
        if !self.IsFrameOwnerElement(node) || self.Node(node).frame_element_loading.is_some() {
            return;
        }
        let attribute = |name| self.Node(node).FindAttribute(name).map(|v| v.value.clone());
        let url = if attribute("srcdoc").is_some() {
            utf16("about:srcdoc")
        } else {
            strip_html_spaces(
                self.Node(node)
                    .AttributeValueUTF16("src")
                    .as_deref()
                    .unwrap_or(&[]),
            )
        };
        let name = self
            .Node(node)
            .AttributeValueUTF16("name")
            .unwrap_or_default();
        let sandbox = attribute("sandbox").map_or(SandboxFlags::default(), |value| {
            foundation::security::ParseSandboxPolicy(&value).0
        });
        let referrer = referrer_policy(attribute("referrerpolicy").as_deref());
        // DoesParentAllowLazyLoadingChildren, cc:101-113. The current DOM
        // creates only root Documents; a future child constructor passes its
        // actual embedding owner's retained flag through the setter below.
        self.NodeMut(node).frame_element_loading = Some(Rc::new(FrameElementLoadingState::new(
            url, name, true, sandbox, referrer,
        )));
    }
    pub fn SetFrameElementParentAllowsLazyChildren(&self, node: usize, allowed: bool) {
        self.Node(node)
            .frame_element_loading
            .as_ref()
            .expect("frame element state")
            .should_lazy_load_children
            .set(allowed);
    }
    fn NotifyFrameOwnerLoading(&self, notification: FrameOwnerLoadNotification) {
        let context = self.FrameLoadingDocument().creation_context.get();
        if self.AttributeLifecycle().HasFrameOwnerDocument() {
            self.AttributeLifecycle()
                .RecordFrameOwnerLoading(FrameOwnerLoadChange::Notification(context, notification));
            return;
        }
        self.FrameLoadingDocument()
            .notifications
            .borrow_mut()
            .push_back((context, notification));
    }
    pub(crate) fn FrameElementDidNotifySubtreeInsertions(&self, node: usize) {
        self.NotifyFrameOwnerLoading(FrameOwnerLoadNotification::DidNotifySubtreeInsertions(
            self.NodeHandle(node),
        ));
    }
    /// The owning synchronous DOM transaction drains these before parser or
    /// script reentry, just like AttributeLifecycle. A notification is not a Frame.
    pub fn TakeFrameOwnerLoadNotifications(
        &self,
    ) -> VecDeque<(FrameOwnerCreationContext, FrameOwnerLoadNotification)> {
        std::mem::take(&mut *self.FrameLoadingDocument().notifications.borrow_mut())
    }
    pub fn TakeFrameOwnerLoadEffects(&self) -> VecDeque<FrameOwnerLoadEffect> {
        std::mem::take(&mut *self.FrameLoadingDocument().effects.borrow_mut())
    }
    /// HTMLFrameOwnerElement.cc:672-679. Invoke the real retained observer;
    /// presence of loading="lazy" alone does not imply a pending load.
    pub fn LoadFrameImmediatelyIfLazy(&self, node: usize) -> Result<bool, FrameOwnerLoadError> {
        let Some(observer) = self.FrameLazyLoadObserver(node) else {
            return Ok(false);
        };
        let pending = observer.IsLazyLoadPending();
        if pending {
            observer.LoadImmediately()?;
        }
        Ok(pending)
    }
    fn FrameLoadEffect(&self, effect: FrameOwnerLoadEffect) {
        if self.AttributeLifecycle().HasFrameOwnerDocument() {
            self.AttributeLifecycle()
                .RecordFrameOwnerLoading(FrameOwnerLoadChange::Effect(effect));
            return;
        }
        self.FrameLoadingDocument()
            .effects
            .borrow_mut()
            .push_back(effect);
    }
    pub(crate) fn FrameElementLoadingAttributeChanged(
        &mut self,
        node: usize,
        change: &crate::attribute_lifecycle::AttributeChange,
    ) {
        if !self.IsFrameOwnerElement(node) || !change.namespace.is_empty() {
            return;
        }
        self.InitializeFrameElementLoading(node);
        let state = self
            .Node(node)
            .frame_element_loading
            .as_ref()
            .unwrap()
            .clone();
        let owner = self.NodeHandle(node);
        let new_location = match change.name.as_str() {
            "srcdoc" => {
                if let Some(frame) = self.ContentFrame(node) {
                    self.FrameLoadEffect(FrameOwnerLoadEffect::SrcDocChanged {
                        frame_identity: frame.Identity(),
                        value: change.new_value_utf16.clone().unwrap_or_default(),
                    });
                }
                if change.new_value.is_some() {
                    Some(utf16("about:srcdoc"))
                } else if let Some(src) = self.Node(node).AttributeValueUTF16("src") {
                    Some(strip_html_spaces(&src))
                } else if change.old_value.is_some() {
                    Some(utf16("about:blank"))
                } else {
                    None
                }
            }
            "src" if self.Node(node).FindAttribute("srcdoc").is_none() => Some(strip_html_spaces(
                change.new_value_utf16.as_deref().unwrap_or(&[]),
            )),
            "id" | "name" => {
                *state.name.borrow_mut() = change.new_value_utf16.clone().unwrap_or_default();
                None
            }
            "loading" => {
                self.NotifyFrameOwnerLoading(FrameOwnerLoadNotification::LoadingAttributeChanged(
                    owner,
                ));
                None
            }
            "sandbox" => {
                state.sandbox.set(
                    change
                        .new_value
                        .as_deref()
                        .map_or(SandboxFlags::default(), |value| {
                            foundation::security::ParseSandboxPolicy(value).0
                        }),
                );
                self.NotifyFrameOwnerLoading(FrameOwnerLoadNotification::UpdatePolicy(owner));
                None
            }
            "referrerpolicy" => {
                state
                    .referrer_policy
                    .set(referrer_policy(change.new_value.as_deref()));
                None
            }
            "allow" | "policy" => {
                self.NotifyFrameOwnerLoading(FrameOwnerLoadNotification::UpdatePolicy(owner));
                None
            }
            _ => None,
        };
        if let Some(url) = new_location {
            *state.url.borrow_mut() = url;
            if change.was_connected {
                self.NotifyFrameOwnerLoading(FrameOwnerLoadNotification::OpenURL {
                    owner,
                    replace_current_item: false,
                });
            }
        }
    }
    pub fn EnterFrameOwnerCreationContext(
        &self,
        context: FrameOwnerCreationContext,
    ) -> FrameOwnerCreationScope {
        let cell = self.FrameLoadingDocument().creation_context.clone();
        let previous = cell.replace(context);
        FrameOwnerCreationScope {
            context: cell,
            previous,
        }
    }
    fn ProcessFrameOwnerLoadNotificationWithContext(
        &mut self,
        notification: FrameOwnerLoadNotification,
        context: FrameOwnerCreationContext,
        services: &dyn FrameOwnerLoadingServices,
    ) -> Result<bool, FrameOwnerLoadError> {
        let _context = self.EnterFrameOwnerCreationContext(context);
        self.ProcessFrameOwnerLoadNotification(notification, services)
    }
    pub fn ProcessFrameOwnerLoadNotification(
        &mut self,
        notification: FrameOwnerLoadNotification,
        services: &dyn FrameOwnerLoadingServices,
    ) -> Result<bool, FrameOwnerLoadError> {
        match notification {
            FrameOwnerLoadNotification::UpdatePolicy(owner) => {
                let node = self.FrameLoadingIndex(owner)?;
                if services
                    .ParentFrame(self.NodeDocumentHandle(node))?
                    .is_none()
                {
                    return Ok(false);
                }
                self.UpdateFrameOwnerLoadingPolicy(node)?;
                Ok(true)
            }
            FrameOwnerLoadNotification::DidNotifySubtreeInsertions(owner) => {
                let node = self.FrameLoadingIndex(owner)?;
                // cc:185-196: re-read ContentFrame after any insertion reentry.
                if services
                    .ParentFrame(self.NodeDocumentHandle(node))?
                    .is_none()
                    || !self.CanLoadSubframe(node)
                    || self.ContentFrame(node).is_some()
                {
                    return Ok(false);
                }
                let state = self.Node(node).frame_element_loading.as_ref().unwrap();
                *state.name.borrow_mut() = self
                    .Node(node)
                    .AttributeValueUTF16("name")
                    .unwrap_or_default();
                // SetNameAndOpenURL -> OpenURL's header default is true.
                self.OpenFrameElementURL(node, true, services)
            }
            FrameOwnerLoadNotification::OpenURL {
                owner,
                replace_current_item,
            } => {
                let node = self.FrameLoadingIndex(owner)?;
                self.OpenFrameElementURL(node, replace_current_item, services)
            }
            FrameOwnerLoadNotification::LoadingAttributeChanged(owner) => {
                let node = self.FrameLoadingIndex(owner)?;
                let Some(parent) = services.ParentFrame(self.NodeDocumentHandle(node))? else {
                    return Ok(false);
                };
                let loading = self
                    .Node(node)
                    .FindAttribute("loading")
                    .map(|v| v.value.as_str())
                    .unwrap_or("");
                let lazy = loading.eq_ignore_ascii_case("lazy");
                if loading.eq_ignore_ascii_case("eager")
                    || !(parent.lazy_load_enabled && parent.page_visible && lazy)
                {
                    self.Node(node)
                        .frame_element_loading
                        .as_ref()
                        .unwrap()
                        .should_lazy_load_children
                        .set(false);
                    if let Some(observer) = self.FrameLazyLoadObserver(node) {
                        if observer.IsLazyLoadPending() {
                            observer.LoadImmediately()?;
                        }
                    }
                }
                Ok(true)
            }
        }
    }
    fn OpenFrameElementURL(
        &mut self,
        node: usize,
        replace: bool,
        services: &dyn FrameOwnerLoadingServices,
    ) -> Result<bool, FrameOwnerLoadError> {
        // HTMLFrameElementBase.cc:62-87.
        let Some(parent) = services.ParentFrame(self.NodeDocumentHandle(node))? else {
            return Ok(false);
        };
        let state = self
            .Node(node)
            .frame_element_loading
            .as_ref()
            .unwrap()
            .clone();
        if state.url.borrow().is_empty() {
            *state.url.borrow_mut() = utf16("about:blank");
        }
        // KURL consumes scalar input here; retain the original WTF-16 members
        // until this URL boundary, including unpaired surrogates in names/srcdoc.
        let source: String = char::decode_utf16(state.url.borrow().iter().copied())
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        let url = DocumentURLValue::Resolve(&self.BaseURL(), &source);
        if let Some(frame) = self.ContentFrame(node) {
            if !services.CanNavigate(frame.as_ref(), &url)? {
                return Ok(false);
            }
        }
        let owner = self.NodeHandle(node);
        if !url.IsValid() && self.BaseURL().starts_with("data:") {
            self.FrameLoadEffect(FrameOwnerLoadEffect::InvalidRelativeURL { owner, source });
        }
        // HTMLFrameOwnerElement.cc:727-799.
        if url.starts_with("http:") || url.starts_with("https:") {
            *state.fallback_timing.borrow_mut() = Some(FrameOwnerFallbackTiming {
                started_at: services.MonotonicTime(),
                url: url.clone(),
            });
        }
        if self
            .Node(node)
            .FindAttribute("loading")
            .is_some_and(|a| a.value.eq_ignore_ascii_case("eager"))
        {
            state.should_lazy_load_children.set(false);
        }
        let policy = self.UpdateFrameOwnerLoadingPolicy(node)?;
        let trust_token = match self.Node(node).FindAttribute("trusttoken") {
            Some(value) => services.TrustTokenParams(owner, &value.value)?,
            None => None,
        };
        let request = FrameOwnerResourceRequest {
            url: url.clone(),
            method: "GET".into(),
            referrer: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
            encoded_body: None,
            navigation: None,
            referrer_policy: state.referrer_policy.get(),
            trust_token,
            bypass_cache: false,
            skip_service_worker: false,
        };
        if let Some(frame) = self.ContentFrame(node) {
            let navigation = FrameOwnerNavigation {
                owner,
                request,
                policy,
                load_type: if replace {
                    FrameLoadType::ReplaceCurrentItem
                } else {
                    FrameLoadType::Standard
                },
                reason: FrameOwnerNavigationReason::FrameNavigation,
                is_container_initiated: true,
            };
            if !self.LazyLoadFrameIfPossible(
                node,
                &url,
                &navigation.request,
                navigation.load_type,
                services,
            )? {
                self.FrameLoadEffect(FrameOwnerLoadEffect::Navigate { frame, navigation });
            }
            return Ok(true);
        }
        if !self.CanLoadSubframe(node) || parent.subframe_count >= parent.maximum_frames {
            return Ok(false);
        }
        let name = state.name.borrow().clone();
        self.FrameLoadEffect(FrameOwnerLoadEffect::CreateLocalFrame(
            CreateLocalFrameRequest {
                owner,
                name,
                is_created_by_script: self.FrameLoadingDocument().creation_context.get()
                    == FrameOwnerCreationContext::Script,
                navigation: FrameOwnerNavigation {
                    owner,
                    request,
                    policy,
                    load_type: FrameLoadType::ReplaceCurrentItem,
                    reason: FrameOwnerNavigationReason::InitialFrameNavigation,
                    is_container_initiated: true,
                },
                original_url: url,
            },
        ));
        Ok(true)
    }
    fn UpdateFrameOwnerLoadingPolicy(
        &mut self,
        node: usize,
    ) -> Result<FrameOwnerPolicy, FrameOwnerLoadError> {
        // Source cc:445-483, called in this order by cc:759-760.
        let state = self
            .Node(node)
            .frame_element_loading
            .as_ref()
            .unwrap()
            .clone();
        let sandbox = state.sandbox.get();
        let source: String = char::decode_utf16(state.url.borrow().iter().copied())
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        let url = DocumentURLValue::Resolve(&self.BaseURL(), &source);
        let container = self.ConstructFrameContainerPolicy(node, &url, sandbox);
        let mut policy = FrameOwnerPolicy {
            sandbox,
            container,
            required_document: state
                .policy
                .borrow()
                .as_ref()
                .map(|policy| policy.required_document.clone())
                .unwrap_or_default(),
        };
        *state.policy.borrow_mut() = Some(policy.clone());
        if let Some(frame) = self.ContentFrame(node) {
            self.FrameLoadEffect(FrameOwnerLoadEffect::DidChangeFramePolicy {
                frame_identity: frame.Identity(),
                policy: policy.clone(),
            });
        }
        if foundation::RuntimeEnabledFeatures::DocumentPolicyNegotiationEnabled() {
            policy.required_document = if self.Node(node).IsHTMLElement("iframe") {
                self.UpdateIFrameRequiredDocumentPolicy(node);
                self.IFrameRequiredDocumentPolicy(node)
            } else {
                self.RequiredDocumentPolicy()
            };
            *state.policy.borrow_mut() = Some(policy.clone());
            if let Some(frame) = self.ContentFrame(node) {
                self.FrameLoadEffect(FrameOwnerLoadEffect::DidChangeFramePolicy {
                    frame_identity: frame.Identity(),
                    policy: policy.clone(),
                });
            }
        }
        Ok(policy)
    }
    fn LazyLoadFrameIfPossible(
        &mut self,
        node: usize,
        url: &Rc<DocumentURLValue>,
        request: &FrameOwnerResourceRequest,
        load_type: FrameLoadType,
        services: &dyn FrameOwnerLoadingServices,
    ) -> Result<bool, FrameOwnerLoadError> {
        // HTMLFrameOwnerElement.cc:115-161,681-713. Never infer an observer
        // from an HTML loading attribute: create its original native owner.
        let Some(parent) = services.ParentFrame(self.NodeDocumentHandle(node))? else {
            return Ok(false);
        };
        let lazy = self
            .Node(node)
            .FindAttribute("loading")
            .is_some_and(|a| a.value.eq_ignore_ascii_case("lazy"));
        let state = self
            .Node(node)
            .frame_element_loading
            .as_ref()
            .unwrap()
            .clone();
        if !(url.starts_with("http:") || url.starts_with("https:"))
            || !url.IsValid()
            || !parent.can_execute_scripts
            || (!lazy && (!state.should_lazy_load_children.get() || services.CanAccessURL(url)))
        {
            return Ok(false);
        }
        state.should_lazy_load_children.set(false);
        if let Some(observer) = self.FrameLazyLoadObserver(node) {
            observer.CancelPendingLazyLoad();
        }
        let frame = self
            .ContentFrame(node)
            .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
        let observer = services.NewLazyLoadObserver(self.NodeHandle(node), frame.as_ref())?;
        self.BindFrameLazyLoadObserver(node, observer.clone());
        if parent.lazy_load_enabled && parent.page_visible && lazy {
            observer.DeferLoadUntilNearViewport(request.clone(), load_type)?;
            return Ok(true);
        }
        Ok(false)
    }
    /// Resume immediately after the client's source CreateFrame call. The
    /// client must already have called InitializeFrame on this same owner.
    /// A null result aborts; reentry may not substitute a different child Frame.
    pub fn CompleteLocalFrameCreation(
        &mut self,
        pending: CreateLocalFrameRequest,
        created: Option<Rc<dyn ContentFrameSource>>,
    ) -> Result<bool, FrameOwnerLoadError> {
        // HTMLFrameOwnerElement.cc:800-852.
        let node = self.FrameLoadingIndex(pending.owner)?;
        let Some(frame) = created else {
            return Ok(false);
        };
        if self
            .ContentFrame(node)
            .is_none_or(|bound| !Rc::ptr_eq(&bound, &frame))
        {
            return Err(FrameOwnerLoadError::UnexpectedCreatedFrame);
        }
        self.FrameLoadEffect(FrameOwnerLoadEffect::DidChangeAttributes {
            owner: pending.owner,
            frame_identity: frame.Identity(),
        });
        self.FrameLoadEffect(FrameOwnerLoadEffect::ContinueInitialNavigation { pending, frame });
        Ok(true)
    }
    fn ContinueInitialFrameNavigation(
        &mut self,
        mut pending: CreateLocalFrameRequest,
        frame: Rc<dyn ContentFrameSource>,
        services: &dyn FrameOwnerLoadingServices,
    ) -> Result<bool, FrameOwnerLoadError> {
        let node = self.FrameLoadingIndex(pending.owner)?;
        if let Some(policy) = self
            .Node(node)
            .frame_element_loading
            .as_ref()
            .unwrap()
            .policy
            .borrow()
            .as_ref()
        {
            pending.navigation.policy = policy.clone();
        }
        if let Some(parent) = services.ParentFrame(self.NodeDocumentHandle(node))? {
            if pending.original_url.AsStr() != "about:blank"
                && !self.LoadEventFinished()
                && parent.load_type == FrameLoadType::ReloadBypassingCache
            {
                pending.navigation.load_type = FrameLoadType::ReloadBypassingCache;
                pending.navigation.request.bypass_cache = true;
            }
        }
        pending.navigation.request.skip_service_worker =
            self.Node(node).IsHTMLElement("object") || self.Node(node).IsHTMLElement("embed");
        if self.FrameLazyLoadObserver(node).is_none()
            && self.LazyLoadFrameIfPossible(
                node,
                &pending.original_url,
                &pending.navigation.request,
                pending.navigation.load_type,
                services,
            )?
        {
            return Ok(true);
        }
        if !services.AllowRequestForThisFrame(frame.as_ref(), &pending.navigation)? {
            return Ok(false);
        }
        self.FrameLoadEffect(FrameOwnerLoadEffect::StartNavigation {
            frame,
            navigation: pending.navigation,
        });
        Ok(true)
    }
}

#[cfg(test)]
mod creation_context_tests {
    use super::*;
    #[test]
    fn retains_creation_entry_across_deferred_notifications_and_reentry() {
        let mut owner = crate::DOM::new();
        let document = owner.GetDocumentMut();
        let node = document.CreateElementDefault(
            crate::persistent_document::DOMNamespace::kHTML,
            "iframe".into(),
        );
        document.FrameElementDidNotifySubtreeInsertions(node);
        {
            let _script =
                document.EnterFrameOwnerCreationContext(FrameOwnerCreationContext::Script);
            document.FrameElementDidNotifySubtreeInsertions(node);
            {
                let _parser = document
                    .EnterFrameOwnerCreationContext(FrameOwnerCreationContext::ParserOrHost);
                document.FrameElementDidNotifySubtreeInsertions(node);
            }
            document.FrameElementDidNotifySubtreeInsertions(node);
        }
        document.FrameElementDidNotifySubtreeInsertions(node);
        let contexts: Vec<_> = document
            .TakeFrameOwnerLoadNotifications()
            .into_iter()
            .map(|(context, _)| context)
            .collect();
        assert_eq!(
            contexts,
            [
                FrameOwnerCreationContext::ParserOrHost,
                FrameOwnerCreationContext::Script,
                FrameOwnerCreationContext::ParserOrHost,
                FrameOwnerCreationContext::Script,
                FrameOwnerCreationContext::ParserOrHost
            ]
        );
    }
}
