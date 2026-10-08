#![allow(non_snake_case)]
//! Document-scoped module requests, import map, compiled records and graph readiness.
//! Pump never waits for a URL response; the engine resolver only reads this cache.
use crate::{
    resource_loader::{RequireResponse, ResourceLoader, StartResource},
    text_decode::DecodeText,
    url_reference::ResolveUrl,
};
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptExceptionKind, JavaScriptModuleCompilation,
    JavaScriptModuleCompilationJob, JavaScriptModuleCompilationPoll, JavaScriptModuleResolver,
    JavaScriptModuleSource, JavaScriptRealm, JavaScriptRuntime,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    io,
    rc::Rc,
};
use url_loader::{RequestDestination, URLLoader, URLRequest};

#[derive(Clone, Debug)]
pub enum ModuleLoadError {
    Resource { url: String, message: String },
    Compilation(JavaScriptException),
}
#[derive(Clone, PartialEq, Eq, Hash)]
enum ModuleKey {
    URL(String),
    Inline(u64),
}
struct PreparedModule {
    source: JavaScriptModuleSource,
    dependencies: Vec<String>,
    base_url: String,
    resolved_imports: HashMap<String, String>,
}
enum ModuleState {
    Compiling,
    BackgroundCompiling {
        job: JavaScriptModuleCompilationJob,
        url: String,
        source: String,
        base_url: String,
        import_map: HashMap<Vec<u8>, Vec<u8>>,
    },
    Fetching(ResourceLoader),
    Uncompiled {
        url: String,
        source: String,
        base_url: String,
    },
    Prepared(PreparedModule),
    Failed(ModuleLoadError),
}

pub struct ModuleResources {
    loader: Rc<RefCell<dyn URLLoader>>,
    entries: RefCell<HashMap<ModuleKey, ModuleState>>,
    work: RefCell<VecDeque<ModuleKey>>,
    roots: RefCell<HashMap<u64, ModuleKey>>,
    aliases: RefCell<HashMap<String, String>>,
    import_map: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
}
impl ModuleResources {
    pub fn new(loader: Rc<RefCell<dyn URLLoader>>) -> Self {
        Self {
            loader,
            entries: RefCell::new(HashMap::new()),
            work: RefCell::new(VecDeque::new()),
            roots: RefCell::new(HashMap::new()),
            aliases: RefCell::new(HashMap::new()),
            import_map: RefCell::new(HashMap::new()),
        }
    }
    fn URLKey(&self, url: &str) -> ModuleKey {
        ModuleKey::URL(
            self.aliases
                .borrow()
                .get(url)
                .cloned()
                .unwrap_or_else(|| url.to_owned()),
        )
    }
    // cpp: browser/browser.cc:790-798
    /// Also used by modulepreload. Compilation and dependent fetches occur in Pump.
    pub fn StartModuleLoad(&self, url: &str, referrer: &str) {
        if url.is_empty() {
            return;
        }
        let key = self.URLKey(url);
        if self.entries.borrow().contains_key(&key) {
            return;
        }
        let pending = StartResource(
            &mut *self.loader.borrow_mut(),
            &URLRequest {
                url: url.to_owned(),
                referrer: referrer.to_owned(),
                destination: RequestDestination::kScript,
                ..Default::default()
            },
        );
        self.entries
            .borrow_mut()
            .insert(key.clone(), ModuleState::Fetching(pending));
        self.work.borrow_mut().push_back(key);
    }
    pub fn StartExternalScript(&self, id: u64, url: &str, referrer: &str) {
        if self.roots.borrow().contains_key(&id) {
            return;
        }
        self.StartModuleLoad(url, referrer);
        self.roots.borrow_mut().insert(id, self.URLKey(url));
    }
    /// Inline roots retain distinct engine records and a captured base URL.
    pub fn StartInlineScript(&self, id: u64, source: &str, source_url: &str, base_url: &str) {
        if self.roots.borrow().contains_key(&id) {
            return;
        }
        let key = ModuleKey::Inline(id);
        self.entries.borrow_mut().insert(
            key.clone(),
            ModuleState::Uncompiled {
                url: source_url.to_owned(),
                source: source.to_owned(),
                base_url: base_url.to_owned(),
            },
        );
        self.work.borrow_mut().push_back(key.clone());
        self.roots.borrow_mut().insert(id, key);
    }
    /// The engine supplies the root's source URL, while inline imports resolve
    /// against its captured document base URL. No synthetic source URL is needed.
    pub fn ResolverForScript(self: &Rc<Self>, id: u64) -> Rc<dyn JavaScriptModuleResolver> {
        let key = self.roots.borrow().get(&id).cloned();
        if matches!(key, Some(ModuleKey::Inline(_))) {
            if let Some(ModuleKey::Inline(id)) = key {
                let entries = self.entries.borrow();
                if let Some(ModuleState::Prepared(module)) = entries.get(&ModuleKey::Inline(id)) {
                    return Rc::new(InlineResolver {
                        resources: self.clone(),
                        source_url: module.source.url.clone(),
                        base_url: module.base_url.clone(),
                        resolved_imports: module.resolved_imports.clone(),
                    });
                }
            }
        }
        self.clone()
    }
    fn ResourceError(url: &str, error: impl ToString) -> ModuleLoadError {
        ModuleLoadError::Resource {
            url: url.to_owned(),
            message: error.to_string(),
        }
    }
    fn ImportError(url: &str, error: impl ToString) -> ModuleLoadError {
        ModuleLoadError::Compilation(JavaScriptException {
            kind: JavaScriptExceptionKind::kTypeError,
            message: error.to_string(),
            source_name: url.to_owned(),
            ..Default::default()
        })
    }
    fn Compile(
        &self,
        url: String,
        source: String,
        base_url: String,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
    ) -> ModuleState {
        // Capture resolution at the original synchronous compilation point.
        // A later import map must not rebind requests while the worker runs.
        let import_map = self.import_map.borrow().clone();
        if let Some(job) = runtime.BeginCompileModule(realm, &source, &url) {
            return ModuleState::BackgroundCompiling {
                job,
                url,
                source,
                base_url,
                import_map,
            };
        }
        let compilation = match runtime.CompileModule(realm, &source, &url) {
            Ok(compilation) => compilation,
            Err(error) => return ModuleState::Failed(ModuleLoadError::Compilation(error)),
        };
        self.PrepareCompilation(url, source, base_url, &import_map, compilation)
    }
    fn PrepareCompilation(
        &self,
        url: String,
        source: String,
        base_url: String,
        import_map: &HashMap<Vec<u8>, Vec<u8>>,
        compilation: JavaScriptModuleCompilation,
    ) -> ModuleState {
        let mut dependencies = Vec::new();
        let mut resolved_imports = HashMap::new();
        for request in compilation.requests {
            let dependency = match Self::ResolveSpecifierWithMap(&request, &base_url, import_map) {
                Ok(Some(url)) => url,
                Ok(None) => {
                    return ModuleState::Failed(Self::ImportError(
                        &url,
                        format!("Unresolved module import: {request}"),
                    ))
                }
                Err(error) => return ModuleState::Failed(Self::ImportError(&url, error)),
            };
            resolved_imports.insert(request, dependency.clone());
            if !dependencies.contains(&dependency) {
                self.StartModuleLoad(&dependency, &url);
                dependencies.push(dependency);
            }
        }
        ModuleState::Prepared(PreparedModule {
            source: JavaScriptModuleSource {
                url,
                source,
                module: Some(compilation.module),
            },
            dependencies,
            base_url,
            resolved_imports,
        })
    }
    /// Advance at most budget modules; each pending response is polled once in
    /// this turn. New dependencies are queued for subsequent turns. Compilation
    /// retains the real engine record and never evaluates user code.
    pub fn Pump(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        budget: usize,
    ) -> usize {
        let count = budget.min(self.work.borrow().len());
        let mut progressed = 0;
        for _ in 0..count {
            let key = self.work.borrow_mut().pop_front().unwrap();
            let Some(state) = self.entries.borrow_mut().remove(&key) else {
                continue;
            };
            self.entries
                .borrow_mut()
                .insert(key.clone(), ModuleState::Compiling);
            let next = match state {
                ModuleState::Fetching(mut pending) => {
                    if !pending.Poll() {
                        self.entries
                            .borrow_mut()
                            .insert(key.clone(), ModuleState::Fetching(pending));
                        self.work.borrow_mut().push_back(key);
                        continue;
                    }
                    progressed += 1;
                    let ModuleKey::URL(requested) = &key else {
                        unreachable!()
                    };
                    match RequireResponse(pending.TakeResult()) {
                        Err(error) => ModuleState::Failed(Self::ResourceError(requested, error)),
                        Ok(response) => {
                            if (response.status_code != 0
                                && !(200..300).contains(&response.status_code))
                                || !JavaScriptMIME(&response.mime_type)
                            {
                                self.entries.borrow_mut().insert(
                                    key.clone(),
                                    ModuleState::Failed(Self::ResourceError(
                                        requested,
                                        format!(
                                            "Module response rejected: HTTP {}, MIME {}",
                                            response.status_code, response.mime_type
                                        ),
                                    )),
                                );
                                continue;
                            }
                            let url = if response.final_url.is_empty() {
                                requested.clone()
                            } else {
                                response.final_url.clone()
                            };
                            if url != *requested {
                                self.aliases
                                    .borrow_mut()
                                    .entry(url.clone())
                                    .or_insert(requested.clone());
                            }
                            match DecodeText(&response) {
                                Ok(source) => {
                                    self.Compile(url.clone(), source, url, runtime, realm)
                                }
                                Err(error) => {
                                    ModuleState::Failed(Self::ResourceError(requested, error))
                                }
                            }
                        }
                    }
                }
                ModuleState::Uncompiled {
                    url,
                    source,
                    base_url,
                } => {
                    progressed += 1;
                    self.Compile(url, source, base_url, runtime, realm)
                }
                ModuleState::BackgroundCompiling {
                    mut job,
                    url,
                    source,
                    base_url,
                    import_map,
                } => match runtime.PollCompileModule(realm, &mut job) {
                    JavaScriptModuleCompilationPoll::Pending => ModuleState::BackgroundCompiling {
                        job,
                        url,
                        source,
                        base_url,
                        import_map,
                    },
                    JavaScriptModuleCompilationPoll::Ready(Err(error)) => {
                        progressed += 1;
                        ModuleState::Failed(ModuleLoadError::Compilation(error))
                    }
                    JavaScriptModuleCompilationPoll::Ready(Ok(compilation)) => {
                        progressed += 1;
                        self.PrepareCompilation(url, source, base_url, &import_map, compilation)
                    }
                },
                terminal => terminal,
            };
            if matches!(next, ModuleState::BackgroundCompiling { .. }) {
                self.work.borrow_mut().push_back(key.clone());
            }
            self.entries.borrow_mut().insert(key, next);
        }
        progressed
    }
    fn GraphResult(&self, root: &ModuleKey) -> Option<Result<(), ModuleLoadError>> {
        let entries = self.entries.borrow();
        let mut stack = vec![root.clone()];
        let mut visited = HashSet::new();
        let mut pending = false;
        while let Some(key) = stack.pop() {
            if !visited.insert(key.clone()) {
                continue;
            }
            match entries.get(&key) {
                Some(ModuleState::Prepared(module)) => {
                    stack.extend(module.dependencies.iter().map(|url| self.URLKey(url)));
                }
                Some(ModuleState::Failed(error)) => return Some(Err(error.clone())),
                _ => pending = true,
            }
        }
        if pending {
            None
        } else {
            Some(Ok(()))
        }
    }
    /// Ready includes terminal failure, so a failed graph never stalls the script queue.
    pub fn ScriptReady(&self, id: u64) -> bool {
        self.roots
            .borrow()
            .get(&id)
            .is_some_and(|key| self.GraphResult(key).is_some())
    }
    pub fn ScriptModule(&self, id: u64) -> Result<JavaScriptModuleSource, ModuleLoadError> {
        let key = self
            .roots
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| Self::ResourceError("", "module script was not registered"))?;
        self.GraphResult(&key)
            .unwrap_or_else(|| Err(Self::ResourceError("", "module graph is not ready")))?;
        let entries = self.entries.borrow();
        match entries.get(&key) {
            Some(ModuleState::Prepared(module)) => Ok(module.source.clone()),
            _ => unreachable!("a successful graph has a prepared root"),
        }
    }
    /// Cancel unfinished requests. Prepared records and failed module-map
    /// entries remain document-scoped, including after HTML loading finishes.
    pub fn StopLoading(&self) {
        self.work.borrow_mut().clear();
        for (key, state) in self.entries.borrow_mut().iter_mut() {
            if !matches!(state, ModuleState::Prepared(_) | ModuleState::Failed(_)) {
                let url = match key {
                    ModuleKey::URL(url) => url.as_str(),
                    _ => "",
                };
                *state = ModuleState::Failed(Self::ResourceError(url, "module loading cancelled"));
            }
        }
    }
    // cpp: browser/browser.cc:1558-1573
    pub fn ProcessImportMap(&self, source: &str, base_url: &str) -> Result<(), &'static str> {
        let entries = crate::import_map::JSONCursor::new(source.as_bytes()).ParseImportMap()?;
        let mut map = self.import_map.borrow_mut();
        for (key, target) in entries {
            if !target.is_empty() {
                map.insert(
                    key,
                    crate::import_map::ResolveReferenceBytes(base_url.as_bytes(), &target),
                );
            }
        }
        Ok(())
    }
    fn ResolveSpecifier(&self, specifier: &str, referrer: &str) -> io::Result<Option<String>> {
        Self::ResolveSpecifierWithMap(specifier, referrer, &self.import_map.borrow())
    }
    fn ResolveSpecifierWithMap(
        specifier: &str,
        referrer: &str,
        map: &HashMap<Vec<u8>, Vec<u8>>,
    ) -> io::Result<Option<String>> {
        let url_like = specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier.starts_with('/')
            || specifier
                .find(':')
                .is_some_and(|colon| colon < specifier.find('/').unwrap_or(usize::MAX));
        if url_like {
            return ResolveUrl(referrer, specifier).map(Some);
        }
        let target = if let Some(exact) = map.get(specifier.as_bytes()) {
            Some(exact.clone())
        } else {
            map.iter()
                .filter(|(key, _)| key.ends_with(b"/") && specifier.as_bytes().starts_with(key))
                .max_by_key(|(key, _)| key.len())
                .map(|(key, target)| {
                    let mut resolved = target.clone();
                    resolved.extend_from_slice(&specifier.as_bytes()[key.len()..]);
                    resolved
                })
        };
        Ok(target.map(|target| String::from_utf8_lossy(&target).into_owned()))
    }
}
fn JavaScriptMIME(mime: &str) -> bool {
    matches!(
        mime.split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "application/ecmascript"
            | "application/javascript"
            | "application/x-ecmascript"
            | "application/x-javascript"
            | "text/ecmascript"
            | "text/javascript"
            | "text/javascript1.0"
            | "text/javascript1.1"
            | "text/javascript1.2"
            | "text/javascript1.3"
            | "text/javascript1.4"
            | "text/javascript1.5"
            | "text/jscript"
            | "text/livescript"
            | "text/x-ecmascript"
            | "text/x-javascript"
    )
}
impl JavaScriptModuleResolver for ModuleResources {
    // cpp: browser/browser.cc:800-850
    fn ResolveModule(
        &self,
        specifier: &str,
        referrer: &str,
    ) -> io::Result<Option<JavaScriptModuleSource>> {
        let captured = {
            let entries = self.entries.borrow();
            match entries.get(&self.URLKey(referrer)) {
                Some(ModuleState::Prepared(module)) => {
                    module.resolved_imports.get(specifier).cloned()
                }
                _ => None,
            }
        };
        let Some(url) = (match captured {
            Some(url) => Some(url),
            None => self.ResolveSpecifier(specifier, referrer)?,
        }) else {
            return Ok(None);
        };
        self.ResolvePreparedURL(&url)
    }

    fn RequestDynamicModule(&self, specifier: &str, referrer: &str) -> io::Result<Option<String>> {
        let Some(url) = self.ResolveSpecifier(specifier, referrer)? else {
            return Ok(None);
        };
        self.StartModuleLoad(&url, referrer);
        Ok(Some(url))
    }

    fn PollDynamicModule(&self, url: &str) -> io::Result<Option<JavaScriptModuleSource>> {
        let key = self.URLKey(url);
        match self.GraphResult(&key) {
            None => Ok(None),
            Some(Err(ModuleLoadError::Resource { message, .. })) => Err(io::Error::other(message)),
            Some(Err(ModuleLoadError::Compilation(error))) => Err(io::Error::other(error.message)),
            Some(Ok(())) => self.ResolvePreparedURL(url),
        }
    }
}
impl ModuleResources {
    fn ResolvePreparedURL(&self, url: &str) -> io::Result<Option<JavaScriptModuleSource>> {
        let key = self.URLKey(&url);
        let entries = self.entries.borrow();
        match entries.get(&key) {
            Some(ModuleState::Prepared(module)) => Ok(Some(module.source.clone())),
            Some(ModuleState::Failed(ModuleLoadError::Resource { message, .. })) => {
                Err(io::Error::other(message.clone()))
            }
            Some(ModuleState::Failed(ModuleLoadError::Compilation(error))) => {
                Err(io::Error::other(error.message.clone()))
            }
            _ => Err(io::Error::other(format!("Module is not prepared: {url}"))),
        }
    }
}

struct InlineResolver {
    resources: Rc<ModuleResources>,
    source_url: String,
    base_url: String,
    resolved_imports: HashMap<String, String>,
}
impl JavaScriptModuleResolver for InlineResolver {
    fn ResolveModule(
        &self,
        specifier: &str,
        referrer: &str,
    ) -> io::Result<Option<JavaScriptModuleSource>> {
        if referrer == self.source_url {
            if let Some(url) = self.resolved_imports.get(specifier) {
                return self.resources.ResolvePreparedURL(url);
            }
        }
        let base = if referrer == self.source_url {
            &self.base_url
        } else {
            referrer
        };
        self.resources.ResolveModule(specifier, base)
    }

    fn RequestDynamicModule(&self, specifier: &str, referrer: &str) -> io::Result<Option<String>> {
        let base = if referrer == self.source_url {
            &self.base_url
        } else {
            referrer
        };
        self.resources.RequestDynamicModule(specifier, base)
    }

    fn PollDynamicModule(&self, url: &str) -> io::Result<Option<JavaScriptModuleSource>> {
        self.resources.PollDynamicModule(url)
    }
}
