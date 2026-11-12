//! Chromium 6c1d401f: local_network_access_checker.{h,cc}:43-210;
//! local_network_access_url_loader_interceptor.{h,cc}:20-238;
//! content/browser/renderer_host/local_network_access_util.cc:102-214;
//! content/browser/storage_partition_impl.cc:2347-2448;
//! components/permissions/permission_context_base.cc:286-288,506-527,894-899.
use super::{AddressSpaceFromIP, RawFetchAddressSpace as Space, RawFetchSecurityContext};
use std::{future::Future, net::SocketAddr, pin::Pin};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawLocalNetworkRequestPolicy {
    Allow,
    Warn,
    Block,
    PermissionBlock,
    PermissionWarn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawLocalNetworkClientSecurityState {
    pub ip_address_space: Space,
    pub is_web_secure_context: bool,
    pub local_network_access_request_policy: RawLocalNetworkRequestPolicy,
}
/// Source subresource branch. Flags are actual embedding feature admissions;
/// secure-context and address-space facts come from the same PolicyContainer.
pub fn DeriveRawLocalNetworkClientSecurityState(
    ip_address_space: Space,
    is_web_secure_context: bool,
    checks_enabled: bool,
    warning_only: bool,
    disable_web_security: bool,
) -> RawLocalNetworkClientSecurityState {
    use RawLocalNetworkRequestPolicy::*;
    let policy = if disable_web_security {
        Allow
    } else if checks_enabled {
        if warning_only {
            PermissionWarn
        } else if is_web_secure_context {
            PermissionBlock
        } else {
            Block
        }
    } else if !is_web_secure_context {
        if ip_address_space == Space::Unknown {
            Allow
        } else {
            Warn
        }
    } else {
        Allow
    };
    RawLocalNetworkClientSecurityState {
        ip_address_space,
        is_web_secure_context,
        local_network_access_request_policy: policy,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawNetworkTransport {
    Direct,
    Proxied,
}
#[derive(Clone, Copy, Debug)]
pub struct RawNetworkConnection {
    pub transport: RawNetworkTransport,
    /// Actual socket peer from the connector, after its existing DNS resolution.
    pub peer: SocketAddr,
}
impl RawNetworkConnection {
    pub fn AddressSpace(&self, security: &RawFetchSecurityContext) -> Space {
        match self.transport {
            RawNetworkTransport::Proxied => Space::Unknown,
            RawNetworkTransport::Direct => security
                .address_space_overrides
                .as_ref()
                .and_then(|owner| owner.ForEndpoint(self.peer.ip(), self.peer.port()))
                .unwrap_or_else(|| AddressSpaceFromIP(self.peer.ip())),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RawLocalNetworkCheckResult {
    AllowedMissingClientSecurityState = 0,
    AllowedNoLessPublic = 1,
    AllowedByPolicyAllow = 2,
    AllowedByPolicyWarn = 3,
    BlockedByLoadOption = 4,
    BlockedByPolicyBlock = 5,
    BlockedByInconsistentIpAddressSpace = 11,
    AllowedPotentiallyTrustworthySameOrigin = 12,
    PermissionRequired = 13,
    AllowedByPermissionWarn = 14,
    BlockedByRequiredIpAddressSpaceMismatch = 15,
}
impl RawLocalNetworkCheckResult {
    pub fn IsAllowed(self) -> bool {
        matches!(
            self,
            Self::AllowedMissingClientSecurityState
                | Self::AllowedNoLessPublic
                | Self::AllowedByPolicyAllow
                | Self::AllowedByPolicyWarn
                | Self::AllowedPotentiallyTrustworthySameOrigin
                | Self::AllowedByPermissionWarn
        )
    }
}
/// One URLLoader's retained checker; no document-wide or URL-derived IP cache.
pub struct RawLocalNetworkAccessChecker {
    response_address_space: Option<Space>,
}
impl RawLocalNetworkAccessChecker {
    pub fn new() -> Self {
        Self {
            response_address_space: None,
        }
    }
    pub fn ResetForRedirect(&mut self) {
        self.ResetForRetry();
    }
    pub fn ResetForRetry(&mut self) {
        self.response_address_space = None;
    }
    pub fn ResponseAddressSpace(&self) -> Option<Space> {
        self.response_address_space
    }
    pub fn Check(
        &mut self,
        security: &RawFetchSecurityContext,
        address_space: Space,
        potentially_trustworthy_same_origin: bool,
    ) -> RawLocalNetworkCheckResult {
        let result =
            self.CheckAddressSpace(security, address_space, potentially_trustworthy_same_origin);
        self.response_address_space = Some(address_space);
        result
    }
    fn CheckAddressSpace(
        &self,
        s: &RawFetchSecurityContext,
        target: Space,
        same_origin: bool,
    ) -> RawLocalNetworkCheckResult {
        use RawLocalNetworkCheckResult as R;
        use RawLocalNetworkRequestPolicy as P;
        if s.block_local_request && rank(target) < rank(Space::Public) {
            return R::BlockedByLoadOption;
        }
        if same_origin {
            return R::AllowedPotentiallyTrustworthySameOrigin;
        }
        let Some(client) = s.local_network_access_client_security_state else {
            return R::AllowedMissingClientSecurityState;
        };
        if s.local_network_access_checks
            && s.target_address_space != Space::Unknown
            && target != s.target_address_space
        {
            return R::BlockedByRequiredIpAddressSpaceMismatch;
        }
        let policy = client.local_network_access_request_policy;
        if policy == P::Allow {
            return R::AllowedByPolicyAllow;
        }
        if self
            .response_address_space
            .is_some_and(|last| last != target)
        {
            return if policy == P::Warn {
                R::AllowedByPolicyWarn
            } else {
                R::BlockedByInconsistentIpAddressSpace
            };
        }
        let (mut target_rank, mut client_rank) = (rank(target), rank(client.ip_address_space));
        if s.local_network_access_checks {
            target_rank = target_rank.max(rank(Space::Local));
            client_rank = client_rank.max(rank(Space::Local));
        }
        if target_rank >= client_rank {
            return R::AllowedNoLessPublic;
        }
        match policy {
            P::Allow => unreachable!(),
            P::Warn => R::AllowedByPolicyWarn,
            P::Block => R::BlockedByPolicyBlock,
            P::PermissionBlock => R::PermissionRequired,
            P::PermissionWarn => R::AllowedByPermissionWarn,
        }
    }
}
fn rank(space: Space) -> u8 {
    match space {
        Space::Unknown | Space::Public => 2,
        Space::Local => 1,
        Space::Loopback => 0,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawLocalNetworkPermission {
    LocalNetwork,
    LoopbackNetwork,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawLocalNetworkPermissionStatus {
    Granted,
    Denied,
    Ask,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawLocalNetworkPermissionResult {
    Granted,
    Denied,
    UnsupportedPrompt,
    MissingDocument,
}
pub type RawLocalNetworkPermissionContinuation =
    Pin<Box<dyn Future<Output = RawLocalNetworkPermissionResult> + Send>>;
/// Same current-document permission owner. GetPermissionStatus must include the
/// inherited PermissionsPolicy feature (local-network / loopback-network) and
/// actual document lifetime. RequestPermission is asynchronous; no UI capability
/// must return UnsupportedPrompt/Denied, never an implicit grant.
pub trait RawFetchLocalNetworkAccessPolicy: Send + Sync {
    fn IsDocumentCurrent(&self) -> bool;
    fn GetPermissionStatus(
        &self,
        permission: RawLocalNetworkPermission,
    ) -> RawLocalNetworkPermissionStatus;
    fn IsFeatureEnabled(&self, permission: RawLocalNetworkPermission) -> bool;
    fn RequestPermission(
        &self,
        permission: RawLocalNetworkPermission,
    ) -> RawLocalNetworkPermissionContinuation;
}
pub async fn RequestRawLocalNetworkPermission(
    s: &RawFetchSecurityContext,
    target: Space,
) -> RawLocalNetworkPermissionResult {
    use RawLocalNetworkPermissionResult as R;
    let Some(client) = s.local_network_access_client_security_state else {
        return R::MissingDocument;
    };
    if !client.is_web_secure_context {
        return R::Denied;
    }
    let permission = match target {
        Space::Local => RawLocalNetworkPermission::LocalNetwork,
        Space::Loopback => RawLocalNetworkPermission::LoopbackNetwork,
        _ => return R::Denied,
    };
    let Some(owner) = &s.local_network_access_policy else {
        return R::UnsupportedPrompt;
    };
    if !owner.IsDocumentCurrent() {
        return R::MissingDocument;
    }
    if !owner.IsFeatureEnabled(permission) {
        return R::Denied;
    }
    match owner.GetPermissionStatus(permission) {
        RawLocalNetworkPermissionStatus::Granted => R::Granted,
        RawLocalNetworkPermissionStatus::Denied => R::Denied,
        RawLocalNetworkPermissionStatus::Ask => {
            let result = owner.RequestPermission(permission).await;
            if owner.IsDocumentCurrent() {
                result
            } else {
                R::MissingDocument
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct RawLocalNetworkAccessEffect {
    pub url: String,
    pub client_address_space: Space,
    pub response_address_space: Space,
    pub required_address_space: Space,
    pub result: RawLocalNetworkCheckResult,
    pub permission: Option<RawLocalNetworkPermissionResult>,
}

/// Native current-document permission owner. Policy inheritance and stored
/// permission statuses arrive as typed facts from their owning mechanisms;
/// network tasks retain this same owner, and read live values on each connection.
/// No prompt implementation is installed implicitly.
pub struct RawLocalNetworkPermissionController {
    current: std::sync::atomic::AtomicBool,
    state: std::sync::RwLock<RawLocalNetworkDocumentPermissions>,
    prompt: Option<std::sync::Arc<dyn RawLocalNetworkPermissionPrompt>>,
}
#[derive(Clone, Copy, Debug)]
pub struct RawLocalNetworkDocumentPermissions {
    pub local_network_feature_enabled: bool,
    pub loopback_network_feature_enabled: bool,
    pub local_network: RawLocalNetworkPermissionStatus,
    pub loopback_network: RawLocalNetworkPermissionStatus,
}
pub trait RawLocalNetworkPermissionPrompt: Send + Sync {
    fn RequestPermission(
        &self,
        permission: RawLocalNetworkPermission,
    ) -> RawLocalNetworkPermissionContinuation;
}
impl RawLocalNetworkPermissionController {
    pub fn new(
        state: RawLocalNetworkDocumentPermissions,
        prompt: Option<std::sync::Arc<dyn RawLocalNetworkPermissionPrompt>>,
    ) -> Self {
        Self {
            current: std::sync::atomic::AtomicBool::new(true),
            state: std::sync::RwLock::new(state),
            prompt,
        }
    }
    pub fn ContextDestroyed(&self) {
        self.current
            .store(false, std::sync::atomic::Ordering::Release);
    }
    /// Called at the existing PermissionsPolicy/content-setting mutation point,
    /// not reconstructed from URL, frame ancestry, or a second document walk.
    pub fn UpdateDocumentPermissions(&self, state: RawLocalNetworkDocumentPermissions) {
        *self.state.write().expect("LNA permission state poisoned") = state;
    }
}
impl RawFetchLocalNetworkAccessPolicy for RawLocalNetworkPermissionController {
    fn IsDocumentCurrent(&self) -> bool {
        self.current.load(std::sync::atomic::Ordering::Acquire)
    }
    fn GetPermissionStatus(&self, p: RawLocalNetworkPermission) -> RawLocalNetworkPermissionStatus {
        if !self.IsDocumentCurrent() {
            return RawLocalNetworkPermissionStatus::Denied;
        }
        let state = self.state.read().expect("LNA permission state poisoned");
        match p {
            RawLocalNetworkPermission::LocalNetwork => state.local_network,
            RawLocalNetworkPermission::LoopbackNetwork => state.loopback_network,
        }
    }
    fn IsFeatureEnabled(&self, p: RawLocalNetworkPermission) -> bool {
        if !self.IsDocumentCurrent() {
            return false;
        }
        let state = self.state.read().expect("LNA permission state poisoned");
        match p {
            RawLocalNetworkPermission::LocalNetwork => state.local_network_feature_enabled,
            RawLocalNetworkPermission::LoopbackNetwork => state.loopback_network_feature_enabled,
        }
    }
    fn RequestPermission(
        &self,
        p: RawLocalNetworkPermission,
    ) -> RawLocalNetworkPermissionContinuation {
        if !self.IsDocumentCurrent() {
            return Box::pin(async { RawLocalNetworkPermissionResult::MissingDocument });
        }
        match &self.prompt {
            Some(prompt) => prompt.RequestPermission(p),
            None => Box::pin(async { RawLocalNetworkPermissionResult::UnsupportedPrompt }),
        }
    }
}
