#![allow(non_snake_case)]

//! Source script scheduling over the persistent parser arena.

use crate::{dynamic_scripts::DynamicScriptTasks, script_execution::ParserScriptTasks};
use document_loader::{DecodeText, DocumentLoader, ResolveCSSStyleSheetURLs, ResolveUrl};
use document_loader::{RequireResponse, ResourceLoader};
use dom::persistent_document::DOMNodeType;
use dom::{Document, DOM};
use html::html_parser::{HTMLDocumentParser, HTMLParserStatus, ParserScript};
use html::html_parser_host::ParserElementPhase;
use html::{HTMLParserHost, ParserElementEvent};
use interaction::event::{EventListenerInvocation, EventPhase, EventType, MakeSyntheticEvent};
use javascript::javascript_runtime::{JavaScriptException, JavaScriptRealm, JavaScriptRuntime};
use resource::ResourceEngine;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::rc::Rc;
use std::time::Duration;
use url_loader::{RequestDestination, URLLoader, URLRequest};
use webapi::dom_bindings::DOMJavaScriptBindings;

pub trait ScriptLoadClient {
    // cpp: browser/browser.cc:1687-1691
    // Page invalidates caches after Pump,
    // before ready asynchronous scripts can observe the newly parsed nodes.
    fn DidPumpParser(&mut self) {}
    fn DidReportScriptError(&mut self, _error: &JavaScriptException) {}
    fn DidFailResource(&mut self, _url: &str, _error: &str) {}
    // Diagnostic observers; no engine/runtime behavior is supplied by them.
    fn DidExecuteScript(&mut self, _script: ParserScript, _source_name: &str, _succeeded: bool) {}
    fn DidSkipScript(&mut self, _script: ParserScript, _script_type: &str) {}
    fn DidApplyStyleSheet(&mut self, _source_url: &str) {}
    fn DidDispatchLifecycleEvent(&mut self, _name: &str) {}
}

enum PendingStyle {
    Inline {
        node: u64,
        base: String,
        sheet: Option<cssom::CSSStyleSheet>,
        render_blocking: bool,
    },
    External {
        url: String,
        resource: ResourceLoader,
        render_blocking: bool,
    },
}
struct ParserResources {
    resources: Rc<ResourceEngine>,
    current_url: String,
    base_url: String,
    base_seen: bool,
    body_seen: bool,
    styles: VecDeque<PendingStyle>,
    page: Option<Rc<document_loader::ResourceFetcher>>,
    dynamic: Option<std::rc::Weak<DynamicScriptTasks>>,
    modules: Rc<document_loader::ModuleResources>,
    supports_modules: bool,
}
struct ParserHost(Rc<RefCell<ParserResources>>);
impl HTMLParserHost for ParserHost {
    // cpp: browser/browser.cc:852-892
    fn HandleParserElement(&mut self, event: ParserElementEvent<'_>) {
        let node = event.element;
        let attr = |name| node.FindAttribute(name).map_or("", |a| a.value.as_str());
        let mut resources = self.0.borrow_mut();
        if event.phase == ParserElementPhase::kInserted {
            if node.IsHTMLElement("body") || node.IsHTMLElement("frameset") {
                resources.body_seen = true;
            }
            if node.IsHTMLElement("base") && !resources.base_seen && !attr("href").is_empty() {
                resources.base_url = ResolveUrl(&resources.current_url, attr("href"))
                    .expect("source URL resolution");
                resources.base_seen = true;
                if let Some(dynamic) = resources.dynamic.as_ref().and_then(|d| d.upgrade()) {
                    dynamic.SetBaseURL(resources.base_url.clone());
                }
                if let Some(page) = &resources.page {
                    page.SetBaseURL(resources.base_url.clone());
                }
            }
            if node.IsHTMLElement("link")
                && attr("rel")
                    .split(|c: char| matches!(c, ' ' | '\t'..='\r'))
                    .any(|s| s.eq_ignore_ascii_case("stylesheet"))
                && !attr("href").is_empty()
            {
                let url =
                    ResolveUrl(&resources.base_url, attr("href")).expect("source URL resolution");
                let request = URLRequest {
                    url: url.clone(),
                    referrer: resources.current_url.clone(),
                    destination: RequestDestination::kStyleSheet,
                    ..Default::default()
                };
                let resource = resources.resources.Start(request);
                let render_blocking = !resources.body_seen;
                resources.styles.push_back(PendingStyle::External {
                    url,
                    resource,
                    render_blocking,
                });
            }
            if node.IsHTMLElement("img") && !attr("src").is_empty() {
                if let Some(page) = &resources.page {
                    page.QueueImageWithLoadBlocking(
                        attr("src"),
                        None,
                        !attr("loading").eq_ignore_ascii_case("lazy"),
                    )
                    .expect("source URL resolution");
                }
            }
            if node.IsHTMLElement("link")
                && attr("rel")
                    .split(|c: char| matches!(c, ' ' | '\t'..='\r'))
                    .any(|s| s.eq_ignore_ascii_case("modulepreload"))
                && !attr("href").is_empty()
            {
                let url =
                    ResolveUrl(&resources.base_url, attr("href")).expect("source URL resolution");
                resources
                    .modules
                    .StartModuleLoad(&url, &resources.current_url);
            }
        } else if node.IsHTMLElement("style") {
            let base = resources.base_url.clone();
            let render_blocking = !resources.body_seen;
            resources.styles.push_back(PendingStyle::Inline {
                node: node.Id(),
                base,
                sheet: None,
                render_blocking,
            });
        }
    }

    fn HandleParserElementInDocument(
        &mut self,
        document: &mut Document,
        element: usize,
        phase: ParserElementPhase,
    ) {
        // Source snapshots the style text at children-finished, before later
        // scripts can mutate/remove it while preceding CSS is still pending.
        let sheet = if phase == ParserElementPhase::kChildrenFinished
            && document.Node(element).IsHTMLElement("style")
        {
            let mut css = String::new();
            AppendText(document, element, &mut css);
            let mut sheet = style::ParseCSS(&css);
            sheet.owner_node_id = document.Node(element).Id();
            Some(sheet)
        } else {
            None
        };
        self.HandleParserElement(ParserElementEvent {
            element: document.Node(element),
            phase,
        });
        if let Some(sheet) = sheet {
            let mut resources = self.0.borrow_mut();
            let Some(PendingStyle::Inline { sheet: stored, .. }) = resources.styles.back_mut()
            else {
                unreachable!("children-finished style queues a stylesheet");
            };
            *stored = Some(sheet);
        }
    }
}

/// Scoped arena handoff required by the existing JS parser-script adapters.
/// It lasts one turn, never the lifetime of a document load.
struct ParserArena {
    document: Rc<RefCell<DOM>>,
    owner: DOM,
}
impl ParserArena {
    fn Take(document: Rc<RefCell<DOM>>) -> Self {
        let mut arena = Self {
            document,
            owner: DOM::new(),
        };
        arena
            .document
            .borrow_mut()
            .SwapDocument(arena.owner.GetDocumentMut());
        arena
    }
}
impl Drop for ParserArena {
    fn drop(&mut self) {
        self.document
            .borrow_mut()
            .SwapDocument(self.owner.GetDocumentMut());
    }
}
fn PollParserStyleSheets(
    resources: &Rc<RefCell<ParserResources>>,
    document: &Rc<RefCell<DOM>>,
    mut parser: Option<&mut HTMLDocumentParser<'_>>,
    client: &mut dyn ScriptLoadClient,
) -> io::Result<bool> {
    for _ in 0..32 {
        let pending = resources.borrow_mut().styles.pop_front();
        let Some(pending) = pending else {
            return Ok(true);
        };
        let (mut sheet, base) = match pending {
            PendingStyle::Inline {
                node, base, sheet, ..
            } => {
                let read = |d: &Document| {
                    let mut css = String::new();
                    let i = d.FindNodeById(node).expect("style node retained in arena");
                    AppendText(d, i, &mut css);
                    let mut sheet = style::ParseCSS(&css);
                    sheet.owner_node_id = node;
                    sheet
                };
                let sheet = match sheet {
                    Some(sheet) => sheet,
                    None => match parser.as_deref_mut() {
                        Some(parser) => parser.WithDocument(|d| read(d)),
                        None => read(document.borrow().GetDocument()),
                    },
                };
                (sheet, base)
            }
            PendingStyle::External {
                url,
                mut resource,
                render_blocking,
            } => {
                if !resource.Poll() {
                    resources
                        .borrow_mut()
                        .styles
                        .push_front(PendingStyle::External {
                            url,
                            resource,
                            render_blocking,
                        });
                    return Ok(false);
                }
                match RequireResponse(resource.TakeResult()).and_then(|response| {
                    let base = if response.final_url.is_empty() {
                        url.clone()
                    } else {
                        response.final_url.clone()
                    };
                    Ok((style::ParseCSS(&DecodeText(&response)?), base))
                }) {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        client.DidFailResource(&url, &error.to_string());
                        continue;
                    }
                }
            }
        };
        let page = resources.borrow().page.clone();
        if page.is_none() {
            ResolveCSSStyleSheetURLs(&mut sheet, &base)?;
        }
        let apply = |d: &mut Document| {
            if let Some(page) = page {
                page.AddParsedStyleSheetToDocument(d, sheet, &base)?;
            } else {
                d.AppendStyleSheet(sheet);
            }
            Ok::<_, io::Error>(())
        };
        if let Some(parser) = parser.as_deref_mut() {
            parser.WithDocument(apply)?;
        } else {
            apply(document.borrow_mut().GetDocumentMut())?;
        }
        client.DidApplyStyleSheet(&base);
    }
    Ok(resources.borrow().styles.is_empty())
}

#[derive(Default)]
struct ScriptDescriptor {
    is_script: bool,
    src: String,
    script_type: String,
    asynchronous: bool,
    deferred: bool,
    nomodule: bool,
    source: String,
}

pub struct ScriptScheduler {
    document: Rc<RefCell<DOM>>,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    tasks: ParserScriptTasks,
    resources: Rc<RefCell<ParserResources>>,
    script_resources: HashMap<u64, ResourceLoader>,
    async_scripts: Vec<ParserScript>,
    deferred_scripts: Vec<ParserScript>,
    running_async: bool,
    // None preserves the explicit synchronous parser adapter. Page supplies a
    // shared script-work allowance for its nonblocking loading/task turn.
    remaining_script_tasks: Option<usize>,
    document_started: bool,
    parsing_finished: bool,
    content_loaded: bool,
    started_scripts: crate::dynamic_scripts::StartedScripts,
    dynamic_scripts: Option<Rc<DynamicScriptTasks>>,
    pending_host_errors: Option<Rc<dyn Fn() -> Vec<JavaScriptException>>>,
}

impl ScriptScheduler {
    pub fn new(
        document: Rc<RefCell<DOM>>,
        bindings: Rc<RefCell<DOMJavaScriptBindings>>,
        loader: Rc<RefCell<dyn URLLoader>>,
        current_url: String,
    ) -> Self {
        let resources = Rc::new(ResourceEngine::new(loader, current_url));
        Self::WithResourceEngine(document, bindings, resources, None)
    }

    pub fn WithInteractionState(
        document: Rc<RefCell<DOM>>,
        bindings: Rc<RefCell<DOMJavaScriptBindings>>,
        loader: Rc<RefCell<dyn URLLoader>>,
        current_url: String,
        _state: Rc<RefCell<dom::UserInteractionState>>,
    ) -> Self {
        let resources = Rc::new(ResourceEngine::new(loader, current_url));
        Self::WithResourceEngine(document, bindings, resources, None)
    }

    pub fn WithResourceEngine(
        document: Rc<RefCell<DOM>>,
        bindings: Rc<RefCell<DOMJavaScriptBindings>>,
        resources: Rc<ResourceEngine>,
        pending_host_errors: Option<Rc<dyn Fn() -> Vec<JavaScriptException>>>,
    ) -> Self {
        let mut tasks = ParserScriptTasks::new(document.clone(), bindings.clone());
        if let Some(source) = pending_host_errors.clone() {
            tasks.SetPendingHostErrors(source);
        }
        let current_url = resources.DocumentURL();
        let modules = Rc::new(document_loader::ModuleResources::WithResourceEngine(
            resources.clone(),
        ));
        Self {
            tasks,
            document,
            bindings,
            resources: Rc::new(RefCell::new(ParserResources {
                resources,
                base_url: current_url.clone(),
                current_url,
                base_seen: false,
                body_seen: false,
                styles: VecDeque::new(),
                page: None,
                dynamic: None,
                modules,
                supports_modules: false,
            })),
            script_resources: HashMap::new(),
            async_scripts: Vec::new(),
            deferred_scripts: Vec::new(),
            running_async: false,
            remaining_script_tasks: None,
            document_started: false,
            parsing_finished: false,
            content_loaded: false,
            started_scripts: Rc::new(RefCell::new(Default::default())),
            dynamic_scripts: None,
            pending_host_errors,
        }
    }

    pub fn InstallResourceFetcher(
        &mut self,
        page: Rc<document_loader::ResourceFetcher>,
    ) -> io::Result<()> {
        if self.document_started {
            return Err(io::Error::other(
                "page resources must be installed before parsing",
            ));
        }
        if !page.OwnsDocument(&self.document) {
            return Err(io::Error::other(
                "page resources must own the parser document",
            ));
        }
        page.SetBaseURL(self.resources.borrow().base_url.clone());
        self.resources.borrow_mut().page = Some(page);
        Ok(())
    }

    pub fn StartedScripts(&self) -> crate::dynamic_scripts::StartedScripts {
        self.started_scripts.clone()
    }
    pub fn BeginTaskTurn(&mut self, nonblocking: bool) {
        self.remaining_script_tasks = nonblocking.then_some(1);
    }
    pub fn HasTaskBudget(&self) -> bool {
        self.remaining_script_tasks
            .is_none_or(|remaining| remaining != 0)
    }
    pub fn ConsumeTaskBudget(&mut self) {
        if let Some(remaining) = &mut self.remaining_script_tasks {
            *remaining = remaining.saturating_sub(1);
        }
    }
    pub fn InstallDynamicScripts(&mut self, dynamic: &Rc<DynamicScriptTasks>) -> io::Result<()> {
        if self.document_started {
            return Err(io::Error::other(
                "dynamic scripts must be installed before parsing",
            ));
        }
        if !dynamic.OwnsDocument(&self.document)
            || !dynamic.SharesStartedScripts(&self.started_scripts)
        {
            return Err(io::Error::other(
                "dynamic and parser scripts must share the persistent document and started-script registry",
            ));
        }
        dynamic.SetBaseURL(self.resources.borrow().base_url.clone());
        dynamic.SetModuleResources(self.resources.borrow().modules.clone());
        if let Some(source) = &self.pending_host_errors {
            dynamic.SetPendingHostErrors(source.clone());
        }
        self.resources.borrow_mut().dynamic = Some(Rc::downgrade(dynamic));
        self.dynamic_scripts = Some(dynamic.clone());
        Ok(())
    }

    pub fn ReportHostErrors(&self, client: &mut dyn ScriptLoadClient) {
        if let Some(source) = &self.pending_host_errors {
            for error in source() {
                client.DidReportScriptError(&error);
            }
        }
    }
    fn descriptor(
        &self,
        parser: Option<&mut HTMLDocumentParser<'_>>,
        script: ParserScript,
    ) -> io::Result<ScriptDescriptor> {
        let read = |tree: &Document| {
            let node = tree
                .FindNodeById(script.node_id)
                .ok_or_else(|| io::Error::other("parser script node disappeared"))?;
            let element = tree.Node(node);
            let attribute = |name| {
                element
                    .FindAttribute(name)
                    .map_or_else(String::new, |a| a.value.clone())
            };
            let mut source = String::new();
            AppendText(tree, node, &mut source);
            Ok(ScriptDescriptor {
                is_script: element.IsHTMLElement("script"),
                src: attribute("src"),
                script_type: attribute("type"),
                asynchronous: element.FindAttribute("async").is_some(),
                deferred: element.FindAttribute("defer").is_some(),
                nomodule: element.FindAttribute("nomodule").is_some(),
                source,
            })
        };
        if let Some(parser) = parser {
            parser.WithDocument(|tree| read(tree))
        } else {
            read(self.document.borrow().GetDocument())
        }
    }
    // cpp: browser/browser.cc:1533-1551
    fn classic(descriptor: &ScriptDescriptor, runtime: &dyn JavaScriptRuntime) -> bool {
        let kind = descriptor
            .script_type
            .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
            .to_ascii_lowercase();
        descriptor.is_script
            && matches!(
                kind.as_str(),
                "" | "text/javascript"
                    | "application/javascript"
                    | "text/ecmascript"
                    | "application/ecmascript"
            )
            && !(descriptor.nomodule && runtime.SupportsModules())
    }
    fn module(descriptor: &ScriptDescriptor, runtime: &dyn JavaScriptRuntime) -> bool {
        descriptor.is_script
            && runtime.SupportsModules()
            && descriptor
                .script_type
                .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
                .eq_ignore_ascii_case("module")
    }
    // cpp: browser/browser.cc:1427-1439
    fn start_script(
        &mut self,
        script: ParserScript,
        descriptor: &ScriptDescriptor,
    ) -> io::Result<()> {
        let resources = self.resources.borrow();
        if resources.supports_modules
            && descriptor
                .script_type
                .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
                .eq_ignore_ascii_case("module")
        {
            if descriptor.src.is_empty() {
                resources.modules.StartInlineScript(
                    script.node_id,
                    &descriptor.source,
                    &resources.current_url,
                    &resources.base_url,
                );
            } else {
                resources.modules.StartExternalScript(
                    script.node_id,
                    &ResolveUrl(&resources.base_url, &descriptor.src)?,
                    &resources.current_url,
                );
            }
            return Ok(());
        }
        if descriptor.src.is_empty() || self.script_resources.contains_key(&script.node_id) {
            return Ok(());
        }
        let request = URLRequest {
            url: ResolveUrl(&resources.base_url, &descriptor.src)?,
            referrer: resources.current_url.clone(),
            destination: RequestDestination::kScript,
            ..Default::default()
        };
        self.script_resources
            .insert(script.node_id, resources.resources.Start(request));
        Ok(())
    }
    // cpp: browser/browser.cc:1441-1451
    fn script_ready(&mut self, script: ParserScript, descriptor: &ScriptDescriptor) -> bool {
        let resources = self.resources.borrow();
        if resources.supports_modules
            && descriptor
                .script_type
                .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
                .eq_ignore_ascii_case("module")
        {
            return resources.modules.ScriptReady(script.node_id);
        }
        self.script_resources
            .get_mut(&script.node_id)
            .is_none_or(|p| p.Poll())
    }
    // cpp: browser/browser.cc:1453-1467
    fn run_async(
        &mut self,
        mut parser: Option<&mut HTMLDocumentParser<'_>>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        if self.running_async {
            return Ok(());
        }
        self.running_async = true;
        let result = (|| {
            let mut index = 0;
            while index < self.async_scripts.len() && self.HasTaskBudget() {
                let script = self.async_scripts[index];
                let descriptor = self.descriptor(parser.as_deref_mut(), script)?;
                if !self.script_ready(script, &descriptor) {
                    index += 1;
                    continue;
                }
                self.async_scripts.remove(index);
                self.run_script(
                    parser.as_deref_mut(),
                    script,
                    false,
                    false,
                    runtime,
                    realm,
                    client,
                )?;
            }
            Ok(())
        })();
        self.running_async = false;
        result
    }
    // cpp: browser/browser.cc:1469-1499
    fn script_source(
        &mut self,
        script: ParserScript,
        descriptor: &ScriptDescriptor,
    ) -> io::Result<(String, String)> {
        if descriptor.src.is_empty() {
            return Ok((
                descriptor.source.clone(),
                self.resources.borrow().current_url.clone(),
            ));
        }
        self.start_script(script, descriptor)?;
        if !self.script_ready(script, descriptor) {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "script is not ready",
            ));
        }
        let mut pending = self.script_resources.remove(&script.node_id).unwrap();
        let response = RequireResponse(pending.TakeResult())?;
        let source_name = if response.final_url.is_empty() {
            ResolveUrl(&self.resources.borrow().base_url, &descriptor.src)?
        } else {
            response.final_url.clone()
        };
        Ok((DecodeText(&response)?, source_name))
    }
    // cpp: browser/browser.cc:1321-1354
    pub fn FinishStyleSheets(
        &mut self,
        mut parser: Option<&mut HTMLDocumentParser<'_>>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        loop {
            let pending = self.resources.borrow_mut().styles.pop_front();
            let Some(pending) = pending else {
                break;
            };
            let loaded = match pending {
                PendingStyle::Inline {
                    node, base, sheet, ..
                } => {
                    let read = |tree: &Document| {
                        let index = tree
                            .FindNodeById(node)
                            .expect("inline style node retained in arena");
                        let mut css = String::new();
                        AppendText(tree, index, &mut css);
                        let mut sheet = style::ParseCSS(&css);
                        sheet.owner_node_id = node;
                        sheet
                    };
                    let sheet = if let Some(sheet) = sheet {
                        sheet
                    } else if let Some(parser) = parser.as_deref_mut() {
                        parser.WithDocument(|tree| read(tree))
                    } else {
                        read(self.document.borrow().GetDocument())
                    };
                    Ok((sheet, base))
                }
                PendingStyle::External {
                    url, mut resource, ..
                } => {
                    while !resource.Poll() {
                        self.run_async(parser.as_deref_mut(), runtime, realm, client)?;
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    match RequireResponse(resource.TakeResult()).and_then(|response| {
                        let base = if response.final_url.is_empty() {
                            url.clone()
                        } else {
                            response.final_url.clone()
                        };
                        Ok((style::ParseCSS(&DecodeText(&response)?), base))
                    }) {
                        Ok(loaded) => Ok(loaded),
                        Err(error) => {
                            client.DidFailResource(&url, &error.to_string());
                            Err(error)
                        }
                    }
                }
            };
            if let Ok((mut sheet, base)) = loaded {
                let page = self.resources.borrow().page.clone();
                if page.is_none() {
                    ResolveCSSStyleSheetURLs(&mut sheet, &base)?;
                }
                let apply = |tree: &mut Document| {
                    if let Some(page) = page {
                        page.AddParsedStyleSheetToDocument(tree, sheet, &base)?;
                    } else {
                        tree.AppendStyleSheet(sheet);
                    }
                    Ok::<_, io::Error>(())
                };
                if let Some(parser) = parser.as_deref_mut() {
                    parser.WithDocument(apply)?;
                } else {
                    apply(self.document.borrow_mut().GetDocumentMut())?;
                }
                client.DidApplyStyleSheet(&base);
            }
        }
        Ok(())
    }
    // cpp: browser/browser.cc:1615-1643
    fn run_script(
        &mut self,
        mut parser: Option<&mut HTMLDocumentParser<'_>>,
        script: ParserScript,
        wait_styles: bool,
        parser_blocking: bool,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        self.ConsumeTaskBudget();
        self.started_scripts.borrow_mut().insert(script.node_id);
        let descriptor = self.descriptor(parser.as_deref_mut(), script)?;
        self.start_script(script, &descriptor)?;
        if wait_styles {
            self.FinishStyleSheets(parser.as_deref_mut(), runtime, realm, client)?;
        }
        let descriptor = self.descriptor(parser.as_deref_mut(), script)?;
        let module_source = if Self::module(&descriptor, runtime) {
            match self.resources.borrow().modules.ScriptModule(script.node_id) {
                Ok(module) => Some(module),
                Err(document_loader::ModuleLoadError::Compilation(error)) => {
                    client.DidReportScriptError(&error);
                    return Ok(());
                }
                Err(document_loader::ModuleLoadError::Resource { url, message }) => {
                    client.DidFailResource(&url, &message);
                    return Ok(());
                }
            }
        } else {
            None
        };
        let (source, source_name) = if let Some(module) = &module_source {
            (module.source.clone(), module.url.clone())
        } else {
            match self.script_source(script, &descriptor) {
                Ok(source) => source,
                Err(error) => {
                    self.script_resources.remove(&script.node_id);
                    client
                        .DidFailResource(&self.resources.borrow().current_url, &error.to_string());
                    return Ok(());
                }
            }
        };
        // Re-read type after waits, as RunScript does before Evaluate.
        let current = self.descriptor(parser.as_deref_mut(), script)?;
        let module = Self::module(&current, runtime);
        if !Self::classic(&current, runtime) && !module {
            self.bindings.borrow_mut().SetCurrentScript(0);
            runtime.PerformMicrotaskCheckpoint();
            for error in runtime.TakePendingExceptions(realm) {
                client.DidReportScriptError(&error);
            }
            return Ok(());
        }
        if !self
            .started_scripts
            .borrow_mut()
            .ClaimExecution(script.node_id)
        {
            return Ok(());
        }
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let result = if module {
            let modules = self.resources.borrow().modules.clone();
            self.tasks.ExecuteModule(
                parser,
                script,
                module_source.as_ref().unwrap().module.as_ref().unwrap(),
                &source_name,
                modules.ResolverForScript(script.node_id),
                runtime,
                realm,
                &mut |error| client.DidReportScriptError(error),
            )
        } else if parser_blocking {
            self.tasks.ExecuteClassic(
                parser
                    .as_deref_mut()
                    .expect("blocking script requires parser"),
                script,
                Some(&source),
                &source_name,
                runtime,
                realm,
                &mut |error| client.DidReportScriptError(error),
            )
        } else {
            self.tasks.ExecuteNonBlocking(
                parser,
                script,
                &source,
                &source_name,
                runtime,
                realm,
                &mut |error| client.DidReportScriptError(error),
            )
        };
        if let Some(started) = started {
            eprintln!(
                "parser-script-profile id={} source={source_name:?} bytes={} module={module} ms={:.3} exception={:?}",
                script.node_id,
                source.len(),
                started.elapsed().as_secs_f64() * 1000.0,
                result
                    .exception
                    .as_ref()
                    .map(|exception| &exception.message)
            );
        }
        client.DidExecuteScript(script, &source_name, result.Succeeded());
        self.ReportHostErrors(client);
        Ok(())
    }
    // cpp: browser/browser.cc:1501-1532
    pub fn DispatchLifecycleEvent(
        &mut self,
        kind: EventType,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) {
        let tree = self.document.borrow();
        let root = tree.GetDocument().Node(tree.GetDocument().Root()).Id();
        drop(tree);
        let mut event = MakeSyntheticEvent(kind, root);
        let dispatch = |event: &mut interaction::event::Event,
                        current,
                        phase,
                        capture,
                        runtime: &mut dyn JavaScriptRuntime,
                        client: &mut dyn ScriptLoadClient| {
            let result = DOMJavaScriptBindings::DispatchEventListeners(
                &self.bindings,
                &EventListenerInvocation {
                    event,
                    target_node_id: root,
                    current_target_node_id: current,
                    phase,
                    capture_listeners: capture,
                    focused_node_id: None,
                },
                runtime,
                realm,
                &mut |error| {
                    self.ReportHostErrors(client);
                    client.DidReportScriptError(error);
                },
            );
            self.ReportHostErrors(client);
            event.propagation_stopped |= result.stop_propagation;
            event.immediate_propagation_stopped |= result.stop_immediate_propagation;
        };
        if kind == EventType::kLoad {
            dispatch(&mut event, 0, EventPhase::kAtTarget, true, runtime, client);
            if !event.immediate_propagation_stopped {
                dispatch(&mut event, 0, EventPhase::kAtTarget, false, runtime, client);
            }
        } else {
            dispatch(&mut event, 0, EventPhase::kCapturing, true, runtime, client);
            if !event.propagation_stopped {
                dispatch(
                    &mut event,
                    root,
                    EventPhase::kAtTarget,
                    true,
                    runtime,
                    client,
                );
                if !event.immediate_propagation_stopped {
                    dispatch(
                        &mut event,
                        root,
                        EventPhase::kAtTarget,
                        false,
                        runtime,
                        client,
                    );
                }
            }
            if event.bubbles && !event.propagation_stopped {
                dispatch(&mut event, 0, EventPhase::kBubbling, false, runtime, client);
            }
        }
        runtime.PerformMicrotaskCheckpoint();
        for error in runtime.TakePendingExceptions(realm) {
            client.DidReportScriptError(&error);
        }
        self.ReportHostErrors(client);
        client.DidDispatchLifecycleEvent(interaction::event::EventTypeName(kind));
    }
    pub fn BeginDocumentLoader(
        &mut self,
        loader: &mut DocumentLoader,
        runtime: &dyn JavaScriptRuntime,
    ) -> io::Result<()> {
        if self.document_started {
            return Err(io::Error::other("document has already been built"));
        }
        self.resources.borrow_mut().supports_modules = runtime.SupportsModules();
        let mut host = ParserHost(self.resources.clone());
        loader.BeginParsing(self.document.borrow_mut().GetDocumentMut(), &mut host)?;
        self.document_started = true;
        Ok(())
    }
    /// The arena is returned to Page after every bounded loading turn, including
    /// script waits and unwinds. Script adapters may temporarily expose it to JS.
    pub fn PumpDocumentLoader(
        &mut self,
        loader: &mut DocumentLoader,
        budget: document_loader::DocumentLoadBudget,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<document_loader::DocumentLoadProgress> {
        if self.remaining_script_tasks.is_none() {
            self.PumpModules(runtime, realm, 16);
        }
        PollParserStyleSheets(&self.resources, &self.document, None, client)?;
        let mut arena = ParserArena::Take(self.document.clone());
        let mut host = ParserHost(self.resources.clone());
        let progress = loader.Pump(arena.owner.GetDocumentMut(), &mut host, budget)?;
        if progress.made_progress {
            client.DidPumpParser();
        }
        loader.WithParser(arena.owner.GetDocumentMut(), &mut host, |parser| {
            // Reserve executable parser scripts before async JS can mutate the
            // waiting element. Unsupported script types remain unstarted.
            if let Some(script)=progress.parser.script {
                let descriptor=self.descriptor(Some(parser),script)?;
                if Self::classic(&descriptor,runtime) || Self::module(&descriptor,runtime) {
                    self.started_scripts.borrow_mut().insert(script.node_id);
                }
            }
            self.run_async(Some(parser), runtime, realm, client)?;
            // An async script is a separate ScriptRunner task. Leave the
            // resident parser paused if this turn already ran that task.
            if !self.HasTaskBudget() { return Ok::<_, io::Error>(()); }
            if progress.parser.status == HTMLParserStatus::kWaitingForScript {
                if let Some(script) = progress.parser.script {
                    let descriptor = self.descriptor(Some(parser), script)?;
                    let module = Self::module(&descriptor, runtime);
                    if descriptor.is_script && descriptor.script_type.trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r')).eq_ignore_ascii_case("importmap") {
                        let resources = self.resources.borrow();
                        if let Err(message) = resources.modules.ProcessImportMap(&descriptor.source, &resources.base_url) {
                            client.DidReportScriptError(&JavaScriptException {
                                kind: javascript::javascript_runtime::JavaScriptExceptionKind::kSyntaxError,
                                message: message.into(), source_name: resources.current_url.clone(), ..Default::default()
                            });
                        }
                    } else if Self::classic(&descriptor, runtime) || module {
                        self.start_script(script, &descriptor)?;
                        if (module || !descriptor.src.is_empty()) && descriptor.asynchronous {
                            self.started_scripts.borrow_mut().insert(script.node_id);
                            self.async_scripts.push(script);
                        } else if module || (!descriptor.src.is_empty() && descriptor.deferred) {
                            self.started_scripts.borrow_mut().insert(script.node_id);
                            self.deferred_scripts.push(script);
                        } else {
                            if !PollParserStyleSheets(&self.resources, &self.document, Some(parser), client)?
                                || !self.script_ready(script, &descriptor) { return Ok::<_, io::Error>(()); }
                            self.run_script(Some(parser), script, false, true, runtime, realm, client)?;
                        }
                    } else { client.DidSkipScript(script, &descriptor.script_type); }
                }
                parser.ResumeAfterScript();
            }
            Ok::<_, io::Error>(())
        })??;
        drop(arena);
        PollParserStyleSheets(&self.resources, &self.document, None, client)?;
        Ok(progress)
    }
    pub fn PumpModules(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        budget: usize,
    ) -> usize {
        let modules = self.resources.borrow().modules.clone();
        modules.Pump(runtime, realm, budget)
    }
    pub fn HasPendingDynamicScripts(&self) -> bool {
        self.resources
            .borrow()
            .dynamic
            .as_ref()
            .and_then(|d| d.upgrade())
            .is_some_and(|dynamic| dynamic.PendingLoads() != 0)
    }
    pub fn TraceLoadingState(&self) {
        let resources = self.resources.borrow();
        let dynamic = resources.dynamic.as_ref().and_then(|d| d.upgrade());
        eprintln!(
            "script-loading-state parsing_finished={} content_loaded={} async={:?} defer={:?} styles={} dynamic={}",
            self.parsing_finished,
            self.content_loaded,
            self.async_scripts
                .iter()
                .map(|s| s.node_id)
                .collect::<Vec<_>>(),
            self.deferred_scripts
                .iter()
                .map(|s| s.node_id)
                .collect::<Vec<_>>(),
            resources.styles.len(),
            dynamic.as_ref().map_or(0, |d| d.PendingLoads())
        );
    }
    pub fn StopModuleLoading(&self) {
        self.resources.borrow().modules.StopLoading();
    }
    pub fn HasPendingStyleSheets(&self) -> bool {
        !self.resources.borrow().styles.is_empty()
    }
    pub fn HasPendingRenderBlockingStyleSheets(&self) -> bool {
        self.resources
            .borrow()
            .styles
            .iter()
            .any(|style| match style {
                PendingStyle::Inline {
                    render_blocking, ..
                }
                | PendingStyle::External {
                    render_blocking, ..
                } => *render_blocking,
            })
    }
    /// Page invokes this after the parser is done; it never waits on a deferred
    /// or asynchronous script. DOMContentLoaded precedes image/font completion.
    pub fn PollParsingCompletion(
        &mut self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<bool> {
        if !self.HasTaskBudget() {
            return Ok(false);
        }
        if self.remaining_script_tasks.is_none() {
            self.PumpModules(runtime, realm, 16);
        }
        if !self.parsing_finished {
            self.parsing_finished = true;
            self.bindings
                .borrow_mut()
                .SetReadyState("interactive".into());
            self.DispatchLifecycleEvent(EventType::kReadyStateChange, runtime, realm, client);
            if self.remaining_script_tasks.is_some() {
                self.ConsumeTaskBudget();
                return Ok(false);
            }
        }
        self.run_async(None, runtime, realm, client)?;
        if !self.HasTaskBudget() {
            return Ok(false);
        }
        if !PollParserStyleSheets(&self.resources, &self.document, None, client)? {
            return Ok(false);
        }
        // Preserve document order. Only already-ready deferred scripts execute.
        let mut remaining = 16;
        while !self.deferred_scripts.is_empty() && remaining > 0 && self.HasTaskBudget() {
            let script = self.deferred_scripts[0];
            let descriptor = self.descriptor(None, script)?;
            if !self.script_ready(script, &descriptor) {
                return Ok(false);
            }
            self.deferred_scripts.remove(0);
            self.run_script(None, script, false, false, runtime, realm, client)?;
            remaining -= 1;
        }
        if !self.deferred_scripts.is_empty() {
            return Ok(false);
        }
        if !self.HasTaskBudget() {
            return Ok(false);
        }
        if !self.content_loaded {
            self.content_loaded = true;
            self.ConsumeTaskBudget();
            self.DispatchLifecycleEvent(EventType::kDOMContentLoaded, runtime, realm, client);
            // HTMLParserScriptRunner's stable deferred-task mode returns to
            // the event loop; document load is checked on a later Page turn.
            if self.remaining_script_tasks.is_some() {
                return Ok(false);
            }
        }
        Ok(self.async_scripts.is_empty() && !self.HasPendingStyleSheets())
    }
    // cpp: browser/browser.cc:1646-1681
    pub fn ParseDocument(
        &mut self,
        source: &str,
        input_chunk_size: usize,
        parser_token_budget: usize,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        let mut loader =
            DocumentLoader::WithResourceEngine(self.resources.borrow().resources.clone());
        self.ParseDocumentWithLoader(
            &mut loader,
            source,
            input_chunk_size,
            parser_token_budget,
            runtime,
            realm,
            client,
        )
    }
    pub fn ParseDocumentWithLoader(
        &mut self,
        document_loader: &mut DocumentLoader,
        source: &str,
        input_chunk_size: usize,
        parser_token_budget: usize,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        if parser_token_budget == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "parser budgets must be positive",
            ));
        }
        self.BeginDocumentLoader(document_loader, runtime)?;
        let budget = document_loader::DocumentLoadBudget {
            body_bytes: input_chunk_size.max(1),
            parser_tokens: parser_token_budget,
        };
        let mut offset = 0;
        while offset < source.len() {
            let end = UTF8ChunkEnd(source, offset, input_chunk_size);
            document_loader.Append(&source[offset..end])?;
            offset = end;
            loop {
                let result =
                    self.PumpDocumentLoader(document_loader, budget, runtime, realm, client)?;
                if matches!(
                    result.parser.status,
                    HTMLParserStatus::kNeedMoreInput | HTMLParserStatus::kFinished
                ) {
                    break;
                }
                if result.parser.status == HTMLParserStatus::kWaitingForScript {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        document_loader.FinishInput()?;
        while document_loader.Status() != document_loader::DocumentLoaderStatus::kFinished {
            self.PumpDocumentLoader(document_loader, budget, runtime, realm, client)?;
        }
        while !self.PollParsingCompletion(runtime, realm, client)? {
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }
}

/// HTML resource discovery and stylesheet application when script execution
/// is disabled. Parsing itself stays in the resident DocumentLoader.
// cpp: browser/browser.cc:852-892,1533-1580,1646-1727
pub struct ParserResourceDiscovery {
    document: Rc<RefCell<DOM>>,
    resources: Rc<RefCell<ParserResources>>,
    import_map: HashMap<Vec<u8>, Vec<u8>>,
}
impl ParserResourceDiscovery {
    pub fn new(
        document: Rc<RefCell<DOM>>,
        loader: Rc<RefCell<dyn URLLoader>>,
        page: Rc<document_loader::ResourceFetcher>,
    ) -> Self {
        let resources = Rc::new(ResourceEngine::new(loader, String::new()));
        Self::WithResourceEngine(document, resources, page)
    }

    pub fn WithResourceEngine(
        document: Rc<RefCell<DOM>>,
        resources: Rc<ResourceEngine>,
        page: Rc<document_loader::ResourceFetcher>,
    ) -> Self {
        let modules = Rc::new(document_loader::ModuleResources::WithResourceEngine(
            resources.clone(),
        ));
        let current_url = resources.DocumentURL();
        Self {
            document,
            resources: Rc::new(RefCell::new(ParserResources {
                resources,
                current_url: current_url.clone(),
                base_url: current_url,
                base_seen: false,
                body_seen: false,
                styles: VecDeque::new(),
                page: Some(page),
                dynamic: None,
                modules,
                supports_modules: false,
            })),
            import_map: HashMap::new(),
        }
    }
    pub fn SetDocumentURL(&self, url: String) {
        let mut resources = self.resources.borrow_mut();
        resources.resources.SetDocumentURL(url.clone());
        resources.current_url = url.clone();
        resources.base_url = url;
    }
    pub fn BeginDocumentLoader(&mut self, loader: &mut DocumentLoader) -> io::Result<()> {
        let mut host = ParserHost(self.resources.clone());
        loader.BeginParsing(self.document.borrow_mut().GetDocumentMut(), &mut host)
    }
    pub fn PumpDocumentLoader(
        &mut self,
        loader: &mut DocumentLoader,
        budget: document_loader::DocumentLoadBudget,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<document_loader::DocumentLoadProgress> {
        PollParserStyleSheets(&self.resources, &self.document, None, client)?;
        let document = self.document.clone();
        let mut owner = document.borrow_mut();
        let mut host = ParserHost(self.resources.clone());
        let progress = loader.Pump(owner.GetDocumentMut(), &mut host, budget)?;
        if progress.made_progress {
            client.DidPumpParser();
        }
        if progress.parser.status == HTMLParserStatus::kWaitingForScript {
            loader.WithParser(owner.GetDocumentMut(), &mut host, |parser| {
                self.HandleParserScript(parser, progress.parser.script, client);
                parser.ResumeAfterScript();
            })?;
        }
        drop(owner);
        PollParserStyleSheets(&self.resources, &self.document, None, client)?;
        Ok(progress)
    }
    pub fn HasPendingStyleSheets(&self) -> bool {
        !self.resources.borrow().styles.is_empty()
    }
    pub fn HasPendingRenderBlockingStyleSheets(&self) -> bool {
        self.resources
            .borrow()
            .styles
            .iter()
            .any(|style| match style {
                PendingStyle::Inline {
                    render_blocking, ..
                }
                | PendingStyle::External {
                    render_blocking, ..
                } => *render_blocking,
            })
    }
    pub fn PollStyleSheets(&mut self, client: &mut dyn ScriptLoadClient) -> io::Result<bool> {
        PollParserStyleSheets(&self.resources, &self.document, None, client)
    }
    fn HandleParserScript(
        &mut self,
        parser: &mut HTMLDocumentParser<'_>,
        script: Option<ParserScript>,
        client: &mut dyn ScriptLoadClient,
    ) {
        if let Some(script) = script {
            let source = parser.WithDocument(|d| {
                let i = d.FindNodeById(script.node_id)?;
                let n = d.Node(i);
                let kind = n.FindAttribute("type").map_or("", |a| a.value.as_str());
                if !n.IsHTMLElement("script")
                    || !kind
                        .trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r'))
                        .eq_ignore_ascii_case("importmap")
                {
                    return None;
                }
                let mut source = String::new();
                AppendText(d, i, &mut source);
                Some(source)
            });
            if let Some(source) = source {
                match document_loader::ParseImportMap(source.as_bytes()) {
                    Ok(entries) => {
                        let resources = self.resources.borrow();
                        for (key, target) in entries {
                            if !target.is_empty() {
                                self.import_map.insert(
                                    key,
                                    document_loader::ResolveModuleReference(
                                        resources.base_url.as_bytes(),
                                        &target,
                                    ),
                                );
                            }
                        }
                    }
                    Err(message) => client.DidReportScriptError(&JavaScriptException {
                        kind: javascript::javascript_runtime::JavaScriptExceptionKind::kSyntaxError,
                        message: message.into(),
                        source_name: self.resources.borrow().current_url.clone(),
                        ..Default::default()
                    }),
                }
            }
        }
    }
    pub fn ParseDocument(
        &mut self,
        document_loader: &mut DocumentLoader,
        source: &str,
        chunk_size: usize,
        token_budget: usize,
        client: &mut dyn ScriptLoadClient,
    ) -> io::Result<()> {
        if token_budget == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "parser budgets must be positive",
            ));
        }
        if chunk_size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input chunk size must be positive",
            ));
        }
        self.BeginDocumentLoader(document_loader)?;
        let budget = document_loader::DocumentLoadBudget {
            body_bytes: chunk_size,
            parser_tokens: token_budget,
        };
        let mut offset = 0;
        while offset < source.len() {
            let end = UTF8ChunkEnd(source, offset, chunk_size);
            document_loader.Append(&source[offset..end])?;
            offset = end;
            loop {
                let result = self.PumpDocumentLoader(document_loader, budget, client)?;
                if matches!(
                    result.parser.status,
                    HTMLParserStatus::kNeedMoreInput | HTMLParserStatus::kFinished
                ) {
                    break;
                }
            }
        }
        document_loader.FinishInput()?;
        while document_loader.Status() != document_loader::DocumentLoaderStatus::kFinished {
            self.PumpDocumentLoader(document_loader, budget, client)?;
        }

        Ok(())
    }
    // cpp: browser/browser.cc:1321-1354
    pub fn FinishStyleSheets(&mut self, client: &mut dyn ScriptLoadClient) -> io::Result<()> {
        loop {
            let pending = self.resources.borrow_mut().styles.pop_front();
            let Some(pending) = pending else {
                return Ok(());
            };
            let (sheet, base) = match pending {
                PendingStyle::Inline {
                    node, base, sheet, ..
                } => {
                    let sheet = sheet.unwrap_or_else(|| {
                        let owner = self.document.borrow();
                        let d = owner.GetDocument();
                        let i = d.FindNodeById(node).expect("style remains in arena");
                        let mut css = String::new();
                        AppendText(d, i, &mut css);
                        let mut sheet = style::ParseCSS(&css);
                        sheet.owner_node_id = node;
                        sheet
                    });
                    (sheet, base)
                }
                PendingStyle::External {
                    url, mut resource, ..
                } => {
                    while !resource.Poll() {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    let loaded = RequireResponse(resource.TakeResult()).and_then(|r| {
                        let base = if r.final_url.is_empty() {
                            url.clone()
                        } else {
                            r.final_url.clone()
                        };
                        Ok((style::ParseCSS(&DecodeText(&r)?), base))
                    });
                    match loaded {
                        Ok(loaded) => loaded,
                        Err(error) => {
                            client.DidFailResource(&url, &error.to_string());
                            continue;
                        }
                    }
                }
            };
            let page = self
                .resources
                .borrow()
                .page
                .clone()
                .expect("no-runtime page resources");
            page.AddParsedStyleSheet(sheet, &base)?;
            client.DidApplyStyleSheet(&base);
        }
    }
}

// cpp: browser/browser.cc:240-251
fn UTF8ChunkEnd(source: &str, begin: usize, target_size: usize) -> usize {
    let mut end = source.len().min(begin.saturating_add(target_size));
    if end == source.len() {
        return end;
    }
    while end > begin && !source.is_char_boundary(end) {
        end -= 1;
    }
    if end != begin {
        return end;
    }
    end = begin + 1;
    while end < source.len() && !source.is_char_boundary(end) {
        end += 1;
    }
    end
}
// cpp: browser/browser.cc:179-184
fn AppendText(tree: &Document, index: usize, output: &mut String) {
    let node = tree.Node(index);
    if node.Type() == DOMNodeType::kText {
        output.push_str(node.Data());
    }
    for &child in node.Children() {
        AppendText(tree, child, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom_mutation::ApplyDOMTreeMutation;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    use url_loader::{URLLoadOperation, URLResponse};

    struct DelayedResponse {
        polls: usize,
        result: Option<URLResponse>,
    }
    impl URLLoadOperation for DelayedResponse {
        fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
            if self.polls > 0 {
                self.polls -= 1;
                return Ok(None);
            }
            Ok(self.result.take())
        }
    }
    struct MockLoader(Rc<RefCell<Vec<URLRequest>>>);
    impl URLLoader for MockLoader {
        fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
            self.0.borrow_mut().push(request.clone());
            let (polls, source, final_url, mime) = if request.url.ends_with("style.css") {
                (
                    2,
                    ".a{background-image:url('../image.png')}",
                    "https://cdn.test/styles/main.css",
                    "text/css",
                )
            } else if request.url.ends_with("async.js") {
                (
                    1,
                    "log.push('async'); if(!document.currentScript)throw Error('async currentScript'); try{document.write('bad');throw Error('write succeeded')}catch(e){if(!(e instanceof TypeError))throw e}",
                    "https://cdn.test/async-final.js",
                    "text/javascript",
                )
            } else if request.url.ends_with("blocking.js") {
                (
                    3,
                    "log.push('blocking');if(document.getElementById('later'))throw Error('parsed beyond blocking script');document.getElementById('written').className='from-external';",
                    "https://cdn.test/blocking-final.js",
                    "text/javascript",
                )
            } else if request.url.ends_with("defer.js") {
                (
                    0,
                    "if(document.readyState!=='interactive'||!document.getElementById('later'))throw Error('defer boundary');log.push('defer');",
                    "https://cdn.test/defer-final.js",
                    "text/javascript",
                )
            } else {
                return Err(io::Error::other("resource load failure"));
            };
            Ok(Box::new(DelayedResponse {
                polls,
                result: Some(URLResponse {
                    final_url: final_url.into(),
                    mime_type: mime.into(),
                    body: source.as_bytes().to_vec(),
                    status_code: 200,
                    ..Default::default()
                }),
            }))
        }
    }
    #[derive(Default)]
    struct Client {
        errors: Vec<JavaScriptException>,
        failed: Vec<(String, String)>,
        executed: Vec<String>,
        styles: Vec<String>,
        skipped: usize,
    }
    impl ScriptLoadClient for Client {
        fn DidReportScriptError(&mut self, e: &JavaScriptException) {
            self.errors.push(e.clone());
        }
        fn DidFailResource(&mut self, url: &str, error: &str) {
            self.failed.push((url.into(), error.into()));
        }
        fn DidExecuteScript(&mut self, _: ParserScript, url: &str, _: bool) {
            self.executed.push(url.into());
        }
        fn DidApplyStyleSheet(&mut self, url: &str) {
            self.styles.push(url.into());
        }
        fn DidSkipScript(&mut self, _: ParserScript, _: &str) {
            self.skipped += 1;
        }
    }
    #[test]
    fn external_classic_order_base_styles_and_failure_recovery() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMTreeMutation(&mut emit_document.borrow_mut(), mutation);
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let requests = Rc::new(RefCell::new(Vec::new()));
        let loader = Rc::new(RefCell::new(MockLoader(requests.clone())));
        let mut scheduler = ScriptScheduler::new(
            document.clone(),
            bindings,
            loader,
            "https://page.test/original/page.html".into(),
        );
        let source = r#"<html><head><base href='/assets/'><link rel=stylesheet href=style.css><style>p{color:red}</style>
          <script>var log=['inline'];document.write('<i id=written>你好</i>');Promise.resolve().then(()=>{if(document.currentScript!==null)throw Error('microtask currentScript');log.push('microtask')})</script>
          <script async src=async.js></script><script src=blocking.js>throw Error('inline fallback')</script>
          <script defer src=defer.js></script><script src=missing.js></script><script type=application/json>{bad json}</script>
          </head><body><div id=later></div><script defer>log.push('inline-defer');throw Error('intentional script exception')</script><script>log.push('after-error')</script></body></html>"#;
        let mut client = Client::default();
        scheduler
            .ParseDocument(source, usize::MAX, 64, &mut runtime, &realm, &mut client)
            .unwrap();
        assert_eq!(client.errors.len(), 1, "{:?}", client.errors);
        assert!(client.errors[0]
            .message
            .contains("intentional script exception"));
        assert_eq!(client.failed.len(), 1);
        assert_eq!(client.failed[0].0, "https://page.test/original/page.html");
        assert_eq!(
            client.styles,
            vec![
                "https://cdn.test/styles/main.css",
                "https://page.test/assets/"
            ]
        );
        assert_eq!(client.skipped, 1);
        assert_eq!(
            requests.borrow()[0].url,
            "https://page.test/assets/style.css"
        );
        for request in requests.borrow().iter() {
            assert_eq!(request.referrer, "https://page.test/original/page.html");
        }
        assert!(client
            .executed
            .contains(&"https://cdn.test/blocking-final.js".into()));
        let result=runtime.Evaluate(&realm,"if(JSON.stringify(log)!==JSON.stringify(['inline','microtask','async','blocking','inline-defer','after-error','defer']))throw Error(JSON.stringify(log)); if(document.getElementById('written').className!=='from-external'||document.getElementById('written').textContent!=='你好')throw Error('written DOM');true","verify.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let owner = document.borrow();
        let sheets = owner.GetDocument().StyleSheets(None);
        assert_eq!(sheets.len(), 2);
        assert_eq!(
            sheets[0].rules[0].declarations[0].value,
            "url(\"https://cdn.test/image.png\")"
        );
        drop(owner);
        assert!(scheduler
            .ParseDocument("", 1, 1, &mut runtime, &realm, &mut client)
            .is_err());
    }
    #[test]
    fn chunks_keep_utf8_and_parser_script_insertion() {
        for chunk in [0, 1, 2, 3, 7] {
            let document = Rc::new(RefCell::new(DOM::new()));
            let emit_document = document.clone();
            let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
                document.clone(),
                Box::new(move |mutation| {
                    ApplyDOMTreeMutation(&mut emit_document.borrow_mut(), mutation)
                }),
            )));
            let mut runtime = QuickJsJavaScriptRuntime::new();
            let realm = runtime.CreateRealm(bindings.clone());
            assert!(runtime
                .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
                .Succeeded());
            let mut scheduler = ScriptScheduler::new(
                document,
                bindings,
                Rc::new(RefCell::new(MockLoader(Rc::new(RefCell::new(Vec::new()))))),
                "https://page.test/".into(),
            );
            let mut client = Client::default();
            scheduler.ParseDocument("<p id=text>你好😀</p><script>document.write('<b id=written>世界😀</b>')</script><script>if(document.getElementById('text').textContent!=='你好😀'||document.getElementById('written').textContent!=='世界😀')throw Error('UTF8 split')</script>",chunk,1,&mut runtime,&realm,&mut client).unwrap();
            assert!(
                client.errors.is_empty(),
                "chunk={chunk}: {:?}",
                client.errors
            );
            assert_eq!(client.executed.len(), 2);
        }
    }

    #[test]
    fn lifecycle_capture_target_bubble_checkpoint_and_load_legacy_target() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let changed = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |m| ApplyDOMTreeMutation(&mut changed.borrow_mut(), m)),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let loader = Rc::new(RefCell::new(MockLoader(Rc::new(RefCell::new(Vec::new())))));
        let mut scheduler =
            ScriptScheduler::new(document, bindings, loader, "https://test.test/page".into());
        let source = r#"<html><head><script>
            var log=[],seen;
            function check(v,s){if(!v)throw Error(s)}
            window.addEventListener('readystatechange',function(e){log.push('state-window-capture');check(e.target===document && e.currentTarget===window && e.eventPhase===1,'state capture')},true);
            document.addEventListener('readystatechange',function(e){log.push('interactive');check(document.readyState==='interactive' && e.eventPhase===2 && !e.bubbles && !e.cancelable,'interactive');});
            window.addEventListener('readystatechange',()=>log.push('unreachable-state-bubble'));
            window.addEventListener('DOMContentLoaded',function(e){log.push('window-capture');seen=e;check(this===window && e.currentTarget===window && e.target===document && e.eventPhase===1 && e.bubbles && !e.composed && e.isTrusted,'capture')},true);
            document.addEventListener('DOMContentLoaded',function(e){log.push('document-capture');check(seen===e && e.eventPhase===2,'identity');document.body.setAttribute('data-ready','yes');Promise.resolve().then(()=>log.push('microtask'))},true);
            document.addEventListener('DOMContentLoaded',function(e){log.push('document-bubble');check(document.body.getAttribute('data-ready')==='yes','mutation');throw Error('listener error')});
            window.addEventListener('DOMContentLoaded',function(e){log.push('window-bubble');check(seen===e && e.eventPhase===3,'bubble')});
            window.addEventListener('load',function(e){log.push('load-capture');check(this===window && e.target===document && e.currentTarget===window && e.eventPhase===2 && !e.bubbles,'load legacy target')},true);
            window.onload=function(e){log.push('load-handler');check(e.target===document,'load handler')};
            document.addEventListener('load',()=>log.push('unreachable-document-load'));
        </script></head><body>text</body></html>"#;
        let mut client = Client::default();
        scheduler
            .ParseDocument(source, 17, 5, &mut runtime, &realm, &mut client)
            .unwrap();
        assert_eq!(client.errors.len(), 1, "{:?}", client.errors);
        assert!(client.errors[0].message.contains("listener error"));
        let result=runtime.Evaluate(&realm,"check(JSON.stringify(log)===JSON.stringify(['state-window-capture','interactive','window-capture','document-capture','document-bubble','window-bubble','microtask']),'parse lifecycle order');check(seen.currentTarget===null && seen.eventPhase===0,'end of dispatch')","verify");
        assert!(result.Succeeded(), "{:?}", result.exception);
        scheduler.DispatchLifecycleEvent(EventType::kLoad, &mut runtime, &realm, &mut client);
        let result = runtime.Evaluate(
            &realm,
            "check(log.slice(-2).join(',')==='load-capture,load-handler','load routing')",
            "verify",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
}
