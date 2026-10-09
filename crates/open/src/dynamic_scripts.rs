#![allow(non_snake_case)]
//! Page's connected-subtree script tasks over the existing persistent DOM.
use crate::script_scheduler::ScriptLoadClient;
use document_loader::{DecodeText, RequireResponse, ResolveUrl, ResourceLoader};
use dom::{Document, DOM};
use html::html_parser::ParserScript;
use interaction::event::{EventListenerInvocation, EventPhase, EventType, MakeSyntheticEvent};
use javascript::javascript_runtime::{JavaScriptRealm, JavaScriptRuntime};
use resource::ResourceEngine;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    io,
    rc::Rc,
};
use url_loader::{RequestDestination, URLLoader, URLRequest};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};

// Preparation reserves a script while its resource is still loading. Execution
// is claimed separately, shared by parser and dynamic queues, before entering
// JS: a pending registration must not execute a parser script a second time.
#[derive(Default)]
pub struct StartedScriptRegistry {
    prepared: HashSet<u64>,
    executed: HashSet<u64>,
}
impl StartedScriptRegistry {
    pub fn contains(&self, id: &u64) -> bool {
        self.prepared.contains(id)
    }
    pub fn insert(&mut self, id: u64) -> bool {
        self.prepared.insert(id)
    }
    pub fn ClaimExecution(&mut self, id: u64) -> bool {
        self.prepared.insert(id);
        self.executed.insert(id)
    }
}
pub type StartedScripts = Rc<RefCell<StartedScriptRegistry>>;
pub struct DynamicScriptTasks {
    document: Rc<RefCell<DOM>>,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    resource_engine: Rc<ResourceEngine>,
    current_url: String,
    base_url: RefCell<String>,
    started: StartedScripts,
    supports_modules: bool,
    modules: RefCell<Rc<document_loader::ModuleResources>>,
    resources: RefCell<HashMap<u64, ResourceLoader>>,
    registrations: RefCell<VecDeque<u64>>,
    pending_modules: RefCell<HashSet<u64>>,
    pending_host_errors:
        RefCell<Option<Rc<dyn Fn() -> Vec<javascript::javascript_runtime::JavaScriptException>>>>,
}
#[derive(PartialEq, Eq)]
enum Kind {
    Skip,
    Classic,
    Module,
}
fn KindFor(tree: &Document, node: usize, modules: bool) -> Kind {
    let element = tree.Node(node);
    if !element.IsHTMLElement("script") {
        return Kind::Skip;
    }
    let kind = element
        .FindAttribute("type")
        .map_or("", |a| a.value.as_str())
        .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
        .to_ascii_lowercase();
    if kind == "module" {
        return if modules { Kind::Module } else { Kind::Skip };
    }
    if !matches!(
        kind.as_str(),
        "" | "text/javascript"
            | "application/javascript"
            | "text/ecmascript"
            | "application/ecmascript"
    ) || (modules && element.FindAttribute("nomodule").is_some())
    {
        Kind::Skip
    } else {
        Kind::Classic
    }
}
fn AppendText(tree: &Document, node: usize, text: &mut String) {
    let element = tree.Node(node);
    if element.Type() == dom::persistent_document::DOMNodeType::kText {
        text.push_str(element.Data());
    }
    for &child in element.Children() {
        AppendText(tree, child, text);
    }
}
impl DynamicScriptTasks {
    pub fn new(
        document: Rc<RefCell<DOM>>,
        bindings: Rc<RefCell<DOMJavaScriptBindings>>,
        loader: Rc<RefCell<dyn URLLoader>>,
        current_url: String,
        started: StartedScripts,
        supports_modules: bool,
    ) -> Self {
        let resource_engine = Rc::new(ResourceEngine::new(loader, current_url.clone()));
        Self::WithResourceEngine(
            document,
            bindings,
            resource_engine,
            started,
            supports_modules,
        )
    }

    pub fn WithResourceEngine(
        document: Rc<RefCell<DOM>>,
        bindings: Rc<RefCell<DOMJavaScriptBindings>>,
        resource_engine: Rc<ResourceEngine>,
        started: StartedScripts,
        supports_modules: bool,
    ) -> Self {
        let current_url = resource_engine.DocumentURL();
        let modules = Rc::new(document_loader::ModuleResources::WithResourceEngine(
            resource_engine.clone(),
        ));
        Self {
            document,
            bindings,
            resource_engine,
            base_url: RefCell::new(current_url.clone()),
            current_url,
            started,
            supports_modules,
            modules: RefCell::new(modules),
            resources: RefCell::new(HashMap::new()),
            registrations: RefCell::new(VecDeque::new()),
            pending_modules: RefCell::new(HashSet::new()),
            pending_host_errors: RefCell::new(None),
        }
    }
    pub fn OwnsDocument(&self, document: &Rc<RefCell<DOM>>) -> bool {
        Rc::ptr_eq(&self.document, document)
    }
    pub fn SharesStartedScripts(&self, started: &StartedScripts) -> bool {
        Rc::ptr_eq(&self.started, started)
    }
    pub fn SetBaseURL(&self, base: String) {
        *self.base_url.borrow_mut() = base;
    }
    pub fn SetModuleResources(&self, modules: Rc<document_loader::ModuleResources>) {
        *self.modules.borrow_mut() = modules;
    }
    pub fn PendingLoads(&self) -> usize {
        self.resources.borrow().len() + self.pending_modules.borrow().len()
    }
    pub fn SetPendingHostErrors(
        &self,
        source: Rc<dyn Fn() -> Vec<javascript::javascript_runtime::JavaScriptException>>,
    ) {
        *self.pending_host_errors.borrow_mut() = Some(source);
    }
    fn ReportHostErrors(&self, client: &Rc<RefCell<dyn ScriptLoadClient>>) {
        let source = self.pending_host_errors.borrow().clone();
        if let Some(source) = source {
            for error in source() {
                client.borrow_mut().DidReportScriptError(&error);
            }
        }
    }
    // cpp: browser/browser.cc:1575-1580,1591-1613
    // Called after Page applies a mutation and its notification. The caller
    // supplies the live arena it already borrows; no second Document borrow.
    // Scripts installed through innerHTML have prepare_scripts=false.
    pub fn PrepareConnectedScript(
        &self,
        tree: &Document,
        node: usize,
        prepare_scripts: bool,
    ) -> io::Result<()> {
        let mut root = node;
        while let Some(parent) = tree.Node(root).Parent() {
            root = parent;
        }
        if tree.Node(root).Type() != dom::persistent_document::DOMNodeType::kDocument {
            return Ok(());
        }
        let id = tree.Node(node).Id();
        let kind = KindFor(tree, node, self.supports_modules);
        if prepare_scripts
            && root == tree.Root()
            && kind != Kind::Skip
            && !self.started.borrow().contains(&id)
        {
            let mut text = String::new();
            AppendText(tree, node, &mut text);
            let src = tree
                .Node(node)
                .FindAttribute("src")
                .map_or("", |a| a.value.as_str());
            if !src.is_empty() || !text.is_empty() {
                self.started.borrow_mut().insert(id);
                if kind == Kind::Module {
                    if src.is_empty() {
                        self.modules.borrow().StartInlineScript(
                            id,
                            &text,
                            &self.current_url,
                            &self.base_url.borrow(),
                        );
                    } else {
                        self.modules.borrow().StartExternalScript(
                            id,
                            &ResolveUrl(&self.base_url.borrow(), src)?,
                            &self.current_url,
                        );
                    }
                    self.pending_modules.borrow_mut().insert(id);
                } else {
                    self.StartScriptLoad(id, src)?;
                }
                self.registrations.borrow_mut().push_back(id);
            }
        }
        Ok(())
    }
    pub fn PrepareConnectedScripts(
        &self,
        tree: &Document,
        node: usize,
        prepare_scripts: bool,
    ) -> io::Result<()> {
        self.PrepareConnectedScript(tree, node, prepare_scripts)?;
        for &child in tree.Node(node).Children() {
            self.PrepareConnectedScripts(tree, child, prepare_scripts)?;
        }
        Ok(())
    }
    // cpp: browser/browser.cc:1427-1439
    fn StartScriptLoad(&self, id: u64, src: &str) -> io::Result<()> {
        if src.is_empty() || self.resources.borrow().contains_key(&id) {
            return Ok(());
        }
        let request = URLRequest {
            url: ResolveUrl(&self.base_url.borrow(), src)?,
            referrer: self.current_url.clone(),
            destination: RequestDestination::kScript,
            ..Default::default()
        };
        let pending = self.resource_engine.Start(request);
        self.resources.borrow_mut().insert(id, pending);
        Ok(())
    }
    // cpp: browser/browser.cc:1441-1451
    fn ScriptReady(&self, id: u64) -> bool {
        let owner = self.document.borrow();
        let tree = owner.GetDocument();
        if let Some(index) = tree.FindNodeById(id) {
            if KindFor(tree, index, self.supports_modules) == Kind::Module {
                return self.modules.borrow().ScriptReady(id);
            }
        }
        if let Some(index) = tree.FindNodeById(id) {
            let src = tree
                .Node(index)
                .FindAttribute("src")
                .map_or("", |a| a.value.as_str());
            if !src.is_empty() && self.StartScriptLoad(id, src).is_err() {
                return true;
            }
        }
        self.resources
            .borrow_mut()
            .get_mut(&id)
            .is_none_or(|p| p.Poll())
    }
    // cpp: browser/browser.cc:1469-1499
    fn ScriptSource(&self, id: u64) -> io::Result<(String, String)> {
        let (src, inline) = {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            let index = tree
                .FindNodeById(id)
                .ok_or_else(|| io::Error::other("parser script node disappeared"))?;
            let src = tree
                .Node(index)
                .FindAttribute("src")
                .map_or_else(String::new, |a| a.value.clone());
            let mut text = String::new();
            if src.is_empty() {
                AppendText(tree, index, &mut text);
            }
            (src, text)
        };
        if src.is_empty() {
            return Ok((inline, self.current_url.clone()));
        }
        self.StartScriptLoad(id, &src)?;
        // Readiness started and polled this request before scheduling execution.
        let mut pending = self
            .resources
            .borrow_mut()
            .remove(&id)
            .expect("started external script has a pending load");
        if !pending.Poll() {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "script is not ready",
            ));
        }
        let response = RequireResponse(pending.TakeResult())?;
        let name = if response.final_url.is_empty() {
            ResolveUrl(&self.base_url.borrow(), &src)?
        } else {
            response.final_url.clone()
        };
        Ok((DecodeText(&response)?, name))
    }
    // cpp: browser/browser.cc:1615-1643
    fn RunScript(
        &self,
        id: u64,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &Rc<RefCell<dyn ScriptLoadClient>>,
    ) {
        self.started.borrow_mut().insert(id);
        self.pending_modules.borrow_mut().remove(&id);
        let kind = {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            tree.FindNodeById(id)
                .map_or(Kind::Skip, |i| KindFor(tree, i, runtime.SupportsModules()))
        };
        let module_source = if kind == Kind::Module {
            match self.modules.borrow().ScriptModule(id) {
                Ok(module) => Some(module),
                Err(document_loader::ModuleLoadError::Compilation(error)) => {
                    client.borrow_mut().DidReportScriptError(&error);
                    return;
                }
                Err(document_loader::ModuleLoadError::Resource { url, message }) => {
                    client.borrow_mut().DidFailResource(&url, &message);
                    return;
                }
            }
        } else {
            None
        };
        let (source, name) = if let Some(module) = &module_source {
            (module.source.clone(), module.url.clone())
        } else {
            match self.ScriptSource(id) {
                Ok(source) => source,
                Err(error) => {
                    self.resources.borrow_mut().remove(&id);
                    client
                        .borrow_mut()
                        .DidFailResource(&self.current_url, &error.to_string());
                    return;
                }
            }
        };
        // No Document, service state, Window or client borrow survives Evaluate.
        self.bindings
            .borrow_mut()
            .SetCurrentScript(if kind == Kind::Classic { id } else { 0 });
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let result = match kind {
            Kind::Classic => runtime.Evaluate(realm, &source, &name),
            Kind::Skip => Default::default(),
            Kind::Module => {
                let modules = self.modules.borrow().clone();
                runtime.EvaluateCompiledModule(
                    realm,
                    module_source.as_ref().unwrap().module.as_ref().unwrap(),
                    &name,
                    Some(modules.ResolverForScript(id)),
                )
            }
        };
        if let Some(started) = started {
            eprintln!(
                "dynamic-script-profile id={id} source={name:?} bytes={} ms={:.3} exception={:?}",
                source.len(),
                started.elapsed().as_secs_f64() * 1000.0,
                result
                    .exception
                    .as_ref()
                    .map(|exception| &exception.message)
            );
        }
        self.bindings.borrow_mut().SetCurrentScript(0);
        self.ReportHostErrors(client);
        if let Some(error) = &result.exception {
            client.borrow_mut().DidReportScriptError(error);
        }
        runtime.PerformMicrotaskCheckpoint();
        self.ReportHostErrors(client);
        for error in runtime.TakePendingExceptions(realm) {
            client.borrow_mut().DidReportScriptError(&error);
        }
        client.borrow_mut().DidExecuteScript(
            ParserScript {
                node_id: id,
                ..Default::default()
            },
            &name,
            result.Succeeded(),
        );
    }
    // cpp: browser/browser.cc:1599-1608
    // Flush after the outer host invocation releases its Window borrow. Loads
    // already began at the mutation point, before execution returns to JS.
    pub fn EnqueuePreparedTasks(
        self: &Rc<Self>,
        window: &Rc<RefCell<WindowJavaScriptBindings>>,
        client: Rc<RefCell<dyn ScriptLoadClient>>,
    ) {
        loop {
            let Some(id) = self.registrations.borrow_mut().pop_front() else {
                break;
            };
            let ready = Rc::downgrade(self);
            let run = Rc::downgrade(self);
            let report = Rc::downgrade(&client);
            window.borrow_mut().EnqueueTaskWithRuntime(
                Box::new(move |runtime, realm| {
                    let (Some(service), Some(client)) = (run.upgrade(), report.upgrade()) else {
                        return;
                    };
                    if !service.started.borrow_mut().ClaimExecution(id) {
                        service.resources.borrow_mut().remove(&id);
                        service.pending_modules.borrow_mut().remove(&id);
                        return;
                    }
                    service.RunScript(id, runtime, realm, &client);
                    let event = MakeSyntheticEvent(EventType::kLoad, id);
                    DOMJavaScriptBindings::DispatchEventListeners(
                        &service.bindings,
                        &EventListenerInvocation {
                            event: &event,
                            target_node_id: id,
                            current_target_node_id: id,
                            phase: EventPhase::kAtTarget,
                            capture_listeners: false,
                            focused_node_id: None,
                        },
                        runtime,
                        realm,
                        &mut |error| {
                            service.ReportHostErrors(&client);
                            client.borrow_mut().DidReportScriptError(error);
                        },
                    );
                    service.ReportHostErrors(&client);
                }),
                Some(Rc::new(move || {
                    ready.upgrade().is_none_or(|s| {
                        s.started.borrow().executed.contains(&id) || s.ScriptReady(id)
                    })
                })),
            );
        }
    }
}

/// Page host composition that releases Window before registering mutation tasks.
/// The DOM mutation emitter invokes PrepareConnectedScripts; this adapter owns
/// its queue flush, including mutations from focus/event host continuations.
pub struct DynamicScriptHost {
    window: Rc<RefCell<WindowJavaScriptBindings>>,
    scripts: Rc<DynamicScriptTasks>,
    client: Rc<RefCell<dyn ScriptLoadClient>>,
}
impl DynamicScriptHost {
    pub fn new(
        window: Rc<RefCell<WindowJavaScriptBindings>>,
        scripts: Rc<DynamicScriptTasks>,
        client: Rc<RefCell<dyn ScriptLoadClient>>,
    ) -> Self {
        Self {
            window,
            scripts,
            client,
        }
    }
    fn Flush(&self) {
        self.scripts
            .EnqueuePreparedTasks(&self.window, self.client.clone());
    }
}
impl javascript::javascript_runtime::JavaScriptHostBindings for DynamicScriptHost {
    fn GlobalNames(&self) -> Vec<String> {
        self.window.borrow().GlobalNames()
    }
    fn PrepareInvocation(
        &mut self,
        call: &javascript::javascript_runtime::HostCall<'_>,
    ) -> Option<javascript::javascript_runtime::HostContinuation> {
        let continuation = self.window.borrow_mut().PrepareInvocation(call)?;
        let window = self.window.clone();
        let scripts = self.scripts.clone();
        let client = self.client.clone();
        Some(Box::new(move |runtime| {
            let result = continuation(runtime);
            scripts.EnqueuePreparedTasks(&window, client);
            result
        }))
    }
    fn Invoke(
        &mut self,
        call: &javascript::javascript_runtime::HostCall<'_>,
    ) -> javascript::javascript_runtime::HostResult {
        let result = self.window.borrow_mut().Invoke(call);
        self.Flush();
        result
    }
    fn InvokeWithRuntime(
        &mut self,
        call: &javascript::javascript_runtime::HostCall<'_>,
        runtime: &mut dyn javascript::javascript_runtime::JavaScriptHostRuntime,
    ) -> javascript::javascript_runtime::HostResult {
        let result = self.window.borrow_mut().InvokeWithRuntime(call, runtime);
        self.Flush();
        result
    }
    fn PrototypeFor(
        &mut self,
        id: javascript::javascript_runtime::HostObjectId,
    ) -> javascript::javascript_runtime::HostResult {
        self.window.borrow_mut().PrototypeFor(id)
    }
}
#[cfg(test)]
mod tests;
