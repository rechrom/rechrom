//! DocumentLoader's retained navigation/history owner and synchronous commit.
//! Chromium document_loader.cc:1038-1272,1284-1364; history_item.cc:147-269.
//! Each stage is consumed synchronously; script stages release all DOM/loader
//! borrows. Page routes stages and never decides history identity or load type.
use crate::InitialScrollState;
use dom::persistent_document::SecurityOrigin;
use navigation_history::{
    FrameLoadType, HistoryItemOwner, ReferrerPolicy, RequestorOrigin, SerializedScriptValue,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SameDocumentNavigationType {
    Fragment,
    HistoryApi,
    NavigationApiIntercept,
    PrerenderNoVarySearchActivation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryCommitType {
    Standard,
    BackForward,
    Inert,
}
impl HistoryCommitType {
    fn ForLoad(load: FrameLoadType) -> Self {
        match load {
            FrameLoadType::Standard => Self::Standard,
            FrameLoadType::BackForward | FrameLoadType::Restore => Self::BackForward,
            FrameLoadType::ReplaceCurrentItem
            | FrameLoadType::Reload
            | FrameLoadType::ReloadBypassingCache => Self::Inert,
        }
    }
}
/// Facts read from the original LocalFrame/Document, before the first stage.
/// A missing initiator is resolved to that Document's actual origin below.
pub struct SameDocumentNavigationFacts {
    pub document_url: String,
    pub is_on_initial_empty_document: bool,
    pub was_loading: bool,
    pub has_transient_user_activation: bool,
    pub document_origin: Rc<SecurityOrigin>,
}
/// The original NavigateEvent/history operation's immutable arguments. Values
/// with identity are borrowed shared owners, including a traversal's HistoryItem.
pub struct SameDocumentNavigationParams {
    pub new_url: String,
    pub history_item: Option<Rc<HistoryItemOwner>>,
    pub navigation_type: SameDocumentNavigationType,
    pub state: Option<Rc<dyn SerializedScriptValue>>,
    pub load_type: FrameLoadType,
    pub fire_popstate: bool,
    pub initiator_origin: Option<Rc<dyn foundation::security::SecurityOriginOwner>>,
    pub is_browser_initiated: bool,
    pub is_synchronously_committed: bool,
    pub has_ua_visual_transition: bool,
    pub should_skip_screenshot: bool,
    pub involvement: UserNavigationInvolvement,
    pub interaction_id: Option<[u64; 2]>,
}
pub use navigation_history::UserNavigationInvolvement;
#[derive(Clone, Debug)]
pub struct SameDocumentCommitNotification {
    pub commit_type: HistoryCommitType,
    pub load_type: FrameLoadType,
    pub navigation_type: SameDocumentNavigationType,
    pub is_synchronously_committed: bool,
    pub is_client_redirect: bool,
    pub is_browser_initiated: bool,
    pub should_skip_screenshot: bool,
    pub metrics_token: [u8; 16],
    pub caused_by_ad: bool,
}
/// Source call sites in original order. A caller must apply UpdateDocument
/// before asking for the next stage; event stages may reenter this same owner.
pub enum SameDocumentCommitStage {
    EnqueueHashchange {
        old_url: String,
        new_url: String,
        involvement: UserNavigationInvolvement,
    },
    ProgressStarted,
    UpdateDocument {
        url: String,
    },
    DidCommitProvisionalLoad {
        inert: bool,
    },
    DidFinishSameDocumentNavigation(SameDocumentCommitNotification),
    ProgressCompleted,
    UpdateNavigationApi {
        item: Rc<HistoryItemOwner>,
        load_type: FrameLoadType,
    },
    DispatchPopstate {
        state: Option<Rc<dyn SerializedScriptValue>>,
        has_ua_visual_transition: bool,
        involvement: UserNavigationInvolvement,
    },
    SoftNavigationCommitted {
        old_url: String,
        new_url: String,
        load_type: FrameLoadType,
        metrics_token: [u8; 16],
        interaction_id: Option<[u64; 2]>,
    },
    Done,
}
/// renderer_content_settings.mojom:9-14; DocumentLoader retains the
/// committed RendererContentSettings, not a second sandbox interpretation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RendererContentSettings {
    pub allow_popup: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum HistoryNavigationType {
    DifferentDocument,
    Fragment,
    HistoryApi,
}
// DocumentLoader.h:285-286,347; document_loader.cc:2115,3027.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DocumentLoaderState {
    NotStarted,
    Provisional,
    Committed,
    SentDidFinishLoad,
}
pub struct DocumentLoaderNavigationState {
    state: Cell<DocumentLoaderState>,
    // DocumentLoader.cc:569,2959-2960; LocalDOMWindow stores this same
    // committed loader token. It is independent of Document::Token.
    initiator_state_token: Cell<[u64; 2]>,
    navigation_scroll_allowed: Cell<Option<bool>>,
    cross_document_navigation: RefCell<Option<navigation_history::NavigationRequestMetadata>>,
    load_type: Cell<Option<FrameLoadType>>,
    url: RefCell<String>,
    history_item: RefCell<Option<Rc<HistoryItemOwner>>>,
    previous_document_history_item: RefCell<Option<Rc<HistoryItemOwner>>>,
    history_item_initialized_for_window: Cell<bool>,
    http_method: RefCell<String>,
    http_body: RefCell<Option<Arc<fileapi::encoded_form_data::EncodedFormData>>>,
    content_type: RefCell<Option<String>>,
    response_mime_type: RefCell<String>,
    response_http_status: Cell<i64>,
    internal_scroll_to_text_fragment: RefCell<Option<String>>,
    referrer: RefCell<String>,
    requestor_origin: RefCell<Option<RequestorOrigin>>,
    referrer_policy: Cell<ReferrerPolicy>,
    replaces_current_history_item: Cell<bool>,
    is_client_redirect: Cell<bool>,
    navigation_type: Cell<navigation_history::WebNavigationType>,
    last_navigation_had_trusted_initiator: Cell<bool>,
    last_navigation_had_transient_user_activation: Cell<bool>,
    has_text_fragment_token: Cell<bool>,
    text_fragment_token_had_trusted_initiator: Cell<bool>,
    initial_scroll_state: Rc<InitialScrollState>,
    load_timing: RefCell<Option<Rc<crate::DocumentLoadTiming>>>,
    renderer_content_settings: Cell<RendererContentSettings>,
}
impl DocumentLoaderNavigationState {
    pub(crate) fn new(initial_scroll_state: Rc<InitialScrollState>) -> Rc<Self> {
        Rc::new(Self {
            state: Cell::new(DocumentLoaderState::NotStarted),
            initiator_state_token: Cell::new(dom::navigation_tokens::CreateNavigationToken()),
            navigation_scroll_allowed: Cell::new(None),
            cross_document_navigation: RefCell::new(None),
            load_type: Cell::new(None),
            url: RefCell::new(String::new()),
            history_item: RefCell::new(None),
            previous_document_history_item: RefCell::new(None),
            history_item_initialized_for_window: Cell::new(false),
            http_method: RefCell::new("GET".into()),
            http_body: RefCell::new(None),
            content_type: RefCell::new(None),
            response_mime_type: RefCell::new(String::new()),
            response_http_status: Cell::new(0),
            internal_scroll_to_text_fragment: RefCell::new(None),
            referrer: RefCell::new(String::new()),
            requestor_origin: RefCell::new(None),
            referrer_policy: Cell::new(ReferrerPolicy::Default),
            replaces_current_history_item: Cell::new(false),
            is_client_redirect: Cell::new(false),
            navigation_type: Cell::new(navigation_history::WebNavigationType::Other),
            last_navigation_had_trusted_initiator: Cell::new(false),
            last_navigation_had_transient_user_activation: Cell::new(false),
            has_text_fragment_token: Cell::new(false),
            text_fragment_token_had_trusted_initiator: Cell::new(false),
            initial_scroll_state,
            load_timing: RefCell::new(Some(crate::DocumentLoadTiming::new(Rc::new(
                std::time::Instant::now,
            )))),
            renderer_content_settings: Cell::new(RendererContentSettings::default()),
        })
    }
    pub fn LoadTiming(&self) -> Rc<crate::DocumentLoadTiming> {
        self.load_timing
            .borrow()
            .as_ref()
            .expect("loader constructed its canonical load timing")
            .clone()
    }
    pub fn SetCommitted(&self) {
        self.state.set(DocumentLoaderState::Committed);
    }
    pub fn SetProvisional(&self) {
        self.state.set(DocumentLoaderState::Provisional);
    }
    pub fn SetSentDidFinishLoad(&self) {
        self.state.set(DocumentLoaderState::SentDidFinishLoad);
    }
    pub fn SentDidFinishLoad(&self) -> bool {
        self.state.get() == DocumentLoaderState::SentDidFinishLoad
    }
    /// DocumentLoader.h:291-293; CommitSameDocumentNavigationInternal cc:1882-1886.
    pub fn SetNavigationType(&self, kind: navigation_history::WebNavigationType) {
        self.navigation_type.set(kind);
    }
    pub fn NavigationType(&self) -> navigation_history::WebNavigationType {
        self.navigation_type.get()
    }
    pub fn SetIsClientRedirect(&self, redirect: bool) {
        self.is_client_redirect.set(redirect);
    }
    pub fn SetLoadType(&self, load: FrameLoadType) {
        self.load_type.set(Some(load));
    }
    pub fn RendererContentSettings(&self) -> RendererContentSettings {
        self.renderer_content_settings.get()
    }
    pub fn InitiatorStateToken(&self) -> [u64; 2] {
        self.initiator_state_token.get()
    }
    /// A logical new DocumentLoader replaces the initial transport loader on
    /// response commit. Same-document URL/history changes do not call this.
    pub fn CommitNewDocumentInitiatorState(&self) {
        self.initiator_state_token
            .set(dom::navigation_tokens::CreateNavigationToken());
    }
    pub(crate) fn CommitDocumentPolicy(&self, policy: Rc<dom::document_policy::DocumentPolicy>) {
        // DocumentLoader.cc3174: snapshot from this committed enforce owner.
        self.navigation_scroll_allowed.set(Some(
            !policy.IsFeatureEnabled(dom::document_policy::DocumentPolicyFeature::ForceLoadAtTop),
        ));
    }
    pub fn NavigationScrollAllowed(&self) -> Option<bool> {
        self.navigation_scroll_allowed.get()
    }
    pub fn SetRendererContentSettings(&self, settings: RendererContentSettings) {
        self.renderer_content_settings.set(settings);
    }
    pub fn URL(&self) -> String {
        self.url.borrow().clone()
    }
    pub fn CurrentHistoryItem(&self) -> Option<Rc<HistoryItemOwner>> {
        self.history_item.borrow().clone()
    }
    /// The prior loader's canonical item survives until SetHistoryItemState-
    /// ForCommit; it never becomes the new loader's current history owner.
    pub fn SetPreviousDocumentHistoryItem(&self, item: Option<Rc<HistoryItemOwner>>) {
        *self.previous_document_history_item.borrow_mut() = item;
    }
    pub fn InitialScrollState(&self) -> Rc<InitialScrollState> {
        self.initial_scroll_state.clone()
    }
    pub fn LastNavigationHadTrustedInitiator(&self) -> bool {
        self.last_navigation_had_trusted_initiator.get()
    }
    pub fn LastNavigationHadTransientUserActivation(&self) -> bool {
        self.last_navigation_had_transient_user_activation.get()
    }
    /// Missing request metadata remains explicit; a URL alone does not prove
    /// an Other/browser-initiated load or an activation state.
    pub fn CrossDocumentNavigationMetadata(
        &self,
    ) -> Option<navigation_history::NavigationRequestMetadata> {
        self.cross_document_navigation.borrow().clone()
    }
    pub fn LoadType(&self) -> Option<FrameLoadType> {
        self.load_type.get()
    }
    /// The actual commit URL, before Document::SetURL strips its fragment
    /// directive. This URL belongs to DocumentLoader, independently of DOM URL.
    pub fn SetDocumentNavigationURL(&self, url: String) {
        *self.url.borrow_mut() = url;
    }
    pub fn HasTextFragmentToken(&self) -> bool {
        self.has_text_fragment_token.get()
    }
    /// DocumentLoader.h:392-394; this is the initiator associated with the
    /// consumed permission token, which can differ from the latest navigation.
    pub fn TextFragmentTokenHadTrustedInitiator(&self) -> bool {
        self.text_fragment_token_had_trusted_initiator.get()
    }
    /// Document::contentType uses the committed response MIME, independently
    /// of the POST request Content-Type retained for HistoryItem form data.
    pub fn MimeType(&self) -> String {
        self.response_mime_type.borrow().clone()
    }
    /// DocumentLoader.h:428-437. The value is consumed once by the native
    /// TextFragmentAnchor; absence is the source constructor state.
    pub fn TakeInternalScrollToTextFragment(&self) -> Option<String> {
        self.internal_scroll_to_text_fragment.borrow_mut().take()
    }
    pub fn HasInternalScrollToTextFragment(&self) -> bool {
        self.internal_scroll_to_text_fragment.borrow().is_some()
    }
    pub fn ConsumeTextFragmentToken(&self) -> bool {
        self.has_text_fragment_token.replace(false)
    }
    pub(crate) fn SetRequest(&self, request: &url_loader::URLRequest) {
        self.SetProvisional();
        *self.url.borrow_mut() = request.url.clone();
        *self.cross_document_navigation.borrow_mut() = request.navigation.clone();
        self.load_type.set(
            request
                .navigation
                .as_ref()
                .map(|metadata| metadata.load_type),
        );
        if let Some(navigation) = &request.navigation {
            if navigation.load_type.IsTraversal() {
                *self.history_item.borrow_mut() = Some(HistoryItemOwner::Restore(
                    navigation
                        .history_item
                        .as_deref()
                        .expect("admitted traversal destination"),
                ));
            }
            self.navigation_type.set(navigation.navigation_type);
            *self.requestor_origin.borrow_mut() = navigation.requestor_origin.clone();
            self.last_navigation_had_transient_user_activation
                .set(navigation.had_transient_user_activation);
            let generated = matches!(
                navigation.navigation_type,
                navigation_history::WebNavigationType::LinkClicked
                    | navigation_history::WebNavigationType::Other
            ) && (navigation.had_transient_user_activation
                || navigation.is_browser_initiated);
            self.has_text_fragment_token
                .set(generated || navigation.propagated_text_fragment_token.is_some());
            if let Some(trusted) = navigation.propagated_text_fragment_token {
                self.text_fragment_token_had_trusted_initiator.set(trusted);
            }
            *self.internal_scroll_to_text_fragment.borrow_mut() =
                navigation.internal_scroll_to_text_fragment.clone();
        }
        *self.referrer.borrow_mut() = request.referrer.clone();
        self.referrer_policy.set(match request.referrer_policy {
            None => ReferrerPolicy::Default,
            Some(url_loader::ReferrerPolicy::NoReferrer) => ReferrerPolicy::Never,
            Some(url_loader::ReferrerPolicy::NoReferrerWhenDowngrade) => {
                ReferrerPolicy::NoReferrerWhenDowngrade
            }
            Some(url_loader::ReferrerPolicy::Origin) => ReferrerPolicy::Origin,
            Some(url_loader::ReferrerPolicy::OriginWhenCrossOrigin) => {
                ReferrerPolicy::OriginWhenCrossOrigin
            }
            Some(url_loader::ReferrerPolicy::SameOrigin) => ReferrerPolicy::SameOrigin,
            Some(url_loader::ReferrerPolicy::StrictOrigin) => ReferrerPolicy::StrictOrigin,
            Some(url_loader::ReferrerPolicy::StrictOriginWhenCrossOrigin) => {
                ReferrerPolicy::StrictOriginWhenCrossOrigin
            }
            Some(url_loader::ReferrerPolicy::UnsafeUrl) => ReferrerPolicy::Always,
        });
        *self.http_method.borrow_mut() = request.method.clone();
        *self.http_body.borrow_mut() = request.encoded_body.clone().or_else(|| {
            (!request.body.is_empty()).then(|| {
                fileapi::encoded_form_data::EncodedFormData::FromBytes(Arc::<[u8]>::from(
                    request.body.as_slice(),
                ))
            })
        });
        *self.content_type.borrow_mut() = request
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("Content-Type"))
            .map(|(_, value)| value.clone());
    }
    /// Loader commit installs one HistoryItem before parser/script exposure.
    /// Response and original Document origin are already committed here.
    pub(crate) fn InitializeForDocument(
        &self,
        document: &dom::Document,
        response: Option<&url_loader::URLResponseHead>,
    ) -> std::io::Result<()> {
        self.SetCommitted();
        if self.cross_document_navigation.borrow().is_some() {
            // DocumentLoader.cc:3323-3333: compare the original requestor
            // origin after the response Document has its committed origin.
            let origin = document.SecurityContext().Origin();
            let trusted = self
                .requestor_origin
                .borrow()
                .as_ref()
                .is_none_or(|requestor| match requestor {
                    RequestorOrigin::Tuple { scheme, host, port } => {
                        origin.Tuple().is_some_and(|tuple| {
                            tuple.scheme == *scheme && tuple.host == *host && tuple.port == *port
                        })
                    }
                    RequestorOrigin::Opaque(nonce) => origin.OpaqueNonce() == Some(*nonce),
                });
            self.last_navigation_had_trusted_initiator.set(trusted);
            if self.has_text_fragment_token.get() {
                self.text_fragment_token_had_trusted_initiator.set(trusted);
            }
        }
        if let Some(response) = response {
            *self.url.borrow_mut() = response.final_url.clone();
        }
        *self.response_mime_type.borrow_mut() =
            response.map_or_else(String::new, |head| head.mime_type.clone());
        self.response_http_status
            .set(response.map_or(0, |head| head.status_code));
        if let Some(referrer) = response.and_then(|head| head.request_referrer.clone()) {
            *self.referrer.borrow_mut() = referrer;
        }
        self.EnsureHistoryItemForWindow(document);
        // DidInstallNewDocument document_loader.cc:2295-2297. Restore controls
        // before parser exposure, including documents without script bindings.
        if self.LoadType().is_some_and(FrameLoadType::IsTraversal) {
            let controller = document.FormController().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "history traversal requires the Document's form controller",
                )
            })?;
            controller.SetStateForNewControls(
                &self
                    .CurrentHistoryItem()
                    .expect("admitted traversal")
                    .DocumentState(),
            );
        }
        self.AttachDocumentState(document);
        Ok(())
    }
    /// Window's Navigation owner is initialized at URL commit, before parser
    /// entry. The parser's response initialization retains this same item.
    pub fn EnsureHistoryItemForWindow(&self, document: &dom::Document) {
        if self.history_item_initialized_for_window.replace(true) {
            return;
        }
        if self.history_item.borrow().is_none() {
            let old = self
                .previous_document_history_item
                .borrow_mut()
                .take()
                .or_else(|| {
                    self.cross_document_navigation
                        .borrow()
                        .as_ref()
                        .and_then(|metadata| metadata.history_item.as_deref())
                        .map(HistoryItemOwner::Restore)
                });
            if let Some(load) = self.LoadType() {
                self.SetHistoryItemStateForCommit(
                    old,
                    load,
                    HistoryNavigationType::DifferentDocument,
                    false,
                    document.SecurityContext().Origin(),
                );
            } else {
                let item = HistoryItemOwner::Create();
                self.WriteRequestToHistoryItem(&item);
                *self.history_item.borrow_mut() = Some(item);
            }
        } else if self.LoadType().is_some_and(FrameLoadType::IsTraversal) {
            // The destination identities/state stay intact. As in Chromium's
            // SetHistoryItemStateForCommit, update only committed request facts.
            self.WriteRequestToHistoryItem(self.history_item.borrow().as_ref().unwrap());
        }
        if let Some(state) = self
            .cross_document_navigation
            .borrow()
            .as_ref()
            .and_then(|metadata| metadata.navigation_api_state.clone())
        {
            self.history_item
                .borrow()
                .as_ref()
                .unwrap()
                .SetNavigationApiState(Some(navigation_history::StoredSerializedScriptValue::new(
                    state,
                )));
        }
        // Document::GetFormController attaches ControlStates to the loader's
        // item (document.cc:2367-2374). Page installs the controller before this
        // item exists, so complete that attachment here. A traversal must first
        // consume its stored vector in DidInstallNewDocument above; attaching
        // the new, still-empty live owner earlier would destroy the input.
        if !self.LoadType().is_some_and(FrameLoadType::IsTraversal) {
            self.AttachDocumentState(document);
        }
    }
    fn AttachDocumentState(&self, document: &dom::Document) {
        if let (Some(item), Some(controller)) =
            (self.CurrentHistoryItem(), document.FormController())
        {
            item.SetDocumentStateOwner(controller.ControlStates());
        }
    }
    fn WriteRequestToHistoryItem(&self, item: &HistoryItemOwner) {
        item.SetURL(self.URL());
        item.SetReferrer(self.referrer.borrow().clone());
        item.SetRequestorOrigin(self.requestor_origin.borrow().clone());
        item.SetReferrerPolicy(self.referrer_policy.get());
        if self.http_method.borrow().eq_ignore_ascii_case("POST") {
            item.SetFormData(self.http_body.borrow().clone());
            item.SetFormContentType(self.content_type.borrow().clone());
        } else {
            item.SetFormData(None);
            item.SetFormContentType(None);
        }
    }
    /// DocumentLoader.cc1284-1364. One HistoryItem owner algorithm is shared
    /// by same-document updates and static different-document commits.
    fn SetHistoryItemStateForCommit(
        &self,
        old: Option<Rc<HistoryItemOwner>>,
        load: FrameLoadType,
        kind: HistoryNavigationType,
        force_new_document_sequence_number: bool,
        original_origin: &SecurityOrigin,
    ) -> Rc<HistoryItemOwner> {
        let item = if load.IsTraversal() {
            self.CurrentHistoryItem()
                .expect("traversal's source history item")
        } else {
            HistoryItemOwner::Create()
        };
        self.WriteRequestToHistoryItem(&item);
        if let Some(old) = old.filter(|_| !load.IsTraversal()) {
            let registry = Rc::new(original_origin.SchemeRegistryForRequests());
            let old_origin =
                SecurityOrigin::CreateForNavigation(&old.URL(), None, registry.clone());
            let new_origin = SecurityOrigin::CreateForNavigation(&item.URL(), None, registry);
            let inert = HistoryCommitType::ForLoad(load) == HistoryCommitType::Inert;
            if inert && old_origin.CanAccess(&new_origin) {
                item.SetNavigationApiKey(old.NavigationApiKey());
            }
            if load.IsReload() {
                item.SetNavigationApiId(old.NavigationApiId());
            }
            if kind == HistoryNavigationType::Fragment || load.IsReload() {
                item.SetNavigationApiState(old.NavigationApiState());
            }
            let same_url_without_fragment = {
                let without_fragment = |url: String| url.split('#').next().unwrap_or("").to_owned();
                without_fragment(old.URL()) == without_fragment(item.URL())
            };
            if kind != HistoryNavigationType::DifferentDocument
                || (!force_new_document_sequence_number && inert && same_url_without_fragment)
            {
                item.SetDocumentSequenceNumber(old.DocumentSequenceNumber());
                item.CopyViewStateFrom(&old);
                item.SetScrollRestorationType(old.ScrollRestorationType());
                if inert && (kind == HistoryNavigationType::HistoryApi || old.URL() == item.URL()) {
                    item.SetStateObject(old.StateObject());
                    item.SetItemSequenceNumber(old.ItemSequenceNumber());
                }
            }
        }
        *self.history_item.borrow_mut() = Some(item.clone());
        item
    }
    /// FrameLoadRequest's original initiator, never the response document's
    /// origin. Browser-initiated loads keep the source null constructor value.
    pub fn SetNavigationRequestorOrigin(&self, origin: Option<RequestorOrigin>) {
        *self.requestor_origin.borrow_mut() = origin;
    }
    pub fn RunURLAndHistoryUpdateSteps(
        self: &Rc<Self>,
        params: SameDocumentNavigationParams,
        facts: SameDocumentNavigationFacts,
    ) -> SameDocumentCommit {
        assert_eq!(
            params.load_type.IsTraversal(),
            params.history_item.is_some(),
            "traversals retain their original HistoryItem"
        );
        let current = self.CurrentHistoryItem();
        let same_item_sequence_number = current
            .as_ref()
            .zip(params.history_item.as_ref())
            .is_some_and(|(a, b)| a.ItemSequenceNumber() == b.ItemSequenceNumber());
        if let Some(item) = &params.history_item {
            *self.history_item.borrow_mut() = Some(item.clone());
        }
        let load_type =
            if params.load_type == FrameLoadType::Standard && facts.is_on_initial_empty_document {
                FrameLoadType::ReplaceCurrentItem
            } else {
                params.load_type
            };
        SameDocumentCommit {
            owner: self.clone(),
            old_url: facts.document_url.clone(),
            params,
            facts,
            load_type,
            same_item_sequence_number,
            hashchange_checked: false,
            step: 0,
            document_updated: false,
            notification: None,
        }
    }
}
pub struct SameDocumentCommit {
    owner: Rc<DocumentLoaderNavigationState>,
    old_url: String,
    params: SameDocumentNavigationParams,
    facts: SameDocumentNavigationFacts,
    load_type: FrameLoadType,
    same_item_sequence_number: bool,
    hashchange_checked: bool,
    step: u8,
    document_updated: bool,
    notification: Option<SameDocumentCommitNotification>,
}
impl SameDocumentCommit {
    /// UpdateDocument's source continuation. DocumentState is produced by the
    /// original FormController, and fragment facts by the original directive
    /// parser. Neither can be replaced with an empty/default projection.
    pub fn ApplyDocumentUpdate(
        &mut self,
        document: &mut dom::Document,
        document_state: Rc<dyn navigation_history::DocumentState>,
    ) {
        assert_eq!(self.step, 2, "apply the emitted UpdateDocument stage once");
        assert!(!self.document_updated);
        document.SetDocumentURL(self.params.new_url.clone());
        let fragments = document.FragmentDirectiveState();
        let owner = &self.owner;
        *owner.url.borrow_mut() = self.params.new_url.clone();
        owner
            .replaces_current_history_item
            .set(self.load_type != FrameLoadType::Standard);
        let history_api = self.params.navigation_type != SameDocumentNavigationType::Fragment;
        if history_api {
            *owner.http_method.borrow_mut() = "GET".into();
            *owner.http_body.borrow_mut() = None;
        }
        let initiator: &dyn foundation::security::SecurityOriginOwner = self
            .params
            .initiator_origin
            .as_deref()
            .unwrap_or(self.facts.document_origin.as_ref());
        let http_family = url::Url::parse(&self.params.new_url)
            .is_ok_and(|url| matches!(url.scheme(), "http" | "https"));
        let trusted = foundation::security::IsSameSecurityOrigin(
            initiator,
            self.facts.document_origin.as_ref(),
        ) && http_family;
        owner.last_navigation_had_trusted_initiator.set(trusted);
        owner
            .last_navigation_had_transient_user_activation
            .set(self.facts.has_transient_user_activation);
        if self.load_type == FrameLoadType::Standard
            || matches!(
                self.params.navigation_type,
                SameDocumentNavigationType::Fragment
                    | SameDocumentNavigationType::NavigationApiIntercept
            )
        {
            let token = matches!(
                self.load_type,
                FrameLoadType::Standard | FrameLoadType::ReplaceCurrentItem
            ) && matches!(
                self.params.navigation_type,
                SameDocumentNavigationType::Fragment
                    | SameDocumentNavigationType::NavigationApiIntercept
            ) && trusted
                && fragments.LastNavigationHadFragmentDirective()
                && fragments.HasTextDirective();
            owner.has_text_fragment_token.set(token);
            if token {
                owner.text_fragment_token_had_trusted_initiator.set(trusted);
            }
        }
        let item = owner.SetHistoryItemStateForCommit(
            owner.CurrentHistoryItem(),
            self.load_type,
            if history_api {
                HistoryNavigationType::HistoryApi
            } else {
                HistoryNavigationType::Fragment
            },
            false,
            &self.facts.document_origin,
        );
        item.SetDocumentStateOwner(document_state);
        if history_api {
            item.SetStateObject(self.params.state.clone());
        }
        *owner.history_item.borrow_mut() = Some(item);
        let mut metrics_token = [0u8; 16];
        getrandom::fill(&mut metrics_token).expect("same-document metrics token entropy");
        // Stable revision's AdTagging runtime feature is disabled; the optional
        // AdTracker does not exist. Frame evidence still remains authoritative.
        assert!(!foundation::RuntimeEnabledFeatures::AdTaggingEnabled());
        self.notification = Some(SameDocumentCommitNotification {
            commit_type: HistoryCommitType::ForLoad(self.load_type),
            load_type: self.load_type,
            navigation_type: self.params.navigation_type,
            is_synchronously_committed: self.params.is_synchronously_committed,
            is_client_redirect: owner.is_client_redirect.get(),
            is_browser_initiated: self.params.is_browser_initiated,
            should_skip_screenshot: self.params.should_skip_screenshot,
            metrics_token,
            caused_by_ad: document.IsAdFrame(),
        });
        self.document_updated = true;
    }
    /// The caller reads actual frame attachment after each script-bearing
    /// stage. No script invocation or DOM/loader borrow is retained here.
    pub fn Next(&mut self, frame_attached: bool) -> SameDocumentCommitStage {
        // CommitSameDocumentNavigationInternal, document_loader.cc:1902-1911.
        // History API and intercepted commits enter the URL/history update
        // steps directly and do not take this fragment-navigation branch.
        if !self.hashchange_checked {
            self.hashchange_checked = true;
            if self.params.navigation_type == SameDocumentNavigationType::Fragment {
                let (old_base, old_fragment) =
                    self.old_url.split_once('#').unwrap_or((&self.old_url, ""));
                let (new_base, new_fragment) = self
                    .params
                    .new_url
                    .split_once('#')
                    .unwrap_or((&self.params.new_url, ""));
                if old_base == new_base && old_fragment != new_fragment {
                    return SameDocumentCommitStage::EnqueueHashchange {
                        old_url: self.old_url.clone(),
                        new_url: self.params.new_url.clone(),
                        involvement: self.params.involvement,
                    };
                }
            }
        }
        loop {
            let step = self.step;
            self.step = self.step.saturating_add(1);
            match step {
                0 if !self.facts.was_loading => return SameDocumentCommitStage::ProgressStarted,
                0 => {}
                1 => {
                    return SameDocumentCommitStage::UpdateDocument {
                        url: self.params.new_url.clone(),
                    }
                }
                2 => {
                    assert!(
                        self.document_updated,
                        "UpdateDocument stage must finish before commit notification"
                    );
                    return SameDocumentCommitStage::DidCommitProvisionalLoad {
                        inert: HistoryCommitType::ForLoad(self.load_type)
                            == HistoryCommitType::Inert,
                    };
                }
                3 => {
                    return SameDocumentCommitStage::DidFinishSameDocumentNavigation(
                        self.notification.clone().expect("history committed"),
                    )
                }
                4 if !self.facts.was_loading
                    && self.params.navigation_type
                        != SameDocumentNavigationType::NavigationApiIntercept =>
                {
                    return SameDocumentCommitStage::ProgressCompleted
                }
                4 => {}
                5 if !self.same_item_sequence_number => {
                    return SameDocumentCommitStage::UpdateNavigationApi {
                        item: self.owner.CurrentHistoryItem().expect("history committed"),
                        load_type: self.load_type,
                    }
                }
                5 => {}
                6 => {
                    if !frame_attached {
                        self.step = 9;
                        return SameDocumentCommitStage::Done;
                    }
                    if self.params.fire_popstate {
                        self.owner
                            .initial_scroll_state
                            .was_scrolled_by_user
                            .set(false);
                        if !self.same_item_sequence_number {
                            return SameDocumentCommitStage::DispatchPopstate {
                                state: self
                                    .params
                                    .history_item
                                    .as_ref()
                                    .and_then(|item| item.StateObject()),
                                has_ua_visual_transition: self.params.has_ua_visual_transition,
                                involvement: self.params.involvement,
                            };
                        }
                    }
                }
                7 if frame_attached => {
                    return SameDocumentCommitStage::SoftNavigationCommitted {
                        old_url: self.old_url.clone(),
                        new_url: self.params.new_url.clone(),
                        load_type: self.load_type,
                        metrics_token: self
                            .notification
                            .as_ref()
                            .expect("history committed")
                            .metrics_token,
                        interaction_id: self.params.interaction_id,
                    }
                }
                _ => return SameDocumentCommitStage::Done,
            }
        }
    }
}

#[path = "static_document_commit.rs"]
mod static_document_commit;
pub(crate) use static_document_commit::StaticDataNavigationBodyLoader;
pub use static_document_commit::{
    StaticDocumentCommit, StaticDocumentWindowChoice, StaticDocumentWindowDisposition,
    StaticDocumentWindowReuseFacts,
};
