#![allow(non_snake_case)]

//! Main-document navigation and loading orchestration.
//!
//! `OpenEngine` owns one navigation's loader, URL, script scheduling state and
//! completion policy. The application supplies narrow document and script
//! adapters; rendering remains downstream and independent.

mod connected_resources;
#[cfg(test)]
mod dom_mutation;
pub mod dynamic_scripts;
pub mod script_execution;
pub mod script_scheduler;

pub use connected_resources::{
    ConnectedResourceDiscovery, ConnectedResourceEffects, ConnectedScriptPreparer,
};

use document_loader::{DocumentLoadBudget, DocumentLoader, DocumentLoaderStatus, ResourceFetcher};
use dom::DOM;
use javascript::javascript_runtime::JavaScriptException;
use javascript::script_engine::{ScriptExecutor, ScriptExecutorHandle};
use resource::ResourceEngine;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLRequest, URLResponseHead, URLWakeCallback};
use webapi::dom_bindings::DOMJavaScriptBindings;

/// Stable services supplied by the embedding document. OpenEngine consumes
/// these capabilities to construct its own parser and scheduling objects.
pub struct OpenDocumentServices {
    pub document: Rc<RefCell<DOM>>,
    pub resources: Rc<ResourceFetcher>,
    pub scripts: Option<OpenScriptServices>,
}

pub struct OpenScriptServices {
    pub bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    pub pending_host_errors: Option<Rc<dyn Fn() -> Vec<JavaScriptException>>>,
}

/// The mutually-exclusive parsing pipeline for one opened document. Its
/// concrete scheduler/parser is created and retained by OpenEngine.
pub enum OpenDocumentPipeline {
    Script {
        scheduler: script_scheduler::ScriptScheduler,
        executor: ScriptExecutorHandle,
    },
    Resource(script_scheduler::ParserResourceDiscovery),
}

impl OpenDocumentPipeline {
    fn Scheduler(&self) -> Option<&script_scheduler::ScriptScheduler> {
        match self {
            Self::Script { scheduler, .. } => Some(scheduler),
            Self::Resource(_) => None,
        }
    }

    fn SchedulerMut(
        &mut self,
    ) -> Option<(&mut script_scheduler::ScriptScheduler, &dyn ScriptExecutor)> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => Some((scheduler, &**executor)),
            Self::Resource(_) => None,
        }
    }

    fn HasPendingRenderBlockingStyleSheets(&self) -> bool {
        match self {
            Self::Script { scheduler, .. } => scheduler.HasPendingRenderBlockingStyleSheets(),
            Self::Resource(parser) => parser.HasPendingRenderBlockingStyleSheets(),
        }
    }

    fn BeginDocument(&mut self, loader: &mut DocumentLoader) -> io::Result<()> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => {
                let mut result = None;
                executor.WithRuntimeAndRealm(&mut |runtime, _| {
                    result = Some(scheduler.BeginDocumentLoader(loader, runtime));
                });
                result.expect("ScriptExecutor must invoke the task")
            }
            Self::Resource(parser) => parser.BeginDocumentLoader(loader),
        }
    }

    fn PumpDocument(
        &mut self,
        loader: &mut DocumentLoader,
        budget: DocumentLoadBudget,
        client: &mut dyn script_scheduler::ScriptLoadClient,
    ) -> io::Result<()> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => {
                let mut result = None;
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    result =
                        Some(scheduler.PumpDocumentLoader(loader, budget, runtime, realm, client));
                });
                result.expect("ScriptExecutor must invoke the task")?;
            }
            Self::Resource(parser) => {
                parser.PumpDocumentLoader(loader, budget, client)?;
            }
        }
        Ok(())
    }

    fn PollDocumentCompletion(
        &mut self,
        client: &mut dyn script_scheduler::ScriptLoadClient,
    ) -> io::Result<bool> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => {
                let mut result = None;
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    result = Some(scheduler.PollParsingCompletion(runtime, realm, client));
                });
                result.expect("ScriptExecutor must invoke the task")
            }
            Self::Resource(parser) => parser.PollStyleSheets(client),
        }
    }

    fn ParseSuppliedDocument(
        &mut self,
        loader: &mut DocumentLoader,
        source: &str,
        chunk_size: usize,
        token_budget: usize,
        client: &mut dyn script_scheduler::ScriptLoadClient,
    ) -> io::Result<()> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => {
                let mut result = None;
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    result = Some(scheduler.ParseDocumentWithLoader(
                        loader,
                        source,
                        chunk_size,
                        token_budget,
                        runtime,
                        realm,
                        client,
                    ));
                });
                result.expect("ScriptExecutor must invoke the task")
            }
            Self::Resource(parser) => {
                parser.ParseDocument(loader, source, chunk_size, token_budget, client)
            }
        }
    }

    fn FinishSuppliedStyleSheets(
        &mut self,
        client: &mut dyn script_scheduler::ScriptLoadClient,
    ) -> io::Result<()> {
        match self {
            Self::Script {
                scheduler,
                executor,
            } => {
                let mut result = None;
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    result = Some(scheduler.FinishStyleSheets(None, runtime, realm, client));
                });
                result.expect("ScriptExecutor must invoke the task")
            }
            Self::Resource(parser) => parser.FinishStyleSheets(client),
        }
    }
}

/// The document-specific side of opening a page. `OpenEngine` owns ordering and
/// completion policy; the application composition root supplies concrete
/// document, resource and Web API adapters.
pub trait OpenDocumentAdapter {
    fn CommitURL(&mut self, url: &str);
    fn PrepareDocumentServices(&mut self) -> OpenDocumentServices;
    fn InstallDynamicScriptHooks(&mut self, dynamic: &Rc<dynamic_scripts::DynamicScriptTasks>);
    fn WithScriptLoadClient(
        &mut self,
        callback: &mut dyn FnMut(&mut dyn script_scheduler::ScriptLoadClient),
    );
    fn UpdateDocumentResources(&mut self, allowance: usize) -> io::Result<usize>;
    fn HasPendingImageEvents(&self) -> bool;
    fn HasPendingLoadBlockingImages(&self) -> bool;
    fn FinishLoad(&mut self, pipeline: &mut OpenDocumentPipeline);
    fn FailLoad(&mut self, message: &str);
}

pub enum OpenMutation {
    Open {
        request: URLRequest,
        budget: DocumentLoadBudget,
    },
    CommitURL(String),
    UpdateReadiness(OpenReadiness),
    Stop,
    Fail {
        kind: io::ErrorKind,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OpenReadiness {
    pub parser_finished: bool,
    pub resource_turn_available: bool,
    pub script_task_available: bool,
    pub dynamic_scripts_finished: bool,
    pub image_events_finished: bool,
    pub load_blocking_images_finished: bool,
}

impl OpenReadiness {
    fn IsComplete(self) -> bool {
        self.parser_finished
            && self.resource_turn_available
            && self.script_task_available
            && self.dynamic_scripts_finished
            && self.image_events_finished
            && self.load_blocking_images_finished
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenEffect {
    LoadingStarted,
    URLCommitted(String),
    DocumentStarted,
    AwaitingCompletion,
    LoadFinished,
    Stopped,
    LoadFailed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenStatus {
    Idle,
    Loading,
    Parsing,
    WaitingForCompletion,
    Finished,
    Stopped,
    Failed(String),
}

/// One main-document navigation. Subsystems may share the contained resource
/// engine within this document, but a different document gets a new OpenEngine
/// and ResourceEngine.
pub struct OpenEngine {
    resources: Rc<ResourceEngine>,
    loader: DocumentLoader,
    current_url: String,
    document_started: bool,
    loading: bool,
    stopped: bool,
    failure: Option<String>,
    budget: DocumentLoadBudget,
    pipeline: Option<OpenDocumentPipeline>,
    script_executor: Option<ScriptExecutorHandle>,
    resource_completion_budget: Option<usize>,
}

/// Opaque saved admission state while a rendering opportunity runs. Rendering
/// may resolve styles, but it must not admit unrelated network completions.
pub struct OpenResourceCompletionScope(Option<usize>);

impl OpenEngine {
    pub fn new(
        resources: Rc<ResourceEngine>,
        script_executor: Option<ScriptExecutorHandle>,
    ) -> Self {
        Self {
            loader: DocumentLoader::WithResourceEngine(resources.clone()),
            resources,
            current_url: String::new(),
            document_started: false,
            loading: false,
            stopped: false,
            failure: None,
            budget: DocumentLoadBudget::default(),
            pipeline: None,
            script_executor,
            resource_completion_budget: None,
        }
    }

    pub fn ApplyMutation(&mut self, mutation: OpenMutation) -> io::Result<OpenEffect> {
        match mutation {
            OpenMutation::Open { request, budget } => {
                if request.url.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Open URL is empty",
                    ));
                }
                if budget.body_bytes == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "input_chunk_size must be positive",
                    ));
                }
                if budget.parser_tokens == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "parser_token_budget must be positive",
                    ));
                }
                if self.document_started || self.loader.Status() != DocumentLoaderStatus::kIdle {
                    return Err(io::Error::other("Page already has a document"));
                }
                self.budget = budget;
                self.loader.StartLoading(&request)?;
                self.loading = true;
                self.stopped = false;
                self.failure = None;
                Ok(OpenEffect::LoadingStarted)
            }
            OpenMutation::CommitURL(url) => {
                self.current_url = url;
                self.resources.SetDocumentURL(self.current_url.clone());
                Ok(OpenEffect::URLCommitted(self.current_url.clone()))
            }
            OpenMutation::UpdateReadiness(readiness) => {
                if self.loading
                    && self.document_started
                    && self.loader.Status() == DocumentLoaderStatus::kFinished
                    && readiness.IsComplete()
                {
                    self.loading = false;
                    self.stopped = false;
                    Ok(OpenEffect::LoadFinished)
                } else {
                    Ok(OpenEffect::AwaitingCompletion)
                }
            }
            OpenMutation::Stop => {
                self.loader.StopLoading();
                self.loading = false;
                self.stopped = true;
                Ok(OpenEffect::Stopped)
            }
            OpenMutation::Fail { kind, message } => {
                if self.loader.Status() != DocumentLoaderStatus::kIdle {
                    self.loader.StopLoading();
                }
                self.loading = false;
                self.stopped = false;
                self.failure = Some(message.clone());
                Ok(OpenEffect::LoadFailed(
                    io::Error::new(kind, message).to_string(),
                ))
            }
        }
    }

    pub fn Open(
        &mut self,
        request: URLRequest,
        budget: DocumentLoadBudget,
    ) -> io::Result<OpenEffect> {
        self.ApplyMutation(OpenMutation::Open { request, budget })
    }

    pub fn Stop(&mut self) -> OpenEffect {
        if let Some(scheduler) = self
            .pipeline
            .as_ref()
            .and_then(OpenDocumentPipeline::Scheduler)
        {
            scheduler.StopModuleLoading();
        }
        self.ApplyMutation(OpenMutation::Stop)
            .expect("stopping a navigation cannot fail")
    }

    /// Advance one bounded navigation turn. This is the only live-loading
    /// entry point needed by the host event loop after Open().
    pub fn Advance(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
    ) -> io::Result<Vec<OpenEffect>> {
        if !self.loading {
            return Ok(Vec::new());
        }
        let result = self.AdvanceInternal(adapter);
        if let Err(error) = &result {
            self.Fail(adapter, error);
        }
        result
    }

    pub fn Fail(&mut self, adapter: &mut dyn OpenDocumentAdapter, error: &io::Error) -> OpenEffect {
        let message = error.to_string();
        let effect = self
            .ApplyMutation(OpenMutation::Fail {
                kind: error.kind(),
                message: message.clone(),
            })
            .expect("recording a navigation failure cannot fail");
        if let Some(scheduler) = self
            .pipeline
            .as_ref()
            .and_then(OpenDocumentPipeline::Scheduler)
        {
            scheduler.StopModuleLoading();
        }
        adapter.FailLoad(&message);
        effect
    }

    fn AdvanceInternal(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
    ) -> io::Result<Vec<OpenEffect>> {
        let mut effects = Vec::new();
        if self.loader.Status() == DocumentLoaderStatus::kLoading {
            let Some(response) = self.PollResponse()? else {
                return Ok(effects);
            };
            effects.push(self.CommitURL(adapter, response.final_url.clone())?);
            self.EnsureParserState(adapter, &response.final_url)?;
            if let Some((scheduler, _)) = self
                .pipeline
                .as_mut()
                .and_then(OpenDocumentPipeline::SchedulerMut)
            {
                scheduler.BeginTaskTurn(true);
            }
            self.pipeline
                .as_mut()
                .expect("document pipeline installed")
                .BeginDocument(&mut self.loader)?;
            if !matches!(
                self.loader.Status(),
                DocumentLoaderStatus::kParsing | DocumentLoaderStatus::kFinished
            ) {
                return Err(io::Error::other(
                    "document adapter did not start the parser",
                ));
            }
            self.document_started = true;
            effects.push(OpenEffect::DocumentStarted);
        }
        if self.loader.Status() == DocumentLoaderStatus::kParsing {
            let pipeline = self.pipeline.as_mut().expect("document pipeline installed");
            let loader = &mut self.loader;
            let budget = self.budget;
            let mut result = None;
            adapter.WithScriptLoadClient(&mut |client| {
                result = Some(pipeline.PumpDocument(loader, budget, client));
            });
            result.expect("adapter must provide the script load client")?;
        }
        let parser_finished = if self.loader.Status() == DocumentLoaderStatus::kFinished {
            let pipeline = self.pipeline.as_mut().expect("document pipeline installed");
            let mut result = None;
            adapter.WithScriptLoadClient(&mut |client| {
                result = Some(pipeline.PollDocumentCompletion(client));
            });
            result.expect("adapter must provide the script load client")?
        } else {
            false
        };
        let task_available = self
            .pipeline
            .as_ref()
            .and_then(OpenDocumentPipeline::Scheduler)
            .is_none_or(script_scheduler::ScriptScheduler::HasTaskBudget);
        let allowance = match self.resource_completion_budget {
            Some(remaining) if task_available => remaining,
            Some(_) => 0,
            None => usize::MAX,
        };
        let completed = adapter.UpdateDocumentResources(allowance)?;
        if let Some(remaining) = self.resource_completion_budget.as_mut() {
            *remaining = remaining.saturating_sub(completed);
            if completed != 0 {
                if let Some((scheduler, _)) = self
                    .pipeline
                    .as_mut()
                    .and_then(OpenDocumentPipeline::SchedulerMut)
                {
                    scheduler.ConsumeTaskBudget();
                }
            }
        }
        let readiness = OpenReadiness {
            parser_finished,
            resource_turn_available: self
                .resource_completion_budget
                .is_none_or(|remaining| remaining != 0),
            script_task_available: self
                .pipeline
                .as_ref()
                .and_then(OpenDocumentPipeline::Scheduler)
                .is_none_or(script_scheduler::ScriptScheduler::HasTaskBudget),
            dynamic_scripts_finished: !self
                .pipeline
                .as_ref()
                .and_then(OpenDocumentPipeline::Scheduler)
                .is_some_and(script_scheduler::ScriptScheduler::HasPendingDynamicScripts),
            image_events_finished: !adapter.HasPendingImageEvents(),
            load_blocking_images_finished: !adapter.HasPendingLoadBlockingImages(),
        };
        let completion = self.ApplyMutation(OpenMutation::UpdateReadiness(readiness))?;
        if completion == OpenEffect::LoadFinished {
            if let Some((scheduler, _)) = self
                .pipeline
                .as_mut()
                .and_then(OpenDocumentPipeline::SchedulerMut)
            {
                scheduler.ConsumeTaskBudget();
            }
            adapter.FinishLoad(self.pipeline.as_mut().expect("document pipeline installed"));
        }
        effects.push(completion);
        Ok(effects)
    }

    fn EnsureParserState(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
        url: &str,
    ) -> io::Result<()> {
        if self.pipeline.is_some() {
            return Ok(());
        }
        let services = adapter.PrepareDocumentServices();
        if self.script_executor.is_some() {
            let script_services = services.scripts.ok_or_else(|| {
                io::Error::other("script-enabled OpenEngine requires script services")
            })?;
            self.resources.SetDocumentURL(url.to_owned());
            let mut scheduler = script_scheduler::ScriptScheduler::WithResourceEngine(
                services.document.clone(),
                script_services.bindings.clone(),
                self.resources.clone(),
                script_services.pending_host_errors,
            );
            scheduler.InstallResourceFetcher(services.resources)?;
            let executor = self
                .script_executor
                .as_ref()
                .expect("script-enabled OpenEngine has an executor");
            let dynamic = Rc::new(dynamic_scripts::DynamicScriptTasks::WithResourceEngine(
                services.document,
                script_services.bindings,
                self.resources.clone(),
                scheduler.StartedScripts(),
                executor.SupportsModules(),
            ));
            scheduler.InstallDynamicScripts(&dynamic)?;
            adapter.InstallDynamicScriptHooks(&dynamic);
            self.pipeline = Some(OpenDocumentPipeline::Script {
                scheduler,
                executor: executor.clone(),
            });
        } else {
            let parser = script_scheduler::ParserResourceDiscovery::WithResourceEngine(
                services.document,
                self.resources.clone(),
                services.resources,
            );
            parser.SetDocumentURL(url.to_owned());
            self.pipeline = Some(OpenDocumentPipeline::Resource(parser));
        }
        Ok(())
    }

    /// Parse caller-supplied complete source through the same resident parser
    /// and script scheduler used by network navigation.
    pub fn ParseSuppliedDocument(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
        source: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<OpenEffect> {
        if self.document_started {
            return Err(io::Error::other("document has already been built"));
        }
        if !matches!(
            self.loader.Status(),
            DocumentLoaderStatus::kIdle | DocumentLoaderStatus::kResponseReady
        ) {
            return Err(io::Error::other(
                "document is not ready for supplied source",
            ));
        }
        let url = self.current_url.clone();
        self.EnsureParserState(adapter, &url)?;
        if let Some((scheduler, _)) = self
            .pipeline
            .as_mut()
            .and_then(OpenDocumentPipeline::SchedulerMut)
        {
            scheduler.BeginTaskTurn(true);
        }
        let pipeline = self.pipeline.as_mut().expect("document pipeline installed");
        let loader = &mut self.loader;
        let mut result = None;
        adapter.WithScriptLoadClient(&mut |client| {
            result = Some(pipeline.ParseSuppliedDocument(
                loader,
                source,
                chunk_size,
                token_budget,
                client,
            ));
        });
        result.expect("adapter must provide the script load client")?;
        self.document_started = true;
        Ok(OpenEffect::DocumentStarted)
    }

    pub fn FinishSuppliedStyleSheets(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
    ) -> io::Result<()> {
        let pipeline = self.pipeline.as_mut().expect("document pipeline installed");
        let mut result = None;
        adapter.WithScriptLoadClient(&mut |client| {
            result = Some(pipeline.FinishSuppliedStyleSheets(client));
        });
        result.expect("adapter must provide the script load client")
    }

    pub fn FinishSuppliedLoad(&mut self, adapter: &mut dyn OpenDocumentAdapter) {
        if let Some((scheduler, _)) = self
            .pipeline
            .as_mut()
            .and_then(OpenDocumentPipeline::SchedulerMut)
        {
            scheduler.ConsumeTaskBudget();
        }
        adapter.FinishLoad(self.pipeline.as_mut().expect("document pipeline installed"));
    }

    /// Polling commits the final response URL into this navigation context. Page
    /// receives the response once and routes the commit to its external clients.
    pub fn PollResponse(&mut self) -> io::Result<Option<URLResponseHead>> {
        self.loader.PollResponse()
    }

    pub fn CommitURL(
        &mut self,
        adapter: &mut dyn OpenDocumentAdapter,
        url: String,
    ) -> io::Result<OpenEffect> {
        let effect = self.ApplyMutation(OpenMutation::CommitURL(url.clone()))?;
        adapter.CommitURL(&url);
        Ok(effect)
    }

    pub fn SetWakeCallback(&mut self, callback: URLWakeCallback) {
        if let Some(executor) = &self.script_executor {
            executor.SetModuleCompilationWake(Some(callback.clone()));
        }
        self.loader.SetWakeCallback(callback);
    }

    /// Open owns the per-event-loop-turn admission budget used by script and
    /// resource completion scheduling.
    pub fn BeginTaskTurn(&mut self, milliseconds: f64) {
        self.resource_completion_budget = (!(milliseconds > 0.0)).then_some(1);
        if let Some((scheduler, _)) = self
            .pipeline
            .as_mut()
            .and_then(OpenDocumentPipeline::SchedulerMut)
        {
            scheduler.BeginTaskTurn(!(milliseconds > 0.0));
        }
    }

    pub fn EndTaskTurn(&mut self) {
        self.resource_completion_budget = None;
    }

    pub fn BeginRenderingOpportunity(&mut self) -> OpenResourceCompletionScope {
        OpenResourceCompletionScope(self.resource_completion_budget.replace(0))
    }

    pub fn EndRenderingOpportunity(&mut self, scope: OpenResourceCompletionScope) {
        self.resource_completion_budget = scope.0;
    }

    pub fn HasPendingRenderBlockingStyleSheets(&self) -> bool {
        self.pipeline
            .as_ref()
            .is_some_and(OpenDocumentPipeline::HasPendingRenderBlockingStyleSheets)
    }

    /// Admit loading-related script work for this host task. The caller only
    /// supplies the concrete runtime bridge; OpenEngine owns ordering, limits
    /// and whether ordinary host tasks may run afterward.
    pub fn RunScriptTaskTurn(
        &mut self,
        milliseconds: f64,
        pump: impl FnOnce(
            &mut script_scheduler::ScriptScheduler,
            &dyn ScriptExecutor,
            usize,
        ) -> io::Result<usize>,
    ) -> io::Result<bool> {
        let Some((scheduler, executor)) = self
            .pipeline
            .as_mut()
            .and_then(OpenDocumentPipeline::SchedulerMut)
        else {
            return Ok(true);
        };
        if !scheduler.HasTaskBudget() {
            return Ok(false);
        }
        let progressed = pump(scheduler, executor, if milliseconds > 0.0 { 16 } else { 1 })?;
        if progressed != 0 && !(milliseconds > 0.0) {
            scheduler.ConsumeTaskBudget();
            return Ok(false);
        }
        Ok(true)
    }

    pub fn TraceLoadingState(&self) {
        if let Some(scheduler) = self
            .pipeline
            .as_ref()
            .and_then(OpenDocumentPipeline::Scheduler)
        {
            scheduler.TraceLoadingState();
        }
    }

    /// Adopt a document constructed by an embedding path instead of the
    /// network loader. Parser and scheduler internals remain encapsulated.
    pub fn AdoptDocument(&mut self) -> io::Result<OpenEffect> {
        if self.document_started {
            return Err(io::Error::other("document has already been built"));
        }
        if !matches!(
            self.loader.Status(),
            DocumentLoaderStatus::kIdle | DocumentLoaderStatus::kResponseReady
        ) {
            return Err(io::Error::other("document is not ready to begin parsing"));
        }
        self.document_started = true;
        Ok(OpenEffect::DocumentStarted)
    }

    pub fn LoaderStatus(&self) -> DocumentLoaderStatus {
        self.loader.Status()
    }

    pub fn Resources(&self) -> Rc<ResourceEngine> {
        self.resources.clone()
    }

    pub fn CurrentURL(&self) -> &str {
        &self.current_url
    }

    pub fn IsLoading(&self) -> bool {
        self.loading
    }

    pub fn HasCommittedDocument(&self) -> bool {
        self.document_started
    }

    pub fn LoadingFailure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub fn Status(&self) -> OpenStatus {
        if let Some(message) = &self.failure {
            return OpenStatus::Failed(message.clone());
        }
        if self.stopped {
            return OpenStatus::Stopped;
        }
        if self.loading {
            return match self.loader.Status() {
                DocumentLoaderStatus::kLoading | DocumentLoaderStatus::kResponseReady => {
                    OpenStatus::Loading
                }
                DocumentLoaderStatus::kParsing => OpenStatus::Parsing,
                DocumentLoaderStatus::kFinished => OpenStatus::WaitingForCompletion,
                DocumentLoaderStatus::kFailed | DocumentLoaderStatus::kCancelled => {
                    OpenStatus::Failed("document loading stopped".into())
                }
                DocumentLoaderStatus::kIdle => OpenStatus::Loading,
            };
        }
        if self.document_started {
            OpenStatus::Finished
        } else {
            OpenStatus::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use url_loader::{URLLoadOperation, URLLoader, URLResponse};

    struct Pending;
    impl URLLoadOperation for Pending {
        fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
            Ok(None)
        }
    }

    struct Loader;
    impl URLLoader for Loader {
        fn Load(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
            Ok(Box::new(Pending))
        }
    }

    #[test]
    fn owns_one_document_open_lifecycle() {
        let resources = Rc::new(ResourceEngine::new(
            Rc::new(RefCell::new(Loader)),
            String::new(),
        ));
        let mut engine = OpenEngine::new(resources, None);
        assert_eq!(
            engine
                .ApplyMutation(OpenMutation::Open {
                    request: URLRequest {
                        url: "https://example.test/".into(),
                        ..Default::default()
                    },
                    budget: DocumentLoadBudget::default(),
                })
                .unwrap(),
            OpenEffect::LoadingStarted
        );
        assert!(engine.IsLoading());
        assert!(engine.AdoptDocument().is_err());
        engine.ApplyMutation(OpenMutation::Stop).unwrap();

        let resources = Rc::new(ResourceEngine::new(
            Rc::new(RefCell::new(Loader)),
            String::new(),
        ));
        let mut engine = OpenEngine::new(resources, None);
        assert_eq!(engine.AdoptDocument().unwrap(), OpenEffect::DocumentStarted);
        assert!(engine.HasCommittedDocument());
        assert!(engine
            .ApplyMutation(OpenMutation::Open {
                request: URLRequest {
                    url: "https://other.test/".into(),
                    ..Default::default()
                },
                budget: DocumentLoadBudget::default(),
            })
            .is_err());
    }
}
