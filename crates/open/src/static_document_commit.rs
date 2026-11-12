//! Chromium frame_loader.cc:1503-1558; document_loader.cc:3015-3044,
//! 3144-3192,3300-3362. The same OpenEngine installs a new DocumentLoader and
//! parsing pipeline. StartLoading/history precede the Window transition;
//! InstallNewDocument/parser creation follow it. No URL transport is involved.
use super::*;
use document_loader::{DocumentLoaderNavigationState, InitialScrollState, StaticDocumentCommit};

/// Publish these aliases as one owner snapshot. Retained aliases to the old
/// DocumentLoader continue to name the original navigation/lifetime.
#[derive(Clone)]
pub struct OpenDocumentOwners {
    pub navigation: Rc<DocumentLoaderNavigationState>,
    pub initial_scroll: Rc<InitialScrollState>,
    pub lifecycle: Rc<OpenDocumentLifecycleState>,
}
/// Receipt for the source-installed loader. The Window transition may not be
/// run before BeginStaticDocumentCommit, or be applied to a different loader.
pub struct PreparedStaticDocumentCommit {
    owners: OpenDocumentOwners,
    source_document: dom::persistent_document::DOMOwnerHandle,
    window_initialization_started: Cell<bool>,
}
impl PreparedStaticDocumentCommit {
    pub fn Owners(&self) -> OpenDocumentOwners {
        self.owners.clone()
    }
    /// DocumentLoader.cc3027-3044. Run after the client transition and before
    /// the VM creates/selects the actual Window. Missing initial-window facts
    /// remain explicit; they cannot turn into a guessed reuse decision.
    pub fn BeginWindowInitialization(
        &self,
        commit: &StaticDocumentCommit,
        facts: Option<document_loader::StaticDocumentWindowReuseFacts>,
    ) -> Option<document_loader::StaticDocumentWindowChoice> {
        assert!(Rc::ptr_eq(
            &self.owners.navigation,
            &commit.NavigationState()
        ));
        let choice = commit.WindowDisposition(facts)?;
        assert!(
            !self.window_initialization_started.replace(true),
            "one Window initialization per loader"
        );
        self.owners.navigation.SetCommitted();
        Some(choice)
    }
}
pub struct StaticDocumentInstallation {
    pub window_choice: document_loader::StaticDocumentWindowChoice,
    pub services: OpenDocumentServices,
    pub script_executor: Option<ScriptExecutorHandle>,
}
pub struct StaticDocumentCommitResult {
    pub owners: OpenDocumentOwners,
    pub document: dom::persistent_document::DOMOwnerHandle,
    pub window_disposition: document_loader::StaticDocumentWindowDisposition,
    pub replacement_notification: StaticDocumentReplacementNotification,
}
/// LocalFrameClient::DidCommitDocumentReplacementNavigation, cc3401-3417.
/// The client receives canonical history identity before StartLoadingResponse.
pub struct StaticDocumentReplacementNotification {
    pub document: dom::persistent_document::DOMOwnerHandle,
    pub document_token: [u64; 2],
    pub history_item: Rc<navigation_history::HistoryItemOwner>,
    pub load_type: navigation_history::FrameLoadType,
}
impl OpenEngine {
    pub fn DocumentOwners(&self) -> OpenDocumentOwners {
        OpenDocumentOwners {
            navigation: self.NavigationState(),
            initial_scroll: self.InitialScrollState(),
            lifecycle: self.LifecycleState(),
        }
    }
    /// Called at DetachDocumentLoader, before Document::Shutdown. The caller
    /// separately delivers XHR abort/microtask dispatch, which may remove the
    /// frame; BeginStaticDocumentCommit is never called after such removal.
    pub fn DetachDocumentLoader(&mut self) {
        if let Some(scheduler) = self
            .pipeline
            .as_ref()
            .and_then(OpenDocumentPipeline::Scheduler)
        {
            scheduler.StopModuleLoading();
        }
        self.loader.StopLoading();
        self.lifecycle.loading.set(false);
        self.pipeline.take();
    }
    /// FrameLoader.cc1508-1544: SetDocumentLoader, SetCommitReason(JSURL),
    /// StartLoading(static), history and unload timing. The caller next routes
    /// TransitionToCommittedForNewPage and the source-selected Window change.
    pub fn BeginStaticDocumentCommit(
        &mut self,
        commit: &StaticDocumentCommit,
        resources: Rc<ResourceEngine>,
        budget: DocumentLoadBudget,
        unload_timing: Option<navigation_history::UnloadEventTiming>,
    ) -> io::Result<PreparedStaticDocumentCommit> {
        if budget.body_bytes == 0 || budget.parser_tokens == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "static document budgets must be positive",
            ));
        }
        if self.pipeline.is_some() {
            return Err(io::Error::other(
                "old DocumentLoader must be detached before static commit",
            ));
        }
        self.loader = DocumentLoader::ForStaticDocumentCommit(resources.clone(), commit);
        commit.SetHistoryItemStateForCommit();
        self.loader.ApplyPreviousDocumentUnloadTiming(unload_timing);
        self.resources = resources;
        self.current_url = commit.NavigationState().URL();
        self.resources.SetDocumentURL(self.current_url.clone());
        self.lifecycle = Rc::new(OpenDocumentLifecycleState {
            loading: Cell::new(true),
        });
        self.document_started = false;
        self.document_initialized = false;
        self.stopped = false;
        self.failure = None;
        self.budget = budget;
        if let Some(callback) = self.wake_callback.clone() {
            self.resources.SetWakeCallback(callback.clone());
            self.loader.SetWakeCallback(callback);
        }
        Ok(PreparedStaticDocumentCommit {
            owners: self.DocumentOwners(),
            source_document: commit.SourceDocument(),
            window_initialization_started: Cell::new(false),
        })
    }
    /// DocumentLoader.cc3015-3044,3144-3192,3300-3362. The real Window/realm
    /// transition has completed; install the fresh Document and create only
    /// its parser. Pumping its static body occurs through ordinary Advance.
    pub fn CommitStaticDocument(
        &mut self,
        prepared: PreparedStaticDocumentCommit,
        adapter: &mut dyn OpenDocumentAdapter,
        commit: &StaticDocumentCommit,
        installation: StaticDocumentInstallation,
    ) -> io::Result<StaticDocumentCommitResult> {
        if !Rc::ptr_eq(&prepared.owners.navigation, &self.NavigationState())
            || !Rc::ptr_eq(&prepared.owners.navigation, &commit.NavigationState())
            || prepared.source_document != commit.SourceDocument()
            || !prepared.window_initialization_started.get()
            || self.document_started
            || self.pipeline.is_some()
        {
            return Err(io::Error::other(
                "static installation no longer owns the source loader transaction",
            ));
        }
        if !installation.window_choice.IsFor(commit) {
            return Err(io::Error::other(
                "Window choice belongs to a different static commit",
            ));
        }
        let StaticDocumentInstallation {
            window_choice,
            services,
            script_executor,
        } = installation;
        {
            let document = services.document.borrow();
            if document.GetDocument().RootHandle() == commit.SourceDocument()
                || document.GetDocument().GetDocumentParser().is_some()
            {
                return Err(io::Error::other(
                    "static commit requires the new Document arena",
                ));
            }
        }
        if script_executor.is_some() != services.scripts.is_some() {
            return Err(io::Error::other(
                "new Window script executor and bindings must belong to the same installation",
            ));
        }
        self.script_executor = script_executor;
        self.document_initialized = true;
        if let Some(callback) = self.wake_callback.clone() {
            self.SetWakeCallback(callback);
        }
        // Never pass through CommitURL: it generates a new initiator token.
        // This Document uses the cloned token and compiled inherited policies;
        // no old node, observer, parser, or Window global is rebound to it.
        commit.InstallDocumentSecurity(services.document.borrow_mut().GetDocumentMut());
        let document = services.document.borrow().GetDocument().RootHandle();
        adapter.InitializeWindowNavigation(self.NavigationState());
        self.InstallParserState(adapter, services, &self.current_url.clone())?;
        Ok(StaticDocumentCommitResult {
            owners: self.DocumentOwners(),
            document,
            window_disposition: window_choice.Disposition(),
            replacement_notification: StaticDocumentReplacementNotification {
                document,
                document_token: commit.DocumentToken(),
                history_item: commit
                    .NavigationState()
                    .CurrentHistoryItem()
                    .expect("source history before replacement notification"),
                load_type: commit.LoadType(),
            },
        })
    }
    /// DocumentLoader.cc2189-2280,3519-3523. Route the replacement notification
    /// first, then feed the resident static response in this same source task.
    /// Inline scripts and tokenization run synchronously; an actual pending
    /// external parser-blocking script is the only parser wait boundary.
    pub fn StartStaticDocumentResponse(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
    ) -> io::Result<Vec<OpenEffect>> {
        if self.document_started
            || !self.loader.ForcesSynchronousParsing()
            || self.loader.Status() != DocumentLoaderStatus::kResponseReady
        {
            return Err(io::Error::other(
                "static response does not own an unstarted committed parser",
            ));
        }
        self.pipeline
            .as_mut()
            .expect("new document pipeline")
            .BeginDocument(&mut self.loader)?;
        self.document_started = true;
        let mut effects = vec![OpenEffect::DocumentStarted];
        effects.extend(self.Advance(adapter)?);
        Ok(effects)
    }
}
