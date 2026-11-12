//! URLLoader's OnConnected interception. Native peer metadata is read from the
//! selected pooled connection before any HTTP request headers/body are sent.
use crate::NetworkFetchOptions;
use fetch_policy::{
    RawLocalNetworkAccessChecker, RawLocalNetworkAccessEffect, RawLocalNetworkCheckResult,
    RawLocalNetworkPermissionResult, RawNetworkConnection,
};
use hyper_util::client::legacy::connect::{Connected, ConnectionCheck};
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub(crate) struct LocalNetworkAccessGate {
    pub(crate) fetch: NetworkFetchOptions,
    pub(crate) url: String,
    pub(crate) pending: Arc<LocalNetworkPermissionPending>,
    pub(crate) trace: crate::request_trace::RequestTrace,
    pub(crate) checker: Arc<Mutex<RawLocalNetworkAccessChecker>>,
}
impl ConnectionCheck for LocalNetworkAccessGate {
    fn check(&self, connected: &Connected) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send>> {
        let mut extras = hyper::http::Extensions::new();
        connected.get_extras(&mut extras);
        let connection = extras.get::<RawNetworkConnection>().copied();
        let fetch = self.fetch.clone();
        let url = self.url.clone();
        let checker = self.checker.clone();
        let pending = self.pending.clone();
        let trace = self.trace;
        Box::pin(async move {
            let connection = connection.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native connection omitted actual endpoint/proxy metadata",
                )
            })?;
            let security = fetch.security.as_ref();
            if security.current_frame.is_some()
                && security
                    .local_network_access_client_security_state
                    .is_none()
            {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "document ClientSecurityState is not bound",
                ));
            }
            let address_space = connection.AddressSpace(security);
            let trustworthy_same_origin = fetch.policy.IsURLPotentiallyTrustworthy(&url)
                && fetch.requestor_origin.IsSameOrigin(&url);
            let result = checker.lock().expect("LNA checker poisoned").Check(
                security,
                address_space,
                trustworthy_same_origin,
            );
            // The mutex protects only the synchronous checker. An asynchronous
            // permission continuation never holds it or a Document/engine borrow.
            let permission = if result == RawLocalNetworkCheckResult::PermissionRequired {
                let _wait = pending.Begin();
                if fetch.context_detached.load(Ordering::Acquire) {
                    Some(RawLocalNetworkPermissionResult::MissingDocument)
                } else {
                    let permission =
                        fetch_policy::RequestRawLocalNetworkPermission(security, address_space)
                            .await;
                    Some(if fetch.context_detached.load(Ordering::Acquire) {
                        RawLocalNetworkPermissionResult::MissingDocument
                    } else {
                        permission
                    })
                }
            } else {
                None
            };
            let allowed =
                result.IsAllowed() || permission == Some(RawLocalNetworkPermissionResult::Granted);
            fetch
                .local_network_access_effects
                .lock()
                .expect("LNA effects poisoned")
                .push(RawLocalNetworkAccessEffect {
                    url,
                    client_address_space: security
                        .local_network_access_client_security_state
                        .map_or(fetch_policy::RawFetchAddressSpace::Unknown, |c| {
                            c.ip_address_space
                        }),
                    response_address_space: address_space,
                    required_address_space: security.target_address_space,
                    result,
                    permission,
                });
            trace.event(
                "LocalNetworkAccessCheck",
                &[
                    ("result", result as u8 as f64),
                    ("allowed", allowed as u8 as f64),
                ],
            );
            if let Some(wake) = &fetch.policy_effect_wake {
                wake();
            }
            if allowed {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("local network access blocked: {result:?}, permission={permission:?}"),
                ))
            }
        })
    }
}

/// Permission waiting is not a stalled network transfer. The URLLoader lifetime
/// still owns/cancels the future, including the keepalive disconnection timeout.
#[derive(Default)]
pub(crate) struct LocalNetworkPermissionPending {
    pending: AtomicBool,
    changed: tokio::sync::Notify,
}
impl LocalNetworkPermissionPending {
    fn Begin(&self) -> PermissionWait<'_> {
        self.pending.store(true, Ordering::Release);
        self.changed.notify_one();
        PermissionWait(self)
    }
    pub(crate) async fn WithStallTimeout<F, T>(
        &self,
        future: F,
        options: &crate::HTTPTransportOptions,
    ) -> Result<T, crate::http_request::TransferError>
    where
        F: Future<Output = Result<T, crate::http_request::TransferError>>,
    {
        if options.stall_timeout_ms <= 0 {
            return future.await;
        }
        tokio::pin!(future);
        let duration = std::time::Duration::from_millis(options.stall_timeout_ms as u64);
        let mut deadline = tokio::time::Instant::now() + duration;
        loop {
            let changed = self.changed.notified();
            let waiting = self.pending.load(Ordering::Acquire);
            tokio::select! {
                result=&mut future=>return result,
                _=changed=>{deadline=tokio::time::Instant::now()+duration;},
                _=tokio::time::sleep_until(deadline),if !waiting=>{
                    if !self.pending.load(Ordering::Acquire) {
                        return Err(crate::http_request::TransferError::new("transfer made no byte progress",true));
                    }
                },
            }
        }
    }
}
struct PermissionWait<'a>(&'a LocalNetworkPermissionPending);
impl Drop for PermissionWait<'_> {
    fn drop(&mut self) {
        self.0.pending.store(false, Ordering::Release);
        self.0.changed.notify_one();
    }
}
