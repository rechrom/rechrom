//! Fetch's immutable request-head values at the network-service boundary.
//! Chromium services/network/public/cpp/resource_request.h; FetchParameters;
//! KeepAliveURLLoader cc:150-159,1337-1389 (browser migration is enabled).
use foundation::content_security_policy::{
    CSPRequestContext, CSPRequestPolicy, CSPViolation, CSPWorld,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FetchMode {
    NoCors,
    Cors,
    SameOrigin,
    Navigate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FetchCredentialsMode {
    Omit,
    SameOrigin,
    Include,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FetchRedirectMode {
    Follow,
    Manual,
    Error,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FetchOrigin {
    Tuple {
        scheme: String,
        host: String,
        port: u16,
    },
    Opaque(u64),
}
impl FetchOrigin {
    pub fn Serialize(&self) -> String {
        match self {
            Self::Opaque(_) => "null".into(),
            Self::Tuple { scheme, host, port } => {
                if scheme == "file" {
                    return "file://".into();
                }
                let host = if host.contains(':') && !host.starts_with('[') {
                    format!("[{host}]")
                } else {
                    host.clone()
                };
                if matches!(
                    (scheme.as_str(), *port),
                    ("http", 80) | ("https", 443) | ("ws", 80) | ("wss", 443) | ("ftp", 21)
                ) || !matches!(scheme.as_str(), "http" | "https" | "ws" | "wss" | "ftp")
                {
                    format!("{scheme}://{host}")
                } else {
                    format!("{scheme}://{host}:{port}")
                }
            }
        }
    }
    pub(crate) fn IsSameOrigin(&self, url: &str) -> bool {
        matches!(self, Self::Tuple { .. }) && crate::IsSameOrigin(&self.Serialize(), url)
    }
}
/// FetchUtils::FetchKeepAliveRequestState, fetch_utils.h:32-37.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum FetchKeepaliveRequestState {
    Total,
    Started,
    Succeeded,
    Failed,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchKeepaliveMetricsSnapshot {
    pub states: [[u64; 4]; 3],
    pub detached_states: [[u64; 4]; 3],
}
pub struct FetchKeepaliveMetrics {
    states: [[AtomicU64; 4]; 3],
    detached_states: [[AtomicU64; 4]; 3],
}
impl Default for FetchKeepaliveMetrics {
    fn default() -> Self {
        Self {
            states: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            detached_states: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
        }
    }
}
impl FetchKeepaliveMetrics {
    pub fn Record(
        &self,
        context: CSPRequestContext,
        state: FetchKeepaliveRequestState,
        detached: bool,
    ) {
        let context = match context {
            CSPRequestContext::Ping => 0,
            CSPRequestContext::Beacon => 1,
            CSPRequestContext::CSPReport => 2,
            CSPRequestContext::Image
            | CSPRequestContext::ImageSet
            | CSPRequestContext::Font
            | CSPRequestContext::Track => return,
        };
        self.states[context][state as usize].fetch_add(1, Ordering::Relaxed);
        if detached {
            self.detached_states[context][state as usize].fetch_add(1, Ordering::Relaxed);
        }
        browser_tracing::instant(
            "resource",
            "FetchKeepAliveRequest",
            &[
                ("context", context as f64),
                ("state", state as usize as f64),
                ("detached", detached as u8 as f64),
            ],
        );
    }
    pub fn Snapshot(&self) -> FetchKeepaliveMetricsSnapshot {
        FetchKeepaliveMetricsSnapshot {
            states: std::array::from_fn(|c| {
                std::array::from_fn(|s| self.states[c][s].load(Ordering::Relaxed))
            }),
            detached_states: std::array::from_fn(|c| {
                std::array::from_fn(|s| self.detached_states[c][s].load(Ordering::Relaxed))
            }),
        }
    }
}
/// Policy is the fetch-start PolicyContainer view: shared compiled AST owners,
/// never a callback into Document or parser state from the network task.
#[derive(Clone)]
pub struct NetworkFetchOptions {
    pub mode: FetchMode,
    pub credentials: FetchCredentialsMode,
    pub redirect: FetchRedirectMode,
    pub keepalive: bool,
    pub upgrade_if_insecure: bool,
    pub requestor_origin: FetchOrigin,
    pub world: CSPWorld,
    pub context: CSPRequestContext,
    pub policy: Arc<dyn CSPRequestPolicy>,
    pub policy_violations: Arc<Mutex<Vec<CSPViolation>>>,
    pub metrics: Arc<FetchKeepaliveMetrics>,
    pub context_detached: Arc<AtomicBool>,
    pub succeeded: Arc<AtomicBool>,
    pub failed: Arc<AtomicBool>,
    pub access_check_failed: Arc<AtomicBool>,
    pub security: Arc<fetch_policy::RawFetchSecurityContext>,
    pub mixed_content_effects: Arc<Mutex<Vec<fetch_policy::RawMixedContentEffect>>>,
    pub local_network_access_effects: Arc<Mutex<Vec<fetch_policy::RawLocalNetworkAccessEffect>>>,
    pub policy_effect_wake: Option<crate::HTTPWakeCallback>,
}
impl std::fmt::Debug for NetworkFetchOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkFetchOptions")
            .field("mode", &self.mode)
            .field("credentials", &self.credentials)
            .field("redirect", &self.redirect)
            .field("keepalive", &self.keepalive)
            .field("requestor_origin", &self.requestor_origin)
            .field("world", &self.world)
            .field("context", &self.context)
            .finish_non_exhaustive()
    }
}
