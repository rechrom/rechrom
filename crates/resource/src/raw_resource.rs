//! RawResource::Fetch and ResourceFetcher's nonblocking loader ownership.
//! Chromium raw_resource.cc:65-72; resource_fetcher.cc:2755-2828,
//! 2582-2612/2683-2687. Every native raw request owns its original transfer.
use crate::{ResourceEngine, ResourceLoadStatus, ResourceLoader};
use foundation::content_security_policy::{
    CSPCheckHeaderType, CSPRedirectStatus, CSPReportingDisposition, CSPRequest,
};
use std::sync::Arc;
use url_loader::{FetchKeepaliveMetrics, FetchKeepaliveRequestState, FetchMode, URLRequest};

/// Source Resource object identity/status, not a URL callback or image client.
pub struct RawResource {
    loader: ResourceLoader,
}
impl RawResource {
    pub fn Identity(&self) -> u64 {
        self.loader.Identity()
    }
    pub fn Request(&self) -> Option<Arc<URLRequest>> {
        self.loader.Request()
    }
    pub fn RegisterCancellationClient(
        &self,
        client: &std::rc::Rc<dyn crate::ResourceCancellationClient>,
    ) {
        self.loader.RegisterCancellationClient(client);
    }
    pub fn UnregisterCancellationClient(
        &self,
        client: &std::rc::Rc<dyn crate::ResourceCancellationClient>,
    ) {
        self.loader.UnregisterCancellationClient(client);
    }
    pub fn GetStatus(&self) -> ResourceLoadStatus {
        self.loader.Status()
    }
    pub fn Error(&self) -> String {
        self.loader.Error()
    }
    pub fn FailedWithRequest(request: URLRequest, reason: impl ToString) -> Self {
        let mut loader = ResourceLoader::Prepare(&request);
        let reason = reason.to_string();
        loader.FailPrepared(&reason);
        browser_tracing::instant_with_text(
            "resource",
            "RawResourceAdmissionRejected",
            &reason,
            &[("resource_id", loader.Identity() as f64)],
        );
        Self { loader }
    }
    pub fn Failed(reason: impl ToString) -> Self {
        Self {
            loader: ResourceLoader::Failed(reason),
        }
    }
}
/// ResourceLoadObserver receives original immutable request values, identity
/// and terminal state. It cannot mutate the fetcher's policy or transfer owner.
pub trait RawResourceObserver {
    fn DidDispatch(&self, request: &URLRequest);
    fn WillSendRequest(&self, resource_id: u64, request: &URLRequest);
    fn DidFinishLoading(&self, resource_id: u64, status: ResourceLoadStatus);
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawRequestKind {
    OneWay,
    TextTrack,
}
pub(crate) struct RawLoad {
    pub(crate) loader: ResourceLoader,
    pub(crate) bytes: u32,
    pub(crate) fetch: url_loader::NetworkFetchOptions,
}
impl ResourceEngine {
    pub fn SetRawResourceObserver(&self, observer: Option<std::rc::Rc<dyn RawResourceObserver>>) {
        *self.raw_observer.borrow_mut() = observer;
    }
    pub fn RawResourceMetrics(&self) -> url_loader::FetchKeepaliveMetricsSnapshot {
        self.raw_metrics.Snapshot()
    }
    pub fn RawResourceMetricsOwner(&self) -> Arc<FetchKeepaliveMetrics> {
        self.raw_metrics.clone()
    }
    /// FetchParameters owns a checked current-world PolicyContainer view.
    /// Native one-way consumers are no-cors; other modes require their source
    /// Fetch/CORS owner and are rejected here rather than silently downgraded.
    pub(crate) fn PrepareRawResourceRequest(
        &self,
        mut request: URLRequest,
        kind: RawRequestKind,
    ) -> Result<URLRequest, RawResource> {
        if let Some(fetch) = request.fetch.as_mut() {
            // Retain only the source wake slot weakly. A detached network task
            // must not retain/call the Document's scheduler callback.
            fetch.policy_effect_wake = self.wake_callback.borrow().as_ref().map(|wake| {
                let weak = Arc::downgrade(wake);
                let detached = fetch.context_detached.clone();
                Arc::new(move || {
                    if !detached.load(std::sync::atomic::Ordering::Acquire) {
                        if let Some(wake) = weak.upgrade() {
                            wake();
                        }
                    }
                }) as url_loader::URLWakeCallback
            });
        }
        let Some(fetch) = request.fetch.as_ref() else {
            return Err(RawResource::FailedWithRequest(
                request,
                "RawResource requires typed FetchParameters",
            ));
        };
        let observer = self.raw_observer.borrow().clone();
        if let Some(observer) = observer {
            observer.DidDispatch(&request);
        }
        browser_tracing::instant_with_text(
            "resource",
            if kind == RawRequestKind::OneWay {
                "DidDispatchPingLoader"
            } else {
                "DidDispatchTextTrack"
            },
            &request.url,
            &[("keepalive", fetch.keepalive as u8 as f64)],
        );
        if kind == RawRequestKind::OneWay {
            fetch
                .metrics
                .Record(fetch.context, FetchKeepaliveRequestState::Total, false);
        }
        if !self.active.get() {
            return Err(RawResource::FailedWithRequest(
                request,
                "raw fetch context is detached",
            ));
        }
        if kind == RawRequestKind::OneWay && fetch.mode != FetchMode::NoCors {
            return Err(RawResource::FailedWithRequest(
                request,
                "raw one-way producer requires the no-cors Fetch owner",
            ));
        }
        if kind == RawRequestKind::TextTrack
            && (fetch.keepalive
                || !matches!(fetch.mode, FetchMode::SameOrigin | FetchMode::Cors)
                || request.destination != url_loader::RequestDestination::kTrack
                || fetch.context != foundation::content_security_policy::CSPRequestContext::Track)
        {
            return Err(RawResource::FailedWithRequest(
                request,
                "text track requires its same-origin/CORS request owner",
            ));
        }
        let prepared = fetch.policy.PrepareRequestURL(&request.url);
        request.url = prepared.url;
        request.upgrade_if_insecure |= prepared.upgrade_if_insecure;
        let mixed = fetch_policy::CheckRawMixedContent(
            fetch.security.as_ref(),
            fetch.policy.as_ref(),
            &request.url,
            &request.url,
            fetch.context,
            CSPRedirectStatus::NoRedirect,
            prepared.block_all_mixed_content,
        );
        if let Some(effect) = mixed.effect {
            fetch
                .mixed_content_effects
                .lock()
                .expect("mixed content effect queue poisoned")
                .push(effect);
        }
        if let Some(reason) = mixed.blocked {
            return Err(RawResource::FailedWithRequest(
                request,
                format!("RawResource mixed-content admission: {reason:?}"),
            ));
        }
        let verdict = fetch.policy.AllowRequest(&CSPRequest {
            url: &request.url,
            url_before_redirects: &request.url,
            context: fetch.context,
            world: fetch.world,
            redirect: CSPRedirectStatus::NoRedirect,
            reporting: CSPReportingDisposition::Report,
            check_header_type: CSPCheckHeaderType::All,
        });
        fetch
            .policy_violations
            .lock()
            .expect("CSP effect queue poisoned")
            .extend(verdict.violations);
        if !verdict.allowed {
            return Err(RawResource::FailedWithRequest(
                request,
                "RawResource blocked by Content Security Policy",
            ));
        }
        request
            .fetch
            .as_mut()
            .expect("admitted FetchParameters")
            .upgrade_if_insecure = request.upgrade_if_insecure;
        Ok(self.NormalizeRequest(request))
    }
    pub fn FetchRaw(&self, request: URLRequest) -> RawResource {
        let _request = crate::fetcher_lifecycle::RequestResourceScope::new(
            self.is_in_request_resource.clone(),
        );
        let request = match self.PrepareRawResourceRequest(request, RawRequestKind::OneWay) {
            Ok(request) => request,
            Err(resource) => return resource,
        };
        let fetch = request.fetch.as_ref().unwrap().clone();
        let mut loader = ResourceLoader::Prepare(&request);
        let identity = loader.Identity();
        browser_tracing::instant_with_text(
            "resource",
            "RawResourceWillSendRequest",
            &request.url,
            &[
                ("resource_id", identity as f64),
                ("body_bytes", request.body.len() as f64),
                ("keepalive", fetch.keepalive as u8 as f64),
            ],
        );
        let observer = self.raw_observer.borrow().clone();
        if let Some(observer) = observer {
            observer.WillSendRequest(identity, &request);
        }
        let bytes = if fetch.keepalive {
            let size = u32::try_from(request.body.len())
                .ok()
                .filter(|size| *size <= 65536 - self.raw_inflight_bytes.get());
            let Some(size) = size else {
                loader.FailPrepared("keepalive in-flight body exceeds 64 KiB");
                return RawResource { loader };
            };
            size
        } else {
            0
        };
        self.raw_inflight_bytes
            .set(self.raw_inflight_bytes.get() + bytes);
        if fetch.keepalive {
            fetch
                .metrics
                .Record(fetch.context, FetchKeepaliveRequestState::Started, false);
        }
        // Raw side effects retain one original operation, independently of
        // GET MemoryCache coalescing and of the caller's returned handle.
        self.fetcher_loaders
            .Add(&loader, crate::fetcher_lifecycle::LoaderSet::NonBlocking);
        loader.StartPrepared(&mut *self.loader.borrow_mut(), &request);
        if let Some(wake) = self.wake_callback.borrow().clone() {
            loader.SetWakeCallback(wake);
        }
        let handle = ResourceLoader::FromShared(loader.Shared());
        self.raw_loads.borrow_mut().insert(
            identity,
            RawLoad {
                loader,
                bytes,
                fetch,
            },
        );
        // An already-terminal transport result still belongs to this same
        // Resource; releasing quota never depends on callers retaining handles.
        self.PollRawResources();
        RawResource { loader: handle }
    }
    pub fn HasPendingRawResources(&self) -> bool {
        !self.raw_loads.borrow().is_empty() || !self.text_track_resources.borrow().is_empty()
    }
    pub fn RawResourceInflightBytes(&self) -> u32 {
        self.raw_inflight_bytes.get()
    }
    pub fn PollRawResources(&self) -> usize {
        let mut loads = self.raw_loads.borrow_mut();
        let mut completed = Vec::new();
        loads.retain(|identity, load| {
            if !load.loader.Poll() {
                return true;
            }
            self.raw_inflight_bytes.set(
                self.raw_inflight_bytes
                    .get()
                    .checked_sub(load.bytes)
                    .expect("raw quota ownership"),
            );
            let detached = load
                .fetch
                .context_detached
                .load(std::sync::atomic::Ordering::Acquire);
            if load.fetch.keepalive {
                let status = load.loader.Status();
                if status == ResourceLoadStatus::kComplete
                    && !load
                        .fetch
                        .succeeded
                        .swap(true, std::sync::atomic::Ordering::AcqRel)
                {
                    load.fetch.metrics.Record(
                        load.fetch.context,
                        FetchKeepaliveRequestState::Succeeded,
                        detached,
                    );
                }
                if status == ResourceLoadStatus::kFailed
                    && !load
                        .fetch
                        .failed
                        .swap(true, std::sync::atomic::Ordering::AcqRel)
                {
                    load.fetch.metrics.Record(
                        load.fetch.context,
                        FetchKeepaliveRequestState::Failed,
                        detached,
                    );
                }
            }
            browser_tracing::instant(
                "resource",
                "RawResourceDidFinishLoading",
                &[
                    ("resource_id", *identity as f64),
                    ("status", load.loader.Status() as u8 as f64),
                ],
            );
            completed.push((*identity, load.loader.Status()));
            false
        });
        drop(loads);
        let observer = self.raw_observer.borrow().clone();
        if let Some(observer) = observer.as_ref() {
            for &(identity, status) in &completed {
                observer.DidFinishLoading(identity, status);
            }
        }
        completed.len() + self.PollTextTrackResources()
    }
}
