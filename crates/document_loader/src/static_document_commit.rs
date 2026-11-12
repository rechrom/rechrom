//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! DocumentLoader.cc:739-844; ScriptController.cc:293-305. Snapshot the clone
//! parameters before unload can run script. The original history item is kept
//! separately: it is NOT the new loader's history item (cc:750-752).
use super::*;
use dom::persistent_document::{DOMOwnerHandle, SecurityOrigin};
use std::any::Any;

pub struct StaticDocumentCommit {
    source_document: DOMOwnerHandle,
    document_token: [u64; 2],
    navigation: Rc<DocumentLoaderNavigationState>,
    previous_history_item: Option<Rc<HistoryItemOwner>>,
    previous_origin: Rc<SecurityOrigin>,
    storage_key: foundation::security::StorageKey,
    body: Arc<[u8]>,
    cloned_http_status: i64,
    security: dom::persistent_document::JavaScriptURLDocumentSecurity,
    previous_document_was_initial_empty: bool,
    prepared_document_security: RefCell<Option<PreparedDocumentSecurity>>,
}
struct PreparedDocumentSecurity {
    document: DOMOwnerHandle,
    origin: Rc<SecurityOrigin>,
    csp: Arc<dom::content_security_policy::DocumentContentSecurityPolicy>,
}
impl StaticDocumentCommit {
    pub fn ForJavaScriptURL(
        current: &Rc<DocumentLoaderNavigationState>,
        document: &dom::Document,
        result: &[u16],
    ) -> Rc<Self> {
        let navigation = DocumentLoaderNavigationState::new(Rc::new(InitialScrollState::default()));
        // CloneParams uses LocalDOMWindow::Url, not the unstripped transport
        // URL retained by the previous loader. Redirect/client state is not
        // propagated, and the response is replaced by FillStaticResponse.
        *navigation.url.borrow_mut() = document.DocumentURL().to_owned();
        *navigation.http_method.borrow_mut() = current.http_method.borrow().clone();
        *navigation.http_body.borrow_mut() = current.http_body.borrow().clone();
        *navigation.referrer.borrow_mut() = current.referrer.borrow().clone();
        navigation
            .initiator_state_token
            .set(current.InitiatorStateToken());
        navigation
            .last_navigation_had_transient_user_activation
            .set(current.last_navigation_had_transient_user_activation.get());
        navigation
            .has_text_fragment_token
            .set(current.has_text_fragment_token.get());
        navigation
            .text_fragment_token_had_trusted_initiator
            .set(current.text_fragment_token_had_trusted_initiator.get());
        *navigation.internal_scroll_to_text_fragment.borrow_mut() =
            current.internal_scroll_to_text_fragment.borrow().clone();
        navigation
            .renderer_content_settings
            .set(current.renderer_content_settings.get());
        navigation.replaces_current_history_item.set(true);
        navigation
            .load_type
            .set(Some(FrameLoadType::ReplaceCurrentItem));
        navigation
            .response_http_status
            .set(current.response_http_status.get());
        *navigation.response_mime_type.borrow_mut() = "text/html".into();
        navigation
            .navigation_type
            .set(navigation_history::WebNavigationType::Determine(
                FrameLoadType::ReplaceCurrentItem,
                navigation.http_body.borrow().is_some(),
                false,
            ));
        // The new requestor origin is the WebNavigationParams constructor's
        // null value; CloneParams does not copy requestor_origin_.
        *navigation.cross_document_navigation.borrow_mut() = current
            .cross_document_navigation
            .borrow()
            .as_ref()
            .map(|metadata| navigation_history::NavigationRequestMetadata {
                load_type: FrameLoadType::ReplaceCurrentItem,
                history_item: None,
                back_entries: metadata.back_entries.clone(),
                forward_entries: metadata.forward_entries.clone(),
                navigation_api_state: None,
                navigation_type: navigation_history::WebNavigationType::Determine(
                    FrameLoadType::ReplaceCurrentItem,
                    navigation.http_body.borrow().is_some(),
                    false,
                ),
                is_browser_initiated: metadata.is_browser_initiated,
                had_transient_user_activation: metadata.had_transient_user_activation,
                requestor_origin: None,
                propagated_text_fragment_token: current
                    .has_text_fragment_token
                    .get()
                    .then(|| current.text_fragment_token_had_trusted_initiator.get()),
                internal_scroll_to_text_fragment: current
                    .internal_scroll_to_text_fragment
                    .borrow()
                    .clone(),
            });
        if navigation
            .cross_document_navigation
            .borrow()
            .as_ref()
            .is_some_and(|metadata| {
                matches!(
                    metadata.navigation_type,
                    navigation_history::WebNavigationType::LinkClicked
                        | navigation_history::WebNavigationType::Other
                ) && (metadata.had_transient_user_activation || metadata.is_browser_initiated)
            })
        {
            navigation.has_text_fragment_token.set(true);
        }
        // WTF StringUtf8Adaptor(kStrictReplacingErrors): decode scalar values,
        // replacing each unpaired UTF-16 surrogate, then encode UTF-8 once.
        let text: String = char::decode_utf16(result.iter().copied())
            .map(|value| value.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        Rc::new(Self {
            source_document: document.RootHandle(),
            document_token: document.Token(),
            previous_history_item: current.CurrentHistoryItem(),
            previous_origin: document.SecurityContext().Origin().clone(),
            storage_key: document
                .SecurityContext()
                .PublicURLManager()
                .StorageKey()
                .clone(),
            navigation,
            body: Arc::from(text.into_bytes()),
            cloned_http_status: current.response_http_status.get(),
            security: document.CaptureJavaScriptURLDocumentSecurity(),
            previous_document_was_initial_empty: document.IsInitialEmptyDocument(),
            prepared_document_security: RefCell::new(None),
        })
    }
    /// ShouldReuseDOMWindow cc2663-2707. A loaded Document always needs a
    /// new Window. Initial-empty reuse additionally requires the actual
    /// Window/navigation anonymous and COI-key comparison, never a guessed
    /// non-isolated default when those owners have not been supplied.
    pub fn WindowDisposition(
        &self,
        facts: Option<StaticDocumentWindowReuseFacts>,
    ) -> Option<StaticDocumentWindowChoice> {
        if self.previous_document_was_initial_empty && facts.is_none() {
            return None;
        }
        Some(StaticDocumentWindowChoice::Choose(
            self.source_document,
            self.previous_document_was_initial_empty,
            &self.previous_origin,
            &self.previous_origin.IsolatedCopy(),
            facts,
        ))
    }
    /// InitializeWindow/CreateCSP precede constructing the new Document's
    /// fetch context and bindings. Install these exact compiled owners before
    /// any downstream constructor can retain them; do not replace them later.
    pub fn PrepareDocumentSecurity(&self, document: &mut dom::Document) {
        assert_ne!(document.RootHandle(), self.source_document);
        assert!(
            self.prepared_document_security.borrow().is_none(),
            "one new Document per static commit"
        );
        self.security
            .InstallOnNewDocument(document, self.document_token, self.navigation.URL());
        *self.prepared_document_security.borrow_mut() = Some(PreparedDocumentSecurity {
            document: document.RootHandle(),
            origin: document.SecurityContext().Origin().clone(),
            csp: document.ContentSecurityPolicy(),
        });
    }
    /// Actual InstallNewDocument bookkeeping. Fetcher, bindings, DOM and the
    /// new native graph must already retain the prepared security/CSP owners.
    /// A late installation would leave those consumers on an empty policy.
    pub fn InstallDocumentSecurity(&self, document: &mut dom::Document) {
        let prepared = self.prepared_document_security.borrow();
        let prepared = prepared
            .as_ref()
            .expect("prepare security before constructing new document services");
        assert_eq!(prepared.document, document.RootHandle());
        assert_eq!(document.Token(), self.document_token);
        assert!(Rc::ptr_eq(
            document.SecurityContext().Origin(),
            &prepared.origin
        ));
        assert!(Arc::ptr_eq(
            &document.ContentSecurityPolicy(),
            &prepared.csp
        ));
        self.DidInstallNewDocument();
    }
    pub fn SourceDocument(&self) -> DOMOwnerHandle {
        self.source_document
    }
    pub fn DocumentToken(&self) -> [u64; 2] {
        self.document_token
    }
    pub fn NavigationState(&self) -> Rc<DocumentLoaderNavigationState> {
        self.navigation.clone()
    }
    pub fn PreviousHistoryItem(&self) -> Option<Rc<HistoryItemOwner>> {
        self.previous_history_item.clone()
    }
    pub fn PreviousOrigin(&self) -> &Rc<SecurityOrigin> {
        &self.previous_origin
    }
    pub fn StorageKey(&self) -> &foundation::security::StorageKey {
        &self.storage_key
    }
    pub fn Body(&self) -> Arc<[u8]> {
        self.body.clone()
    }
    /// CloneParams and FillStaticResponse preserve the actual HTTP status
    /// (web_navigation_params.cc:98-107), independently of response headers.
    pub fn ClonedHttpStatus(&self) -> i64 {
        self.cloned_http_status
    }
    /// FrameLoader.cc1518-1525. Invoked after the static loader's StartLoading,
    /// before TransitionToCommittedForNewPage and InstallNewDocument.
    pub(crate) fn DidInstallNewDocument(&self) {
        // DocumentLoader.cc3323-3333: CloneParams leaves requestor_origin null.
        self.navigation
            .last_navigation_had_trusted_initiator
            .set(true);
        if self.navigation.has_text_fragment_token.get() {
            self.navigation
                .text_fragment_token_had_trusted_initiator
                .set(true);
        }
    }
    pub fn SetHistoryItemStateForCommit(&self) {
        assert!(
            self.navigation.CurrentHistoryItem().is_none(),
            "static history committed once"
        );
        self.navigation.SetHistoryItemStateForCommit(
            self.previous_history_item.clone(),
            FrameLoadType::ReplaceCurrentItem,
            HistoryNavigationType::DifferentDocument,
            false,
            &self.previous_origin,
        );
    }
    pub fn Referrer(&self) -> String {
        self.navigation.referrer.borrow().clone()
    }
    pub fn LoadType(&self) -> FrameLoadType {
        FrameLoadType::ReplaceCurrentItem
    }
}
impl navigation_history::StaticDocumentCommitPayload for StaticDocumentCommit {
    fn AsAny(&self) -> &dyn Any {
        self
    }
}

/// StaticDataNavigationBodyLoader.cc:34-85. Data is already resident; dropping
/// this original body loader cancels delivery without touching a URL loader.
pub(crate) struct StaticDataNavigationBodyLoader {
    bytes: Arc<[u8]>,
    offset: usize,
    finished: bool,
}
impl StaticDataNavigationBodyLoader {
    pub(crate) fn new(bytes: Arc<[u8]>) -> Self {
        Self {
            bytes,
            offset: 0,
            finished: false,
        }
    }
}
impl url_loader::URLStreamOperation for StaticDataNavigationBodyLoader {
    fn Poll(&mut self, max_bytes: usize) -> std::io::Result<Option<url_loader::URLLoadEvent>> {
        if max_bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "static body budget must be positive",
            ));
        }
        if self.offset < self.bytes.len() {
            let end = self.offset.saturating_add(max_bytes).min(self.bytes.len());
            let data = self.bytes[self.offset..end].to_vec();
            self.offset = end;
            return Ok(Some(url_loader::URLLoadEvent::Data(data)));
        }
        if !self.finished {
            self.finished = true;
            return Ok(Some(url_loader::URLLoadEvent::Finished));
        }
        Ok(None)
    }
}

/// InitializeWindow cc2851-2906. No existing realm is cleared for a loaded
/// document; initial-empty secure transition keeps the original realm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticDocumentWindowDisposition {
    NewWindow,
    ReuseInitialWindow,
}
/// Equality facts from the original Window/policy-container/agent-cluster
/// owners. Absence remains a required input, not an assumption of no COI key.
pub struct StaticDocumentWindowReuseFacts {
    pub window_anonymous_matching: bool,
    pub cross_origin_isolation_keys_match: bool,
}

/// Source-selected Window transition. The installation cannot fabricate a
/// choice for a different old document or silently reuse a loaded realm.
pub struct StaticDocumentWindowChoice {
    source_document: DOMOwnerHandle,
    disposition: StaticDocumentWindowDisposition,
}
impl StaticDocumentWindowChoice {
    /// DocumentLoader::ShouldReuseDOMWindow (cc2663-2707). Both ordinary
    /// response commits and static replacement commits use this same choice.
    /// Missing agent/anonymous facts forbid reuse rather than guessing them.
    pub fn ForNavigation(
        previous: &dom::Document,
        origin: &SecurityOrigin,
        facts: Option<StaticDocumentWindowReuseFacts>,
    ) -> Self {
        Self::Choose(
            previous.RootHandle(),
            previous.IsInitialEmptyDocument(),
            previous.SecurityContext().Origin(),
            origin,
            facts,
        )
    }
    fn Choose(
        source_document: DOMOwnerHandle,
        initial_empty: bool,
        previous_origin: &SecurityOrigin,
        origin: &SecurityOrigin,
        facts: Option<StaticDocumentWindowReuseFacts>,
    ) -> Self {
        let reuse = initial_empty
            && previous_origin.CanAccess(origin)
            && facts.is_some_and(|facts| {
                facts.window_anonymous_matching && facts.cross_origin_isolation_keys_match
            });
        Self {
            source_document,
            disposition: if reuse {
                StaticDocumentWindowDisposition::ReuseInitialWindow
            } else {
                StaticDocumentWindowDisposition::NewWindow
            },
        }
    }
    pub fn SourceDocument(&self) -> DOMOwnerHandle {
        self.source_document
    }
    pub fn Disposition(&self) -> StaticDocumentWindowDisposition {
        self.disposition
    }
    pub fn IsFor(&self, commit: &StaticDocumentCommit) -> bool {
        self.source_document == commit.source_document
    }
}
