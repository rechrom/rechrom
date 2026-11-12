#![allow(non_snake_case)]

//! One resource request and its terminal result. The URL backend supplies bytes;
//! the resource owner decides how to decode, apply and schedule those bytes.

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::{
    cell::RefCell,
    io,
    rc::{Rc, Weak},
};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLoadStatus {
    kLoading,
    kComplete,
    kFailed,
    kCancelled,
    kNotStarted,
}

#[derive(Clone, Default)]
pub struct ResourceLoadResult {
    pub response: Option<URLResponse>,
    pub error: String,
}

pub(crate) struct SharedResourceLoad {
    identity: u64,
    media_info: Option<Rc<crate::media_resource_info::MediaResourceInfo>>,
    request: Option<std::sync::Arc<URLRequest>>,
    operation: Option<Box<dyn URLLoadOperation>>,
    result: ResourceLoadResult,
    status: ResourceLoadStatus,
    fetcher_membership: Option<crate::fetcher_lifecycle::LoaderMembership>,
    cancellation_clients: Vec<Weak<dyn crate::ResourceCancellationClient>>,
    is_link_preload: bool,
}

static NEXT_RESOURCE_LOAD_IDENTITY: AtomicU64 = AtomicU64::new(1);

pub(crate) fn NextResourceLoadIdentity() -> u64 {
    let identity = NEXT_RESOURCE_LOAD_IDENTITY.fetch_add(1, Ordering::Relaxed);
    assert_ne!(identity, 0, "resource load identity exhausted");
    identity
}

pub struct ResourceLoader {
    shared: Rc<RefCell<SharedResourceLoad>>,
    cancelled: bool,
    took_result: bool,
}

impl SharedResourceLoad {
    pub(crate) fn MemoryUsage(&self) -> (bool, usize) {
        (
            self.status == ResourceLoadStatus::kLoading,
            self.result
                .response
                .as_ref()
                .map_or(0, |response| response.body.capacity()),
        )
    }
}
impl ResourceLoader {
    pub(crate) fn Prepare(request: &URLRequest) -> Self {
        let request = std::sync::Arc::new(request.clone());
        let media_info = Rc::new(crate::media_resource_info::MediaResourceInfo::new(
            request.clone(),
        ));
        Self::FromShared(Rc::new(RefCell::new(SharedResourceLoad {
            identity: NextResourceLoadIdentity(),
            media_info: Some(media_info),
            request: Some(request),
            operation: None,
            result: ResourceLoadResult::default(),
            status: ResourceLoadStatus::kNotStarted,
            fetcher_membership: None,
            cancellation_clients: Vec::new(),
            is_link_preload: false,
        })))
    }
    pub(crate) fn StartPrepared(&mut self, loader: &mut dyn URLLoader, request: &URLRequest) {
        assert_eq!(self.Status(), ResourceLoadStatus::kNotStarted);
        let operation = if request.fetch.as_ref().is_some_and(|fetch| fetch.keepalive) {
            loader.LoadKeepalive(request)
        } else {
            loader.Load(request)
        };
        let mut shared = self.shared.borrow_mut();
        match operation {
            Ok(operation) => {
                shared.operation = Some(operation);
                shared.status = ResourceLoadStatus::kLoading;
            }
            Err(error) => {
                shared.result.error = error.to_string();
                shared.status = ResourceLoadStatus::kFailed;
            }
        }
        let failed = shared.status == ResourceLoadStatus::kFailed;
        drop(shared);
        if failed {
            self.RemoveFetcherMembership();
        }
    }
    pub(crate) fn DetachForKeepalive(&mut self) -> io::Result<()> {
        let operation = {
            let mut shared = self.shared.borrow_mut();
            if shared.status != ResourceLoadStatus::kLoading {
                return Ok(());
            }
            if let Some(operation) = shared.operation.as_mut() {
                operation.DetachForKeepalive()?;
            }
            shared.status = ResourceLoadStatus::kCancelled;
            shared.result.error = "resource load cancelled".into();
            shared.operation.take()
        };
        self.RemoveFetcherMembership();
        drop(operation);
        Ok(())
    }
    pub(crate) fn FailPrepared(&mut self, error: &str) {
        let mut shared = self.shared.borrow_mut();
        assert_eq!(shared.status, ResourceLoadStatus::kNotStarted);
        shared.status = ResourceLoadStatus::kFailed;
        shared.result.error = error.into();
    }
    pub fn Start(loader: &mut dyn URLLoader, request: &URLRequest) -> Self {
        let mut resource = Self::Prepare(request);
        resource.StartPrepared(loader, request);
        resource
    }

    pub fn Failed(error: impl ToString) -> Self {
        Self::FromShared(Rc::new(RefCell::new(SharedResourceLoad {
            identity: NextResourceLoadIdentity(),
            media_info: None,
            request: None,
            operation: None,
            result: ResourceLoadResult {
                response: None,
                error: error.to_string(),
            },
            status: ResourceLoadStatus::kFailed,
            fetcher_membership: None,
            cancellation_clients: Vec::new(),
            is_link_preload: false,
        })))
    }

    pub(crate) fn FromShared(shared: Rc<RefCell<SharedResourceLoad>>) -> Self {
        Self {
            shared,
            cancelled: false,
            took_result: false,
        }
    }

    pub(crate) fn Shared(&self) -> Rc<RefCell<SharedResourceLoad>> {
        self.shared.clone()
    }

    /// Stable identity of the underlying byte transfer. Independent handles
    /// returned for a coalesced request retain the same identity, allowing a
    /// format owner to share derived immutable data without keying it by URL.
    pub(crate) fn Request(&self) -> Option<std::sync::Arc<URLRequest>> {
        self.shared.borrow().request.clone()
    }
    /// The original transport's typed access-check result survives its string
    /// diagnostic and decode boundary; ImageLoader must not infer it from text.
    pub fn AccessCheckFailed(&self) -> bool {
        self.Request()
            .and_then(|request| {
                request.fetch.as_ref().map(|fetch| {
                    fetch
                        .access_check_failed
                        .load(std::sync::atomic::Ordering::Acquire)
                })
            })
            .unwrap_or(false)
    }
    pub fn Identity(&self) -> u64 {
        self.shared.borrow().identity
    }

    pub(crate) fn CancelShared(shared: &Rc<RefCell<SharedResourceLoad>>) -> bool {
        let (operation, membership) = {
            let mut shared = shared.borrow_mut();
            if !matches!(
                shared.status,
                ResourceLoadStatus::kLoading | ResourceLoadStatus::kNotStarted
            ) {
                return false;
            }
            shared.status = ResourceLoadStatus::kCancelled;
            shared.result.error = "resource load cancelled".into();
            (shared.operation.take(), shared.fetcher_membership.take())
        };
        if let Some(membership) = membership {
            membership.Remove();
        }
        drop(operation);
        true
    }
    pub(crate) fn BindFetcherMembership(
        &self,
        membership: crate::fetcher_lifecycle::LoaderMembership,
    ) {
        assert!(
            self.shared.borrow().fetcher_membership.is_none(),
            "one source loader set membership"
        );
        self.shared.borrow_mut().fetcher_membership = Some(membership);
    }
    pub(crate) fn MarkLinkPreload(&self) {
        self.shared.borrow_mut().is_link_preload = true;
    }
    pub(crate) fn PromoteToBlocking(&self) {
        // ResourceFetcher::MakePreloadedResourceBlockIfNeeded cc1160-1172.
        if !self.shared.borrow().is_link_preload {
            return;
        }
        let membership = self.shared.borrow_mut().fetcher_membership.take();
        if let Some(mut membership) = membership {
            membership.PromoteToBlocking(self.shared.clone());
            self.shared.borrow_mut().fetcher_membership = Some(membership);
        }
    }
    fn RemoveFetcherMembership(&self) {
        let membership = self.shared.borrow_mut().fetcher_membership.take();
        if let Some(membership) = membership {
            membership.Remove();
        }
    }
    pub fn RegisterCancellationClient(&self, client: &Rc<dyn crate::ResourceCancellationClient>) {
        let mut shared = self.shared.borrow_mut();
        let weak = Rc::downgrade(client);
        shared
            .cancellation_clients
            .retain(|client| client.strong_count() != 0);
        if !shared
            .cancellation_clients
            .iter()
            .any(|client| Weak::ptr_eq(client, &weak))
        {
            shared.cancellation_clients.push(weak);
        }
    }
    pub fn UnregisterCancellationClient(&self, client: &Rc<dyn crate::ResourceCancellationClient>) {
        let weak = Rc::downgrade(client);
        self.shared
            .borrow_mut()
            .cancellation_clients
            .retain(|client| !Weak::ptr_eq(client, &weak));
    }
    pub(crate) fn NotifyCancellationClients(&self) {
        let (identity, request, clients) = {
            let shared = self.shared.borrow();
            (
                shared.identity,
                shared.request.clone(),
                shared.cancellation_clients.clone(),
            )
        };
        let Some(request) = request else {
            return;
        };
        for weak in clients {
            if !self
                .shared
                .borrow()
                .cancellation_clients
                .iter()
                .any(|client| Weak::ptr_eq(client, &weak))
            {
                continue;
            }
            if let Some(client) = weak.upgrade() {
                client.ResourceCancelled(identity, &request);
            }
        }
    }
    pub(crate) fn IsKeepalive(&self) -> bool {
        self.Request()
            .is_some_and(|request| request.fetch.as_ref().is_some_and(|fetch| fetch.keepalive))
    }
    pub(crate) fn ShouldBeKeptAliveWhenDetached(&self) -> io::Result<bool> {
        if foundation::blink_features::KeepAliveInBrowserMigrationEnabled() || !self.IsKeepalive() {
            return Ok(false);
        }
        let shared = self.shared.borrow();
        if shared.result.response.is_some() {
            return Ok(false);
        }
        if shared.status == ResourceLoadStatus::kNotStarted {
            return Ok(true);
        }
        let started = shared
            .operation
            .as_ref()
            .and_then(|operation| operation.HasResponseStarted())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "renderer keepalive requires actual response-header receipt",
                )
            })?;
        Ok(!started)
    }
    pub(crate) fn CancelKeepalive(&mut self) -> io::Result<()> {
        let operation = {
            let mut shared = self.shared.borrow_mut();
            if shared.status != ResourceLoadStatus::kLoading {
                return Ok(());
            }
            if let Some(operation) = shared.operation.as_mut() {
                operation.CancelKeepalive()?;
            }
            shared.status = ResourceLoadStatus::kCancelled;
            shared.result.error = "resource load cancelled".into();
            shared.operation.take()
        };
        self.RemoveFetcherMembership();
        drop(operation);
        Ok(())
    }

    pub(crate) fn Error(&self) -> String {
        self.shared.borrow().result.error.clone()
    }
    pub fn Status(&self) -> ResourceLoadStatus {
        if self.cancelled {
            ResourceLoadStatus::kCancelled
        } else {
            self.shared.borrow().status
        }
    }

    pub fn TakeResult(&mut self) -> ResourceLoadResult {
        if self.took_result {
            return ResourceLoadResult::default();
        }
        self.took_result = true;
        if self.cancelled {
            return ResourceLoadResult {
                response: None,
                error: "resource load cancelled".into(),
            };
        }
        self.shared.borrow().result.clone()
    }

    pub fn Poll(&mut self) -> bool {
        if self.cancelled {
            return true;
        }
        let mut shared = self.shared.borrow_mut();
        if shared.status == ResourceLoadStatus::kNotStarted {
            return false;
        }
        if shared.status != ResourceLoadStatus::kLoading {
            return true;
        }
        match shared
            .operation
            .as_mut()
            .expect("loading operation exists")
            .Poll()
        {
            Ok(None) => return false,
            Ok(Some(response)) => {
                if let Some(info) = &shared.media_info {
                    info.ResponseReceived(&response);
                }
                shared.result.response = Some(response);
                shared.status = ResourceLoadStatus::kComplete;
            }
            Err(error) => {
                shared.result.error = error.to_string();
                shared.status = ResourceLoadStatus::kFailed;
            }
        }
        let operation = shared.operation.take();
        drop(shared);
        self.RemoveFetcherMembership();
        drop(operation);
        true
    }

    pub(crate) fn SetScheduling(&mut self, request: &url_loader::URLRequest) {
        if let Some(operation) = self.shared.borrow_mut().operation.as_mut() {
            operation.SetScheduling(request.destination, request.defer, request.script_type);
        }
    }

    pub fn MediaResourceInfo(
        &self,
    ) -> Option<Rc<dyn image_resource::media_timing::ImageResourceInfo>> {
        self.shared
            .borrow()
            .media_info
            .as_ref()
            .map(|info| info.clone() as Rc<dyn image_resource::media_timing::ImageResourceInfo>)
    }
    pub fn SetPriority(&mut self, priority: url_loader::ResourceLoadPriority) {
        let mut shared = self.shared.borrow_mut();
        if let Some(info) = &shared.media_info {
            info.SetPriority(priority);
        }
        if let Some(operation) = shared.operation.as_mut() {
            operation.SetPriority(priority);
        }
    }

    pub fn SetWakeCallback(&mut self, callback: url_loader::URLWakeCallback) {
        if self.cancelled {
            return;
        }
        let mut shared = self.shared.borrow_mut();
        if let Some(operation) = shared.operation.as_mut() {
            operation.SetWakeCallback(callback);
        }
    }

    pub fn Cancel(&mut self) {
        self.cancelled = true;
    }
}

pub fn StartResource(loader: &mut dyn URLLoader, request: &URLRequest) -> ResourceLoader {
    ResourceLoader::Start(loader, request)
}

pub fn AwaitResource(pending: &mut ResourceLoader) -> ResourceLoadResult {
    while !pending.Poll() {
        thread::sleep(Duration::from_millis(1));
    }
    pending.TakeResult()
}

pub fn RequireResponse(result: ResourceLoadResult) -> io::Result<URLResponse> {
    result
        .response
        .ok_or_else(|| io::Error::other(result.error))
}

pub fn LoadResponse(loader: &mut dyn URLLoader, request: &URLRequest) -> io::Result<URLResponse> {
    RequireResponse(AwaitResource(&mut StartResource(loader, request)))
}
