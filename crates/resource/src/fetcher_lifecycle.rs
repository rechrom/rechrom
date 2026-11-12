//! Chromium ResourceFetcher.cc:2680-2723,2907-2928,3172-3201;
//! ResourceLoader.cc:490-493,1156-1222,1367-1376. These are the original
//! in-flight loader sets, independently of MemoryCache and returned handles.
use crate::{
    resource_loader::SharedResourceLoad, ResourceEngine, ResourceLoadStatus, ResourceLoader,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use url_loader::URLRequest;

/// FetchParameters::IsLinkPreload and ImageLoadBlockingPolicy remain
/// explicit source inputs; network priority/defer does not imply either.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResourceLoadEventPolicy {
    Regular,
    LinkPreload,
    ForceNonBlockingImage,
}
impl ResourceLoadEventPolicy {
    pub(crate) fn BlocksLoad(self, request: &URLRequest) -> bool {
        use url_loader::RequestDestination as D;
        // Resource::IsLoadEventBlockingResourceType cc1365-1390. Main
        // document bytes use DocumentLoader, not a subresource loader set.
        self == Self::Regular
            && matches!(request.destination, D::kImage | D::kStyleSheet | D::kFont)
    }
}
pub(crate) struct RequestResourceScope {
    state: Rc<std::cell::Cell<bool>>,
    previous: bool,
}
impl RequestResourceScope {
    pub(crate) fn new(state: Rc<std::cell::Cell<bool>>) -> Self {
        let previous = state.replace(true);
        Self { state, previous }
    }
}
impl Drop for RequestResourceScope {
    fn drop(&mut self) {
        self.state.set(self.previous);
    }
}
/// Same freezable task runner used by ResourceFetcher.cc2380-2387. The
/// callback belongs to the fetcher, not to an old Window's script task queue.
pub trait FetcherCancellableTask {
    fn Cancel(&self);
}
pub trait FetcherWakeTask: FetcherCancellableTask {
    fn WakeCallback(&self) -> url_loader::URLWakeCallback;
}
pub trait FetcherTaskRunner {
    /// Register a reusable Page resource task. Its Send+Sync wake merely
    /// enqueues; execution never belongs to an old Window/Document task scope.
    fn CreateWakeTask(&self, task: Box<dyn FnMut() -> io::Result<()>>) -> Rc<dyn FetcherWakeTask>;
    fn PostDelayedTask(
        &self,
        delay: std::time::Duration,
        task: Box<dyn FnOnce() -> io::Result<()>>,
    ) -> Rc<dyn FetcherCancellableTask>;
}
/// Resource::FinishAsError's native cancellation client. Delivery is synchronous
/// and occurs after membership/quota removal, with every engine borrow released.
/// The receiver gets values, never a ResourceEngine/VM/DOM capability.
pub trait ResourceCancellationClient {
    fn ResourceCancelled(&self, resource_id: u64, request: &URLRequest);
}
#[derive(Clone, Copy)]
pub(crate) enum LoaderSet {
    Blocking,
    NonBlocking,
}
#[derive(Default)]
pub(crate) struct FetcherLoaderSets {
    blocking: RefCell<HashMap<u64, Rc<RefCell<SharedResourceLoad>>>>,
    non_blocking: RefCell<HashMap<u64, Rc<RefCell<SharedResourceLoad>>>>,
    keepalive_timeout: RefCell<Option<Rc<dyn FetcherCancellableTask>>>,
    detached_completion: RefCell<Option<Rc<dyn FetcherWakeTask>>>,
}
pub(crate) struct LoaderMembership {
    owner: Weak<FetcherLoaderSets>,
    set: LoaderSet,
    identity: u64,
}
impl FetcherLoaderSets {
    pub(crate) fn Add(self: &Rc<Self>, loader: &ResourceLoader, set: LoaderSet) {
        let identity = loader.Identity();
        self.Map(set).borrow_mut().insert(identity, loader.Shared());
        loader.BindFetcherMembership(LoaderMembership {
            owner: Rc::downgrade(self),
            set,
            identity,
        });
    }
    pub(crate) fn BlockingRequestCount(&self) -> usize {
        self.blocking.borrow().len()
    }
    fn Map(&self, set: LoaderSet) -> &RefCell<HashMap<u64, Rc<RefCell<SharedResourceLoad>>>> {
        match set {
            LoaderSet::Blocking => &self.blocking,
            LoaderSet::NonBlocking => &self.non_blocking,
        }
    }
    fn IsEmpty(&self) -> bool {
        self.blocking.borrow().is_empty() && self.non_blocking.borrow().is_empty()
    }
    fn CancelKeepaliveTimeoutIfEmpty(&self) {
        if self.IsEmpty() {
            let task = self.keepalive_timeout.borrow_mut().take();
            if let Some(task) = task {
                task.Cancel();
            }
            let completion = self.detached_completion.borrow_mut().take();
            if let Some(completion) = completion {
                completion.Cancel();
            }
        }
    }
    fn Contains(&self, shared: &Rc<RefCell<SharedResourceLoad>>) -> bool {
        let id = ResourceLoader::FromShared(shared.clone()).Identity();
        self.non_blocking
            .borrow()
            .get(&id)
            .is_some_and(|actual| Rc::ptr_eq(actual, shared))
            || self
                .blocking
                .borrow()
                .get(&id)
                .is_some_and(|actual| Rc::ptr_eq(actual, shared))
    }
    pub(crate) fn Snapshot(&self, set: LoaderSet) -> Vec<Rc<RefCell<SharedResourceLoad>>> {
        self.Map(set).borrow().values().cloned().collect()
    }
}
impl LoaderMembership {
    pub(crate) fn PromoteToBlocking(&mut self, shared: Rc<RefCell<SharedResourceLoad>>) {
        if matches!(self.set, LoaderSet::Blocking) {
            return;
        }
        if let Some(owner) = self.owner.upgrade() {
            let original = owner.non_blocking.borrow_mut().remove(&self.identity);
            if original
                .as_ref()
                .is_some_and(|original| Rc::ptr_eq(original, &shared))
            {
                owner.blocking.borrow_mut().insert(self.identity, shared);
                self.set = LoaderSet::Blocking;
            }
        }
    }
    pub(crate) fn Remove(self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.Map(self.set).borrow_mut().remove(&self.identity);
            owner.CancelKeepaliveTimeoutIfEmpty();
        }
    }
}
impl ResourceEngine {
    /// The detached property belongs to this fetch context. Every request must
    /// retain the same flag, including requests already handed to the network
    /// keepalive owner before the Document is shut down.
    pub fn FetchContextDetachedState(&self) -> Arc<AtomicBool> {
        self.context_detached.clone()
    }
    pub fn BindFetcherTaskRunner(&self, runner: Rc<dyn FetcherTaskRunner>) {
        *self.fetcher_task_runner.borrow_mut() = Some(runner);
    }
    pub fn BlockingRequestCount(&self) -> usize {
        self.fetcher_loaders.BlockingRequestCount()
    }
    pub fn IsInRequestResource(&self) -> bool {
        self.is_in_request_resource.get()
    }
    pub fn StopFetching(&self) -> io::Result<()> {
        self.StopFetchingInternal(false)
    }
    pub fn StopFetchingIncludingKeepaliveLoaders(&self) -> io::Result<()> {
        self.StopFetchingInternal(true)
    }
    fn StopFetchingInternal(&self, include_keepalive: bool) -> io::Result<()> {
        enum CancelTarget {
            Resource(Rc<RefCell<SharedResourceLoad>>),
            Track(Rc<crate::TextTrackResource>),
            XMLHttpRequest(Rc<crate::RegisteredXMLHttpRequestOperation>),
        }
        // Snapshot ALL original members before invoking the first client.
        let select = |snapshot: Vec<Rc<RefCell<SharedResourceLoad>>>| -> io::Result<Vec<_>> {
            let mut selected = Vec::new();
            for shared in snapshot {
                if include_keepalive
                    || !ResourceLoader::FromShared(shared.clone())
                        .ShouldBeKeptAliveWhenDetached()?
                {
                    selected.push(shared);
                }
            }
            Ok(selected)
        };
        let non_blocking = select(self.fetcher_loaders.Snapshot(LoaderSet::NonBlocking))?;
        let blocking = select(self.fetcher_loaders.Snapshot(LoaderSet::Blocking))?;
        let tracks = self
            .text_track_resources
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let xmlhttp_requests = self.xmlhttp_requests.Snapshot();
        let snapshot = non_blocking
            .into_iter()
            .map(CancelTarget::Resource)
            .chain(tracks.into_iter().map(CancelTarget::Track))
            .chain(
                xmlhttp_requests
                    .into_iter()
                    .map(CancelTarget::XMLHttpRequest),
            )
            .chain(blocking.into_iter().map(CancelTarget::Resource))
            .collect::<Vec<_>>();
        for target in snapshot {
            let shared = match target {
                CancelTarget::Resource(shared) => shared,
                CancelTarget::XMLHttpRequest(operation) => {
                    if self.xmlhttp_requests.Contains(&operation) {
                        operation.Cancel();
                    }
                    continue;
                }
                CancelTarget::Track(track) => {
                    let identity = track.Identity();
                    let current = self.text_track_resources.borrow().get(&identity).cloned();
                    if current
                        .as_ref()
                        .is_some_and(|current| Rc::ptr_eq(current, &track))
                    {
                        self.text_track_resources.borrow_mut().remove(&identity);
                        track.CancelForFetcher();
                    }
                    continue;
                }
            };
            // A preceding client may synchronously cancel/remove this loader,
            // shut down the document, or start another request. New requests
            // are not added to this source snapshot; ClearContext repeats Stop.
            if !self.fetcher_loaders.Contains(&shared) {
                continue;
            }
            let mut loader = ResourceLoader::FromShared(shared.clone());
            if loader.IsKeepalive()
                && foundation::blink_features::KeepAliveInBrowserMigrationEnabled()
            {
                // Cancel the renderer loader while preserving the original
                // browser-owned transfer. Failure means the required native
                // keepalive producer did not implement its accepted contract.
                loader.DetachForKeepalive()?;
            } else if loader.IsKeepalive() {
                loader.CancelKeepalive()?;
            } else {
                ResourceLoader::CancelShared(&shared);
            }
            if loader.Status() != ResourceLoadStatus::kCancelled {
                continue;
            }
            let identity = loader.Identity();
            let raw_load = self.raw_loads.borrow_mut().remove(&identity);
            let was_raw = raw_load.is_some();
            if let Some(load) = raw_load {
                self.raw_inflight_bytes.set(
                    self.raw_inflight_bytes
                        .get()
                        .checked_sub(load.bytes)
                        .expect("original keepalive quota"),
                );
                if load.fetch.keepalive && !load.fetch.failed.swap(true, Ordering::AcqRel) {
                    load.fetch.metrics.Record(
                        load.fetch.context,
                        url_loader::FetchKeepaliveRequestState::Failed,
                        self.context_detached.load(Ordering::Acquire),
                    );
                }
            }
            // FinishAsError precedes ResourceLoadObserver::DidFailLoading.
            loader.NotifyCancellationClients();
            let observer = self.raw_observer.borrow().clone();
            if was_raw {
                if let Some(observer) = observer {
                    observer.DidFinishLoading(identity, ResourceLoadStatus::kCancelled);
                }
            }
        }
        Ok(())
    }
    /// ResourceFetcher::ClearContext cc2343-2390. StopFetching must have run
    /// before this point; repeat after detaching because abort callbacks may
    /// have admitted new requests. The network keepalive owner is independent.
    pub fn ClearFetchContext(self: &Rc<Self>) -> io::Result<()> {
        self.context_detached.store(true, Ordering::Release);
        self.active.set(false);
        self.raw_observer.borrow_mut().take();
        self.StopFetching()?;
        if !self.fetcher_loaders.IsEmpty() {
            assert!(
                !foundation::blink_features::KeepAliveInBrowserMigrationEnabled(),
                "only renderer-owned keepalive loaders survive detached StopFetching"
            );
            let runner = self.fetcher_task_runner.borrow().clone().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "renderer keepalive requires the original fetcher task runner",
                )
            })?;
            if self.fetcher_loaders.detached_completion.borrow().is_none() {
                let fetcher = self.clone();
                let completion = runner.CreateWakeTask(Box::new(move || {
                    fetcher.PollRawResources();
                    Ok(())
                }));
                let wake = completion.WakeCallback();
                *self.fetcher_loaders.detached_completion.borrow_mut() = Some(completion);
                for set in [LoaderSet::NonBlocking, LoaderSet::Blocking] {
                    for shared in self.fetcher_loaders.Snapshot(set) {
                        ResourceLoader::FromShared(shared).SetWakeCallback(wake.clone());
                    }
                }
            }
            if self.fetcher_loaders.keepalive_timeout.borrow().is_none() {
                let fetcher = self.clone();
                let task = runner.PostDelayedTask(
                    std::time::Duration::from_secs(30),
                    Box::new(move || fetcher.StopFetchingIncludingKeepaliveLoaders()),
                );
                *self.fetcher_loaders.keepalive_timeout.borrow_mut() = Some(task);
            }
        }
        Ok(())
    }
}
