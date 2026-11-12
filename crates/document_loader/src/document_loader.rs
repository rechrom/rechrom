#![allow(non_snake_case)]
use dom::Document;
use html::html_parser::{HTMLParserResult, HTMLParserStatus};
use html::{
    document_parser::{DocumentParser, DocumentParserState},
    HTMLParserHost,
};
use resource::ResourceEngine;
use std::{cell::RefCell, io, rc::Rc, time::Duration};
// Bound unread decoded input while scripts block the tree builder. The stream
// can continue delivery and preload discovery without unbounded parser storage.
const MAX_BUFFERED_INPUT_CHARACTERS: usize = 1024 * 1024;
// Blink's early HTMLParserScheduler budget. A parser continuation is an
// ordinary loading task, leaving the next BeginFrame free to paint mutations
// produced by the completed task.
const PARSER_TIME_BUDGET: Duration = Duration::from_millis(10);

use url_loader::{
    RequestDestination, URLLoadEvent, URLLoader, URLRequest, URLResponseHead, URLStreamOperation,
    URLWakeCallback,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentLoaderStatus {
    kIdle,
    kLoading,
    kResponseReady,
    kParsing,
    kFinished,
    kFailed,
    kCancelled,
}
#[derive(Clone, Copy, Debug)]
pub struct DocumentLoadBudget {
    pub body_bytes: usize,
    pub parser_tokens: usize,
}
impl Default for DocumentLoadBudget {
    fn default() -> Self {
        Self {
            body_bytes: 16 * 1024,
            parser_tokens: 4096,
        }
    }
}
// The existing standalone parser adapters specify a token budget only.
impl From<usize> for DocumentLoadBudget {
    fn from(parser_tokens: usize) -> Self {
        Self {
            parser_tokens,
            ..Self::default()
        }
    }
}
pub struct DocumentLoadProgress {
    pub body_finished: bool,
    pub made_progress: bool,
    /// Appending speculative input while paused makes loading progress but
    /// does not change DOM or invalidate the rendering lifecycle.
    pub parser_made_progress: bool,
    /// The parser still has buffered tokens, or the byte budget was exhausted
    /// while the stream remained readable. The embedder can run another
    /// parser task before spending a rendering opportunity on the partial DOM.
    pub immediate_work_pending: bool,
    pub parser: HTMLParserResult,
}
/// One main-document byte stream and its resident parser. Page owns the DOM,
/// navigation commit, JavaScript environment, lifecycle events and rendering.
/// No method waits for IO or retains a borrow of the document between calls.
/// Original DocumentLoader::InitialScrollState (document_loader.h:337-345).
/// Page retains only an alias to this loader-owned state for synchronous edits.
#[derive(Default)]
pub struct InitialScrollState {
    pub was_scrolled_by_user: std::cell::Cell<bool>,
    pub did_restore_from_history: std::cell::Cell<bool>,
}
pub struct DocumentLoader {
    document_policy: crate::document_policy::DocumentPolicyLoader,
    is_javascript_url_commit: bool,
    is_initial_frame_document: bool,
    render_blocking_timing: Rc<dom::render_blocking_resource_manager::RenderBlockingLoadTiming>,
    initial_scroll_state: Rc<InitialScrollState>,
    navigation_state: Rc<crate::DocumentLoaderNavigationState>,
    resources: Rc<ResourceEngine>,
    stream: Option<Box<dyn URLStreamOperation>>,
    response: Option<URLResponseHead>,
    requested_url: String,
    parser: Option<DocumentParserState>,
    status: DocumentLoaderStatus,
    body_finished: bool,
    needs_input: bool,
    failure: Option<(io::ErrorKind, String)>,
    wake: Option<URLWakeCallback>,
    preload_scripting: Option<bool>,
    diagnostic_scripting: Option<bool>,
    diagnostic: Option<crate::diagnostic_read_ahead::DiagnosticReadAhead>,
}
impl DocumentLoader {
    /// FrameLoader::Init cc:251-306: the initial empty Document uses the same
    /// DocumentLoader/parser path with static empty bytes, never a transport
    /// request or a fabricated response from another Frame.
    pub fn ForInitialLocalFrame(resources: Rc<ResourceEngine>) -> Self {
        let mut loader = Self::WithResourceEngine(resources);
        loader.is_initial_frame_document = true;
        loader
            .navigation_state
            .SetLoadType(navigation_history::FrameLoadType::Standard);
        loader
            .navigation_state
            .SetNavigationType(navigation_history::WebNavigationType::Other);
        loader.response = Some(URLResponseHead {
            final_url: String::new(),
            mime_type: "text/html".into(),
            text_encoding: "UTF-8".into(),
            ..Default::default()
        });
        loader.stream = Some(Box::new(
            crate::navigation::StaticDataNavigationBodyLoader::new(std::sync::Arc::from([])),
        ));
        loader.status = DocumentLoaderStatus::kResponseReady;
        loader
    }
    /// Source initialization is ScriptForbidden and has no document observers
    /// yet. All parsing, tree construction and parser teardown still use their
    /// original owners; this method does not synthesize html/head/body nodes.
    pub fn InitializeLocalFrameDocument(&mut self, document: &mut Document) -> io::Result<()> {
        struct InitialEmptyDocumentHost;
        impl HTMLParserHost for InitialEmptyDocumentHost {
            fn HandleParserElement(&mut self, _: html::ParserElementEvent<'_>) {}
        }
        assert!(self.is_initial_frame_document);
        let mut host = InitialEmptyDocumentHost;
        self.BeginParsing(document, &mut host)?;
        while self.status != DocumentLoaderStatus::kFinished {
            let progress = self.Pump(
                document,
                &mut host,
                DocumentLoadBudget {
                    body_bytes: usize::MAX,
                    parser_tokens: usize::MAX,
                },
            )?;
            if !progress.made_progress && self.status != DocumentLoaderStatus::kFinished {
                return Err(io::Error::other(
                    "initial empty Frame document did not finish synchronously",
                ));
            }
        }
        let mut cancellation = document.BeginCancelParsing(true);
        while !matches!(
            cancellation.Next(),
            dom::document_parser_lifecycle::CancelParsingStage::Done
        ) {}
        self.navigation_state.SetSentDidFinishLoad();
        Ok(())
    }
    pub fn new(loader: Rc<RefCell<dyn URLLoader>>) -> Self {
        let resources = Rc::new(ResourceEngine::new(loader, String::new()));
        Self::WithResourceEngine(resources)
    }

    pub fn WithResourceEngine(resources: Rc<ResourceEngine>) -> Self {
        let initial_scroll_state = Rc::new(InitialScrollState::default());
        let navigation_state =
            crate::DocumentLoaderNavigationState::new(initial_scroll_state.clone());
        let render_blocking_timing = navigation_state.LoadTiming();
        Self {
            document_policy: Default::default(),
            is_javascript_url_commit: false,
            is_initial_frame_document: false,
            navigation_state,
            render_blocking_timing,
            initial_scroll_state,
            resources,
            stream: None,
            response: None,
            requested_url: String::new(),
            parser: None,
            status: DocumentLoaderStatus::kIdle,
            body_finished: false,
            needs_input: true,
            failure: None,
            wake: None,
            preload_scripting: None,
            diagnostic_scripting: None,
            diagnostic: None,
        }
    }
    /// The embedding supplies the same scripting flag as the document parser.
    /// HTML owns speculative tokenization; the parser host owns resource admission.
    pub fn EnablePreloadScanning(&mut self, scripting: bool) {
        assert!(self.parser.is_none(), "configure preloads before parsing");
        self.preload_scripting = Some(scripting);
    }
    /// Explicit embedding-only diagnostic. Disabled by default and never admits
    /// a speculative resource. Configure before BeginParsing.
    pub fn EnableDiagnosticReadAhead(&mut self, scripting: bool) {
        assert!(self.parser.is_none(), "configure diagnostic before parsing");
        self.diagnostic_scripting = Some(scripting);
    }
    /// FrameLoader.cc1508-1525; WebNavigationParams.cc98-107. Static data
    /// starts a fresh loader. No URLRequest, transport, or old parser is reused.
    pub fn ForStaticDocumentCommit(
        resources: Rc<ResourceEngine>,
        commit: &crate::StaticDocumentCommit,
    ) -> Self {
        let mut loader = Self::WithResourceEngine(resources);
        loader.navigation_state = commit.NavigationState();
        loader.render_blocking_timing = loader.navigation_state.LoadTiming();
        loader.initial_scroll_state = loader.navigation_state.InitialScrollState();
        loader.requested_url = loader.navigation_state.URL();
        loader.response = Some(URLResponseHead {
            final_url: loader.requested_url.clone(),
            status_code: commit.ClonedHttpStatus(),
            mime_type: "text/html".into(),
            text_encoding: "UTF-8".into(),
            request_referrer: Some(commit.Referrer()),
            ..Default::default()
        });
        loader.stream = Some(Box::new(
            crate::navigation::StaticDataNavigationBodyLoader::new(commit.Body()),
        ));
        loader.status = DocumentLoaderStatus::kResponseReady;
        loader.is_javascript_url_commit = true;
        loader.navigation_state.SetProvisional();
        loader.render_blocking_timing.MarkNavigationStart();
        loader
    }
    /// FrameLoader.cc1531-1544. Only a real captured unload interval transfers
    /// to the new loader; no synthetic zero timestamps are substituted.
    pub fn ApplyPreviousDocumentUnloadTiming(
        &self,
        timing: Option<navigation_history::UnloadEventTiming>,
    ) {
        if let Some(timing) = timing {
            self.render_blocking_timing.unload_event.set(Some(timing));
            self.render_blocking_timing
                .commit_navigation_end
                .set(Some(self.render_blocking_timing.Now()));
        }
    }
    pub fn NavigationState(&self) -> Rc<crate::DocumentLoaderNavigationState> {
        self.navigation_state.clone()
    }
    pub fn BindRequiredDocumentPolicy(
        &mut self,
        required: Rc<dom::document_policy::DocumentPolicyFeatureState>,
    ) {
        self.document_policy.BindRequiredPolicy(required);
    }
    pub fn InitialScrollState(&self) -> Rc<InitialScrollState> {
        self.initial_scroll_state.clone()
    }
    pub fn ForcesSynchronousParsing(&self) -> bool {
        self.is_javascript_url_commit
    }
    pub fn IsWaitingForParserBlockingScript(&self) -> bool {
        self.parser
            .as_ref()
            .is_some_and(DocumentParserState::IsPaused)
    }
    pub fn Status(&self) -> DocumentLoaderStatus {
        self.status
    }
    pub fn URL(&self) -> String {
        // Response headers remain immutable. Same-document commits mutate the
        // original DocumentLoader navigation owner, not that response record.
        if self.navigation_state.CurrentHistoryItem().is_some() {
            self.navigation_state.URL()
        } else {
            self.response
                .as_ref()
                .map_or_else(String::new, |head| head.final_url.clone())
        }
    }
    pub fn SetWakeCallback(&mut self, callback: URLWakeCallback) {
        self.wake = Some(callback.clone());
        if let Some(stream) = &mut self.stream {
            stream.SetWakeCallback(callback);
        }
    }
    /// Static/about/srcdoc responses still admit the original navigation
    /// request before Window initialization. They bypass transport, not the
    /// DocumentLoader's history and structured-state ownership contract.
    pub fn AdmitSuppliedDocumentRequest(&mut self, request: &URLRequest) -> io::Result<()> {
        if self.status != DocumentLoaderStatus::kIdle {
            return Err(io::Error::other("document loading has already started"));
        }
        if request.url.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Open URL is empty",
            ));
        }
        if request.navigation.as_ref().is_some_and(|navigation| {
            navigation.load_type.IsTraversal() && navigation.history_item.is_none()
        }) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "back/forward and restore require the destination history item",
            ));
        }
        self.render_blocking_timing.MarkNavigationStart();
        self.navigation_state.SetRequest(request);
        self.requested_url = request.url.clone();
        Ok(())
    }
    pub fn StartLoading(&mut self, request: &URLRequest) -> io::Result<()> {
        self.AdmitSuppliedDocumentRequest(request)?;
        let mut request = request.clone();
        request.destination = RequestDestination::kDocument;
        let result = self.resources.StartStream(request.clone());
        let mut stream = match result {
            Ok(result) => result,
            Err(error) => return self.Fail(error),
        };
        if let Some(wake) = &self.wake {
            stream.SetWakeCallback(wake.clone());
        }
        self.requested_url = request.url;
        self.stream = Some(stream);
        self.status = DocumentLoaderStatus::kLoading;
        Ok(())
    }
    /// Deliver headers exactly once. Body consumption starts only after Page
    /// has installed its document/environment and called BeginParsing.
    pub fn PollResponse(&mut self) -> io::Result<Option<URLResponseHead>> {
        self.CheckFailure()?;
        if self.status != DocumentLoaderStatus::kLoading {
            return Ok(None);
        }
        match self.stream.as_mut().expect("loading stream exists").Poll(1) {
            Ok(None) => Ok(None),
            Ok(Some(URLLoadEvent::Response(mut head))) => {
                if head.final_url.is_empty() {
                    head.final_url = self.requested_url.clone();
                }
                self.document_policy.Initialize(
                    &head.final_url,
                    &head.document_policy,
                    &head.document_policy_report_only,
                    &head.require_document_policy,
                );
                if self.document_policy.WasBlocked() {
                    // ReplaceWithEmptyDocument, cc1622-1629. The original
                    // response body never reaches parser/preload/script entry.
                    self.stream.take();
                    self.body_finished = true;
                    head = URLResponseHead {
                        final_url: "data:,".into(),
                        status_code: 200,
                        mime_type: "text/html".into(),
                        text_encoding: "UTF-8".into(),
                        ..Default::default()
                    };
                }
                if let Some(policy) = head.referrer_policy {
                    self.resources.SetReferrerPolicy(policy);
                }
                self.response = Some(head.clone());
                self.status = DocumentLoaderStatus::kResponseReady;
                Ok(Some(head))
            }
            Ok(Some(_)) => self.Fail(io::Error::new(
                io::ErrorKind::InvalidData,
                "document body precedes response headers",
            )),
            Err(error) => self.Fail(error),
        }
    }
    /// Bind one resident parser to the Page-owned document. Idle permits the
    /// existing caller-supplied HTML adapter; network loads require headers.
    pub fn BeginParsing(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
    ) -> io::Result<()> {
        self.CheckFailure()?;
        if !matches!(
            self.status,
            DocumentLoaderStatus::kIdle | DocumentLoaderStatus::kResponseReady
        ) {
            return Err(io::Error::other("document is not ready to begin parsing"));
        }
        if let Some(scripting) = self.preload_scripting {
            document.SetScriptingEnabled(scripting);
        }
        // ExecutionContextCSPDelegate cc:147-167 reads the original Loader
        // response and Document.referrer. An idle supplied-document loader has
        // ResourceResponse's source status-code constructor value, zero.
        document.ContentSecurityPolicy().BindReportContext(
            Some(
                self.response
                    .as_ref()
                    .map_or(0, |head| head.status_code as u16),
            ),
            self.response
                .as_ref()
                .map_or_else(|| Some(String::new()), |head| head.request_referrer.clone()),
        );
        // DocumentLoader.cc3144-3172: JavaScript documents inherit the
        // previous Window's policies during installation. Parsing must not
        // replace them with the empty static response's header policies.
        if !self.is_javascript_url_commit {
            if let Some(head) = &self.response {
                // DocumentLoader.cc2312-2322. Only the first language is used,
                // stripped with HTML-space semantics; an empty value does not set
                // the Document's retained ContentLanguage.
                let language = head
                    .content_language
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '));
                if !language.is_empty() {
                    document.SetContentLanguage(&language.encode_utf16().collect::<Vec<_>>());
                }
                if !self.is_initial_frame_document {
                    document.SetResponseAddressSpace(head.response_address_space);
                }
                document.AddResponseContentSecurityPolicies(
                    &head.final_url,
                    &head.content_security_policy,
                    &head.content_security_policy_report_only,
                );
            }
            document.ApplyResponsePolicySandbox();
            if !self.document_policy.Initialized() {
                self.document_policy
                    .Initialize(document.DocumentURL(), "", "", "");
            }
            self.document_policy.Commit(document);
            self.navigation_state.CommitDocumentPolicy(
                document
                    .SecurityContext()
                    .DocumentPolicy()
                    .expect("source committed policy"),
            );
            if let Some(head) = &self.response {
                let context = document.SecurityContext();
                context
                    .NetworkPermissionsPolicy()
                    .SetHeaderPoliciesForOutermostDocument(
                        &head.feature_policy,
                        &head.permissions_policy,
                        context.Origin(),
                        &head.final_url,
                    );
            }
        } else if let Some(policy) = document.SecurityContext().DocumentPolicy() {
            self.navigation_state.CommitDocumentPolicy(policy);
        }
        self.navigation_state
            .InitializeForDocument(document, self.response.as_ref())?;
        document.AttachDocumentLoader();
        // DocumentInit::ComputeDocumentType cc:169-173 and Document::CreateParser
        // cc:3653-3685: the response owner selects SVG/XML before parsing bytes.
        let svg = self.response.as_ref().is_some_and(|head| {
            head.mime_type
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("image/svg+xml")
        });
        if svg {
            document.SetDocumentType(
                document.Root(),
                dom::persistent_document::DOMDocumentType::kSVG,
            );
        }
        // DocumentLoader.cc:3335-3349, kPaintHolding is enabled by default
        // (blink/common/features.cc:1706). kInitialWebUISurfaceSync is disabled
        // and this ordinary navigation has no WebUI FrameClient admission.
        document.SetDeferredCompositorCommitIsAllowed(
            document
                .DocumentClasses(document.Root())
                .Has(dom::document_metadata::DocumentClass::HTML)
                && self.response.as_ref().is_some_and(|head| {
                    url::Url::parse(&head.final_url)
                        .is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
                }),
        );
        if self.render_blocking_timing.navigation_start.get().is_none() {
            self.render_blocking_timing.MarkNavigationStart();
        }
        document.BindRenderBlockingLoadTiming(self.render_blocking_timing.clone());
        let mut parser = if svg {
            DocumentParserState::NewXML(document)
        } else {
            DocumentParserState::NewHTML(document, host)
        };
        if self.document_policy.WasBlocked() {
            parser.FinishInput();
        }
        if let Some(head) = &self.response {
            if let Err(error) = parser.SetTextEncoding(&head.text_encoding) {
                return self.Fail(error);
            }
        }
        // Until a CSP owner can admit speculative requests, do not speculate
        // past HTTP policy. The scanner similarly stops at a meta CSP boundary.
        if let Some(scripting) = self.preload_scripting {
            parser.EnablePreloadScanning(scripting);
            if self
                .response
                .as_ref()
                .is_some_and(|head| !head.content_security_policy.is_empty())
            {
                parser.DisablePreloadScanning();
            }
        }
        if let Some(scripting) = self.diagnostic_scripting.filter(|_| !svg) {
            let encoding = self
                .response
                .as_ref()
                .map_or("utf-8", |head| head.text_encoding.as_str());
            self.diagnostic = Some(crate::diagnostic_read_ahead::DiagnosticReadAhead::new(
                encoding, scripting,
            )?);
        }
        if self.is_javascript_url_commit {
            let DocumentParserState::HTML(html) = &mut parser else {
                return Err(io::Error::other(
                    "JavaScript URL replacement requires a new HTML document",
                ));
            };
            html.SetSynchronizationPolicy(
                html::html_parser::ParserSynchronizationPolicy::ForceSynchronousParsing,
            );
        }
        self.parser = Some(parser);
        self.status = DocumentLoaderStatus::kParsing;
        Ok(())
    }
    pub fn Pump(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        budget: impl Into<DocumentLoadBudget>,
    ) -> io::Result<DocumentLoadProgress> {
        let mut trace = browser_tracing::span("loading", "DocumentLoader.Pump");
        let mut body_bytes = 0usize;
        let budget = budget.into();
        let budget = if self.is_javascript_url_commit {
            DocumentLoadBudget {
                body_bytes: usize::MAX,
                parser_tokens: usize::MAX,
            }
        } else {
            budget
        };
        if budget.body_bytes == 0 || budget.parser_tokens == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "document loading budgets must be positive",
            ));
        }
        self.CheckFailure()?;
        if self.status == DocumentLoaderStatus::kFinished {
            return Ok(DocumentLoadProgress {
                body_finished: self.body_finished,
                made_progress: false,
                parser_made_progress: false,
                immediate_work_pending: false,
                parser: HTMLParserResult {
                    status: HTMLParserStatus::kFinished,
                    script: None,
                },
            });
        }
        self.Parser()?;
        let needed_input_at_start = self.needs_input;
        let mut made_progress = !self.needs_input && !self.parser.as_ref().unwrap().IsPaused();
        let mut body_budget_exhausted = false;
        let parser_paused = self.parser.as_ref().unwrap().IsPaused();
        let allow_paused_read = parser_paused && self.preload_scripting.is_some();
        if parser_paused && !allow_paused_read {
            if let (Some(diagnostic), Some(stream)) = (&mut self.diagnostic, &mut self.stream) {
                if diagnostic.read_ahead(stream.as_mut()) {
                    if let Some(wake) = &self.wake {
                        wake();
                    }
                }
            }
        }
        let room = if self.is_javascript_url_commit {
            usize::MAX
        } else {
            MAX_BUFFERED_INPUT_CHARACTERS
                .saturating_sub(self.parser.as_ref().unwrap().BufferedInputCharacters())
        };
        // HTMLDocumentParser::Append continues network input while paused.
        // Only AppendBytes and speculative tokenization run here; the tree
        // builder still waits for its blocking script, including at EOF.
        if (self.needs_input || allow_paused_read)
            && room != 0
            && !self.body_finished
            && self.stream.is_some()
        {
            let mut remaining = budget.body_bytes.min(room);
            while remaining > 0 {
                let queued = self
                    .diagnostic
                    .as_mut()
                    .and_then(|d| d.poll_queued(remaining));
                let from_stream = queued.is_none();
                let event = match queued
                    .map(|event| event.map(Some))
                    .unwrap_or_else(|| self.stream.as_mut().unwrap().Poll(remaining))
                {
                    Ok(event) => event,
                    Err(error) => return self.Fail(error),
                };
                match event {
                    None => break,
                    Some(URLLoadEvent::Data(bytes)) => {
                        if bytes.is_empty() || bytes.len() > remaining {
                            return self.Fail(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "invalid document body chunk",
                            ));
                        }
                        remaining -= bytes.len();
                        body_bytes += bytes.len();
                        made_progress = true;
                        if let Some(diagnostic) = &mut self.diagnostic {
                            if from_stream {
                                diagnostic.observe(&bytes, parser_paused);
                            }
                            diagnostic.appended += bytes.len();
                        }
                        if let Err(error) = self.parser.as_mut().unwrap().AppendBytes(&bytes) {
                            return self.Fail(error);
                        }
                        self.parser
                            .as_ref()
                            .unwrap()
                            .CommitDecodedEncodingData(document);
                    }
                    Some(URLLoadEvent::Finished) => {
                        if let Err(error) = self.parser.as_mut().unwrap().FinishBytes() {
                            return self.Fail(error);
                        }
                        self.parser
                            .as_ref()
                            .unwrap()
                            .CommitDecodedEncodingData(document);
                        self.body_finished = true;
                        made_progress = true;
                        self.stream.take();
                        break;
                    }
                    Some(URLLoadEvent::Response(_)) => {
                        return self.Fail(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "duplicate document response headers",
                        ));
                    }
                }
            }
            body_budget_exhausted = remaining == 0 && self.stream.is_some();
        }
        let scan = self
            .parser
            .as_mut()
            .unwrap()
            .ScanPreloads(budget.parser_tokens);
        for candidate in &scan.candidates {
            host.HandlePreloadCandidate(candidate);
        }
        let preload_work_pending = scan.work_pending;
        if scan.stopped_for_csp {
            self.parser.as_mut().unwrap().DisablePreloadScanning();
        }
        let result = match self.parser.as_mut().unwrap().PumpWithTimeBudget(
            document,
            host,
            budget.parser_tokens,
            PARSER_TIME_BUDGET,
        ) {
            Ok(result) => result,
            Err(error) => return self.Fail(error),
        };
        self.needs_input = result.status == HTMLParserStatus::kNeedMoreInput;
        // ResumeParsingAfterPause schedules another tokenizer pump in Blink.
        // Our parser can consume its resident input after a blocking script and
        // only then discover that it needs the next stream chunk.  That chunk
        // may already be queued, so its edge-triggered wake has already fired.
        // Post one recheck turn for this false -> true transition.  If the
        // stream is still empty that turn stops normally and the producer's
        // next wake owns further progress.
        let parser_input_recheck = !needed_input_at_start
            && self.needs_input
            && !self.body_finished
            && self.stream.is_some();
        if result.status == HTMLParserStatus::kFinished {
            self.status = DocumentLoaderStatus::kFinished;
        }
        if result.status == HTMLParserStatus::kYielded
            || preload_work_pending
            || (body_budget_exhausted && (self.needs_input || allow_paused_read))
            || parser_input_recheck
        {
            if let Some(wake) = &self.wake {
                wake();
            }
        }
        if let Some(diagnostic) = &self.diagnostic {
            diagnostic.report();
        }
        trace.set("body_bytes", body_bytes as f64);
        trace.set("parser_status", result.status as u8 as f64);
        trace.set("script_node", result.script.map_or(0, |s| s.node_id) as f64);
        trace.set("body_finished", self.body_finished as u8 as f64);
        trace.set("made_progress", made_progress as u8 as f64);
        trace.set("needs_input", self.needs_input as u8 as f64);
        trace.set("parser_input_recheck", parser_input_recheck as u8 as f64);
        Ok(DocumentLoadProgress {
            body_finished: self.body_finished,
            made_progress,
            parser_made_progress: made_progress && !parser_paused,
            immediate_work_pending: result.status == HTMLParserStatus::kYielded
                || preload_work_pending
                || parser_input_recheck
                || ((self.needs_input || allow_paused_read)
                    && body_budget_exhausted
                    && self.parser.as_ref().unwrap().BufferedInputCharacters()
                        < MAX_BUFFERED_INPUT_CHARACTERS),
            parser: result,
        })
    }
    pub fn StopLoading(&mut self) {
        self.diagnostic.take();
        self.stream.take();
        self.parser.take();
        if !matches!(
            self.status,
            DocumentLoaderStatus::kFinished | DocumentLoaderStatus::kFailed
        ) {
            self.status = DocumentLoaderStatus::kCancelled;
        }
    }
    fn CheckFailure(&self) -> io::Result<()> {
        match &self.failure {
            Some((kind, message)) => Err(io::Error::new(*kind, message.clone())),
            None => Ok(()),
        }
    }
    fn Fail<T>(&mut self, error: io::Error) -> io::Result<T> {
        self.failure = Some((error.kind(), error.to_string()));
        self.diagnostic.take();
        self.stream.take();
        self.parser.take();
        self.status = DocumentLoaderStatus::kFailed;
        Err(error)
    }
    fn Parser(&mut self) -> io::Result<&mut DocumentParserState> {
        self.CheckFailure()?;
        if self.status != DocumentLoaderStatus::kParsing {
            return Err(io::Error::other("document parser is not active"));
        }
        Ok(self.parser.as_mut().expect("active parser exists"))
    }
    // Caller-supplied HTML and parser script adapters. Ordinary navigation uses
    // only StartLoading/PollResponse/BeginParsing/Pump/StopLoading above.
    pub fn Append(&mut self, text: &str) -> io::Result<()> {
        if self.stream.is_some() {
            return Err(io::Error::other("network document input is loader-owned"));
        }
        self.Parser()?.Append(text);
        if !text.is_empty() {
            self.needs_input = false;
        }
        Ok(())
    }
    pub fn FinishInput(&mut self) -> io::Result<()> {
        if self.stream.is_some() {
            return Err(io::Error::other("network document EOF is loader-owned"));
        }
        self.Parser()?.FinishInput();
        self.body_finished = true;
        self.needs_input = false;
        Ok(())
    }
    pub fn WithParser<R>(
        &mut self,
        document: &mut Document,
        host: &mut dyn HTMLParserHost,
        task: impl FnOnce(&mut DocumentParser<'_, '_>) -> R,
    ) -> io::Result<R> {
        // EOF has been parsed, but deferred scripts may still need scoped access.
        self.CheckFailure()?;
        let parser = self
            .parser
            .as_mut()
            .ok_or_else(|| io::Error::other("document parser is not active"))?;
        let was_paused = parser.IsPaused();
        let input_before = self
            .diagnostic
            .as_ref()
            .map(|_| parser.BufferedInputCharacters());
        let result = parser.WithParser(document, host, task);
        if was_paused
            && input_before.is_some_and(|before| {
                !parser.IsPaused() || parser.BufferedInputCharacters() != before
            })
        {
            if let Some(diagnostic) = &mut self.diagnostic {
                diagnostic.cancel_scan(1);
            }
        }
        if parser.IsFinished() {
            self.status = DocumentLoaderStatus::kFinished;
        }
        if was_paused && !parser.IsPaused() {
            self.needs_input = false;
            if let Some(wake) = &self.wake {
                wake();
            }
        }
        Ok(result)
    }
    pub fn InsertFromScript(&mut self, text: &str) -> io::Result<()> {
        if let Some(diagnostic) = &mut self.diagnostic {
            diagnostic.cancel_scan(2);
        }
        self.Parser()?.InsertFromScript(text)?;
        Ok(())
    }
    pub fn ResumeAfterScript(&mut self) -> io::Result<()> {
        if let Some(diagnostic) = &mut self.diagnostic {
            diagnostic.cancel_scan(1);
        }
        self.Parser()?.ResumeAfterScript();
        self.needs_input = false;
        if let Some(wake) = &self.wake {
            wake();
        }
        Ok(())
    }
}
